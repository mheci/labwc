//! XDG Autostart specification — freedesktop.org Autostart Specification 0.5.
//!
//! Scans `$XDG_CONFIG_DIRS/autostart/` for `.desktop` files and manages
//! autostart lifecycle: automatically launched at session start,
//! conditionally launched based on `OnlyShowIn`/`NotShowIn`, and
//! toggleable via the control center.

use crate::entry::{DesktopEntry, EntryType};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct AutostartEntry {
    pub entry: DesktopEntry,
    pub source: AutostartSource,
    pub enabled: bool,
    pub pid: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutostartSource {
    UserConfig,
    UserAutostart,
    SystemAutostart,
    SystemEtc,
}

impl AutostartSource {
    pub fn priority(&self) -> u8 {
        match self {
            Self::UserConfig => 0,
            Self::UserAutostart => 1,
            Self::SystemAutostart => 2,
            Self::SystemEtc => 3,
        }
    }
}

#[derive(Debug)]
pub struct AutostartManager {
    pub entries: Vec<AutostartEntry>,
    pub desktop_name: String,
    pub launched: Vec<i32>,
}

impl AutostartManager {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            desktop_name: "labwc-rs".to_string(),
            launched: Vec::new(),
        }
    }

    pub fn scan(&mut self) {
        self.entries.clear();

        let xdg_config_home = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));

        let xdg_config_dirs =
            std::env::var("XDG_CONFIG_DIRS").unwrap_or_else(|_| "/etc/xdg".to_string());

        let mut seen: HashMap<String, Vec<AutostartSource>> = HashMap::new();

        // Priority order: user config > user autostart > system autostart > /etc/xdg
        let mut search_paths = vec![
            (
                xdg_config_home.join("labwc/autostart"),
                AutostartSource::UserConfig,
            ),
            (
                xdg_config_home.join("autostart"),
                AutostartSource::UserAutostart,
            ),
        ];

        for sysdir in xdg_config_dirs.split(':') {
            let sys_path = PathBuf::from(sysdir);
            search_paths.push((sys_path.join("autostart"), AutostartSource::SystemAutostart));
        }

        search_paths.push((
            PathBuf::from("/etc/xdg/autostart"),
            AutostartSource::SystemEtc,
        ));

        for (dir, source) in &search_paths {
            if !dir.exists() {
                continue;
            }

            let entries = match std::fs::read_dir(dir) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e != "desktop").unwrap_or(true) {
                    continue;
                }

                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                // If already seen from a higher-priority source, skip
                if seen.contains_key(&file_name) {
                    continue;
                }

                match DesktopEntry::parse(&path) {
                    Some(desktop_entry) => {
                        if desktop_entry.entry_type != EntryType::Application {
                            continue;
                        }

                        if !desktop_entry.should_show_in(&self.desktop_name) {
                            debug!("Autostart {}: skipped (OnlyShowIn/NotShowIn)", file_name);
                            continue;
                        }

                        if desktop_entry.hidden {
                            debug!("Autostart {}: skipped (Hidden=true)", file_name);
                            continue;
                        }

                        let enabled = !desktop_entry.no_display;

                        seen.insert(file_name, vec![*source]);

                        debug!(
                            "Autostart {}: enabled={} source={:?}",
                            desktop_entry.name, enabled, source
                        );

                        self.entries.push(AutostartEntry {
                            entry: desktop_entry,
                            source: *source,
                            enabled,
                            pid: None,
                        });
                    }
                    None => {
                        debug!("Autostart {}: failed to parse", path.display());
                    }
                }
            }
        }

        // Sort: higher-priority sources first, then by name
        self.entries.sort_by(|a, b| {
            a.source
                .priority()
                .cmp(&b.source.priority())
                .then(a.entry.name.cmp(&b.entry.name))
        });

        info!("Scanned autostart: {} entries", self.entries.len());
    }

    pub fn launch_all(&mut self) {
        let count = self.entries.iter().filter(|e| e.enabled).count();
        info!("Launching {} autostart entries", count);

        for entry in &mut self.entries.iter_mut().filter(|e| e.enabled) {
            if !entry.entry.exec_command().is_some() {
                warn!("Autostart {}: no executable", entry.entry.name);
                continue;
            }

            match labwc_util::spawn_async_no_shell(&entry.entry.exec_command().unwrap()) {
                Ok(pid) => {
                    entry.pid = Some(pid);
                    debug!("Autostart {}: launched (pid={})", entry.entry.name, pid);
                    self.launched.push(pid);
                }
                Err(e) => {
                    error!("Autostart {}: failed to launch: {}", entry.entry.name, e);
                }
            }
        }
    }

    pub fn toggle(&mut self, name: &str) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.entry.name == name) {
            entry.enabled = !entry.enabled;
            debug!(
                "Autostart {}: {}",
                name,
                if entry.enabled { "enabled" } else { "disabled" }
            );
            true
        } else {
            false
        }
    }

    pub fn enabled_count(&self) -> usize {
        self.entries.iter().filter(|e| e.enabled).count()
    }

    pub fn disabled_count(&self) -> usize {
        self.entries.len() - self.enabled_count()
    }
}

impl Default for AutostartManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_new() {
        let mgr = AutostartManager::new();
        assert_eq!(mgr.entries.len(), 0);
        assert_eq!(mgr.desktop_name, "labwc-rs");
    }

    #[test]
    fn test_toggle() {
        let mut mgr = AutostartManager::new();
        mgr.entries.push(AutostartEntry {
            entry: DesktopEntry::parse_str(
                "[Desktop Entry]\nType=Application\nName=Test\nExec=test\n",
                Path::new("/tmp/test.desktop"),
            )
            .unwrap(),
            source: AutostartSource::UserAutostart,
            enabled: true,
            pid: None,
        });
        assert!(mgr.toggle("Test"));
        assert!(!mgr.entries[0].enabled);
        assert!(mgr.toggle("Test"));
        assert!(mgr.entries[0].enabled);
    }

    #[test]
    fn test_source_priority() {
        assert!(
            AutostartSource::UserConfig.priority() < AutostartSource::SystemAutostart.priority()
        );
    }
}
