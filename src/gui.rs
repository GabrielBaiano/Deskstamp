use crate::config::WatermarkConfig;
use crate::ipc::{send_command, IpcCommand};
use eframe::egui;

/// Custom COSMIC-styled toggle switch with orange active accent (#e95420)
fn cosmic_switch(ui: &mut egui::Ui, value: &mut bool) -> egui::Response {
    let desired_size = egui::vec2(42.0, 22.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *value);

        let off_color = egui::Color32::from_rgb(54, 57, 64);
        let on_color = egui::Color32::from_rgb(233, 84, 32); // COSMIC Orange
        let bg_color = off_color.lerp_to_gamma(on_color, how_on);

        let radius = rect.height() * 0.5;
        ui.painter().rect_filled(rect, radius, bg_color);

        let knob_radius = radius - 2.5;
        let knob_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let knob_center = egui::pos2(knob_x, rect.center().y);
        ui.painter().circle_filled(knob_center, knob_radius, egui::Color32::WHITE);
    }

    response
}

/// Row toggle matching Pop!_OS COSMIC settings popover
fn cosmic_row_toggle(ui: &mut egui::Ui, title: &str, subtitle: Option<&str>, value: &mut bool) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(title).size(13.0).strong().color(egui::Color32::from_rgb(235, 235, 240)));
            if let Some(sub) = subtitle {
                ui.label(egui::RichText::new(sub).size(10.5).color(egui::Color32::from_rgb(150, 155, 165)));
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if cosmic_switch(ui, value).changed() {
                changed = true;
            }
        });
    });
    changed
}

// ---------------------------------------------------------------------------
// Deskstamp Settings Studio (Full Window)
// ---------------------------------------------------------------------------

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
            let res = std::process::Command::new(exe).arg("daemon").spawn();

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
        visuals.panel_fill = egui::Color32::from_rgb(26, 28, 34);
        visuals.window_fill = egui::Color32::from_rgb(26, 28, 34);
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(6);
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
        visuals.selection.bg_fill = egui::Color32::from_rgb(233, 84, 32);
        ui.ctx().set_visuals(visuals);

        if self.logo.is_none() {
            self.logo = Some(load_logo_texture(ui.ctx()));
        }

        // Header Panel
        ui.horizontal(|ui| {
            if let Some(logo) = &self.logo {
                ui.image(egui::load::SizedTexture::new(logo.id(), egui::vec2(44.0, 44.0)));
            }
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Deskstamp Studio").strong().size(19.0));
                ui.label(egui::RichText::new("Desktop Watermark & Security Layer").weak().size(12.0));
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Restart Daemon").clicked() {
                    self.restart_daemon();
                }

                if self.config.active && self.daemon_running {
                    ui.colored_label(egui::Color32::from_rgb(80, 220, 120), "● Overlay Active");
                } else if !self.config.active {
                    ui.colored_label(egui::Color32::from_rgb(240, 180, 70), "○ Watermark Hidden");
                } else {
                    ui.colored_label(egui::Color32::from_rgb(240, 90, 90), "✕ Daemon Offline");
                }
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);

            // 1. Famous Watermark Presets
            ui.group(|ui| {
                ui.label(egui::RichText::new("FAMOUS WATERMARK PRESETS").strong().size(11.5).color(egui::Color32::from_rgb(233, 84, 32)));
                ui.label(egui::RichText::new("Click any preset to instantly apply professional watermark patterns:").weak().size(11.0));

                ui.columns(2, |cols| {
                    if cols[0].button("DLP Confidential").on_hover_text("Red vertical dashed security lines + confidential host & user warning").clicked() {
                        self.config.text = "CONFIDENTIAL • {user}@{hostname} • {time:%H:%M:%S}".to_string();
                        self.config.opacity = 0.24;
                        self.config.angle_deg = -20.0;
                        self.config.font_size = 18.0;
                        self.config.show_vertical_lines = true;
                        self.config.line_width = 1.5;
                        self.config.line_dashed = true;
                        self.config.line_color_rgba = [255, 80, 80, 210];
                        self.config.show_diagonal_lines = false;
                        self.config.mode = "text".to_string();
                        self.config.spacing_x = 460.0;
                        self.config.spacing_y = 180.0;
                        changed = true;
                    }
                    if cols[1].button("Security Crosshatch").on_hover_text("Intersecting vertical & diagonal lines with dense repeating timestamp matrix").clicked() {
                        self.config.text = "INTERNAL ONLY • {user}@{hostname} • {date:%Y-%m-%d} {time:%H:%M:%S}".to_string();
                        self.config.opacity = 0.22;
                        self.config.angle_deg = -25.0;
                        self.config.font_size = 16.0;
                        self.config.show_vertical_lines = true;
                        self.config.line_width = 1.0;
                        self.config.line_dashed = true;
                        self.config.line_color_rgba = [255, 255, 255, 180];
                        self.config.show_diagonal_lines = true;
                        self.config.diagonal_line_angle = 45.0;
                        self.config.diagonal_line_width = 1.0;
                        self.config.diagonal_line_dashed = true;
                        self.config.diagonal_line_color_rgba = [255, 255, 255, 180];
                        self.config.diagonal_line_spacing = 180.0;
                        self.config.mode = "text".to_string();
                        self.config.spacing_x = 420.0;
                        self.config.spacing_y = 200.0;
                        changed = true;
                    }
                });

                ui.columns(3, |cols| {
                    if cols[0].button("Enterprise Asset Guard").on_hover_text("3D Stamp logo + host/user/date text stack").clicked() {
                        self.config.text = "PROPERTY OF THE COMPANY • {user}@{hostname}".to_string();
                        self.config.opacity = 0.25;
                        self.config.angle_deg = -15.0;
                        self.config.font_size = 17.0;
                        self.config.show_vertical_lines = true;
                        self.config.line_width = 1.2;
                        self.config.line_dashed = false;
                        self.config.line_color_rgba = [233, 84, 32, 190];
                        self.config.show_diagonal_lines = false;
                        self.config.mode = "both".to_string();
                        self.config.image_path = None;
                        self.config.image_scale = 1.0;
                        self.config.spacing_x = 480.0;
                        self.config.spacing_y = 240.0;
                        changed = true;
                    }
                    if cols[1].button("Forensic Microgrid").on_hover_text("Ultra-dense repeating micro-text with dual intersecting lines").clicked() {
                        self.config.text = "{user} • {hostname} • {time:%H:%M:%S}".to_string();
                        self.config.opacity = 0.28;
                        self.config.angle_deg = 0.0;
                        self.config.font_size = 11.0;
                        self.config.spacing_x = 220.0;
                        self.config.spacing_y = 80.0;
                        self.config.stagger_offset = 40.0;
                        self.config.show_vertical_lines = true;
                        self.config.line_width = 0.8;
                        self.config.line_dashed = false;
                        self.config.line_color_rgba = [255, 255, 255, 120];
                        self.config.show_diagonal_lines = true;
                        self.config.diagonal_line_angle = -45.0;
                        self.config.diagonal_line_width = 0.8;
                        self.config.diagonal_line_dashed = false;
                        self.config.diagonal_line_color_rgba = [255, 255, 255, 120];
                        self.config.diagonal_line_spacing = 110.0;
                        self.config.mode = "text".to_string();
                        changed = true;
                    }
                    if cols[2].button("Clean Streamer").on_hover_text("Subtle streaming watermark without lines").clicked() {
                        self.config.text = "LIVE STREAM • @{user}".to_string();
                        self.config.opacity = 0.16;
                        self.config.angle_deg = -18.0;
                        self.config.font_size = 22.0;
                        self.config.show_vertical_lines = false;
                        self.config.show_diagonal_lines = false;
                        self.config.mode = "text".to_string();
                        self.config.spacing_x = 480.0;
                        self.config.spacing_y = 260.0;
                        changed = true;
                    }
                });
            });

            // 2. Mode & Content (Text, Image Logo, or Both)
            ui.group(|ui| {
                ui.label(egui::RichText::new("WATERMARK CONTENT & MODE").strong().size(11.5).color(egui::Color32::from_rgb(233, 84, 32)));

                ui.horizontal(|ui| {
                    ui.label("Watermark Type:");
                    if ui.selectable_value(&mut self.config.mode, "text".to_string(), "Text Only").clicked() {
                        changed = true;
                    }
                    if ui.selectable_value(&mut self.config.mode, "image".to_string(), "Logo / Image").clicked() {
                        changed = true;
                    }
                    if ui.selectable_value(&mut self.config.mode, "both".to_string(), "Logo + Text").clicked() {
                        changed = true;
                    }
                });

                if self.config.mode == "text" || self.config.mode == "both" {
                    ui.label("Watermark Text Template:");
                    if ui.text_edit_singleline(&mut self.config.text).changed() {
                        changed = true;
                    }

                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new("Insert Token:").weak().size(11.0));
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
                }

                if self.config.mode == "image" || self.config.mode == "both" {
                    ui.separator();
                    ui.label("Logo / Image Settings:");
                    let mut path_str = self.config.image_path.clone().unwrap_or_default();
                    ui.horizontal(|ui| {
                        ui.label("File Path:");
                        if ui.add(egui::TextEdit::singleline(&mut path_str).hint_text("Leave blank for built-in 3D stamp icon")).changed() {
                            self.config.image_path = if path_str.trim().is_empty() {
                                None
                            } else {
                                Some(path_str.trim().to_string())
                            };
                            changed = true;
                        }
                        if self.config.image_path.is_some() {
                            if ui.button("Reset to Default Stamp").clicked() {
                                self.config.image_path = None;
                                changed = true;
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Logo Scale:");
                        if ui.add(egui::Slider::new(&mut self.config.image_scale, 0.2..=3.0).suffix("x")).changed() {
                            changed = true;
                        }
                    });
                }
            });

            // 3. Security Lines (Vertical & Diagonal)
            ui.group(|ui| {
                ui.label(egui::RichText::new("SECURITY LINES (VERTICAL & DIAGONAL)").strong().size(11.5).color(egui::Color32::from_rgb(233, 84, 32)));

                ui.columns(2, |cols| {
                    // Vertical Security Lines
                    cols[0].group(|ui| {
                        ui.label(egui::RichText::new("Vertical Column Lines").strong());
                        if ui.checkbox(&mut self.config.show_vertical_lines, "Enable Vertical Lines").changed() {
                            changed = true;
                        }
                        if self.config.show_vertical_lines {
                            if ui.checkbox(&mut self.config.line_dashed, "Dashed Pattern").changed() {
                                changed = true;
                            }
                            ui.horizontal(|ui| {
                                ui.label("Width:");
                                if ui.add(egui::Slider::new(&mut self.config.line_width, 0.5..=5.0).suffix(" px")).changed() {
                                    changed = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Color:");
                                let mut c = [
                                    self.config.line_color_rgba[0] as f32 / 255.0,
                                    self.config.line_color_rgba[1] as f32 / 255.0,
                                    self.config.line_color_rgba[2] as f32 / 255.0,
                                    self.config.line_color_rgba[3] as f32 / 255.0,
                                ];
                                if ui.color_edit_button_rgba_unmultiplied(&mut c).changed() {
                                    self.config.line_color_rgba = [
                                        (c[0] * 255.0) as u8,
                                        (c[1] * 255.0) as u8,
                                        (c[2] * 255.0) as u8,
                                        (c[3] * 255.0) as u8,
                                    ];
                                    changed = true;
                                }
                            });
                        }
                    });

                    // Diagonal Crosshatch Lines
                    cols[1].group(|ui| {
                        ui.label(egui::RichText::new("Diagonal Security Lines").strong());
                        if ui.checkbox(&mut self.config.show_diagonal_lines, "Enable Diagonal Lines").changed() {
                            changed = true;
                        }
                        if self.config.show_diagonal_lines {
                            if ui.checkbox(&mut self.config.diagonal_line_dashed, "Dashed Pattern").changed() {
                                changed = true;
                            }
                            ui.horizontal(|ui| {
                                ui.label("Angle:");
                                if ui.add(egui::Slider::new(&mut self.config.diagonal_line_angle, -80.0..=80.0).suffix("°")).changed() {
                                    changed = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Spacing:");
                                if ui.add(egui::Slider::new(&mut self.config.diagonal_line_spacing, 50.0..=500.0).suffix(" px")).changed() {
                                    changed = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Width:");
                                if ui.add(egui::Slider::new(&mut self.config.diagonal_line_width, 0.5..=5.0).suffix(" px")).changed() {
                                    changed = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Color:");
                                let mut c = [
                                    self.config.diagonal_line_color_rgba[0] as f32 / 255.0,
                                    self.config.diagonal_line_color_rgba[1] as f32 / 255.0,
                                    self.config.diagonal_line_color_rgba[2] as f32 / 255.0,
                                    self.config.diagonal_line_color_rgba[3] as f32 / 255.0,
                                ];
                                if ui.color_edit_button_rgba_unmultiplied(&mut c).changed() {
                                    self.config.diagonal_line_color_rgba = [
                                        (c[0] * 255.0) as u8,
                                        (c[1] * 255.0) as u8,
                                        (c[2] * 255.0) as u8,
                                        (c[3] * 255.0) as u8,
                                    ];
                                    changed = true;
                                }
                            });
                        }
                    });
                });
            });

            // 4. Typography & Geometry
            ui.group(|ui| {
                ui.label(egui::RichText::new("TYPOGRAPHY & GEOMETRY").strong().size(11.5).color(egui::Color32::from_rgb(233, 84, 32)));

                // Font Family Dropdown
                ui.horizontal(|ui| {
                    ui.label("Font Family:");
                    egui::ComboBox::from_id_salt("font_family_combo")
                        .selected_text(&self.config.font_family)
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(&mut self.config.font_family, "Liberation Sans".to_string(), "Liberation Sans").clicked() {
                                changed = true;
                            }
                            if ui.selectable_value(&mut self.config.font_family, "Roboto".to_string(), "Roboto").clicked() {
                                changed = true;
                            }
                            if ui.selectable_value(&mut self.config.font_family, "Roboto Slab".to_string(), "Roboto Slab").clicked() {
                                changed = true;
                            }
                            if ui.selectable_value(&mut self.config.font_family, "Liberation Mono".to_string(), "Liberation Mono").clicked() {
                                changed = true;
                            }
                            if ui.selectable_value(&mut self.config.font_family, "Liberation Serif".to_string(), "Liberation Serif").clicked() {
                                changed = true;
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Font Size:");
                    if ui.add(egui::Slider::new(&mut self.config.font_size, 10.0..=64.0).suffix(" px")).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Rotation Angle:");
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
                    if ui.button("-45°").clicked() {
                        self.config.angle_deg = -45.0;
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
                    if ui.add(egui::Slider::new(&mut self.config.spacing_y, 60.0..=600.0).suffix(" px")).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Stagger Offset:");
                    if ui.add(egui::Slider::new(&mut self.config.stagger_offset, 0.0..=400.0).suffix(" px")).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Stroke Outline:");
                    if ui.add(egui::Slider::new(&mut self.config.stroke_width, 0.0..=5.0).suffix(" px")).changed() {
                        changed = true;
                    }
                });
            });

            // 5. Colors & Contrast
            ui.group(|ui| {
                ui.label(egui::RichText::new("COLORS & CONTRAST").strong().size(11.5).color(egui::Color32::from_rgb(233, 84, 32)));

                ui.horizontal(|ui| {
                    ui.label("Master Opacity:");
                    let mut op = self.config.opacity * 100.0;
                    if ui.add(egui::Slider::new(&mut op, 2.0..=90.0).suffix("%")).changed() {
                        self.config.opacity = op / 100.0;
                        changed = true;
                    }
                });

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

                    ui.label("Stroke Outline Color:");
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

            // 6. Live Mini Preview Canvas
            ui.group(|ui| {
                ui.label(egui::RichText::new("LIVE PATTERN PREVIEW").strong().size(11.5).color(egui::Color32::from_rgb(233, 84, 32)));

                let (response, painter) = ui.allocate_painter(egui::vec2(ui.available_width(), 100.0), egui::Sense::hover());
                let rect = response.rect;

                // Dark desktop background
                painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(18, 20, 24));

                // Paint vertical lines simulation
                if self.config.show_vertical_lines {
                    let v_step = (self.config.spacing_x * 0.25).max(30.0);
                    let mut vx = rect.left() + 20.0;
                    while vx < rect.right() {
                        painter.line_segment(
                            [egui::pos2(vx, rect.top()), egui::pos2(vx, rect.bottom())],
                            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(200, 200, 200, 50)),
                        );
                        vx += v_step;
                    }
                }

                // Paint diagonal lines simulation
                if self.config.show_diagonal_lines {
                    let d_step = (self.config.diagonal_line_spacing * 0.3).max(25.0);
                    let mut dx = rect.left() - 100.0;
                    while dx < rect.right() + 100.0 {
                        painter.line_segment(
                            [egui::pos2(dx, rect.top()), egui::pos2(dx + 100.0, rect.bottom())],
                            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 120, 60, 45)),
                        );
                        dx += d_step;
                    }
                }

                // Sample text badge
                let resolved = crate::tokens::resolve_tokens(&self.config.text);
                let display_sample = if resolved.len() > 36 {
                    format!("{}...", &resolved[..33])
                } else {
                    resolved
                };
                let text_color = egui::Color32::from_rgba_unmultiplied(
                    self.config.color_rgba[0],
                    self.config.color_rgba[1],
                    self.config.color_rgba[2],
                    ((self.config.opacity * 255.0) as u8).max(40),
                );
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{} (angle: {:.0}°)", display_sample, self.config.angle_deg),
                    egui::FontId::proportional(14.0),
                    text_color,
                );
            });

            ui.add_space(6.0);
            ui.label(egui::RichText::new(&self.status_msg).italics());
        });

        if changed {
            self.notify_daemon_reload();
        }
    }
}

pub fn run_gui() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = egui::ViewportBuilder::default()
        .with_inner_size([720.0, 840.0])
        .with_min_inner_size([550.0, 600.0])
        .with_title("Deskstamp Studio - Settings");

    if let Some(icon) = load_app_icon() {
        builder = builder.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport: builder,
        ..Default::default()
    };

    eframe::run_native(
        "Deskstamp Studio",
        native_options,
        Box::new(|cc| Ok(Box::new(SettingsApp::new(cc)))),
    ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// COSMIC Quick Menu Applet Popover
// ---------------------------------------------------------------------------

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
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Color32::TRANSPARENT.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.last_check_frame += 1;
        if self.last_check_frame % 60 == 0 {
            self.daemon_running = send_command(&IpcCommand::Status).is_ok();
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }

        let mut changed = false;

        if self.logo.is_none() {
            self.logo = Some(load_logo_texture(ui.ctx()));
        }

        // Native COSMIC Popover Card: dark rounded card with subtle 1px border
        let card_frame = egui::Frame::new()
            .fill(egui::Color32::from_rgb(30, 32, 36))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(53, 57, 66)))
            .corner_radius(14.0)
            .inner_margin(egui::Margin::same(14));

        card_frame.show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);

            // Popover Header: Icon + Title + Status + Close
            ui.horizontal(|ui| {
                if let Some(logo) = &self.logo {
                    ui.image(egui::load::SizedTexture::new(logo.id(), egui::vec2(28.0, 28.0)));
                }
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Deskstamp").strong().size(15.0).color(egui::Color32::WHITE));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(egui::RichText::new("✕").size(13.0)).frame(false)).clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if self.config.active && self.daemon_running {
                        ui.colored_label(egui::Color32::from_rgb(80, 220, 120), "● Active");
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(180, 185, 195), "○ Off");
                    }
                });
            });

            ui.separator();

            // 1. Primary Toggle: Watermark Enabled (Pop!_OS COSMIC toggle row)
            if cosmic_row_toggle(ui, "Watermark Overlay", Some("Display watermark on desktop"), &mut self.config.active) {
                changed = true;
                if self.daemon_running {
                    let _ = send_command(&IpcCommand::Toggle);
                }
            }

            ui.separator();

            // 2. Security Lines Toggles
            if cosmic_row_toggle(ui, "Vertical Lines", Some("Column security borders"), &mut self.config.show_vertical_lines) {
                changed = true;
            }

            if cosmic_row_toggle(ui, "Diagonal Lines", Some("Crosshatch diagonal lines"), &mut self.config.show_diagonal_lines) {
                changed = true;
            }

            ui.separator();

            // 3. Mode Selection (Segmented Pills)
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Mode:").size(12.0).weak());
                if ui.selectable_value(&mut self.config.mode, "text".to_string(), "Text").clicked() {
                    changed = true;
                }
                if ui.selectable_value(&mut self.config.mode, "image".to_string(), "Logo").clicked() {
                    changed = true;
                }
                if ui.selectable_value(&mut self.config.mode, "both".to_string(), "Both").clicked() {
                    changed = true;
                }
            });

            // 4. Quick Presets Grid
            ui.label(egui::RichText::new("PRESETS").strong().size(11.0).color(egui::Color32::from_rgb(233, 84, 32)));
            ui.columns(2, |cols| {
                if cols[0].add_sized([cols[0].available_width(), 26.0], egui::Button::new("DLP Guard")).clicked() {
                    self.config.text = "CONFIDENTIAL • {user}@{hostname} • {time:%H:%M:%S}".to_string();
                    self.config.opacity = 0.24;
                    self.config.angle_deg = -20.0;
                    self.config.show_vertical_lines = true;
                    self.config.line_width = 1.5;
                    self.config.line_dashed = true;
                    self.config.line_color_rgba = [255, 80, 80, 210];
                    self.config.show_diagonal_lines = false;
                    self.config.mode = "text".to_string();
                    changed = true;
                }
                if cols[1].add_sized([cols[1].available_width(), 26.0], egui::Button::new("Crosshatch")).clicked() {
                    self.config.text = "INTERNAL ONLY • {user}@{hostname}".to_string();
                    self.config.opacity = 0.22;
                    self.config.angle_deg = -25.0;
                    self.config.show_vertical_lines = true;
                    self.config.line_dashed = true;
                    self.config.show_diagonal_lines = true;
                    self.config.diagonal_line_angle = 45.0;
                    self.config.diagonal_line_dashed = true;
                    self.config.mode = "text".to_string();
                    changed = true;
                }
            });
            ui.columns(2, |cols| {
                if cols[0].add_sized([cols[0].available_width(), 26.0], egui::Button::new("Asset Guard")).clicked() {
                    self.config.text = "PROPERTY OF {hostname}".to_string();
                    self.config.opacity = 0.25;
                    self.config.angle_deg = -15.0;
                    self.config.show_vertical_lines = true;
                    self.config.show_diagonal_lines = false;
                    self.config.mode = "both".to_string();
                    self.config.image_path = None;
                    self.config.image_scale = 1.0;
                    changed = true;
                }
                if cols[1].add_sized([cols[1].available_width(), 26.0], egui::Button::new("Clean Stream")).clicked() {
                    self.config.text = "LIVE STREAM • @{user}".to_string();
                    self.config.opacity = 0.16;
                    self.config.angle_deg = -18.0;
                    self.config.show_vertical_lines = false;
                    self.config.show_diagonal_lines = false;
                    self.config.mode = "text".to_string();
                    changed = true;
                }
            });

            // 5. Opacity Slider
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Opacity:").size(12.0));
                let mut op = self.config.opacity * 100.0;
                if ui.add(egui::Slider::new(&mut op, 2.0..=70.0).suffix("%")).changed() {
                    self.config.opacity = op / 100.0;
                    changed = true;
                }
            });

            ui.separator();

            // 6. Action Footer: Deskstamp Studio settings... (Matching COSMIC bottom action link)
            let studio_btn = egui::Button::new(
                egui::RichText::new("Deskstamp Studio settings...")
                    .size(13.0)
                    .color(egui::Color32::from_rgb(220, 225, 235)),
            )
            .frame(false);

            if ui.add_sized([ui.available_width(), 28.0], studio_btn).clicked() {
                if let Ok(exe) = std::env::current_exe() {
                    let _ = std::process::Command::new(exe).arg("settings").spawn();
                }
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
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
                        // Already open! Toggle OFF instantly
                        libc::kill(pid, libc::SIGTERM);
                        let _ = std::fs::remove_file(&pid_path);
                        return Ok(());
                    }
                }
            }
        }
        let _ = std::fs::remove_file(&pid_path);
    }

    let current_pid = std::process::id();
    let _ = std::fs::write(&pid_path, current_pid.to_string());
    let _guard = MenuPidGuard;

    let mut builder = egui::ViewportBuilder::default()
        .with_inner_size([350.0, 520.0])
        .with_min_inner_size([320.0, 480.0])
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top()
        .with_resizable(false)
        .with_title("Deskstamp Menu");

    if let Some(icon) = load_app_icon() {
        builder = builder.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport: builder,
        ..Default::default()
    };

    eframe::run_native(
        "Deskstamp Menu",
        native_options,
        Box::new(|cc| Ok(Box::new(QuickMenuApp::new(cc)))),
    ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    Ok(())
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
