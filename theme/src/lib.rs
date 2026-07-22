use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub border_width: i32,
    pub titlebar_height: i32,
    pub padding_width: i32,
    pub button_size: i32,
    pub menu_overlap: i32,
    pub window: WindowTheme,
    pub menu: MenuTheme,
    pub fonts: FontTheme,
    pub colors: ColorTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowTheme {
    pub active_border_color: String,
    pub inactive_border_color: String,
    pub active_title_bg: String,
    pub inactive_title_bg: String,
    pub active_title_fg: String,
    pub inactive_title_fg: String,
    pub active_label_justify: Justify,
    pub button_order: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Justify {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuTheme {
    pub bg_color: String,
    pub fg_color: String,
    pub active_bg: String,
    pub active_fg: String,
    pub separator_color: String,
    pub border_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontTheme {
    pub active_font: String,
    pub inactive_font: String,
    pub menu_font: String,
    pub osd_font: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorTheme {
    pub custom: HashMap<String, String>,
}

impl Theme {
    pub fn load(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path =
            Theme::find_theme_path(name).ok_or_else(|| format!("theme '{}' not found", name))?;
        let content = std::fs::read_to_string(&path)?;
        Self::parse_themerc(&content)
    }

    fn find_theme_path(name: &str) -> Option<std::path::PathBuf> {
        let candidates = [
            dirs::config_dir()?
                .join("labwc/themes")
                .join(name)
                .join("themerc"),
            std::path::PathBuf::from("/usr/share/themes")
                .join(name)
                .join("openbox-3/themerc"),
            std::path::PathBuf::from("/usr/share/labwc/themes")
                .join(name)
                .join("themerc"),
        ];
        candidates.into_iter().find(|p| p.exists())
    }

    fn parse_themerc(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut theme = Theme::default();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
                continue;
            }
            if let Some((key, value)) = line.split_once(':') {
                let value = value.trim();
                match key.trim() {
                    "border.width" => theme.border_width = value.parse().unwrap_or(1),
                    "window.titlebar.height" => theme.titlebar_height = value.parse().unwrap_or(26),
                    "padding.width" => theme.padding_width = value.parse().unwrap_or(3),
                    "window.active.border.color" => theme.window.active_border_color = value.into(),
                    "window.inactive.border.color" => {
                        theme.window.inactive_border_color = value.into()
                    }
                    "window.active.title.bg" => theme.window.active_title_bg = value.into(),
                    "window.inactive.title.bg" => theme.window.inactive_title_bg = value.into(),
                    "window.active.title.fg" => theme.window.active_title_fg = value.into(),
                    "window.inactive.title.fg" => theme.window.inactive_title_fg = value.into(),
                    "window.label.text.justify" => {
                        theme.window.active_label_justify = match value {
                            "Left" => Justify::Left,
                            "Right" => Justify::Right,
                            _ => Justify::Center,
                        }
                    }
                    "menu.items.bg.color" => theme.menu.bg_color = value.into(),
                    "menu.items.text.color" => theme.menu.fg_color = value.into(),
                    "menu.items.active.bg" => theme.menu.active_bg = value.into(),
                    "menu.items.active.text.color" => theme.menu.active_fg = value.into(),
                    "menu.separator.color" => theme.menu.separator_color = value.into(),
                    "menu.border.color" => theme.menu.border_color = value.into(),
                    "window.active.font" => theme.fonts.active_font = value.into(),
                    "window.inactive.font" => theme.fonts.inactive_font = value.into(),
                    "menu.items.font" => theme.fonts.menu_font = value.into(),
                    "osd.font" => theme.fonts.osd_font = value.into(),
                    _ => {
                        theme.colors.custom.insert(key.trim().into(), value.into());
                    }
                }
            }
        }
        theme.name = "loaded_theme".into();
        Ok(theme)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".into(),
            border_width: 1,
            titlebar_height: 26,
            padding_width: 3,
            button_size: 18,
            menu_overlap: 0,
            window: WindowTheme {
                active_border_color: "#1a1a1a".into(),
                inactive_border_color: "#555555".into(),
                active_title_bg: "#1a1a1a".into(),
                inactive_title_bg: "#333333".into(),
                active_title_fg: "#ffffff".into(),
                inactive_title_fg: "#999999".into(),
                active_label_justify: Justify::Center,
                button_order: "CIM".into(),
            },
            menu: MenuTheme {
                bg_color: "#1a1a1a".into(),
                fg_color: "#ffffff".into(),
                active_bg: "#555555".into(),
                active_fg: "#ffffff".into(),
                separator_color: "#555555".into(),
                border_color: "#1a1a1a".into(),
            },
            fonts: FontTheme {
                active_font: "sans 10".into(),
                inactive_font: "sans 10".into(),
                menu_font: "sans 10".into(),
                osd_font: "sans 12".into(),
            },
            colors: ColorTheme {
                custom: HashMap::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_default() {
        let t = Theme::default();
        assert_eq!(t.border_width, 1);
        assert_eq!(t.titlebar_height, 26);
    }

    #[test]
    fn test_parse_themerc() {
        let content =
            "border.width: 2\nwindow.titlebar.height: 30\nwindow.active.border.color: #ff0000\n";
        let theme = Theme::parse_themerc(content).unwrap();
        assert_eq!(theme.border_width, 2);
        assert_eq!(theme.titlebar_height, 30);
        assert_eq!(theme.window.active_border_color, "#ff0000");
    }

    #[test]
    fn test_parse_empty() {
        let theme = Theme::parse_themerc("").unwrap();
        assert_eq!(theme.border_width, 1);
    }

    #[test]
    fn test_parse_comment() {
        let content = "# this is a comment\n! also a comment\nborder.width: 3\n";
        let theme = Theme::parse_themerc(content).unwrap();
        assert_eq!(theme.border_width, 3);
    }
}
