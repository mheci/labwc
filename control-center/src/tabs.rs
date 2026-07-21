use crate::AppState;
use eframe::egui::{self, Color32, RichText, Slider};

fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(title).size(14.0).strong());
    ui.separator();
}

fn row(ui: &mut egui::Ui, label: &str, desc: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(label).strong());
            ui.label(RichText::new(desc).size(10.0).color(Color32::GRAY));
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            add(ui);
        });
    });
}

fn combo(ui: &mut egui::Ui, id: &str, selected: &mut usize, items: &[&str]) {
    egui::ComboBox::from_id_source(id)
        .selected_text(items[*selected])
        .show_ui(ui, |ui| {
            for (i, name) in items.iter().enumerate() {
                if ui.selectable_label(*selected == i, *name).clicked() {
                    *selected = i;
                }
            }
        });
}

fn hex_color(hex: &str) -> Color32 {
    let h = hex.trim_start_matches('#');
    if h.len() >= 6 {
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
        Color32::from_rgb(r, g, b)
    } else {
        Color32::GRAY
    }
}

pub fn general(ui: &mut egui::Ui, s: &mut AppState) {
    section(ui, "General");
    row(ui, "Gap", "Space between windows", |ui| {
        ui.add(Slider::new(&mut s.gap, 0..=50).text("px"));
    });
    row(ui, "Theme", "Current theme", |ui| {
        ui.text_edit_singleline(&mut s.theme_name);
    });
    combo(
        ui,
        "placement",
        &mut s.placement_idx,
        &["Center", "Cursor", "Automatic", "Cascade"],
    );
    row(ui, "Primary selection", "Middle-click paste", |ui| {
        ui.checkbox(&mut s.primary_selection, "");
    });
    row(ui, "Magnifier scale", "Zoom level", |ui| {
        ui.add(Slider::new(&mut s.magnifier_scale, 1.0..=5.0));
    });
    ui.add_space(8.0);
    section(ui, "NVIDIA");
    row(ui, "Explicit sync", "linux-drm-syncobj", |ui| {
        ui.checkbox(&mut s.nvidia_sync, "");
    });
    row(ui, "DMA-BUF", "Buffer sharing", |ui| {
        ui.checkbox(&mut s.nvidia_dmabuf, "");
    });
}

pub fn appearance(ui: &mut egui::Ui, s: &mut AppState) {
    section(ui, "Colors");
    let colors: Vec<(&str, &mut String)> = vec![
        ("Active border", &mut s.active_border),
        ("Inactive border", &mut s.inactive_border),
        ("Active title bg", &mut s.active_title_bg),
        ("Inactive title bg", &mut s.inactive_title_bg),
        ("Active title fg", &mut s.active_title_fg),
        ("Inactive title fg", &mut s.inactive_title_fg),
    ];
    for (label, color_ref) in colors {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.add(
                egui::Button::new("  ")
                    .fill(hex_color(color_ref))
                    .min_size(egui::vec2(24.0, 18.0))
                    .rounding(3.0),
            );
            ui.text_edit_singleline(color_ref);
        });
    }
    ui.add_space(6.0);
    section(ui, "Layout");
    row(ui, "Border width", "", |ui| {
        ui.add(Slider::new(&mut s.border_width, 0..=10).text("px"));
    });
    row(ui, "Titlebar height", "", |ui| {
        ui.add(Slider::new(&mut s.titlebar_height, 12..=48).text("px"));
    });
    row(ui, "Button order", "C=Close I=Iconify M=Max", |ui| {
        ui.text_edit_singleline(&mut s.button_order);
    });
    combo(
        ui,
        "justify",
        &mut s.title_justify,
        &["Left", "Center", "Right"],
    );
}

pub fn windows(ui: &mut egui::Ui, s: &mut AppState) {
    section(ui, "Focus");
    row(ui, "Raise on focus", "", |ui| {
        ui.checkbox(&mut s.raise_on_focus, "");
    });
    row(ui, "Raise delay", "ms", |ui| {
        let mut v = s.raise_delay;
        ui.add(Slider::new(&mut v, 0..=2000).text("ms"));
        s.raise_delay = v;
    });
    row(ui, "Focus follows mouse", "", |ui| {
        ui.checkbox(&mut s.focus_follow_mouse, "");
    });
    row(ui, "Focus delay", "ms", |ui| {
        let mut v = s.focus_delay;
        ui.add(Slider::new(&mut v, 0..=1000).text("ms"));
        s.focus_delay = v;
    });
    row(ui, "Double click time", "ms", |ui| {
        let mut v = s.double_click_time;
        ui.add(Slider::new(&mut v, 100..=1000).text("ms"));
        s.double_click_time = v;
    });
}

pub fn keyboard(ui: &mut egui::Ui, s: &mut AppState) {
    section(ui, "Keyboard");
    row(ui, "Repeat rate", "chars/sec", |ui| {
        ui.add(Slider::new(&mut s.repeat_rate, 1..=100));
    });
    row(ui, "Repeat delay", "ms", |ui| {
        ui.add(Slider::new(&mut s.repeat_delay, 100..=2000).text("ms"));
    });
    row(ui, "Per-window layout", "", |ui| {
        ui.checkbox(&mut s.kb_per_window, "");
    });
    ui.add_space(8.0);
    ui.label("Keybinds are configured in rc.xml.");
}

pub fn mouse(ui: &mut egui::Ui, _s: &mut AppState) {
    section(ui, "Mouse");
    ui.label("Mouse bindings are configured in rc.xml.");
    ui.label("Supported contexts: Frame, Titlebar, Client, Root, Toplevel");
    ui.label("Supported buttons: Left, Right, Middle, Side, Extra, Scroll");
}

pub fn workspaces(ui: &mut egui::Ui, s: &mut AppState) {
    section(ui, "Workspaces");
    row(ui, "Count", "", |ui| {
        let mut c = s.workspace_count;
        ui.add(Slider::new(&mut c, 1..=32));
        s.workspace_count = c;
    });
    for i in 0..s.workspace_count {
        if i >= s.workspace_names.len() {
            s.workspace_names.push(format!("{}", i + 1));
        }
        ui.horizontal(|ui| {
            ui.label(format!("WS {}:", i + 1));
            ui.text_edit_singleline(&mut s.workspace_names[i]);
        });
    }
}

pub fn decoration(ui: &mut egui::Ui, s: &mut AppState) {
    section(ui, "Server-Side Decoration");
    row(ui, "XDG shell SSD", "", |ui| {
        ui.checkbox(&mut s.xdg_shell_deco, "");
    });
    row(ui, "Keep border", "When toggling deco", |ui| {
        ui.checkbox(&mut s.ssd_keep_border, "");
    });
    row(ui, "Hide titlebar if max", "", |ui| {
        ui.checkbox(&mut s.hide_maximized_titlebar, "");
    });
    ui.add_space(6.0);
    section(ui, "Preview");
    let border_c = hex_color(&s.active_border.clone());
    let title_c = hex_color(&s.active_title_bg.clone());
    let fg_c = hex_color(&s.active_title_fg.clone());
    let frame = egui::Frame::none()
        .fill(title_c)
        .stroke(egui::Stroke::new(s.border_width as f32, border_c))
        .inner_margin(4.0)
        .rounding(4.0);
    frame.show(ui, |ui| {
        ui.set_min_width(280.0);
        ui.set_min_height(36.0);
        ui.horizontal(|ui| {
            ui.colored_label(border_c, "● □ ×");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.colored_label(fg_c, "Window Preview");
            });
        });
    });
}

pub fn snap(ui: &mut egui::Ui, s: &mut AppState) {
    section(ui, "Snapping");
    row(ui, "Top maximize", "Drag to top = maximize", |ui| {
        ui.checkbox(&mut s.snap_top_maximize, "");
    });
    combo(
        ui,
        "tiling",
        &mut s.tiling_mode,
        &["Never", "Region only", "Edge only", "Always"],
    );
    row(ui, "Resize indicator", "Show dimensions", |ui| {
        ui.checkbox(&mut s.resize_indicator, "");
    });
}

pub fn protocols(ui: &mut egui::Ui, s: &mut AppState) {
    section(ui, "Wayland Protocols");
    let protos: Vec<(&str, &mut bool)> = vec![
        ("Content type manager", &mut s.proto_content_type),
        ("Toplevel drag", &mut s.proto_toplevel_drag),
        ("FIFO presentation", &mut s.proto_fifo),
        ("Pointer warp", &mut s.proto_pointer_warp),
        ("Toplevel tagging", &mut s.proto_toplevel_tag),
        ("Image capture", &mut s.proto_image_capture),
        ("Commit timing", &mut s.proto_commit_timing),
    ];
    for (name, val) in protos {
        ui.checkbox(val, name);
    }
}

pub fn panel(ui: &mut egui::Ui, s: &mut AppState) {
    section(ui, "Desktop Panel");
    row(ui, "Height", "", |ui| {
        ui.add(Slider::new(&mut s.panel_height, 20..=64).text("px"));
    });
    combo(
        ui,
        "panel_pos",
        &mut s.panel_position,
        &["Bottom", "Top", "Left", "Right"],
    );
    row(ui, "Auto-hide", "", |ui| {
        ui.checkbox(&mut s.panel_auto_hide, "");
    });
    row(ui, "Transparency", "", |ui| {
        ui.add(Slider::new(&mut s.panel_transparency, 0.0..=1.0));
    });
    row(ui, "Blur", "", |ui| {
        ui.checkbox(&mut s.panel_blur, "");
    });
    row(ui, "Multi-monitor", "", |ui| {
        ui.checkbox(&mut s.panel_multi, "");
    });
    ui.add_space(6.0);
    section(ui, "Modules");
    ui.label("✓ Launcher  ✓ Taskbar  ✓ Workspaces  ✓ Tray  ✓ Clock  ✓ Audio  ✓ Power");
}

pub fn about(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(40.0);
        ui.heading("labwc-rs Control Center");
        ui.label("v0.21.0");
        ui.add_space(12.0);
        ui.label("26 crates · 5,000+ lines Rust");
        ui.label("7 Wayland protocol extensions");
        ui.label("Integrated desktop panel");
        ui.label("NVIDIA optimized · Hot reload");
        ui.add_space(8.0);
        ui.separator();
        ui.label("github.com/mheci/labwc  ·  GPL-2.0");
    });
}
