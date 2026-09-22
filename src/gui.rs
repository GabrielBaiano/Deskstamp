use crate::config::WatermarkConfig;
use crate::ipc::{get_socket_path, send_command, IpcCommand};
use eframe::egui;

pub struct SettingsApp {
    config: WatermarkConfig,
    daemon_running: bool,
    status_msg: String,
    last_check_frame: u64,
}

impl SettingsApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = WatermarkConfig::load();
        let daemon_running = send_command(&IpcCommand::Status).is_ok();

        Self {
            config,
            daemon_running,
            status_msg: if daemon_running {
                "Overlay daemon is active and running.".to_string()
            } else {
                "Daemon is offline. Click 'Start Overlay' below.".to_string()
            },
            last_check_frame: 0,
        }
    }

    fn notify_daemon_reload(&mut self) {
        let _ = self.config.save();
        if self.daemon_running {
            let _ = send_command(&IpcCommand::Reload);
        }
    }

    fn start_daemon_process(&mut self) {
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::process::Command::new(exe)
                .arg("daemon")
                .spawn();
            std::thread::sleep(std::time::Duration::from_millis(200));
            self.daemon_running = send_command(&IpcCommand::Status).is_ok();
            self.status_msg = if self.daemon_running {
                "Overlay daemon started successfully.".to_string()
            } else {
                "Failed to start overlay daemon.".to_string()
            };
        }
    }
}

impl eframe::App for SettingsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Periodic check for daemon status
        self.last_check_frame += 1;
        if self.last_check_frame % 60 == 0 {
            self.daemon_running = send_command(&IpcCommand::Status).is_ok();
        }

        let mut changed = false;

        ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);

        // Header Section
        ui.horizontal(|ui| {
            ui.heading("🛡️ Deskstamp Settings");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.daemon_running {
                    ui.colored_label(egui::Color32::from_rgb(80, 200, 120), "● Daemon Active");
                    if ui.button("Stop Daemon").clicked() {
                        let _ = send_command(&IpcCommand::Quit);
                        self.daemon_running = false;
                        self.status_msg = "Daemon stopped.".to_string();
                    }
                } else {
                    ui.colored_label(egui::Color32::from_rgb(200, 80, 80), "○ Daemon Inactive");
                    if ui.button("▶ Start Daemon").clicked() {
                        self.start_daemon_process();
                    }
                }
            });
        });

        ui.separator();

        // Active Toggle
        ui.horizontal(|ui| {
            let prev_active = self.config.active;
            let label = if self.config.active { " Watermark: ENABLED " } else { " Watermark: DISABLED " };
            ui.toggle_value(&mut self.config.active, label);
            if self.config.active != prev_active {
                changed = true;
                if self.daemon_running {
                    let _ = send_command(&IpcCommand::Toggle);
                }
            }

            ui.label(format!("Socket: {:?}", get_socket_path()));
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Text Configuration
            ui.group(|ui| {
                ui.label(egui::RichText::new("Watermark Template").strong());
                ui.horizontal(|ui| {
                    if ui.text_edit_singleline(&mut self.config.text).changed() {
                        changed = true;
                    }
                });

                ui.horizontal_wrapped(|ui| {
                    ui.label("Insert dynamic token:");
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

            // Geometry and Matrix Transformation Sliders
            ui.group(|ui| {
                ui.label(egui::RichText::new("Layout & Grid Transformation").strong());

                ui.horizontal(|ui| {
                    ui.label("Rotation Angle:");
                    if ui.add(egui::Slider::new(&mut self.config.angle_deg, -90.0..=90.0).suffix("°")).changed() {
                        changed = true;
                    }
                    if ui.button("Reset Angle").clicked() {
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
                    if ui.add(egui::Slider::new(&mut self.config.font_size, 10.0..=72.0).suffix(" px")).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Spacing X:");
                    if ui.add(egui::Slider::new(&mut self.config.spacing_x, 150.0..=1000.0).suffix(" px")).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Spacing Y:");
                    if ui.add(egui::Slider::new(&mut self.config.spacing_y, 80.0..=600.0).suffix(" px")).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Stagger / Offset:");
                    if ui.add(egui::Slider::new(&mut self.config.stagger_offset, 0.0..=500.0).suffix(" px")).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Stroke Width:");
                    if ui.add(egui::Slider::new(&mut self.config.stroke_width, 0.0..=5.0).suffix(" px")).changed() {
                        changed = true;
                    }
                });
            });

            // Colors
            ui.group(|ui| {
                ui.label(egui::RichText::new("Colors & Contrast").strong());
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

            // Presets
            ui.group(|ui| {
                ui.label(egui::RichText::new("Presets").strong());
                ui.horizontal(|ui| {
                    if ui.button("Confidential NDA").clicked() {
                        self.config.text = "CONFIDENTIAL • {user}@{hostname} • {time:%H:%M:%S}".to_string();
                        self.config.opacity = 0.18;
                        self.config.angle_deg = -25.0;
                        self.config.font_size = 22.0;
                        self.config.spacing_x = 420.0;
                        self.config.spacing_y = 220.0;
                        changed = true;
                    }
                    if ui.button("Live Streaming").clicked() {
                        self.config.text = "LIVE STREAM • @{user}".to_string();
                        self.config.opacity = 0.15;
                        self.config.angle_deg = -20.0;
                        self.config.font_size = 26.0;
                        self.config.spacing_x = 500.0;
                        self.config.spacing_y = 280.0;
                        changed = true;
                    }
                    if ui.button("Classroom / Teaching").clicked() {
                        self.config.text = "EDUCATIONAL COPY • DO NOT DISTRIBUTE".to_string();
                        self.config.opacity = 0.22;
                        self.config.angle_deg = -30.0;
                        self.config.font_size = 24.0;
                        self.config.spacing_x = 450.0;
                        self.config.spacing_y = 240.0;
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

pub fn run_gui() -> Result<(), Box<dyn std::error::Error>> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([620.0, 720.0])
            .with_min_inner_size([500.0, 500.0])
            .with_title("Deskstamp - Settings"),
        ..Default::default()
    };

    eframe::run_native(
        "Deskstamp",
        native_options,
        Box::new(|cc| Ok(Box::new(SettingsApp::new(cc)))),
    ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    Ok(())
}
