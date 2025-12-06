use anyhow::Result;
use chrono::Local;
use clap::Parser;
use rand::Rng;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};

#[derive(Parser, Debug)]
#[command(name = "logstorm")]
#[command(about = "High-performance log generator for stress testing tinywatcher", long_about = None)]
struct Args {
    /// Logs per second
    #[arg(short, long, default_value = "100")]
    rate: u64,

    /// Duration in seconds (0 = infinite)
    #[arg(short, long, default_value = "0")]
    duration: u64,

    /// Output file path (defaults to stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Log format: text, json, apache, nginx
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Error rate (0.0 - 1.0)
    #[arg(short, long, default_value = "0.01")]
    error_rate: f64,

    /// Enable burst mode
    #[arg(short, long)]
    burst: bool,

    /// Burst interval in seconds
    #[arg(long, default_value = "60")]
    burst_interval: u64,

    /// Burst multiplier
    #[arg(long, default_value = "10")]
    burst_multiplier: u64,

    /// Show statistics
    #[arg(short, long)]
    stats: bool,

    /// Line size: short, medium, long, variable, xl
    #[arg(short, long, default_value = "medium")]
    line_size: String,

    /// Batch size for writes (higher = better performance, more memory)
    #[arg(long, default_value = "100")]
    batch_size: usize,

    /// Complex patterns for regex testing (stack traces, SQL, etc)
    #[arg(long)]
    complex_patterns: bool,
}

struct LogGenerator {
    format: String,
    error_rate: f64,
    counter: AtomicU64,
    line_size: String,
    complex_patterns: bool,
}

impl LogGenerator {
    fn new(format: String, error_rate: f64, line_size: String, complex_patterns: bool) -> Self {
        Self {
            format,
            error_rate,
            counter: AtomicU64::new(0),
            line_size,
            complex_patterns,
        }
    }

    fn generate_log(&self) -> String {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        let mut rng = rand::thread_rng();
        let timestamp = Local::now();
        
        let is_error = rng.gen::<f64>() < self.error_rate;
        let level = if is_error {
            if rng.gen::<f64>() < 0.2 {
                "CRITICAL"
            } else {
                "ERROR"
            }
        } else {
            match rng.gen_range(0..10) {
                0..=6 => "INFO",
                7..=8 => "WARN",
                _ => "DEBUG",
            }
        };

        let components = [
            "auth-service", "api-gateway", "database", "cache", 
            "worker", "scheduler", "notifier", "analytics"
        ];
        let component = components[rng.gen_range(0..components.len())];

        let (message, extra_data) = if self.complex_patterns {
            self.generate_complex_message(is_error, &mut rng)
        } else {
            let messages = if is_error {
                vec![
                    "Connection timeout to upstream service",
                    "Database query failed: connection pool exhausted",
                    "Authentication failed for user",
                    "Rate limit exceeded",
                    "Invalid request payload",
                    "Service unavailable",
                    "Memory allocation failed",
                    "Disk space critical",
                    "Network unreachable",
                    "Internal server error",
                ]
            } else {
                vec![
                    "Request processed successfully",
                    "User logged in",
                    "Cache hit",
                    "Task completed",
                    "Health check passed",
                    "Configuration reloaded",
                    "Scheduled job executed",
                    "Session created",
                    "Data synchronized",
                    "Metrics published",
                ]
            };
            (messages[rng.gen_range(0..messages.len())].to_string(), String::new())
        };
        
        let padding = self.generate_padding(&mut rng);

        let request_id = format!("req-{:016x}", rng.gen::<u64>());
        let duration_ms = rng.gen_range(1..500);
        let user_id = rng.gen_range(1000..9999);

        match self.format.as_str() {
            "json" => {
                serde_json::json!({
                    "timestamp": timestamp.to_rfc3339(),
                    "level": level,
                    "component": component,
                    "message": message,
                    "request_id": request_id,
                    "duration_ms": duration_ms,
                    "user_id": user_id,
                    "count": count,
                }).to_string()
            }
            "apache" => {
                let status = if is_error { 500 } else { 200 };
                let bytes = rng.gen_range(100..5000);
                format!(
                    "127.0.0.1 - user{} [{}] \"GET /api/{} HTTP/1.1\" {} {} \"-\" \"LogStorm/1.0\"",
                    user_id,
                    timestamp.format("%d/%b/%Y:%H:%M:%S %z"),
                    component,
                    status,
                    bytes
                )
            }
            "nginx" => {
                let status = if is_error { 502 } else { 200 };
                let bytes = rng.gen_range(100..5000);
                format!(
                    "{} - {} [{}] \"GET /api/{} HTTP/1.1\" {} {} \"-\" \"LogStorm/1.0\" rt={:.3}",
                    "127.0.0.1",
                    user_id,
                    timestamp.format("%d/%b/%Y:%H:%M:%S %z"),
                    component,
                    status,
                    bytes,
                    duration_ms as f64 / 1000.0
                )
            }
            _ => {
                let base = format!(
                    "{} [{}] {}: {} (request_id={}, duration={}ms, user={}, count={})",
                    timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                    level,
                    component,
                    message,
                    request_id,
                    duration_ms,
                    user_id,
                    count
                );
                if !extra_data.is_empty() {
                    format!("{} {}{}", base, extra_data, padding)
                } else {
                    format!("{}{}", base, padding)
                }
            }
        }
    }

    fn generate_complex_message(&self, is_error: bool, rng: &mut impl Rng) -> (String, String) {
        if is_error {
            let error_types = vec![
                ("NullPointerException", self.generate_stack_trace(rng)),
                ("SQLException", format!("Query failed: {}", self.generate_sql_query(rng))),
                ("TimeoutException", format!("Timeout after {}ms connecting to {}", 
                    rng.gen_range(5000..30000), self.generate_url(rng))),
                ("OutOfMemoryError", format!("Heap space exhausted: {}/{}MB used", 
                    rng.gen_range(1800..2048), 2048)),
                ("ConnectionRefusedException", format!("Failed to connect to {}:{}", 
                    self.generate_ip(rng), rng.gen_range(3000..9000))),
            ];
            let (msg, extra) = &error_types[rng.gen_range(0..error_types.len())];
            (msg.to_string(), extra.clone())
        } else {
            let info_types = vec![
                ("Request completed", format!(" endpoint={}", self.generate_url(rng))),
                ("Database query executed", format!(" query={}", self.generate_sql_query(rng))),
                ("API call successful", format!(" url={}", self.generate_url(rng))),
                ("Cache operation", format!(" key={}", self.generate_cache_key(rng))),
                ("User activity", format!(" ip={} agent={}", self.generate_ip(rng), self.generate_user_agent(rng))),
            ];
            let (msg, extra) = &info_types[rng.gen_range(0..info_types.len())];
            (msg.to_string(), extra.clone())
        }
    }

    fn generate_stack_trace(&self, rng: &mut impl Rng) -> String {
        let classes = ["UserService", "DatabaseConnection", "ApiController", "AuthManager"];
        let methods = ["process", "execute", "handle", "validate", "connect"];
        format!(
            "\n  at com.example.{}.{}({}:{})\n  at com.example.app.Main.main(Main.java:42)",
            classes[rng.gen_range(0..classes.len())],
            methods[rng.gen_range(0..methods.len())],
            classes[rng.gen_range(0..classes.len())],
            rng.gen_range(10..500)
        )
    }

    fn generate_sql_query(&self, rng: &mut impl Rng) -> String {
        let tables = ["users", "orders", "products", "sessions", "logs"];
        let table = tables[rng.gen_range(0..tables.len())];
        format!("SELECT * FROM {} WHERE id = {} LIMIT 100", table, rng.gen_range(1..10000))
    }

    fn generate_url(&self, rng: &mut impl Rng) -> String {
        let paths = ["/api/v1/users", "/api/v2/orders", "/health", "/metrics", "/api/products"];
        format!("https://api.example.com{}/{}", 
            paths[rng.gen_range(0..paths.len())], 
            rng.gen_range(1..1000))
    }

    fn generate_ip(&self, rng: &mut impl Rng) -> String {
        format!("{}.{}.{}.{}", 
            rng.gen_range(1..255), 
            rng.gen_range(0..255), 
            rng.gen_range(0..255), 
            rng.gen_range(1..255))
    }

    fn generate_cache_key(&self, rng: &mut impl Rng) -> String {
        format!("cache:user:{}:session:{}", 
            rng.gen_range(1000..9999), 
            format!("{:016x}", rng.gen::<u64>()))
    }

    fn generate_user_agent(&self, rng: &mut impl Rng) -> String {
        let agents = [
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
            "curl/7.68.0",
            "python-requests/2.28.0",
        ];
        agents[rng.gen_range(0..agents.len())].to_string()
    }

    fn generate_padding(&self, rng: &mut impl Rng) -> String {
        match self.line_size.as_str() {
            "short" => String::new(),
            "medium" => {
                if rng.gen_bool(0.3) {
                    format!(" metadata={}", "x".repeat(rng.gen_range(20..50)))
                } else {
                    String::new()
                }
            }
            "long" => {
                format!(" additional_context={} trace_id={} span_id={}", 
                    "x".repeat(rng.gen_range(100..200)),
                    format!("{:032x}", rng.gen::<u128>()),
                    format!("{:016x}", rng.gen::<u64>()))
            }
            "xl" => {
                let json_data = format!("{{\"nested\":{{\"data\":\"{}\"}},\"array\":[{}],\"timestamp\":{}}}",
                    "x".repeat(rng.gen_range(200..400)),
                    (0..rng.gen_range(5..15)).map(|_| rng.gen_range(1..100).to_string()).collect::<Vec<_>>().join(","),
                    rng.gen_range(1000000000..2000000000));
                format!(" payload={}", json_data)
            }
            "variable" => {
                let size_type = rng.gen_range(0..4);
                match size_type {
                    0 => String::new(),
                    1 => format!(" data={}", "x".repeat(rng.gen_range(10..100))),
                    2 => format!(" context={}", "x".repeat(rng.gen_range(100..300))),
                    _ => format!(" large_payload={}", "x".repeat(rng.gen_range(300..800))),
                }
            }
            _ => String::new(),
        }
    }
}

async fn write_logs(
    generator: Arc<LogGenerator>,
    output: Option<String>,
    rate: u64,
    running: Arc<AtomicBool>,
    stats_counter: Arc<AtomicU64>,
    batch_size: usize,
) -> Result<()> {
    // For high throughput, batch operations
    let batch_interval = if rate > 1000 {
        Duration::from_micros((1_000_000 * batch_size as u64) / rate)
    } else {
        Duration::from_micros(1_000_000 / rate)
    };
    
    let mut ticker = interval(batch_interval);
    let logs_per_tick = if rate > 1000 { batch_size } else { 1 };

    match output {
        Some(path) => {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;
            let mut writer = io::BufWriter::with_capacity(256 * 1024, file); // 256KB buffer

            let mut flush_counter = 0;
            let flush_interval = if rate > 10000 { 500 } else { 100 };

            while running.load(Ordering::SeqCst) {
                ticker.tick().await;
                
                // Generate and write batch
                for _ in 0..logs_per_tick {
                    let log = generator.generate_log();
                    writeln!(writer, "{}", log)?;
                    stats_counter.fetch_add(1, Ordering::SeqCst);
                }
                
                // Only flush periodically for high throughput
                flush_counter += logs_per_tick;
                if flush_counter >= flush_interval {
                    writer.flush()?;
                    flush_counter = 0;
                }
            }
            
            writer.flush()?;
        }
        None => {
            // For stdout, still batch but flush more frequently
            while running.load(Ordering::SeqCst) {
                ticker.tick().await;
                
                for _ in 0..logs_per_tick {
                    let log = generator.generate_log();
                    println!("{}", log);
                    stats_counter.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    }

    Ok(())
}

async fn stats_reporter(
    stats_counter: Arc<AtomicU64>,
    running: Arc<AtomicBool>,
    show_stats: bool,
) {
    if !show_stats {
        return;
    }

    let mut last_count = 0u64;
    let mut ticker = interval(Duration::from_secs(1));

    while running.load(Ordering::SeqCst) {
        ticker.tick().await;
        let current_count = stats_counter.load(Ordering::SeqCst);
        let rate = current_count - last_count;
        eprintln!("[STATS] Total: {} | Rate: {} logs/sec", current_count, rate);
        last_count = current_count;
    }

    let final_count = stats_counter.load(Ordering::SeqCst);
    eprintln!("[STATS] Final total: {} logs generated", final_count);
}

async fn burst_controller(
    current_rate: Arc<AtomicU64>,
    base_rate: u64,
    burst_interval: u64,
    burst_multiplier: u64,
    running: Arc<AtomicBool>,
) {
    let mut ticker = interval(Duration::from_secs(burst_interval));
    let burst_duration = Duration::from_secs(5);

    while running.load(Ordering::SeqCst) {
        ticker.tick().await;
        
        eprintln!("[BURST] Starting burst mode: {}x traffic", burst_multiplier);
        current_rate.store(base_rate * burst_multiplier, Ordering::SeqCst);
        
        sleep(burst_duration).await;
        
        eprintln!("[BURST] Returning to normal rate");
        current_rate.store(base_rate, Ordering::SeqCst);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let generator = Arc::new(LogGenerator::new(
        args.format.clone(),
        args.error_rate,
        args.line_size.clone(),
        args.complex_patterns,
    ));

    let running = Arc::new(AtomicBool::new(true));
    let stats_counter = Arc::new(AtomicU64::new(0));
    let current_rate = Arc::new(AtomicU64::new(args.rate));

    // Setup Ctrl+C handler
    let running_clone = running.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        eprintln!("\n[INFO] Shutting down gracefully...");
        running_clone.store(false, Ordering::SeqCst);
    });

    // Duration handler
    if args.duration > 0 {
        let running_clone = running.clone();
        let duration = args.duration;
        tokio::spawn(async move {
            sleep(Duration::from_secs(duration)).await;
            eprintln!("\n[INFO] Duration elapsed, shutting down...");
            running_clone.store(false, Ordering::SeqCst);
        });
    }

    // Stats reporter
    let stats_handle = tokio::spawn(stats_reporter(
        stats_counter.clone(),
        running.clone(),
        args.stats,
    ));

    // Burst controller
    if args.burst {
        let current_rate_clone = current_rate.clone();
        let running_clone = running.clone();
        tokio::spawn(burst_controller(
            current_rate_clone,
            args.rate,
            args.burst_interval,
            args.burst_multiplier,
            running_clone,
        ));
    }

    eprintln!("[INFO] Starting logstorm...");
    eprintln!("[INFO] Format: {}", args.format);
    eprintln!("[INFO] Base rate: {} logs/sec", args.rate);
    eprintln!("[INFO] Line size: {}", args.line_size);
    eprintln!("[INFO] Batch size: {}", args.batch_size);
    eprintln!("[INFO] Error rate: {:.1}%", args.error_rate * 100.0);
    eprintln!("[INFO] Complex patterns: {}", args.complex_patterns);
    if let Some(ref output) = args.output {
        eprintln!("[INFO] Output: {}", output);
    } else {
        eprintln!("[INFO] Output: stdout");
    }
    if args.burst {
        eprintln!("[INFO] Burst mode: enabled (interval={}s, multiplier={}x)", 
                 args.burst_interval, args.burst_multiplier);
    }

    // Main log writer
    let write_handle = tokio::spawn(write_logs(
        generator,
        args.output,
        args.rate,
        running.clone(),
        stats_counter.clone(),
        args.batch_size,
    ));

    // Wait for completion
    write_handle.await??;
    stats_handle.await?;

    eprintln!("[INFO] Shutdown complete");
    Ok(())
}
