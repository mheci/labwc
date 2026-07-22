pub mod autostart;
pub mod entry;

pub use autostart::{AutostartEntry, AutostartManager, AutostartSource};
pub use entry::{DesktopAction, DesktopEntry, EntryType};

use tracing::info;

/// Scan all application directories for `.desktop` files.
///
/// Searches:
/// - `$XDG_DATA_HOME/applications/`
/// - `$XDG_DATA_DIRS/applications/`
/// - `/usr/share/applications/`
/// - `/usr/local/share/applications/`
///
/// Returns entries sorted by: user locale match, then by category priority, then by name.
pub fn scan_applications() -> Vec<DesktopEntry> {
    let mut entries = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let data_home = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.local/share"))
        .join("applications");

    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());

    let mut search_paths = vec![data_home];
    for dir in data_dirs.split(':') {
        search_paths.push(std::path::PathBuf::from(dir).join("applications"));
    }

    for dir in &search_paths {
        if !dir.exists() {
            continue;
        }
        let dir_entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in dir_entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e != "desktop").unwrap_or(true) {
                continue;
            }

            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if seen.contains(&file_name) {
                continue;
            }

            if let Some(desktop_entry) = DesktopEntry::parse(&path) {
                if !desktop_entry.should_show_in("labwc-rs") {
                    continue;
                }
                seen.insert(file_name);
                entries.push(desktop_entry);
            }
        }
    }

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    info!("Scanned applications: {} entries", entries.len());
    entries
}
