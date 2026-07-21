//! Common utility functions for labwc-rs.
//!
//! This crate provides string helpers, process spawning, file descriptor
//! utilities, and search-path resolution — equivalent to labwc's `src/common/`.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tracing::debug;

/// Spawn a process asynchronously without a shell.
///
/// The command string is split on whitespace. The first token is the
/// executable and the rest are arguments.
///
/// # Note
///
/// Only run commands from trusted configuration sources.
pub fn spawn_async_no_shell(cmd: &str) -> std::io::Result<i32> {
    debug!("spawn_async_no_shell: {}", cmd);
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty command",
        ));
    }
    let child = Command::new(parts[0])
        .args(&parts[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(child.id() as i32)
}

/// Check if a string is null, empty, or whitespace-only.
#[must_use]
pub fn string_is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

/// Compare two strings for equality, handling null-equivalent values.
#[must_use]
pub fn string_equal(a: &str, b: &str) -> bool {
    a == b
}

/// Duplicate a string, returning an empty string for None.
#[must_use]
pub fn xstrdup(s: Option<&str>) -> String {
    s.unwrap_or("").to_string()
}

/// Replace the value behind a mutable `Option<String>` reference.
pub fn xstrdup_replace(dst: &mut String, src: &str) {
    *dst = src.to_string();
}

/// Increase the process file descriptor limit.
pub fn increase_nofile_limit() {
    #[cfg(unix)]
    {
        let mut rlim = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: getrlimit is a standard POSIX syscall with well-defined behavior
        let ret = unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlim) };
        if ret == 0 {
            rlim.rlim_cur = rlim.rlim_max;
            // SAFETY: setrlimit with valid values is safe
            unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) };
        }
    }
}

/// Find an executable in PATH, or return None.
#[must_use]
pub fn find_executable(name: &str) -> Option<PathBuf> {
    if let Ok(paths) = std::env::var("PATH") {
        for dir in paths.split(':') {
            let full = Path::new(dir).join(name);
            if full.is_file() {
                return Some(full);
            }
        }
    }
    None
}

/// Resolve a path relative to XDG base directories.
#[must_use]
pub fn resolve_config_path(relative: &str) -> Option<PathBuf> {
    if let Some(config_home) = dirs::config_dir() {
        let p = config_home.join(relative);
        if p.exists() {
            return Some(p);
        }
    }

    // Search XDG_CONFIG_DIRS
    let config_dirs = std::env::var("XDG_CONFIG_DIRS").unwrap_or_else(|_| "/etc/xdg".to_string());
    for dir in config_dirs.split(':') {
        let p = Path::new(dir).join(relative);
        if p.exists() {
            return Some(p);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_is_blank() {
        assert!(string_is_blank(""));
        assert!(string_is_blank("   "));
        assert!(!string_is_blank("hello"));
    }

    #[test]
    fn test_string_equal() {
        assert!(string_equal("hello", "hello"));
        assert!(!string_equal("hello", "world"));
    }

    #[test]
    fn test_xstrdup() {
        assert_eq!(xstrdup(Some("test")), "test");
        assert_eq!(xstrdup(None), "");
    }
}
