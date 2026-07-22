#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui;

mod tabs;

#[derive(Clone)]
struct AppState {
    gap: i32,
    theme_name: String,
    placement_idx: usize,
    primary_selection: bool,
    magnifier_scale: f32,
    border_width: i32,
    titlebar_height: i32,
    active_border: String,
    inactive_border: String,
    active_title_bg: String,
    inactive_title_bg: String,
    active_title_fg: String,
    inactive_title_fg: String,
    button_order: String,
    title_justify: usize,
    raise_on_focus: bool,
    raise_delay: u32,
    focus_follow_mouse: bool,
    focus_delay: u32,
    double_click_time: u32,
    repeat_rate: i32,
    repeat_delay: i32,
    kb_per_window: bool,
    xdg_shell_deco: bool,
    ssd_keep_border: bool,
    hide_maximized_titlebar: bool,
    snap_top_maximize: bool,
    resize_indicator: bool,
    tiling_mode: usize,
    workspace_count: usize,
    workspace_names: Vec<String>,
    panel_height: i32,
    panel_position: usize,
    panel_auto_hide: bool,
    panel_transparency: f32,
    panel_blur: bool,
    panel_multi: bool,
    proto_content_type: bool,
    proto_toplevel_drag: bool,
    proto_fifo: bool,
    proto_pointer_warp: bool,
    proto_toplevel_tag: bool,
    proto_image_capture: bool,
    proto_commit_timing: bool,
    nvidia_sync: bool,
    nvidia_dmabuf: bool,
    selected_tab: usize,
    status_msg: String,
    dirty: bool,
    autostart_entries: Vec<AutostartItem>,
}

#[derive(Clone)]
struct AutostartItem {
    name: String,
    exec: String,
    enabled: bool,
    source: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            gap: 0,
            theme_name: "default".into(),
            placement_idx: 0,
            primary_selection: true,
            magnifier_scale: 2.0,
            border_width: 1,
            titlebar_height: 26,
            active_border: "#1a1a1a".into(),
            inactive_border: "#555555".into(),
            active_title_bg: "#1a1a1a".into(),
            inactive_title_bg: "#333333".into(),
            active_title_fg: "#ffffff".into(),
            inactive_title_fg: "#999999".into(),
            button_order: "CIM".into(),
            title_justify: 1,
            raise_on_focus: false,
            raise_delay: 0,
            focus_follow_mouse: false,
            focus_delay: 0,
            double_click_time: 400,
            repeat_rate: 25,
            repeat_delay: 600,
            kb_per_window: false,
            xdg_shell_deco: true,
            ssd_keep_border: false,
            hide_maximized_titlebar: false,
            snap_top_maximize: true,
            resize_indicator: true,
            tiling_mode: 0,
            workspace_count: 4,
            workspace_names: vec!["1".into(), "2".into(), "3".into(), "4".into()],
            panel_height: 30,
            panel_position: 0,
            panel_auto_hide: false,
            panel_transparency: 0.95,
            panel_blur: true,
            panel_multi: true,
            proto_content_type: true,
            proto_toplevel_drag: true,
            proto_fifo: true,
            proto_pointer_warp: true,
            proto_toplevel_tag: true,
            proto_image_capture: true,
            proto_commit_timing: true,
            nvidia_sync: true,
            nvidia_dmabuf: true,
            selected_tab: 0,
            status_msg: String::new(),
            dirty: false,
            autostart_entries: vec![
                AutostartItem {
                    name: "PolicyKit Agent".into(),
                    exec: "/usr/lib/polkit-gnome/polkit-gnome-authentication-agent-1".into(),
                    enabled: true,
                    source: "system".into(),
                },
                AutostartItem {
                    name: "PipeWire".into(),
                    exec: "/usr/bin/pipewire".into(),
                    enabled: true,
                    source: "user".into(),
                },
                AutostartItem {
                    name: "WirePlumber".into(),
                    exec: "/usr/bin/wireplumber".into(),
                    enabled: true,
                    source: "user".into(),
                },
                AutostartItem {
                    name: "xdg-desktop-portal".into(),
                    exec: "/usr/lib/xdg-desktop-portal".into(),
                    enabled: true,
                    source: "system".into(),
                },
                AutostartItem {
                    name: "Network Manager Applet".into(),
                    exec: "nm-applet".into(),
                    enabled: true,
                    source: "user".into(),
                },
            ],
        }
    }
}

impl AppState {
    fn apply(&mut self) {
        self.dirty = false;
    }
    fn refresh(&mut self) {
        self.dirty = false;
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "labwc-rs Control Center",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}

#[derive(Default)]
struct App {
    state: AppState,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let tab_names = [
            "General",
            "Appearance",
            "Windows",
            "Keyboard",
            "Mouse",
            "Workspaces",
            "Decoration",
            "Snapping",
            "Protocols",
            "Autostart",
            "Panel",
            "About",
        ];
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("labwc-rs Control Center");
                ui.separator();
                if ui.button("Apply").clicked() {
                    self.state.apply();
                    self.state.status_msg = "Applied".into();
                }
                if ui.button("Reload").clicked() {
                    self.state.refresh();
                    self.state.status_msg = "Reloaded".into();
                }
            });
            ui.horizontal(|ui| {
                for (i, name) in tab_names.iter().enumerate() {
                    if ui
                        .selectable_label(self.state.selected_tab == i, *name)
                        .clicked()
                    {
                        self.state.selected_tab = i;
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| match self.state.selected_tab {
                0 => tabs::general(ui, &mut self.state),
                1 => tabs::appearance(ui, &mut self.state),
                2 => tabs::windows(ui, &mut self.state),
                3 => tabs::keyboard(ui, &mut self.state),
                4 => tabs::mouse(ui, &mut self.state),
                5 => tabs::workspaces(ui, &mut self.state),
                6 => tabs::decoration(ui, &mut self.state),
                7 => tabs::snap(ui, &mut self.state),
                8 => tabs::protocols(ui, &mut self.state),
                9 => tabs::autostart(ui, &mut self.state),
                10 => tabs::panel(ui, &mut self.state),
                11 => tabs::about(ui),
                _ => {}
            });
        });
        if !self.state.status_msg.is_empty() {
            egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
                ui.colored_label(egui::Color32::GREEN, &self.state.status_msg);
            });
            self.state.status_msg.clear();
        }
        ctx.request_repaint();
    }
}
