use anyhow::Result;
use std::path::{Path, PathBuf};

/// Check if the current process is running with elevated privileges
#[cfg(unix)]
pub fn is_elevated() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(windows)]
pub fn is_elevated() -> bool {
    // On Windows, check if running as Administrator
    // This is a simplified check - in production might want more robust detection
    use std::process::Command;
    
    Command::new("net")
        .args(&["session"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if a file requires elevated privileges to read
#[cfg(unix)]
pub fn file_needs_elevation(path: &Path) -> Result<bool> {
    use std::os::unix::fs::MetadataExt;
    
    // Try to check if we can access the file
    if std::fs::metadata(path).is_err() {
        // If we can't even stat it, we might need elevation
        // But it could also be that the file doesn't exist yet
        // For now, we'll be conservative and check the parent directory
        if let Some(parent) = path.parent() {
            if let Ok(metadata) = std::fs::metadata(parent) {
                let uid = metadata.uid();
                let mode = metadata.mode();
                let current_uid = unsafe { libc::getuid() };
                
                // If owned by root and we're not root
                if uid == 0 && current_uid != 0 {
                    // Check if we have read access via group or other
                    let group_read = (mode & 0o040) != 0;
                    let other_read = (mode & 0o004) != 0;
                    
                    // If neither group nor other can read, we need elevation
                    if !group_read && !other_read {
                        return Ok(true);
                    }
                }
            }
        }
        return Ok(false);
    }
    
    let metadata = std::fs::metadata(path)?;
    let uid = metadata.uid();
    let mode = metadata.mode();
    let current_uid = unsafe { libc::getuid() };
    
    // If owned by root and we're not root
    if uid == 0 && current_uid != 0 {
        // Check if we have read access via group or other
        let group_read = (mode & 0o040) != 0;
        let other_read = (mode & 0o004) != 0;
        
        // If neither group nor other can read, we need elevation
        return Ok(!group_read && !other_read);
    }
    
    // If owned by us, check if we can read
    if uid == current_uid {
        let owner_read = (mode & 0o400) != 0;
        return Ok(!owner_read);
    }
    
    // For other users, check group and other permissions
    let group_read = (mode & 0o040) != 0;
    let other_read = (mode & 0o004) != 0;
    
    Ok(!group_read && !other_read)
}

#[cfg(windows)]
pub fn file_needs_elevation(path: &Path) -> Result<bool> {
    // On Windows, try to open the file for reading
    // If it fails with access denied, we might need elevation
    match std::fs::File::open(path) {
        Ok(_) => Ok(false),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File doesn't exist yet, check parent directory
            if let Some(parent) = path.parent() {
                return file_needs_elevation(parent);
            }
            Ok(false)
        }
        Err(_) => Ok(false),
    }
}

/// Check if any of the provided files need elevated privileges
pub fn any_file_needs_elevation<P: AsRef<Path>>(paths: &[P]) -> Result<bool> {
    for path in paths {
        if file_needs_elevation(path.as_ref())? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Get a list of files that need elevated privileges
pub fn get_files_needing_elevation<P: AsRef<Path>>(paths: &[P]) -> Result<Vec<PathBuf>> {
    let mut needs_elevation = Vec::new();
    
    for path in paths {
        if file_needs_elevation(path.as_ref())? {
            needs_elevation.push(path.as_ref().to_path_buf());
        }
    }
    
    Ok(needs_elevation)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[cfg(unix)]
    fn test_is_elevated() {
        // This test will pass differently based on whether we're root
        let elevated = is_elevated();
        let uid = unsafe { libc::geteuid() };
        assert_eq!(elevated, uid == 0);
    }
    
    #[test]
    fn test_file_needs_elevation() {
        // Test with /tmp which should be readable
        let result = file_needs_elevation(Path::new("/tmp"));
        assert!(result.is_ok());
        
        #[cfg(unix)]
        {
            // /var/log/system.log typically needs root on macOS
            // But we can't guarantee it exists
            if Path::new("/var/log/system.log").exists() {
                let needs_root = file_needs_elevation(Path::new("/var/log/system.log"));
                // Result depends on current permissions
                assert!(needs_root.is_ok());
            }
        }
    }
}
