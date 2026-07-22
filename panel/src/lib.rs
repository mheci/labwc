use labwc_actions::Action;
use labwc_workspace::WorkspaceManager;
use labwc_xdg_desktop::DesktopEntry;

pub struct Panel {
    pub config: PanelConfig,
    pub state: PanelState,
    pub modules: Vec<PanelModule>,
}

#[derive(Debug, Clone)]
pub struct PanelConfig {
    pub height: i32,
    pub position: PanelPosition,
    pub auto_hide: bool,
    pub auto_hide_delay_ms: u64,
    pub transparency: f32,
    pub blur_enabled: bool,
    pub blur_radius: f32,
    pub multi_monitor: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelPosition {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct PanelState {
    pub visible: bool,
    pub hovered: bool,
    pub active_module: Option<usize>,
    pub clock_visible: bool,
    pub calendar_open: bool,
}

pub enum PanelModule {
    Launcher(LauncherWidget),
    Taskbar(TaskbarWidget),
    WorkspaceIndicator(WorkspaceWidget),
    SystemTray(TrayWidget),
    Clock(ClockWidget),
    PowerMenu(PowerWidget),
    AudioControl(AudioWidget),
    NetworkStatus(NetworkWidget),
    BatteryStatus(BatteryWidget),
    NotificationCenter(NotificationWidget),
    ClipboardHistory(ClipboardWidget),
    SearchLauncher(SearchWidget),
}

pub struct LauncherWidget {
    pub pinned_apps: Vec<PinnedApp>,
    pub recent_apps: Vec<RecentApp>,
    pub search_active: bool,
    pub search_query: String,
    pub search_results: Vec<DesktopEntry>,
    pub all_apps: Vec<DesktopEntry>,
}

#[derive(Debug, Clone)]
pub struct PinnedApp {
    pub name: String,
    pub icon: Option<String>,
    pub exec: Option<String>,
    pub desktop_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RecentApp {
    pub name: String,
    pub exec: Option<String>,
    pub launch_count: u32,
}

pub struct TaskbarWidget {
    pub show_labels: bool,
    pub group_windows: bool,
    pub show_previews: bool,
    pub icon_size: i32,
    pub tasks: Vec<TaskbarEntry>,
}
pub struct TaskbarEntry {
    pub window_id: u64,
    pub title: String,
    pub app_id: String,
    pub workspace: String,
    pub focused: bool,
    pub minimized: bool,
}
pub struct WorkspaceWidget {
    pub show_labels: bool,
    pub compact: bool,
    pub count: usize,
    pub active: usize,
}
pub struct TrayWidget {
    pub icon_size: i32,
    pub items: Vec<TrayItem>,
}
pub struct TrayItem {
    pub id: String,
    pub title: String,
    pub icon_name: String,
    pub category: TrayCategory,
    pub attention: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCategory {
    ApplicationStatus,
    Communications,
    SystemServices,
    Hardware,
    Other,
}
pub struct ClockWidget {
    pub format_24h: bool,
    pub show_seconds: bool,
    pub show_date: bool,
    pub timezone: Option<String>,
}
pub struct PowerWidget {
    pub show_lock: bool,
    pub show_suspend: bool,
    pub show_hibernate: bool,
    pub show_reboot: bool,
    pub show_shutdown: bool,
}
pub struct AudioWidget {
    pub show_volume: bool,
    pub show_mute: bool,
    pub volume: i32,
    pub muted: bool,
}
pub struct NetworkWidget {
    pub show_wifi: bool,
    pub show_ethernet: bool,
    pub connected: bool,
    pub ssid: Option<String>,
    pub signal_strength: u8,
}
pub struct BatteryWidget {
    pub show_percentage: bool,
    pub percentage: u8,
    pub charging: bool,
    pub low_battery: bool,
}
pub struct NotificationWidget {
    pub max_visible: usize,
    pub notifications: Vec<Notification>,
}
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u64,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub urgency: NotificationUrgency,
    pub timestamp: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationUrgency {
    Low,
    Normal,
    Critical,
}
pub struct ClipboardWidget {
    pub max_entries: usize,
    pub entries: Vec<ClipboardEntry>,
}
#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub id: u64,
    pub content_preview: String,
    pub content_type: ClipboardContentType,
    pub timestamp: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardContentType {
    Text,
    Image,
    FileList,
}
pub struct SearchWidget {
    pub history: Vec<String>,
    pub results: Vec<SearchResult>,
    pub query: String,
}
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub description: String,
    pub action: Action,
    pub icon: Option<String>,
}

impl Panel {
    pub fn new(theme: &labwc_theme::Theme) -> Self {
        let all_apps = labwc_xdg_desktop::scan_applications();

        let pinned_apps: Vec<PinnedApp> = all_apps
            .iter()
            .filter(|e| {
                e.categories
                    .iter()
                    .any(|c| c == "TerminalEmulator" || c == "WebBrowser" || c == "FileManager")
            })
            .map(|e| PinnedApp {
                name: e.locale_name.clone(),
                icon: e.icon.clone(),
                exec: e.exec_command(),
                desktop_id: Some(e.path.to_string_lossy().to_string()),
            })
            .collect();

        Self {
            config: PanelConfig {
                height: theme.titlebar_height + 4,
                position: PanelPosition::Bottom,
                auto_hide: false,
                auto_hide_delay_ms: 500,
                transparency: 0.95,
                blur_enabled: true,
                blur_radius: 12.0,
                multi_monitor: true,
            },
            state: PanelState {
                visible: true,
                hovered: false,
                active_module: None,
                clock_visible: true,
                calendar_open: false,
            },
            modules: vec![
                PanelModule::Launcher(LauncherWidget {
                    pinned_apps,
                    recent_apps: vec![],
                    search_active: false,
                    search_query: String::new(),
                    search_results: vec![],
                    all_apps,
                }),
                PanelModule::Taskbar(TaskbarWidget {
                    show_labels: true,
                    group_windows: true,
                    show_previews: true,
                    icon_size: 24,
                    tasks: vec![],
                }),
                PanelModule::WorkspaceIndicator(WorkspaceWidget {
                    show_labels: false,
                    compact: true,
                    count: 4,
                    active: 0,
                }),
                PanelModule::Clock(ClockWidget {
                    format_24h: true,
                    show_seconds: false,
                    show_date: true,
                    timezone: None,
                }),
                PanelModule::AudioControl(AudioWidget {
                    show_volume: true,
                    show_mute: true,
                    volume: 75,
                    muted: false,
                }),
                PanelModule::PowerMenu(PowerWidget {
                    show_lock: true,
                    show_suspend: true,
                    show_reboot: true,
                    show_shutdown: true,
                    show_hibernate: false,
                }),
            ],
        }
    }

    pub fn search_apps(&mut self, query: &str) {
        if let Some(PanelModule::Launcher(ref mut l)) = self
            .modules
            .iter_mut()
            .find(|m| matches!(m, PanelModule::Launcher(_)))
        {
            l.search_active = !query.is_empty();
            l.search_query = query.to_string();
            l.search_results = if query.is_empty() {
                vec![]
            } else {
                l.all_apps
                    .iter()
                    .filter(|e| {
                        e.name.to_lowercase().contains(&query.to_lowercase())
                            || e.generic_name
                                .as_ref()
                                .map(|g| g.to_lowercase().contains(&query.to_lowercase()))
                                .unwrap_or(false)
                            || e.comment
                                .as_ref()
                                .map(|c| c.to_lowercase().contains(&query.to_lowercase()))
                                .unwrap_or(false)
                    })
                    .take(20)
                    .cloned()
                    .collect()
            };
        }
    }

    pub fn refresh_apps(&mut self) {
        if let Some(PanelModule::Launcher(ref mut l)) = self
            .modules
            .iter_mut()
            .find(|m| matches!(m, PanelModule::Launcher(_)))
        {
            l.all_apps = labwc_xdg_desktop::scan_applications();
        }
    }

    pub fn launch_app(&self, name: &str) -> bool {
        if let Some(PanelModule::Launcher(ref l)) = self
            .modules
            .iter()
            .find(|m| matches!(m, PanelModule::Launcher(_)))
        {
            if let Some(entry) = l
                .all_apps
                .iter()
                .find(|e| e.name == name || e.locale_name == name)
            {
                return entry.launch();
            }
        }
        false
    }

    pub fn update_workspaces(&mut self, ws: &WorkspaceManager) {
        for m in &mut self.modules {
            if let PanelModule::WorkspaceIndicator(ref mut w) = m {
                w.count = ws.workspaces.len();
                w.active = ws.current_idx;
            }
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.state.visible = !self.state.visible;
    }
    pub fn handle_clock_click(&mut self) {
        self.state.calendar_open = !self.state.calendar_open;
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new(&labwc_theme::Theme::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_new() {
        let panel = Panel::new(&labwc_theme::Theme::default());
        assert!(panel.modules.len() >= 6);
    }

    #[test]
    fn test_toggle_visibility() {
        let mut panel = Panel::new(&labwc_theme::Theme::default());
        panel.toggle_visibility();
        assert!(!panel.state.visible);
    }

    #[test]
    fn test_search_empty() {
        let mut panel = Panel::new(&labwc_theme::Theme::default());
        panel.search_apps("");
        // Searching with empty query clears search state
        for m in &panel.modules {
            if let PanelModule::Launcher(ref l) = m {
                assert!(!l.search_active);
                assert!(l.search_query.is_empty());
                assert!(l.search_results.is_empty());
            }
        }
    }

    #[test]
    fn test_default() {
        let panel = Panel::default();
        assert!(panel.config.blur_enabled);
    }
}
