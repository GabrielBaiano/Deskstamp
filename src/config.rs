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

    #[serde(default = "default_font_path")]
    pub font_path: Option<String>,

    #[serde(default = "default_active")]
    pub active: bool,

    #[serde(default = "default_update_interval")]
    pub update_interval_secs: u64,
}

fn default_text() -> String {
    "CONFIDENTIAL • {user}@{hostname} • {time:%H:%M:%S}".to_string()
}
fn default_font_size() -> f32 { 22.0 }
fn default_angle() -> f32 { -25.0 }
fn default_opacity() -> f32 { 0.18 }
fn default_color() -> [u8; 4] { [255, 255, 255, 255] }
fn default_spacing_x() -> f32 { 420.0 }
fn default_spacing_y() -> f32 { 220.0 }
fn default_stagger() -> f32 { 210.0 }
fn default_stroke_width() -> f32 { 1.0 }
fn default_stroke_color() -> [u8; 4] { [0, 0, 0, 180] }
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
            font_path: default_font_path(),
            active: default_active(),
            update_interval_secs: default_update_interval(),
        }
    }
}

impl WatermarkConfig {
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
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
