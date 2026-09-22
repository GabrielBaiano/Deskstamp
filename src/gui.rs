use crate::config::WatermarkConfig;
use crate::ipc::{send_command, IpcCommand};
use eframe::egui;

pub struct SettingsApp {
    config: WatermarkConfig,
    daemon_running: bool,
    status_msg: String,
    logo: Option<egui::TextureHandle>,
    last_check_frame: u64,
}

impl SettingsApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = WatermarkConfig::load();
        let daemon_running = send_command(&IpcCommand::Status).is_ok();

        let mut app = Self {
            config,
            daemon_running,
            status_msg: String::new(),
            logo: None,
            last_check_frame: 0,
        };

        if !app.daemon_running {
            app.start_daemon_process();
        } else {
            app.status_msg = "Overlay daemon is active and running.".to_string();
        }

        app
    }

    fn notify_daemon_reload(&mut self) {
        let _ = self.config.save();
        if self.daemon_running {
            let _ = send_command(&IpcCommand::Reload);
        } else {
            self.start_daemon_process();
        }
    }

    fn restart_daemon(&mut self) {
        let _ = send_command(&IpcCommand::Quit);
        std::thread::sleep(std::time::Duration::from_millis(200));
        self.start_daemon_process();
    }

    fn start_daemon_process(&mut self) {
        if send_command(&IpcCommand::Status).is_ok() {
            self.daemon_running = true;
            let _ = send_command(&IpcCommand::Reload);
            self.status_msg = "Connected to running overlay daemon.".to_string();
            return;
        }

        if let Ok(exe) = std::env::current_exe() {
            let res = std::process::Command::new(exe)
                .arg("daemon")
                .spawn();

            if res.is_ok() {
                for _ in 0..15 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    if send_command(&IpcCommand::Status).is_ok() {
                        self.daemon_running = true;
                        self.status_msg = "Overlay daemon started and displaying live.".to_string();
                        return;
                    }
                }
            }
            self.daemon_running = false;
            self.status_msg = "Failed to connect to overlay daemon.".to_string();
        }
    }
}

impl eframe::App for SettingsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.last_check_frame += 1;
        if self.last_check_frame % 60 == 0 {
            self.daemon_running = send_command(&IpcCommand::Status).is_ok();
        }

        let mut changed = false;

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(22, 25, 32);
        visuals.window_fill = egui::Color32::from_rgb(22, 25, 32);
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(6);
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
        ui.ctx().set_visuals(visuals);

        if self.logo.is_none() {
            self.logo = Some(load_logo_texture(ui.ctx()));
        }

        ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);

        // Header Section
        ui.horizontal(|ui| {
            if let Some(logo) = &self.logo {
                ui.image(egui::load::SizedTexture::new(logo.id(), egui::vec2(32.0, 32.0)));
            }
            ui.heading("Deskstamp Settings");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.daemon_running {
                    ui.colored_label(egui::Color32::from_rgb(80, 220, 120), "● Live on Screen");
                    if ui.button("⏹ Stop").clicked() {
                        let _ = send_command(&IpcCommand::Quit);
                        self.daemon_running = false;
                        self.status_msg = "Overlay daemon stopped.".to_string();
                    }
                    if ui.button("🔄 Restart").clicked() {
                        self.restart_daemon();
                    }
                } else {
                    ui.colored_label(egui::Color32::from_rgb(230, 90, 90), "○ Stopped");
                    if ui.button("▶ Start Overlay").clicked() {
                        self.start_daemon_process();
                    }
                }
            });
        });

        ui.separator();

        // Active Toggle
        ui.horizontal(|ui| {
            let prev_active = self.config.active;
            let label = if self.config.active { " Watermark Overlay: ENABLED " } else { " Watermark Overlay: DISABLED " };
            ui.toggle_value(&mut self.config.active, label);
            if self.config.active != prev_active {
                changed = true;
                if self.daemon_running {
                    let _ = send_command(&IpcCommand::Toggle);
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("Pop!_OS COSMIC Tray active").small().weak());
            });
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Presets
            ui.group(|ui| {
                ui.label(egui::RichText::new("⚡ Quick Presets").strong());
                ui.horizontal_wrapped(|ui| {
                    if ui.button("🛡️ Column Guard (Lines + Horizontal)").clicked() {
                        self.config.text = "CONFIDENTIAL • {user}@{hostname} • {time:%H:%M:%S}".to_string();
                        self.config.opacity = 0.22;
                        self.config.angle_deg = 0.0;
                        self.config.font_size = 18.0;
                        self.config.show_vertical_lines = true;
                        self.config.line_width = 1.5;
                        self.config.line_dashed = true;
                        self.config.line_color_rgba = [255, 255, 255, 200];
                        self.config.spacing_x = 480.0;
                        self.config.spacing_y = 160.0;
                        self.config.stagger_offset = 80.0;
                        changed = true;
                    }
                    if ui.button("🏢 Security Grid (Lines + Diagonal)").clicked() {
                        self.config.text = "CONFIDENTIAL • {user}@{hostname} • {time:%H:%M:%S}".to_string();
                        self.config.opacity = 0.22;
                        self.config.angle_deg = -25.0;
                        self.config.font_size = 18.0;
                        self.config.show_vertical_lines = true;
                        self.config.line_width = 1.5;
                        self.config.line_dashed = true;
                        self.config.line_color_rgba = [255, 255, 255, 200];
                        self.config.spacing_x = 380.0;
                        self.config.spacing_y = 200.0;
                        self.config.stagger_offset = 100.0;
                        changed = true;
                    }
                    if ui.button("🔒 High-Density Matrix").clicked() {
                        self.config.text = "INTERNAL ONLY • {user} • {time:%H:%M:%S}".to_string();
                        self.config.opacity = 0.28;
                        self.config.angle_deg = -30.0;
                        self.config.font_size = 16.0;
                        self.config.show_vertical_lines = true;
                        self.config.line_width = 1.0;
                        self.config.line_dashed = false;
                        self.config.line_color_rgba = [255, 255, 255, 180];
                        self.config.spacing_x = 280.0;
                        self.config.spacing_y = 140.0;
                        self.config.stagger_offset = 70.0;
                        changed = true;
                    }
                    if ui.button("🎥 Clean Stream (No Lines)").clicked() {
                        self.config.text = "LIVE STREAM • @{user}".to_string();
                        self.config.opacity = 0.18;
                        self.config.angle_deg = -20.0;
                        self.config.font_size = 24.0;
                        self.config.show_vertical_lines = false;
                        self.config.spacing_x = 450.0;
                        self.config.spacing_y = 250.0;
                        changed = true;
                    }
                });
            });

            // Mini Interactive Screen Preview
            ui.group(|ui| {
                ui.label(egui::RichText::new("🖥️ Live Preview").strong());
                let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 120.0), egui::Sense::hover());
                let painter = ui.painter_at(rect);

                // Preview monitor wallpaper
                painter.rect_filled(rect, egui::CornerRadius::same(6), egui::Color32::from_rgb(26, 29, 36));

                if self.config.active {
                    let w = rect.width();
                    let _h = rect.height();
                    let preview_scale = w / 1920.0;
                    let step_x = (self.config.spacing_x * preview_scale).max(30.0);
                    let step_y = (self.config.spacing_y * preview_scale).max(20.0);

                    // 1. Scaled vertical lines
                    if self.config.show_vertical_lines {
                        let l_alpha = ((self.config.line_color_rgba[3] as f32 / 255.0) * self.config.opacity.clamp(0.0, 1.0) * 255.0) as u8;
                        let line_color = egui::Color32::from_rgba_premultiplied(
                            self.config.line_color_rgba[0],
                            self.config.line_color_rgba[1],
                            self.config.line_color_rgba[2],
                            l_alpha,
                        );

                        let mut cx = rect.left();
                        while cx < rect.right() + 10.0 {
                            if self.config.line_dashed {
                                let mut cy = rect.top();
                                while cy < rect.bottom() {
                                    let dash_end = (cy + 6.0).min(rect.bottom());
                                    painter.line_segment([egui::pos2(cx, cy), egui::pos2(cx, dash_end)], egui::Stroke::new(self.config.line_width.max(1.0), line_color));
                                    cy += 11.0;
                                }
                            } else {
                                painter.line_segment([egui::pos2(cx, rect.top()), egui::pos2(cx, rect.bottom())], egui::Stroke::new(self.config.line_width.max(1.0), line_color));
                            }
                            cx += step_x;
                        }
                    }

                    // 2. Scaled preview text
                    let t_alpha = (self.config.opacity.clamp(0.0, 1.0) * 255.0) as u8;
                    let text_color = egui::Color32::from_rgba_unmultiplied(
                        self.config.color_rgba[0],
                        self.config.color_rgba[1],
                        self.config.color_rgba[2],
                        t_alpha,
                    );
                    let font_id = egui::FontId::proportional((self.config.font_size * preview_scale * 1.6).max(8.0));
                    let sample_text = if self.config.text.len() > 22 {
                        &self.config.text[..22]
                    } else {
                        &self.config.text
                    };

                    let mut cy = rect.top() + 14.0;
                    let mut row = 0;
                    while cy < rect.bottom() {
                        let stagger = if row % 2 == 1 { self.config.stagger_offset * preview_scale } else { 0.0 };
                        let mut cx = rect.left() + step_x * 0.5 + stagger;
                        while cx < rect.right() + 40.0 {
                            painter.text(
                                egui::pos2(cx, cy),
                                egui::Align2::CENTER_CENTER,
                                sample_text,
                                font_id.clone(),
                                text_color,
                            );
                            cx += step_x;
                        }
                        cy += step_y;
                        row += 1;
                    }
                } else {
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Watermark Overlay Disabled",
                        egui::FontId::proportional(13.0),
                        egui::Color32::from_gray(120),
                    );
                }
            });

            // Vertical Lines Settings
            ui.group(|ui| {
                ui.label(egui::RichText::new("📏 Vertical Lines").strong());

                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.config.show_vertical_lines, "Show Vertical Column Lines").changed() {
                        changed = true;
                    }
                    if self.config.show_vertical_lines {
                        if ui.checkbox(&mut self.config.line_dashed, "Dashed Pattern").changed() {
                            changed = true;
                        }
                    }
                });

                if self.config.show_vertical_lines {
                    ui.horizontal(|ui| {
                        ui.label("Line Width:");
                        if ui.add(egui::Slider::new(&mut self.config.line_width, 0.5..=6.0).suffix(" px")).changed() {
                            changed = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Line Color:");
                        let mut lc = [
                            self.config.line_color_rgba[0] as f32 / 255.0,
                            self.config.line_color_rgba[1] as f32 / 255.0,
                            self.config.line_color_rgba[2] as f32 / 255.0,
                        ];
                        if ui.color_edit_button_rgb(&mut lc).changed() {
                            self.config.line_color_rgba[0] = (lc[0] * 255.0) as u8;
                            self.config.line_color_rgba[1] = (lc[1] * 255.0) as u8;
                            self.config.line_color_rgba[2] = (lc[2] * 255.0) as u8;
                            changed = true;
                        }

                        ui.label("Alpha:");
                        let mut line_a = self.config.line_color_rgba[3];
                        if ui.add(egui::Slider::new(&mut line_a, 10..=255)).changed() {
                            self.config.line_color_rgba[3] = line_a;
                            changed = true;
                        }
                    });
                }
            });

            // Text Template & Tokens
            ui.group(|ui| {
                ui.label(egui::RichText::new("✍️ Text Template").strong());
                ui.horizontal(|ui| {
                    if ui.text_edit_singleline(&mut self.config.text).changed() {
                        changed = true;
                    }
                });

                ui.horizontal_wrapped(|ui| {
                    ui.label("Quick tokens:");
                    if ui.button("+ User").clicked() {
                        self.config.text.push_str(" {user}");
                        changed = true;
                    }
                    if ui.button("+ Hostname").clicked() {
                        self.config.text.push_str(" {hostname}");
                        changed = true;
                    }
                    if ui.button("+ Time").clicked() {
                        self.config.text.push_str(" {time:%H:%M:%S}");
                        changed = true;
                    }
                    if ui.button("+ Date").clicked() {
                        self.config.text.push_str(" {date:%Y-%m-%d}");
                        changed = true;
                    }
                });
            });

            // Matrix Transformation & Sliders
            ui.group(|ui| {
                ui.label(egui::RichText::new("📐 Geometry & Transformations").strong());

                ui.horizontal(|ui| {
                    ui.label("Rotation Angle:");
                    if ui.add(egui::Slider::new(&mut self.config.angle_deg, -90.0..=90.0).suffix("°")).changed() {
                        changed = true;
                    }
                    if ui.button("0° (Vertical)").clicked() {
                        self.config.angle_deg = 0.0;
                        changed = true;
                    }
                    if ui.button("-25° (Diagonal)").clicked() {
                        self.config.angle_deg = -25.0;
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Opacity:");
                    let mut op_percent = self.config.opacity * 100.0;
                    if ui.add(egui::Slider::new(&mut op_percent, 1.0..=100.0).suffix("%")).changed() {
                        self.config.opacity = op_percent / 100.0;
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Font Size:");
                    if ui.add(egui::Slider::new(&mut self.config.font_size, 10.0..=64.0).suffix(" px")).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Column Spacing (X):");
                    if ui.add(egui::Slider::new(&mut self.config.spacing_x, 150.0..=1000.0).suffix(" px")).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Row Spacing (Y):");
                    if ui.add(egui::Slider::new(&mut self.config.spacing_y, 80.0..=600.0).suffix(" px")).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Stagger / Offset:");
                    if ui.add(egui::Slider::new(&mut self.config.stagger_offset, 0.0..=400.0).suffix(" px")).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Stroke Outline:");
                    if ui.add(egui::Slider::new(&mut self.config.stroke_width, 0.0..=4.0).suffix(" px")).changed() {
                        changed = true;
                    }
                });
            });

            // Text & Stroke Colors
            ui.group(|ui| {
                ui.label(egui::RichText::new("🎨 Text Colors").strong());
                ui.horizontal(|ui| {
                    ui.label("Text Color:");
                    let mut c = [
                        self.config.color_rgba[0] as f32 / 255.0,
                        self.config.color_rgba[1] as f32 / 255.0,
                        self.config.color_rgba[2] as f32 / 255.0,
                    ];
                    if ui.color_edit_button_rgb(&mut c).changed() {
                        self.config.color_rgba[0] = (c[0] * 255.0) as u8;
                        self.config.color_rgba[1] = (c[1] * 255.0) as u8;
                        self.config.color_rgba[2] = (c[2] * 255.0) as u8;
                        changed = true;
                    }

                    ui.separator();

                    ui.label("Stroke Color:");
                    let mut sc = [
                        self.config.stroke_color_rgba[0] as f32 / 255.0,
                        self.config.stroke_color_rgba[1] as f32 / 255.0,
                        self.config.stroke_color_rgba[2] as f32 / 255.0,
                    ];
                    if ui.color_edit_button_rgb(&mut sc).changed() {
                        self.config.stroke_color_rgba[0] = (sc[0] * 255.0) as u8;
                        self.config.stroke_color_rgba[1] = (sc[1] * 255.0) as u8;
                        self.config.stroke_color_rgba[2] = (sc[2] * 255.0) as u8;
                        changed = true;
                    }
                });
            });

            ui.add_space(8.0);
            ui.label(egui::RichText::new(&self.status_msg).italics());
        });

        if changed {
            self.notify_daemon_reload();
        }
    }
}

pub fn load_app_icon() -> Option<egui::IconData> {
    static ICON_BYTES: &[u8] = include_bytes!("../data/icons/hicolor/64x64/apps/io.github.gabrielbaiano.Deskstamp.png");
    let pix = tiny_skia::Pixmap::decode_png(ICON_BYTES).ok()?;
    Some(egui::IconData {
        rgba: pix.data().to_vec(),
        width: pix.width(),
        height: pix.height(),
    })
}

pub fn load_logo_texture(ctx: &egui::Context) -> egui::TextureHandle {
    static ICON_BYTES: &[u8] = include_bytes!("../data/icons/hicolor/128x128/apps/io.github.gabrielbaiano.Deskstamp.png");
    let pix = tiny_skia::Pixmap::decode_png(ICON_BYTES).expect("valid icon");
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [pix.width() as usize, pix.height() as usize],
        pix.data(),
    );
    ctx.load_texture("deskstamp_logo", color_image, egui::TextureOptions::LINEAR)
}

pub fn run_gui() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = egui::ViewportBuilder::default()
        .with_inner_size([620.0, 760.0])
        .with_min_inner_size([500.0, 550.0])
        .with_title("Deskstamp - Settings");

    if let Some(icon) = load_app_icon() {
        builder = builder.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport: builder,
        ..Default::default()
    };

    eframe::run_native(
        "Deskstamp",
        native_options,
        Box::new(|cc| Ok(Box::new(SettingsApp::new(cc)))),
    ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    Ok(())
}

fn get_menu_pid_path() -> std::path::PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    std::path::PathBuf::from(runtime_dir).join("deskstamp-menu.pid")
}

struct MenuPidGuard;
impl Drop for MenuPidGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(get_menu_pid_path());
    }
}

pub struct QuickMenuApp {
    config: WatermarkConfig,
    daemon_running: bool,
    logo: Option<egui::TextureHandle>,
    last_check_frame: u64,
}

impl QuickMenuApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = WatermarkConfig::load();
        let daemon_running = send_command(&IpcCommand::Status).is_ok();
        let mut app = Self {
            config,
            daemon_running,
            logo: None,
            last_check_frame: 0,
        };
        if !app.daemon_running {
            app.start_daemon();
        }
        app
    }

    fn notify_daemon(&mut self) {
        let _ = self.config.save();
        if self.daemon_running {
            let _ = send_command(&IpcCommand::Reload);
        } else {
            self.start_daemon();
        }
    }

    fn start_daemon(&mut self) {
        if send_command(&IpcCommand::Status).is_ok() {
            self.daemon_running = true;
            let _ = send_command(&IpcCommand::Reload);
            return;
        }
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::process::Command::new(exe).arg("daemon").spawn();
            for _ in 0..12 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if send_command(&IpcCommand::Status).is_ok() {
                    self.daemon_running = true;
                    return;
                }
            }
        }
    }
}

impl eframe::App for QuickMenuApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.last_check_frame += 1;
        if self.last_check_frame % 60 == 0 {
            self.daemon_running = send_command(&IpcCommand::Status).is_ok();
        }

        // Close when user presses Escape
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }

        let mut changed = false;

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(22, 25, 32);
        visuals.window_fill = egui::Color32::from_rgb(22, 25, 32);
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(6);
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
        ui.ctx().set_visuals(visuals);

        if self.logo.is_none() {
            self.logo = Some(load_logo_texture(ui.ctx()));
        }

        ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

        // Header with 3D Stamp Icon + Title
        ui.horizontal(|ui| {
            if let Some(logo) = &self.logo {
                ui.image(egui::load::SizedTexture::new(logo.id(), egui::vec2(36.0, 36.0)));
            }
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Deskstamp").strong().size(17.0));
                ui.label(egui::RichText::new("Desktop Watermark").weak().size(11.0));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if self.config.active && self.daemon_running {
                    ui.colored_label(egui::Color32::from_rgb(80, 220, 120), "● Active");
                } else {
                    ui.colored_label(egui::Color32::from_rgb(220, 90, 90), "○ Disabled");
                }
            });
        });

        ui.separator();

        // Big Main Switch Button
        let switch_label = if self.config.active {
            egui::RichText::new("✓ Watermark is Active  •  Click to Hide")
                .color(egui::Color32::from_rgb(120, 240, 150))
                .strong()
        } else {
            egui::RichText::new("○ Watermark is Disabled  •  Click to Show")
                .color(egui::Color32::from_rgb(240, 140, 140))
                .strong()
        };
        if ui.add_sized([ui.available_width(), 36.0], egui::Button::new(switch_label)).clicked() {
            self.config.active = !self.config.active;
            changed = true;
            if self.daemon_running {
                let _ = send_command(&IpcCommand::Toggle);
            }
        }

        ui.add_space(2.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Quick Presets
                ui.group(|ui| {
                    ui.label(egui::RichText::new("PRESETS").strong().size(11.0).color(egui::Color32::from_rgb(160, 180, 220)));
                    ui.columns(2, |cols| {
                        if cols[0].add_sized([cols[0].available_width(), 28.0], egui::Button::new("Column Guard")).clicked() {
                            self.config.text = "CONFIDENTIAL • {user}@{hostname} • {time:%H:%M:%S}".to_string();
                            self.config.opacity = 0.22;
                            self.config.angle_deg = 0.0;
                            self.config.font_size = 18.0;
                            self.config.show_vertical_lines = true;
                            self.config.line_width = 1.5;
                            self.config.line_dashed = true;
                            self.config.line_color_rgba = [255, 255, 255, 200];
                            self.config.spacing_x = 480.0;
                            self.config.spacing_y = 160.0;
                            self.config.stagger_offset = 80.0;
                            changed = true;
                        }
                        if cols[1].add_sized([cols[1].available_width(), 28.0], egui::Button::new("Security Grid")).clicked() {
                            self.config.text = "CONFIDENTIAL • {user}@{hostname} • {time:%H:%M:%S}".to_string();
                            self.config.opacity = 0.22;
                            self.config.angle_deg = -25.0;
                            self.config.font_size = 18.0;
                            self.config.show_vertical_lines = true;
                            self.config.line_width = 1.5;
                            self.config.line_dashed = true;
                            self.config.line_color_rgba = [255, 255, 255, 200];
                            self.config.spacing_x = 380.0;
                            self.config.spacing_y = 200.0;
                            self.config.stagger_offset = 100.0;
                            changed = true;
                        }
                    });
                    ui.columns(2, |cols| {
                        if cols[0].add_sized([cols[0].available_width(), 28.0], egui::Button::new("High-Density")).clicked() {
                            self.config.text = "INTERNAL ONLY • {user} • {time:%H:%M:%S}".to_string();
                            self.config.opacity = 0.28;
                            self.config.angle_deg = -30.0;
                            self.config.font_size = 16.0;
                            self.config.show_vertical_lines = true;
                            self.config.line_width = 1.0;
                            self.config.line_dashed = false;
                            self.config.line_color_rgba = [255, 255, 255, 180];
                            self.config.spacing_x = 280.0;
                            self.config.spacing_y = 140.0;
                            self.config.stagger_offset = 70.0;
                            changed = true;
                        }
                        if cols[1].add_sized([cols[1].available_width(), 28.0], egui::Button::new("Clean Stream")).clicked() {
                            self.config.text = "LIVE STREAM • @{user}".to_string();
                            self.config.opacity = 0.18;
                            self.config.angle_deg = -20.0;
                            self.config.font_size = 24.0;
                            self.config.show_vertical_lines = false;
                            self.config.spacing_x = 450.0;
                            self.config.spacing_y = 250.0;
                            changed = true;
                        }
                    });
                });

                ui.add_space(2.0);

                // Watermark Text Template
                ui.group(|ui| {
                    ui.label(egui::RichText::new("WATERMARK TEXT").strong().size(11.0).color(egui::Color32::from_rgb(160, 180, 220)));
                    if ui.text_edit_singleline(&mut self.config.text).changed() {
                        changed = true;
                    }
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new("Tokens:").weak().size(11.0));
                        if ui.button("+ User").clicked() {
                            self.config.text.push_str(" {user}");
                            changed = true;
                        }
                        if ui.button("+ Host").clicked() {
                            self.config.text.push_str(" {hostname}");
                            changed = true;
                        }
                        if ui.button("+ Time").clicked() {
                            self.config.text.push_str(" {time:%H:%M:%S}");
                            changed = true;
                        }
                        if ui.button("+ Date").clicked() {
                            self.config.text.push_str(" {date:%Y-%m-%d}");
                            changed = true;
                        }
                    });
                });

                ui.add_space(2.0);

                // Appearance & Adjustments
                ui.group(|ui| {
                    ui.label(egui::RichText::new("APPEARANCE").strong().size(11.0).color(egui::Color32::from_rgb(160, 180, 220)));

                    // Opacity
                    ui.horizontal(|ui| {
                        ui.label("Opacity:");
                        let mut op = self.config.opacity * 100.0;
                        if ui.add(egui::Slider::new(&mut op, 2.0..=80.0).suffix("%")).changed() {
                            self.config.opacity = op / 100.0;
                            changed = true;
                        }
                    });

                    // Angle
                    ui.horizontal(|ui| {
                        ui.label("Angle:");
                        if ui.add(egui::Slider::new(&mut self.config.angle_deg, -90.0..=90.0).suffix("°")).changed() {
                            changed = true;
                        }
                        if ui.button("0°").clicked() {
                            self.config.angle_deg = 0.0;
                            changed = true;
                        }
                        if ui.button("-25°").clicked() {
                            self.config.angle_deg = -25.0;
                            changed = true;
                        }
                    });

                    // Font Size
                    ui.horizontal(|ui| {
                        ui.label("Font Size:");
                        if ui.add(egui::Slider::new(&mut self.config.font_size, 10.0..=48.0).suffix(" px")).changed() {
                            changed = true;
                        }
                    });

                    // Column Spacing
                    ui.horizontal(|ui| {
                        ui.label("Columns (X):");
                        if ui.add(egui::Slider::new(&mut self.config.spacing_x, 150.0..=800.0).suffix(" px")).changed() {
                            changed = true;
                        }
                    });

                    // Row Spacing
                    ui.horizontal(|ui| {
                        ui.label("Rows (Y):");
                        if ui.add(egui::Slider::new(&mut self.config.spacing_y, 60.0..=400.0).suffix(" px")).changed() {
                            changed = true;
                        }
                    });
                });

                ui.add_space(2.0);

                // Vertical Column Lines
                ui.group(|ui| {
                    ui.label(egui::RichText::new("VERTICAL LINES").strong().size(11.0).color(egui::Color32::from_rgb(160, 180, 220)));
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut self.config.show_vertical_lines, "Show Lines").changed() {
                            changed = true;
                        }
                        if self.config.show_vertical_lines {
                            if ui.checkbox(&mut self.config.line_dashed, "Dashed").changed() {
                                changed = true;
                            }
                        }
                    });

                    if self.config.show_vertical_lines {
                        ui.horizontal(|ui| {
                            ui.label("Line Width:");
                            if ui.add(egui::Slider::new(&mut self.config.line_width, 0.5..=5.0).suffix(" px")).changed() {
                                changed = true;
                            }
                        });
                    }
                });

                ui.add_space(4.0);

                // Footer Actions
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("🔄 Restart Overlay").clicked() {
                        self.start_daemon();
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Quit Deskstamp").clicked() {
                            let _ = send_command(&IpcCommand::Quit);
                            std::process::exit(0);
                        }
                    });
                });
            });

        if changed {
            self.notify_daemon();
        }
    }
}

pub fn run_quick_menu() -> Result<(), Box<dyn std::error::Error>> {
    let pid_path = get_menu_pid_path();

    if pid_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pid_path) {
            if let Ok(pid) = content.trim().parse::<i32>() {
                unsafe {
                    if libc::kill(pid, 0) == 0 {
                        // Process is running! User clicked tray to toggle OFF.
                        libc::kill(pid, libc::SIGTERM);
                        let _ = std::fs::remove_file(&pid_path);
                        return Ok(());
                    }
                }
            }
        }
        let _ = std::fs::remove_file(&pid_path);
    }

    // Write current PID
    let current_pid = std::process::id();
    let _ = std::fs::write(&pid_path, current_pid.to_string());
    let _guard = MenuPidGuard;

    let mut builder = egui::ViewportBuilder::default()
        .with_inner_size([380.0, 620.0])
        .with_min_inner_size([340.0, 480.0])
        .with_maximize_button(false)
        .with_title("Deskstamp");

    if let Some(icon) = load_app_icon() {
        builder = builder.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport: builder,
        ..Default::default()
    };

    eframe::run_native(
        "Deskstamp",
        native_options,
        Box::new(|cc| Ok(Box::new(QuickMenuApp::new(cc)))),
    ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    Ok(())
}
