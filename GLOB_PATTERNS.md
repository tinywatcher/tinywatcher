# Glob Pattern Support for File Monitoring

## Overview

TinyWatcher now supports glob patterns in file paths, allowing you to monitor multiple files with a single pattern. This is especially useful for rotating logs and monitoring multiple log files without listing each one individually.

## Usage

Simply use standard glob patterns in your `inputs.files` configuration:

```yaml
inputs:
  files:
    - "/var/log/app/*.log"           # All .log files in /var/log/app/
    - "/var/log/services/*/error.log" # error.log from all service directories
    - "/var/log/app-?.log"            # Single-character wildcard
    - "/var/log/app-[12].log"         # Character class matching
```

## Supported Glob Patterns

### Wildcard (`*`)
Matches any number of characters (except path separators):
- `*.log` - All .log files in current directory
- `app-*.log` - All files starting with "app-" and ending with .log
- `/var/log/*/*.log` - All .log files one level deep

### Question Mark (`?`)
Matches exactly one character:
- `app-?.log` - Matches app-1.log, app-2.log, but not app-10.log
- `log????.txt` - Matches files like log2024.txt

### Character Classes (`[...]`)
Matches one character from the set:
- `app-[123].log` - Matches app-1.log, app-2.log, app-3.log
- `[a-z]*.log` - Matches files starting with lowercase letters
- `app-[0-9][0-9].log` - Two-digit numbered files

## Examples

### Example 1: Rotating Logs
Monitor all daily rotating log files:
```yaml
inputs:
  files:
    - "/var/log/myapp/app-*.log"
```
Matches: `app-2024-12-01.log`, `app-2024-12-02.log`, etc.

### Example 2: Multiple Services
Monitor error logs from all microservices:
```yaml
inputs:
  files:
    - "/var/log/services/*/error.log"
```
Matches: `/var/log/services/auth/error.log`, `/var/log/services/api/error.log`, etc.

### Example 3: Mixed Patterns
Combine patterns and specific files:
```yaml
inputs:
  files:
    - "/var/log/nginx/*.log"
    - "/var/log/app/critical.log"
    - "/var/log/services/*/error.log"
```

## How It Works

1. **Pattern Detection**: When loading the config, TinyWatcher checks each file path for glob characters (`*`, `?`, `[`)

2. **Expansion**: Glob patterns are expanded to actual file paths at startup

3. **File Watching**: Each matched file gets its own independent watcher with automatic retry and reconnection

4. **Logging**: TinyWatcher logs how many files each pattern matched:
   ```
   INFO: Glob pattern '/var/log/app/*.log' matched 3 file(s)
   ```

## Important Notes

- **Directories are ignored**: Only actual files are monitored, not directories
- **No matches warning**: If a pattern matches no files, a warning is logged but startup continues
- **Invalid patterns**: Invalid glob syntax will cause an error at startup
- **Static expansion**: Patterns are expanded once at startup. New files created after startup won't be automatically detected (use file watching for dynamic discovery if needed)
- **Performance**: Each matched file creates a separate watcher task, so be mindful of very broad patterns

## Error Handling

### No Matches
```
WARN: Glob pattern '/var/log/app/*.log' matched no files
```
The application continues running. Useful for optional log files.

### Invalid Pattern
```
ERROR: Invalid glob pattern '/var/log/[invalid': Pattern syntax error
```
The application will fail to start. Fix the pattern in your config.

## Testing

Comprehensive unit tests cover:
- ✅ Wildcard patterns (`*`)
- ✅ Question mark patterns (`?`)
- ✅ Character class patterns (`[...]`)
- ✅ Mixed patterns and explicit files
- ✅ No matches scenario
- ✅ Directory filtering
- ✅ Invalid pattern detection

Run tests with:
```bash
cargo test test_expand_file_globs
```

## Implementation Details

- **Crate**: Uses the `glob` crate (v0.3)
- **Location**: `Config::expand_file_globs()` in `src/config.rs`
- **Called from**: `handle_watch()` in `src/main.rs`
- **Time**: O(n) where n is number of files matched by all patterns

## Future Enhancements

Potential future improvements:
- Dynamic file watching (detect new files matching pattern)
- Recursive glob support (`**`)
- Glob pattern support in rule source filters
- Pattern exclusions (e.g., `!*.tmp`)
