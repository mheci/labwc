#![allow(unused_imports)]
//! XDG Desktop Entry parser — full freedesktop.org Desktop Entry Specification.
//!
//! Parses `.desktop` files and provides typed access to all standard keys:
//! Name, Exec, Icon, Type, Categories, Terminal, StartupWMClass, etc.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct DesktopEntry {
    pub path: PathBuf,
    pub name: String,
    pub generic_name: Option<String>,
    pub comment: Option<String>,
    pub icon: Option<String>,
    pub exec: Option<String>,
    pub try_exec: Option<String>,
    pub working_dir: Option<String>,
    pub entry_type: EntryType,
    pub categories: Vec<String>,
    pub mime_types: Vec<String>,
    pub terminal: bool,
    pub no_display: bool,
    pub hidden: bool,
    pub startup_wm_class: Option<String>,
    pub startup_notify: bool,
    pub only_show_in: Vec<String>,
    pub not_show_in: Vec<String>,
    pub dbus_activatable: bool,
    pub prefers_non_default_gpu: bool,
    pub single_main_window: bool,
    pub actions: Vec<DesktopAction>,
    pub raw: HashMap<String, String>,
    pub locale_name: String,
}

#[derive(Debug, Clone)]
pub struct DesktopAction {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub exec: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Application,
    Link,
    Directory,
    Unknown,
}

impl EntryType {
    pub fn parse_type(s: &str) -> Self {
        match s {
            "Application" => Self::Application,
            "Link" => Self::Link,
            "Directory" => Self::Directory,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Application => "Application",
            Self::Link => "Link",
            Self::Directory => "Directory",
            Self::Unknown => "Unknown",
        }
    }
}

impl DesktopEntry {
    pub fn parse(path: &std::path::Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        Self::parse_str(&content, path)
    }

    pub fn parse_str(content: &str, path: &std::path::Path) -> Option<Self> {
        let mut entry = DesktopEntry {
            path: path.to_path_buf(),
            name: String::new(),
            generic_name: None,
            comment: None,
            icon: None,
            exec: None,
            try_exec: None,
            working_dir: None,
            entry_type: EntryType::Application,
            categories: Vec::new(),
            mime_types: Vec::new(),
            terminal: false,
            no_display: false,
            hidden: false,
            startup_wm_class: None,
            startup_notify: true,
            only_show_in: Vec::new(),
            not_show_in: Vec::new(),
            dbus_activatable: false,
            prefers_non_default_gpu: false,
            single_main_window: false,
            actions: Vec::new(),
            raw: HashMap::new(),
            locale_name: String::new(),
        };

        let mut current_group = "";
        let mut in_desktop_entry = false;
        let mut current_action_id = String::new();
        let mut current_action_name = String::new();
        let mut current_action_icon = None;
        let mut current_action_exec = None;

        let locales = Self::user_locales();
        let mut best_name = String::new();
        let mut best_name_prio = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                let group = &line[1..line.len() - 1];
                if group == "Desktop Entry" {
                    in_desktop_entry = true;
                    current_group = "Desktop Entry";
                } else {
                    in_desktop_entry = false;
                    current_group = group;
                    if group.starts_with("Desktop Action ") {
                        current_action_id = group
                            .strip_prefix("Desktop Action ")
                            .unwrap_or(group)
                            .to_string();
                        current_action_name = current_action_id.clone();
                        current_action_icon = None;
                        current_action_exec = None;
                    }
                }
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                let section = if in_desktop_entry { "" } else { current_group };

                if section.is_empty() {
                    match key {
                        "Type" => entry.entry_type = EntryType::parse_type(value),
                        "Name" => {
                            entry.name = value.to_string();
                            best_name = value.to_string();
                            best_name_prio = 100;
                        }
                        "GenericName" => entry.generic_name = Some(value.to_string()),
                        "Comment" => entry.comment = Some(value.to_string()),
                        "Icon" => entry.icon = Some(value.to_string()),
                        "Exec" => entry.exec = Some(value.to_string()),
                        "TryExec" => entry.try_exec = Some(value.to_string()),
                        "Path" => entry.working_dir = Some(value.to_string()),
                        "Categories" => {
                            entry.categories = value
                                .split(';')
                                .filter(|s| !s.is_empty())
                                .map(|s| s.to_string())
                                .collect();
                        }
                        "MimeType" => {
                            entry.mime_types = value
                                .split(';')
                                .filter(|s| !s.is_empty())
                                .map(|s| s.to_string())
                                .collect();
                        }
                        "Terminal" => entry.terminal = value == "true",
                        "NoDisplay" => entry.no_display = value == "true",
                        "Hidden" => entry.hidden = value == "true",
                        "StartupWMClass" => entry.startup_wm_class = Some(value.to_string()),
                        "StartupNotify" => entry.startup_notify = value != "false",
                        "OnlyShowIn" => {
                            entry.only_show_in = value
                                .split(';')
                                .filter(|s| !s.is_empty())
                                .map(|s| s.to_string())
                                .collect();
                        }
                        "NotShowIn" => {
                            entry.not_show_in = value
                                .split(';')
                                .filter(|s| !s.is_empty())
                                .map(|s| s.to_string())
                                .collect();
                        }
                        "DBusActivatable" => entry.dbus_activatable = value == "true",
                        "PrefersNonDefaultGPU" => entry.prefers_non_default_gpu = value == "true",
                        "SingleMainWindow" => entry.single_main_window = value == "true",
                        _ => {
                            // Handle locale-specific Name[lang] keys
                            if key.starts_with("Name[") && key.ends_with(']') {
                                let lang = &key[5..key.len() - 1];
                                for (i, locale) in locales.iter().enumerate() {
                                    if locale.starts_with(lang) {
                                        let prio = 200 - i;
                                        if prio > best_name_prio {
                                            best_name = value.to_string();
                                            best_name_prio = prio;
                                        }
                                        break;
                                    }
                                }
                            }
                            entry.raw.insert(key.to_string(), value.to_string());
                        }
                    }
                } else if current_group.starts_with("Desktop Action ") {
                    match key {
                        "Name" => current_action_name = value.to_string(),
                        "Icon" => current_action_icon = Some(value.to_string()),
                        "Exec" => current_action_exec = Some(value.to_string()),
                        _ => {}
                    }
                }
            }
        }

        if !best_name.is_empty() {
            entry.locale_name = best_name;
        }

        if !current_action_id.is_empty() {
            entry.actions.push(DesktopAction {
                id: current_action_id,
                name: current_action_name,
                icon: current_action_icon,
                exec: current_action_exec,
            });
        }

        if entry.name.is_empty() || entry.entry_type != EntryType::Application {
            return None;
        }

        // Check TryExec
        if let Some(ref try_exec) = entry.try_exec {
            if !Self::find_in_path(try_exec) {
                debug!(
                    "{}: TryExec '{}' not found in PATH — skipping",
                    path.display(),
                    try_exec
                );
                return None;
            }
        }

        if entry.hidden || entry.no_display {
            return None;
        }

        Some(entry)
    }

    fn find_in_path(binary: &str) -> bool {
        if let Ok(paths) = std::env::var("PATH") {
            for dir in paths.split(':') {
                let full = Path::new(dir).join(binary);
                if full.is_file() {
                    return true;
                }
            }
        }
        std::path::Path::new(binary).is_file()
    }

    fn user_locales() -> Vec<String> {
        let mut locales = Vec::new();
        if let Ok(lang) = std::env::var("LANG") {
            locales.push(lang.clone());
            if let Some(short) = lang.split('.').next() {
                if short != lang {
                    locales.push(short.to_string());
                }
            }
            if let Some(primary) = lang.split('_').next() {
                locales.push(primary.to_string());
            }
        }
        locales.push("C".to_string());
        locales
    }

    pub fn exec_command(&self) -> Option<String> {
        self.exec.as_ref().map(|e| Self::substitute_fields(e))
    }

    fn substitute_fields(exec: &str) -> String {
        let result = exec
            .replace("%f", "")
            .replace("%F", "")
            .replace("%u", "")
            .replace("%U", "")
            .replace("%d", "")
            .replace("%D", "")
            .replace("%n", "")
            .replace("%N", "")
            .replace("%i", "")
            .replace("%c", "")
            .replace("%k", "")
            .replace("%v", "")
            .replace("%m", "");
        // Collapse multiple spaces caused by field removal
        let words: Vec<&str> = result.split_whitespace().collect();
        words.join(" ")
    }

    #[allow(clippy::collapsible_if)]
    pub fn should_show_in(&self, desktop: &str) -> bool {
        if !self.only_show_in.is_empty() {
            if !self
                .only_show_in
                .iter()
                .any(|d| d.eq_ignore_ascii_case(desktop))
            {
                return false;
            }
        }
        if self
            .not_show_in
            .iter()
            .any(|d| d.eq_ignore_ascii_case(desktop))
        {
            return false;
        }
        true
    }

    pub fn launch(&self) -> bool {
        if let Some(ref cmd) = self.exec_command() {
            debug!("Launching {}: {}", self.name, cmd);
            if self.terminal {
                // Wrap in terminal
                let term =
                    std::env::var("TERMINAL").unwrap_or_else(|_| "x-terminal-emulator".into());
                let full = format!("{term} -e {cmd}");
                labwc_util::spawn_async_no_shell(&full).is_ok()
            } else {
                labwc_util::spawn_async_no_shell(cmd).is_ok()
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let content = "[Desktop Entry]\nType=Application\nName=Test App\nExec=test-app\n";
        let entry = DesktopEntry::parse_str(content, Path::new("/tmp/test.desktop")).unwrap();
        assert_eq!(entry.name, "Test App");
        assert_eq!(entry.entry_type, EntryType::Application);
    }

    #[test]
    fn test_parse_categories() {
        let content = "[Desktop Entry]\nType=Application\nName=Terminal\nExec=alacritty\nCategories=System;TerminalEmulator;\n";
        let entry = DesktopEntry::parse_str(content, Path::new("/tmp/term.desktop")).unwrap();
        assert_eq!(entry.categories.len(), 2);
        assert!(entry.categories.contains(&"TerminalEmulator".to_string()));
    }

    #[test]
    fn test_try_exec_fail() {
        let content = "[Desktop Entry]\nType=Application\nName=Missing\nExec=nothing\nTryExec=/nonexistent/binary\n";
        assert!(DesktopEntry::parse_str(content, Path::new("/tmp/try.desktop")).is_none());
    }

    #[test]
    fn test_hidden() {
        let content = "[Desktop Entry]\nType=Application\nName=Hidden\nExec=true\nHidden=true\n";
        assert!(DesktopEntry::parse_str(content, Path::new("/tmp/hidden.desktop")).is_none());
    }

    #[test]
    fn test_only_show_in() {
        let content =
            "[Desktop Entry]\nType=Application\nName=GnomeApp\nExec=true\nOnlyShowIn=GNOME;\n";
        let entry = DesktopEntry::parse_str(content, Path::new("/tmp/gnome.desktop")).unwrap();
        assert!(!entry.should_show_in("labwc"));
        assert!(entry.should_show_in("GNOME"));
    }

    #[test]
    fn test_substitute_fields() {
        assert_eq!(DesktopEntry::substitute_fields("app %f"), "app");
        assert_eq!(
            DesktopEntry::substitute_fields("app %F --option"),
            "app --option"
        );
        assert_eq!(DesktopEntry::substitute_fields("app %u file"), "app file");
    }

    #[test]
    fn test_terminal() {
        let content = "[Desktop Entry]\nType=Application\nName=Top\nExec=htop\nTerminal=true\n";
        let entry = DesktopEntry::parse_str(content, Path::new("/tmp/top.desktop")).unwrap();
        assert!(entry.terminal);
    }
}
