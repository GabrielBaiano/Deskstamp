use crate::config::WatermarkConfig;
use crate::ipc::{send_command, IpcCommand};
use ksni::blocking::TrayMethods;
use ksni::menu::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct DeskstampTray {
    pub active: Arc<AtomicBool>,
}

impl ksni::Tray for DeskstampTray {
    const MENU_ON_ACTIVATE: bool = true;

    fn id(&self) -> String {
        "io.github.gabrielbaiano.Deskstamp".to_string()
    }

    fn title(&self) -> String {
        "Deskstamp".to_string()
    }

    fn icon_name(&self) -> String {
        "io.github.gabrielbaiano.Deskstamp".to_string()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        static ICON_PNG: &[u8] = include_bytes!("../data/icons/hicolor/64x64/apps/io.github.gabrielbaiano.Deskstamp.png");
        if let Ok(pix) = tiny_skia::Pixmap::decode_png(ICON_PNG) {
            let width = pix.width() as i32;
            let height = pix.height() as i32;
            let mut data = pix.data().to_vec();
            for pixel in data.chunks_exact_mut(4) {
                pixel.rotate_right(1);
            }
            vec![ksni::Icon { width, height, data }]
        } else {
            vec![]
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let current = self.active.load(Ordering::Relaxed);
        let new_state = !current;
        self.active.store(new_state, Ordering::Relaxed);
        let mut cfg = WatermarkConfig::load();
        cfg.active = new_state;
        let _ = cfg.save();
        let _ = send_command(&IpcCommand::Toggle);
    }

    fn secondary_activate(&mut self, _x: i32, _y: i32) {
        self.activate(_x, _y);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let cfg = WatermarkConfig::load();
        let is_active = self.active.load(Ordering::Relaxed);

        vec![
            // 1. Watermark Main Toggle
            CheckmarkItem {
                label: if is_active {
                    "✓ Watermark Active".into()
                } else {
                    "Watermark Inactive".into()
                },
                checked: is_active,
                activate: Box::new(|this: &mut Self| {
                    let current = this.active.load(Ordering::Relaxed);
                    let new_state = !current;
                    this.active.store(new_state, Ordering::Relaxed);
                    let mut cfg = WatermarkConfig::load();
                    cfg.active = new_state;
                    let _ = cfg.save();
                    let _ = send_command(&IpcCommand::Toggle);
                }),
                ..Default::default()
            }.into(),

            MenuItem::Separator,

            // 2. Vertical Security Lines Toggle
            CheckmarkItem {
                label: "Vertical Security Lines".into(),
                checked: cfg.show_vertical_lines,
                activate: Box::new(|_| {
                    let mut cfg = WatermarkConfig::load();
                    cfg.show_vertical_lines = !cfg.show_vertical_lines;
                    let _ = cfg.save();
                    let _ = send_command(&IpcCommand::Reload);
                }),
                ..Default::default()
            }.into(),

            // 3. Diagonal Security Lines Toggle
            CheckmarkItem {
                label: "Diagonal Security Lines".into(),
                checked: cfg.show_diagonal_lines,
                activate: Box::new(|_| {
                    let mut cfg = WatermarkConfig::load();
                    cfg.show_diagonal_lines = !cfg.show_diagonal_lines;
                    let _ = cfg.save();
                    let _ = send_command(&IpcCommand::Reload);
                }),
                ..Default::default()
            }.into(),

            MenuItem::Separator,

            // 4. Watermark Mode Submenu
            SubMenu {
                label: format!("Watermark Mode ({})", match cfg.mode.as_str() {
                    "image" => "Logo",
                    "both" => "Logo + Text",
                    _ => "Text",
                }),
                submenu: vec![
                    CheckmarkItem {
                        label: "Text Only".into(),
                        checked: cfg.mode == "text",
                        activate: Box::new(|_| {
                            let mut cfg = WatermarkConfig::load();
                            cfg.mode = "text".to_string();
                            let _ = cfg.save();
                            let _ = send_command(&IpcCommand::Reload);
                        }),
                        ..Default::default()
                    }.into(),
                    CheckmarkItem {
                        label: "Logo / Stamp Only".into(),
                        checked: cfg.mode == "image",
                        activate: Box::new(|_| {
                            let mut cfg = WatermarkConfig::load();
                            cfg.mode = "image".to_string();
                            let _ = cfg.save();
                            let _ = send_command(&IpcCommand::Reload);
                        }),
                        ..Default::default()
                    }.into(),
                    CheckmarkItem {
                        label: "Both (Logo + Text)".into(),
                        checked: cfg.mode == "both",
                        activate: Box::new(|_| {
                            let mut cfg = WatermarkConfig::load();
                            cfg.mode = "both".to_string();
                            let _ = cfg.save();
                            let _ = send_command(&IpcCommand::Reload);
                        }),
                        ..Default::default()
                    }.into(),
                ],
                ..Default::default()
            }.into(),

            // 5. Famous Presets Submenu
            SubMenu {
                label: "Famous Presets".into(),
                submenu: vec![
                    StandardItem {
                        label: "DLP Confidential (Red lines + Session stamp)".into(),
                        activate: Box::new(|_| {
                            let mut cfg = WatermarkConfig::load();
                            cfg.text = "CONFIDENTIAL • {user}@{hostname} • {time:%H:%M:%S}".to_string();
                            cfg.opacity = 0.24;
                            cfg.angle_deg = -20.0;
                            cfg.font_size = 18.0;
                            cfg.show_vertical_lines = true;
                            cfg.line_width = 1.5;
                            cfg.line_dashed = true;
                            cfg.line_color_rgba = [255, 80, 80, 210];
                            cfg.show_diagonal_lines = false;
                            cfg.mode = "text".to_string();
                            cfg.spacing_x = 460.0;
                            cfg.spacing_y = 180.0;
                            let _ = cfg.save();
                            let _ = send_command(&IpcCommand::Reload);
                        }),
                        ..Default::default()
                    }.into(),
                    StandardItem {
                        label: "Security Crosshatch (Intersecting matrix)".into(),
                        activate: Box::new(|_| {
                            let mut cfg = WatermarkConfig::load();
                            cfg.text = "INTERNAL ONLY • {user}@{hostname} • {date:%Y-%m-%d} {time:%H:%M:%S}".to_string();
                            cfg.opacity = 0.22;
                            cfg.angle_deg = -25.0;
                            cfg.font_size = 16.0;
                            cfg.show_vertical_lines = true;
                            cfg.line_width = 1.0;
                            cfg.line_dashed = true;
                            cfg.line_color_rgba = [255, 255, 255, 180];
                            cfg.show_diagonal_lines = true;
                            cfg.diagonal_line_angle = 45.0;
                            cfg.diagonal_line_width = 1.0;
                            cfg.diagonal_line_dashed = true;
                            cfg.diagonal_line_color_rgba = [255, 255, 255, 180];
                            cfg.diagonal_line_spacing = 180.0;
                            cfg.mode = "text".to_string();
                            cfg.spacing_x = 420.0;
                            cfg.spacing_y = 200.0;
                            let _ = cfg.save();
                            let _ = send_command(&IpcCommand::Reload);
                        }),
                        ..Default::default()
                    }.into(),
                    StandardItem {
                        label: "Enterprise Asset Guard (3D Stamp + Host text)".into(),
                        activate: Box::new(|_| {
                            let mut cfg = WatermarkConfig::load();
                            cfg.text = "PROPERTY OF THE COMPANY • {user}@{hostname}".to_string();
                            cfg.opacity = 0.25;
                            cfg.angle_deg = -15.0;
                            cfg.font_size = 17.0;
                            cfg.show_vertical_lines = true;
                            cfg.line_width = 1.2;
                            cfg.line_dashed = false;
                            cfg.line_color_rgba = [233, 84, 32, 190];
                            cfg.show_diagonal_lines = false;
                            cfg.mode = "both".to_string();
                            cfg.image_path = None;
                            cfg.image_scale = 1.0;
                            cfg.spacing_x = 480.0;
                            cfg.spacing_y = 240.0;
                            let _ = cfg.save();
                            let _ = send_command(&IpcCommand::Reload);
                        }),
                        ..Default::default()
                    }.into(),
                    StandardItem {
                        label: "Forensic Microgrid (Ultra-dense microtext)".into(),
                        activate: Box::new(|_| {
                            let mut cfg = WatermarkConfig::load();
                            cfg.text = "{user} • {hostname} • {time:%H:%M:%S}".to_string();
                            cfg.opacity = 0.28;
                            cfg.angle_deg = 0.0;
                            cfg.font_size = 11.0;
                            cfg.spacing_x = 220.0;
                            cfg.spacing_y = 80.0;
                            cfg.stagger_offset = 40.0;
                            cfg.show_vertical_lines = true;
                            cfg.line_width = 0.8;
                            cfg.line_dashed = false;
                            cfg.line_color_rgba = [255, 255, 255, 120];
                            cfg.show_diagonal_lines = true;
                            cfg.diagonal_line_angle = -45.0;
                            cfg.diagonal_line_width = 0.8;
                            cfg.diagonal_line_dashed = false;
                            cfg.diagonal_line_color_rgba = [255, 255, 255, 120];
                            cfg.diagonal_line_spacing = 110.0;
                            cfg.mode = "text".to_string();
                            let _ = cfg.save();
                            let _ = send_command(&IpcCommand::Reload);
                        }),
                        ..Default::default()
                    }.into(),
                    StandardItem {
                        label: "Clean Streamer (Subtle streaming tag)".into(),
                        activate: Box::new(|_| {
                            let mut cfg = WatermarkConfig::load();
                            cfg.text = "LIVE STREAM • @{user}".to_string();
                            cfg.opacity = 0.16;
                            cfg.angle_deg = -18.0;
                            cfg.font_size = 22.0;
                            cfg.show_vertical_lines = false;
                            cfg.show_diagonal_lines = false;
                            cfg.mode = "text".to_string();
                            cfg.spacing_x = 480.0;
                            cfg.spacing_y = 260.0;
                            let _ = cfg.save();
                            let _ = send_command(&IpcCommand::Reload);
                        }),
                        ..Default::default()
                    }.into(),
                ],
                ..Default::default()
            }.into(),

            // 6. Opacity Presets Submenu
            SubMenu {
                label: format!("Opacity ({:.0}%)", cfg.opacity * 100.0),
                submenu: vec![
                    CheckmarkItem {
                        label: "10% (Subtle)".into(),
                        checked: (cfg.opacity - 0.10).abs() < 0.04,
                        activate: Box::new(|_| {
                            let mut cfg = WatermarkConfig::load();
                            cfg.opacity = 0.10;
                            let _ = cfg.save();
                            let _ = send_command(&IpcCommand::Reload);
                        }),
                        ..Default::default()
                    }.into(),
                    CheckmarkItem {
                        label: "18% (Default)".into(),
                        checked: (cfg.opacity - 0.18).abs() < 0.04,
                        activate: Box::new(|_| {
                            let mut cfg = WatermarkConfig::load();
                            cfg.opacity = 0.18;
                            let _ = cfg.save();
                            let _ = send_command(&IpcCommand::Reload);
                        }),
                        ..Default::default()
                    }.into(),
                    CheckmarkItem {
                        label: "25% (Medium)".into(),
                        checked: (cfg.opacity - 0.25).abs() < 0.04,
                        activate: Box::new(|_| {
                            let mut cfg = WatermarkConfig::load();
                            cfg.opacity = 0.25;
                            let _ = cfg.save();
                            let _ = send_command(&IpcCommand::Reload);
                        }),
                        ..Default::default()
                    }.into(),
                    CheckmarkItem {
                        label: "35% (Strong)".into(),
                        checked: (cfg.opacity - 0.35).abs() < 0.04,
                        activate: Box::new(|_| {
                            let mut cfg = WatermarkConfig::load();
                            cfg.opacity = 0.35;
                            let _ = cfg.save();
                            let _ = send_command(&IpcCommand::Reload);
                        }),
                        ..Default::default()
                    }.into(),
                    CheckmarkItem {
                        label: "50% (High Security)".into(),
                        checked: (cfg.opacity - 0.50).abs() < 0.04,
                        activate: Box::new(|_| {
                            let mut cfg = WatermarkConfig::load();
                            cfg.opacity = 0.50;
                            let _ = cfg.save();
                            let _ = send_command(&IpcCommand::Reload);
                        }),
                        ..Default::default()
                    }.into(),
                ],
                ..Default::default()
            }.into(),

            MenuItem::Separator,

            // 7. Full Deskstamp Studio Settings Window
            StandardItem {
                label: "Deskstamp Studio (Full Settings)...".into(),
                activate: Box::new(|_| {
                    if let Ok(exe) = std::env::current_exe() {
                        let _ = std::process::Command::new(exe).arg("settings").spawn();
                    }
                }),
                ..Default::default()
            }.into(),

            MenuItem::Separator,

            // 8. Quit
            StandardItem {
                label: "Quit Deskstamp".into(),
                activate: Box::new(|_| {
                    let _ = send_command(&IpcCommand::Quit);
                    std::process::exit(0);
                }),
                ..Default::default()
            }.into(),
        ]
    }
}

pub fn spawn_tray(active: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let tray = DeskstampTray { active };
        if let Ok(_handle) = tray.spawn() {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
            }
        }
    });
}
