//! Configuration system: rc.xml parser, keybinds, mousebinds, window rules.

use labwc_core::{PlacementPolicy, TilingEventsMode};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Debug, Clone, PartialEq)]
pub struct Keybind {
    pub key: String,
    pub modifiers: Vec<String>,
    pub actions: Vec<ActionConfig>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ActionConfig {
    pub name: String,
    pub args: HashMap<String, String>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct WindowRule {
    pub identifier: Option<String>,
    pub title: Option<String>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RcXml {
    pub config_file: Option<PathBuf>,
    pub config_dir: Option<PathBuf>,
    pub merge_config: bool,
    pub theme_name: String,
    pub gap: i32,
    pub placement_policy: PlacementPolicy,
    pub xdg_shell_server_side_deco: bool,
    pub ssd_keep_border: bool,
    pub hide_maximized_window_titlebar: bool,
    pub kb_layout_per_window: bool,
    pub repeat_rate: i32,
    pub repeat_delay: i32,
    pub double_click_time: u32,
    pub raise_on_focus: bool,
    pub raise_on_focus_delay_ms: u32,
    pub focus_follow_mouse: bool,
    pub snap_tiling_events_mode: TilingEventsMode,
    pub snap_top_maximize: bool,
    pub resize_indicator: bool,
    pub primary_selection: bool,
    pub magnifier_scale: f32,
    pub keybinds: Vec<Keybind>,
    pub mousebinds: Vec<Keybind>,
    pub window_rules: Vec<WindowRule>,
    pub libinput_devices: Vec<LibinputDevice>,
}
#[derive(Debug, Clone)]
pub struct LibinputDevice {
    pub category: String,
    pub pointer_speed: f64,
    pub natural_scroll: bool,
    pub tap: bool,
}

impl Default for RcXml {
    fn default() -> Self {
        Self {
            config_file: None,
            config_dir: None,
            merge_config: false,
            theme_name: "default".into(),
            gap: 0,
            placement_policy: PlacementPolicy::Center,
            xdg_shell_server_side_deco: true,
            ssd_keep_border: false,
            hide_maximized_window_titlebar: false,
            kb_layout_per_window: false,
            repeat_rate: 25,
            repeat_delay: 600,
            double_click_time: 400,
            raise_on_focus: false,
            raise_on_focus_delay_ms: 0,
            focus_follow_mouse: false,
            snap_tiling_events_mode: TilingEventsMode::Never,
            snap_top_maximize: true,
            resize_indicator: true,
            primary_selection: true,
            magnifier_scale: 2.0,
            keybinds: Vec::new(),
            mousebinds: Vec::new(),
            window_rules: Vec::new(),
            libinput_devices: Vec::new(),
        }
    }
}

impl RcXml {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&mut self, config_file: Option<&str>, config_dir: Option<&str>) {
        self.config_file = config_file.map(PathBuf::from);
        self.config_dir = config_dir.map(PathBuf::from);
        match self.find_config_path() {
            Some(ref p) => {
                info!("Loading config from {:?}", p);
                let content = std::fs::read_to_string(p).unwrap_or_default();
                self.parse_xml(&content);
            }
            None => info!("No rc.xml found, using defaults"),
        }
    }

    fn find_config_path(&self) -> Option<PathBuf> {
        if let Some(ref f) = self.config_file {
            if f.exists() {
                return Some(f.clone());
            }
        }
        if let Some(ref d) = self.config_dir {
            let p = d.join("rc.xml");
            if p.exists() {
                return Some(p);
            }
        }
        labwc_util::resolve_config_path("labwc/rc.xml")
    }

    fn parse_xml(&mut self, content: &str) {
        let mut reader = Reader::from_str(content);
        reader.trim_text(true);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag == "core" {
                        self.parse_core_values(&mut reader);
                    }
                    if tag == "keyboard" {
                        self.parse_keybinds(&mut reader);
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    error!("XML: {}", e);
                    break;
                }
                _ => {}
            }
            buf.clear();
        }
    }

    fn parse_core_values<R: std::io::BufRead>(&mut self, reader: &mut Reader<R>) {
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::End(ref e)) => {
                    if String::from_utf8_lossy(e.name().as_ref()) == "core" {
                        break;
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag == "gap" {
                        for attr in e.attributes().filter_map(|a| a.ok()) {
                            if attr.key.as_ref() == b"value" {
                                if let Ok(v) = String::from_utf8_lossy(&attr.value).parse() {
                                    self.gap = v;
                                }
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }
    }

    fn parse_keybinds<R: std::io::BufRead>(&mut self, reader: &mut Reader<R>) {
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::End(ref e)) => {
                    if String::from_utf8_lossy(e.name().as_ref()) == "keyboard" {
                        break;
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag == "keybind" {
                        let mut key = String::new();
                        for attr in e.attributes().filter_map(|a| a.ok()) {
                            if attr.key.as_ref() == b"key" {
                                key = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                        if !key.is_empty() {
                            self.keybinds.push(Keybind {
                                key,
                                modifiers: vec![],
                                actions: vec![],
                            });
                        }
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }
    }
}
