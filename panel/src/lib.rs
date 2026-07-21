use labwc_actions::Action;
use labwc_theme::Theme;
use labwc_workspace::WorkspaceManager;

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
    pub pinned_apps: Vec<String>,
    pub recent_apps: Vec<String>,
    pub search_active: bool,
    pub search_query: String,
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
    pub fn new(theme: &Theme) -> Self {
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
                    pinned_apps: vec!["alacritty".into(), "firefox".into()],
                    recent_apps: vec![],
                    search_active: false,
                    search_query: String::new(),
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
        Self::new(&Theme::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use labwc_theme::Theme;

    #[test]
    fn test_panel_new() {
        let panel = Panel::new(&Theme::default());
        assert_eq!(panel.modules.len(), 6);
        assert!(panel.state.visible);
    }

    #[test]
    fn test_panel_toggle() {
        let mut panel = Panel::new(&Theme::default());
        panel.toggle_visibility();
        assert!(!panel.state.visible);
        panel.toggle_visibility();
        assert!(panel.state.visible);
    }

    #[test]
    fn test_panel_workspace_update() {
        let mut panel = Panel::new(&Theme::default());
        let ws = WorkspaceManager::new();
        panel.update_workspaces(&ws);
    }

    #[test]
    fn test_panel_default() {
        let panel = Panel::default();
        assert!(panel.config.blur_enabled);
    }
}
