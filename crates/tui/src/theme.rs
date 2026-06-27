use ratatui::style::Color;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub border_active: Color,
    pub border_inactive: Color,
    pub primary_accent: Color,
    pub secondary_accent: Color,
    pub highlight: Color,
    pub status_done: Color,
    pub status_pending: Color,
    pub status_overdue: Color,
    pub label: Color,
    pub value: Color,
    pub tag: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            border_active: Color::Cyan,
            border_inactive: Color::DarkGray,
            primary_accent: Color::Cyan,
            secondary_accent: Color::Yellow,
            highlight: Color::White,
            status_done: Color::Rgb(166, 227, 161),
            status_pending: Color::Rgb(249, 226, 175),
            status_overdue: Color::Rgb(243, 139, 168),
            label: Color::DarkGray,
            value: Color::LightCyan,
            tag: Color::Magenta,
        }
    }
}

impl Theme {
    pub fn load() -> Self {
        let mut theme = Self::default();
        if let Some(home) = std::env::var_os("HOME") {
            let colors_path = PathBuf::from(home).join(".config/omarchy/current/theme/colors.toml");
            if let Ok(content) = std::fs::read_to_string(colors_path) {
                for line in content.lines() {
                    let parts: Vec<&str> = line.split('=').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim();
                        let val = parts[1].trim();
                        if let Some(color) = parse_hex_color(val) {
                            match key {
                                "accent" => {
                                    theme.border_active = color;
                                    theme.primary_accent = color;
                                }
                                "cursor" => {
                                    theme.secondary_accent = color;
                                }
                                "color0" | "color8" => {
                                    theme.border_inactive = color;
                                }
                                "color1" => {
                                    theme.status_overdue = color;
                                }
                                "color2" => {
                                    theme.status_done = color;
                                }
                                "color3" => {
                                    theme.status_pending = color;
                                }
                                "color5" => {
                                    theme.tag = color;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        theme
    }
}

fn parse_hex_color(hex_str: &str) -> Option<Color> {
    let clean = hex_str.trim().trim_matches('"').trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some(Color::Rgb(r, g, b))
    } else {
        None
    }
}
