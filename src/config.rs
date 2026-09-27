use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatermarkConfig {
    #[serde(default = "default_text")]
    pub text: String,

    #[serde(default = "default_font_size")]
    pub font_size: f32,

    #[serde(default = "default_angle")]
    pub angle_deg: f32,

    #[serde(default = "default_opacity")]
    pub opacity: f32, // 0.0 to 1.0

    #[serde(default = "default_color")]
    pub color_rgba: [u8; 4],

    #[serde(default = "default_spacing_x")]
    pub spacing_x: f32,

    #[serde(default = "default_spacing_y")]
    pub spacing_y: f32,

    #[serde(default = "default_stagger")]
    pub stagger_offset: f32,

    #[serde(default = "default_stroke_width")]
    pub stroke_width: f32,

    #[serde(default = "default_stroke_color")]
    pub stroke_color_rgba: [u8; 4],

    #[serde(default = "default_show_lines")]
    pub show_vertical_lines: bool,

    #[serde(default = "default_line_width")]
    pub line_width: f32,

    #[serde(default = "default_line_dashed")]
    pub line_dashed: bool,

    #[serde(default = "default_line_color")]
    pub line_color_rgba: [u8; 4],

    #[serde(default = "default_show_diagonal")]
    pub show_diagonal_lines: bool,

    #[serde(default = "default_diagonal_angle")]
    pub diagonal_line_angle: f32,

    #[serde(default = "default_diagonal_width")]
    pub diagonal_line_width: f32,

    #[serde(default = "default_diagonal_dashed")]
    pub diagonal_line_dashed: bool,

    #[serde(default = "default_diagonal_color")]
    pub diagonal_line_color_rgba: [u8; 4],

    #[serde(default = "default_diagonal_spacing")]
    pub diagonal_line_spacing: f32,

    #[serde(default = "default_mode")]
    pub mode: String, // "text", "image", "both"

    #[serde(default = "default_image_path")]
    pub image_path: Option<String>,

    #[serde(default = "default_image_scale")]
    pub image_scale: f32,

    #[serde(default = "default_image_size")]
    pub image_size: f32,

    #[serde(default = "default_font_family")]
    pub font_family: String,

    #[serde(default = "default_font_path")]
    pub font_path: Option<String>,

    #[serde(default = "default_active")]
    pub active: bool,

    #[serde(default = "default_show_text")]
    pub show_text: bool,

    #[serde(default = "default_show_icon")]
    pub show_icon: bool,

    #[serde(default = "default_show_lines_next_to_text")]
    pub show_lines_next_to_text: bool,

    #[serde(default = "default_line_length")]
    pub line_length: f32,

    #[serde(default = "default_line_gap")]
    pub line_gap: f32,

    #[serde(default = "default_monochrome_icon")]
    pub monochrome_icon: bool,

    #[serde(default = "default_update_interval")]
    pub update_interval_secs: u64,

    #[serde(default = "default_layout")]
    pub layout: String, // "grid" or "corner"

    #[serde(default = "default_corner_position")]
    pub corner_position: String, // "bottom_right", "bottom_left", "top_right", "top_left"

    #[serde(default = "default_corner_margin_x")]
    pub corner_margin_x: f32,

    #[serde(default = "default_corner_margin_y")]
    pub corner_margin_y: f32,

    #[serde(default = "default_workspace_stamps")]
    pub workspace_stamps: std::collections::HashMap<String, String>,
}

fn default_layout() -> String { "grid".to_string() }
fn default_corner_position() -> String { "bottom_right".to_string() }
fn default_corner_margin_x() -> f32 { 40.0 }
fn default_corner_margin_y() -> f32 { 40.0 }
fn default_workspace_stamps() -> std::collections::HashMap<String, String> { std::collections::HashMap::new() }

fn default_show_text() -> bool { true }
fn default_show_icon() -> bool { true }
fn default_show_lines_next_to_text() -> bool { true }
fn default_line_length() -> f32 { 60.0 }
fn default_line_gap() -> f32 { 14.0 }
fn default_monochrome_icon() -> bool { false }

fn default_text() -> String {
    "CONFIDENTIAL • {user}@{hostname} • {time:%H:%M:%S}".to_string()
}
fn default_font_size() -> f32 { 18.0 }
fn default_angle() -> f32 { 0.0 }
fn default_opacity() -> f32 { 0.22 }
fn default_color() -> [u8; 4] { [255, 255, 255, 255] }
fn default_spacing_x() -> f32 { 480.0 }
fn default_spacing_y() -> f32 { 160.0 }
fn default_stagger() -> f32 { 80.0 }
fn default_stroke_width() -> f32 { 1.0 }
fn default_stroke_color() -> [u8; 4] { [0, 0, 0, 180] }
fn default_show_lines() -> bool { true }
fn default_line_width() -> f32 { 1.5 }
fn default_line_dashed() -> bool { true }
fn default_line_color() -> [u8; 4] { [255, 255, 255, 200] }
fn default_show_diagonal() -> bool { false }
fn default_diagonal_angle() -> f32 { -45.0 }
fn default_diagonal_width() -> f32 { 1.5 }
fn default_diagonal_dashed() -> bool { true }
fn default_diagonal_color() -> [u8; 4] { [255, 255, 255, 180] }
fn default_diagonal_spacing() -> f32 { 240.0 }
fn default_mode() -> String { "text".to_string() }
fn default_image_path() -> Option<String> { None }
fn default_image_scale() -> f32 { 1.0 }
fn default_image_size() -> f32 { 28.0 }
fn default_font_family() -> String { "Liberation Sans".to_string() }
fn default_font_path() -> Option<String> { None }
fn default_active() -> bool { true }
fn default_update_interval() -> u64 { 1 }

impl Default for WatermarkConfig {
    fn default() -> Self {
        Self {
            text: default_text(),
            font_size: default_font_size(),
            angle_deg: default_angle(),
            opacity: default_opacity(),
            color_rgba: default_color(),
            spacing_x: default_spacing_x(),
            spacing_y: default_spacing_y(),
            stagger_offset: default_stagger(),
            stroke_width: default_stroke_width(),
            stroke_color_rgba: default_stroke_color(),
            show_vertical_lines: default_show_lines(),
            line_width: default_line_width(),
            line_dashed: default_line_dashed(),
            line_color_rgba: default_line_color(),
            show_diagonal_lines: default_show_diagonal(),
            diagonal_line_angle: default_diagonal_angle(),
            diagonal_line_width: default_diagonal_width(),
            diagonal_line_dashed: default_diagonal_dashed(),
            diagonal_line_color_rgba: default_diagonal_color(),
            diagonal_line_spacing: default_diagonal_spacing(),
            mode: default_mode(),
            image_path: default_image_path(),
            image_scale: default_image_scale(),
            image_size: default_image_size(),
            font_family: default_font_family(),
            font_path: default_font_path(),
            active: default_active(),
            show_text: default_show_text(),
            show_icon: default_show_icon(),
            show_lines_next_to_text: default_show_lines_next_to_text(),
            line_length: default_line_length(),
            line_gap: default_line_gap(),
            monochrome_icon: default_monochrome_icon(),
            update_interval_secs: default_update_interval(),
            layout: default_layout(),
            corner_position: default_corner_position(),
            corner_margin_x: default_corner_margin_x(),
            corner_margin_y: default_corner_margin_y(),
            workspace_stamps: default_workspace_stamps(),
        }
    }
}


impl WatermarkConfig {
    pub fn is_corner_mode(&self) -> bool {
        self.layout == "corner"
    }

    pub fn get_text_for_workspace(&self, ws: Option<&str>) -> String {
        if let Some(w) = ws {
            if let Some(custom) = self.workspace_stamps.get(w) {
                if !custom.trim().is_empty() {
                    return custom.clone();
                }
            }
        }
        self.text.clone()
    }
    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config/deskstamp/config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str(&content) {
                return cfg;
            }
        }
        let def = Self::default();
        let _ = def.save();
        def
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        let tmp_path = path.with_extension("tmp");
        std::fs::write(&tmp_path, json)?;
        std::fs::rename(tmp_path, path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_and_backward_compatibility() {
        let def = WatermarkConfig::default();
        assert_eq!(def.layout, "grid");
        assert_eq!(def.corner_position, "bottom_right");
        assert!(!def.is_corner_mode());

        // Test deserializing legacy config without new fields
        let legacy_json = r#"{
            "text": "LEGACY STAMP",
            "active": true
        }"#;
        let parsed: WatermarkConfig = serde_json::from_str(legacy_json).unwrap();
        assert_eq!(parsed.text, "LEGACY STAMP");
        assert_eq!(parsed.layout, "grid");
        assert_eq!(parsed.corner_position, "bottom_right");
        assert_eq!(parsed.corner_margin_x, 40.0);
        assert_eq!(parsed.corner_margin_y, 40.0);
        assert!(parsed.workspace_stamps.is_empty());
    }

    #[test]
    fn test_workspace_stamps_and_corner_mode() {
        let mut cfg = WatermarkConfig::default();
        cfg.layout = "corner".to_string();
        assert!(cfg.is_corner_mode());

        cfg.text = "GLOBAL STAMP".to_string();
        cfg.workspace_stamps.insert("1".to_string(), "PROD SERVER - DO NOT TOUCH".to_string());
        cfg.workspace_stamps.insert("Dev".to_string(), "SANDBOX ENVIRONMENT".to_string());

        assert_eq!(cfg.get_text_for_workspace(Some("1")), "PROD SERVER - DO NOT TOUCH");
        assert_eq!(cfg.get_text_for_workspace(Some("Dev")), "SANDBOX ENVIRONMENT");
        assert_eq!(cfg.get_text_for_workspace(Some("2")), "GLOBAL STAMP");
        assert_eq!(cfg.get_text_for_workspace(None), "GLOBAL STAMP");
    }
}
