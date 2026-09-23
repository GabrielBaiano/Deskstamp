use crate::config::WatermarkConfig;
use crate::ipc::{send_command, IpcCommand};
use eframe::egui;

fn cosmic_switch(ui: &mut egui::Ui, value: &mut bool) -> egui::Response {
    let desired_size = egui::vec2(42.0, 22.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *value);
        let off_color = egui::Color32::from_rgb(52, 55, 62);
        let on_color = egui::Color32::from_rgb(46, 125, 246); // Sleek modern blue like screenshot
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

fn switch_row(ui: &mut egui::Ui, title: &str, value: &mut bool) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(title).size(13.5).color(egui::Color32::from_rgb(225, 230, 240)));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if cosmic_switch(ui, value).changed() {
                changed = true;
            }
        });
    });
    changed
}

pub struct SettingsApp {
    config: WatermarkConfig,
    daemon_running: bool,
    selected_tab: usize, // 0 = General, 1 = About
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
            selected_tab: 0,
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
            let res = std::process::Command::new(exe).arg("daemon").spawn();
            if res.is_ok() {
                for _ in 0..15 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    if send_command(&IpcCommand::Status).is_ok() {
                        self.daemon_running = true;
                        return;
                    }
                }
            }
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

        // Clean dark theme matching user's screenshot
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(24, 25, 29);
        visuals.window_fill = egui::Color32::from_rgb(24, 25, 29);
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(8);
        visuals.selection.bg_fill = egui::Color32::from_rgb(46, 125, 246);
        ui.ctx().set_visuals(visuals);

        if self.logo.is_none() {
            self.logo = Some(load_logo_texture(ui.ctx()));
        }

        let total_width = ui.available_width();
        let sidebar_width = 170.0;

        ui.horizontal(|ui| {
            // ---------------------------------------------------------------
            // Left Sidebar (Navigation)
            // ---------------------------------------------------------------
            ui.allocate_ui(egui::vec2(sidebar_width, ui.available_height()), |ui| {
                let sidebar_frame = egui::Frame::new()
                    .fill(egui::Color32::from_rgb(20, 21, 25))
                    .inner_margin(egui::Margin::symmetric(12, 16))
                    .corner_radius(egui::CornerRadius::same(12));

                sidebar_frame.show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 6.0);

                    // Window traffic lights dummy dots for authentic aesthetic
                    ui.horizontal(|ui| {
                        ui.painter().circle_filled(ui.cursor().min + egui::vec2(6.0, 6.0), 5.5, egui::Color32::from_rgb(255, 95, 87));
                        ui.painter().circle_filled(ui.cursor().min + egui::vec2(22.0, 6.0), 5.5, egui::Color32::from_rgb(254, 188, 46));
                        ui.painter().circle_filled(ui.cursor().min + egui::vec2(38.0, 6.0), 5.5, egui::Color32::from_rgb(40, 200, 64));
                        ui.add_space(50.0);
                    });

                    ui.add_space(20.0);

                    // Tab buttons
                    let general_selected = self.selected_tab == 0;
                    let general_btn = egui::Button::new(
                        egui::RichText::new("⚙  General")
                            .size(13.5)
                            .strong()
                            .color(if general_selected { egui::Color32::WHITE } else { egui::Color32::from_rgb(170, 175, 185) }),
                    )
                    .fill(if general_selected { egui::Color32::from_rgb(40, 42, 50) } else { egui::Color32::TRANSPARENT })
                    .corner_radius(egui::CornerRadius::same(8));

                    if ui.add_sized([sidebar_width - 24.0, 32.0], general_btn).clicked() {
                        self.selected_tab = 0;
                    }

                    let about_selected = self.selected_tab == 1;
                    let about_btn = egui::Button::new(
                        egui::RichText::new("ℹ  About")
                            .size(13.5)
                            .strong()
                            .color(if about_selected { egui::Color32::WHITE } else { egui::Color32::from_rgb(170, 175, 185) }),
                    )
                    .fill(if about_selected { egui::Color32::from_rgb(40, 42, 50) } else { egui::Color32::TRANSPARENT })
                    .corner_radius(egui::CornerRadius::same(8));

                    if ui.add_sized([sidebar_width - 24.0, 32.0], about_btn).clicked() {
                        self.selected_tab = 1;
                    }

                    ui.add_space(ui.available_height() - 60.0);

                    // Daemon status
                    if self.daemon_running && self.config.active {
                        ui.colored_label(egui::Color32::from_rgb(80, 220, 120), "● Overlay Active");
                    } else if !self.config.active {
                        ui.colored_label(egui::Color32::from_rgb(220, 180, 80), "○ Hidden");
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(220, 90, 90), "✕ Daemon Off");
                    }
                });
            });

            ui.add_space(8.0);

            // ---------------------------------------------------------------
            // Right Main Content
            // ---------------------------------------------------------------
            let content_width = total_width - sidebar_width - 16.0;
            ui.allocate_ui(egui::vec2(content_width, ui.available_height()), |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 14.0);

                    if self.selected_tab == 0 {
                        // Header
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("General").strong().size(18.0).color(egui::Color32::WHITE));
                        });

                        // Top Master Toggles Card
                        let top_card = egui::Frame::new()
                            .fill(egui::Color32::from_rgb(32, 34, 40))
                            .corner_radius(egui::CornerRadius::same(12))
                            .inner_margin(egui::Margin::symmetric(16, 12));

                        top_card.show(ui, |ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(0.0, 10.0);

                            if switch_row(ui, "Enable Watermark", &mut self.config.active) {
                                changed = true;
                                if self.daemon_running {
                                    let _ = send_command(&IpcCommand::Toggle);
                                }
                            }
                        });

                        // Appearance Card (Rotation, Opacity, Font Size, Spacing, Text Color)
                        let appearance_card = egui::Frame::new()
                            .fill(egui::Color32::from_rgb(32, 34, 40))
                            .corner_radius(egui::CornerRadius::same(12))
                            .inner_margin(egui::Margin::symmetric(16, 14));

                        appearance_card.show(ui, |ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(0.0, 12.0);

                            // Rotation
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Rotation").size(13.5).color(egui::Color32::from_rgb(220, 225, 235)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(egui::Slider::new(&mut self.config.angle_deg, -90.0..=90.0).suffix("°")).changed() {
                                        changed = true;
                                    }
                                });
                            });

                            ui.separator();

                            // Opacity / Transparency
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Opacity").size(13.5).color(egui::Color32::from_rgb(220, 225, 235)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let mut op_percent = self.config.opacity * 100.0;
                                    if ui.add(egui::Slider::new(&mut op_percent, 2.0..=100.0).suffix("%")).changed() {
                                        self.config.opacity = op_percent / 100.0;
                                        changed = true;
                                    }
                                });
                            });

                            ui.separator();

                            // Font Size / Scale
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Font Size").size(13.5).color(egui::Color32::from_rgb(220, 225, 235)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(egui::Slider::new(&mut self.config.font_size, 10.0..=64.0).suffix(" px")).changed() {
                                        changed = true;
                                    }
                                });
                            });

                            ui.separator();

                            // Spacing
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Spacing").size(13.5).color(egui::Color32::from_rgb(220, 225, 235)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let mut avg_spacing = self.config.spacing_x;
                                    if ui.add(egui::Slider::new(&mut avg_spacing, 100.0..=800.0).suffix(" px")).changed() {
                                        self.config.spacing_x = avg_spacing;
                                        self.config.spacing_y = (avg_spacing * 0.45).max(60.0);
                                        self.config.stagger_offset = avg_spacing * 0.25;
                                        changed = true;
                                    }
                                });
                            });

                            ui.separator();

                            // Text Color
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Text Color").size(13.5).color(egui::Color32::from_rgb(220, 225, 235)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let mut c = [
                                        self.config.color_rgba[0] as f32 / 255.0,
                                        self.config.color_rgba[1] as f32 / 255.0,
                                        self.config.color_rgba[2] as f32 / 255.0,
                                    ];
                                    if ui.color_edit_button_rgb(&mut c).changed() {
                                        self.config.color_rgba[0] = (c[0] * 255.0) as u8;
                                        self.config.color_rgba[1] = (c[1] * 255.0) as u8;
                                        self.config.color_rgba[2] = (c[2] * 255.0) as u8;
                                        // Also update line color for clean harmony
                                        self.config.line_color_rgba = self.config.color_rgba;
                                        changed = true;
                                    }
                                });
                            });
                        });

                        // Content Card (Text, Icon, Lines with spacing)
                        let content_card = egui::Frame::new()
                            .fill(egui::Color32::from_rgb(32, 34, 40))
                            .corner_radius(egui::CornerRadius::same(12))
                            .inner_margin(egui::Margin::symmetric(16, 14));

                        content_card.show(ui, |ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(0.0, 12.0);

                            // Show Text toggle
                            if switch_row(ui, "Show Text", &mut self.config.show_text) {
                                changed = true;
                            }

                            if self.config.show_text {
                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Watermark Text").size(13.5).color(egui::Color32::from_rgb(220, 225, 235)));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.add(egui::TextEdit::singleline(&mut self.config.text).desired_width(180.0)).changed() {
                                            changed = true;
                                        }
                                    });
                                });
                            }

                            ui.separator();

                            // Show Icon toggle
                            if switch_row(ui, "Show Icon", &mut self.config.show_icon) {
                                changed = true;
                            }

                            if self.config.show_icon {
                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Icon Path").size(13.5).color(egui::Color32::from_rgb(220, 225, 235)));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let mut path_str = self.config.image_path.clone().unwrap_or_default();
                                        if ui.add(egui::TextEdit::singleline(&mut path_str).hint_text("Default built-in icon").desired_width(180.0)).changed() {
                                            self.config.image_path = if path_str.trim().is_empty() { None } else { Some(path_str.trim().to_string()) };
                                            changed = true;
                                        }
                                    });
                                });
                            }

                            ui.separator();

                            // Lines alongside text (with clean gap so lines don't draw on top of text)
                            if switch_row(ui, "Lines Alongside Text", &mut self.config.show_lines_next_to_text) {
                                changed = true;
                            }

                            if self.config.show_lines_next_to_text {
                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Line Length").size(13.5).color(egui::Color32::from_rgb(220, 225, 235)));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.add(egui::Slider::new(&mut self.config.line_length, 20.0..=160.0).suffix(" px")).changed() {
                                            changed = true;
                                        }
                                    });
                                });

                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Spacing to Text (Gap)").size(13.5).color(egui::Color32::from_rgb(220, 225, 235)));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.add(egui::Slider::new(&mut self.config.line_gap, 6.0..=40.0).suffix(" px")).changed() {
                                            changed = true;
                                        }
                                    });
                                });

                                ui.separator();
                                if switch_row(ui, "Dashed Lines", &mut self.config.line_dashed) {
                                    changed = true;
                                }
                            }
                        });

                        // Mini live preview box
                        let preview_frame = egui::Frame::new()
                            .fill(egui::Color32::from_rgb(18, 19, 22))
                            .corner_radius(egui::CornerRadius::same(12))
                            .inner_margin(egui::Margin::same(12));

                        preview_frame.show(ui, |ui| {
                            let (response, painter) = ui.allocate_painter(egui::vec2(ui.available_width(), 64.0), egui::Sense::hover());
                            let rect = response.rect;
                            let center = rect.center();

                            let sample_text = if self.config.show_text && !self.config.text.is_empty() {
                                crate::tokens::resolve_tokens(&self.config.text)
                            } else {
                                String::new()
                            };

                            let col = egui::Color32::from_rgba_unmultiplied(
                                self.config.color_rgba[0],
                                self.config.color_rgba[1],
                                self.config.color_rgba[2],
                                ((self.config.opacity * 255.0) as u8).max(60),
                            );

                            let text_len = sample_text.len() as f32 * 7.5;
                            let gap = self.config.line_gap.clamp(8.0, 30.0);
                            let line_len = self.config.line_length.clamp(20.0, 80.0);

                            if self.config.show_lines_next_to_text {
                                // Left line
                                painter.line_segment(
                                    [egui::pos2(center.x - text_len * 0.5 - gap - line_len, center.y), egui::pos2(center.x - text_len * 0.5 - gap, center.y)],
                                    egui::Stroke::new(1.5, col),
                                );
                                // Right line
                                painter.line_segment(
                                    [egui::pos2(center.x + text_len * 0.5 + gap, center.y), egui::pos2(center.x + text_len * 0.5 + gap + line_len, center.y)],
                                    egui::Stroke::new(1.5, col),
                                );
                            }

                            let display = if self.config.show_icon && self.config.show_text {
                                format!("● {}", sample_text)
                            } else if self.config.show_icon {
                                "●".to_string()
                            } else {
                                sample_text
                            };

                            painter.text(
                                center,
                                egui::Align2::CENTER_CENTER,
                                display,
                                egui::FontId::proportional(14.0),
                                col,
                            );
                        });

                    } else {
                        // About Tab
                        ui.label(egui::RichText::new("About Deskstamp").strong().size(18.0).color(egui::Color32::WHITE));
                        let about_card = egui::Frame::new()
                            .fill(egui::Color32::from_rgb(32, 34, 40))
                            .corner_radius(egui::CornerRadius::same(12))
                            .inner_margin(egui::Margin::same(16));

                        about_card.show(ui, |ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(0.0, 8.0);
                            ui.label(egui::RichText::new("Deskstamp").strong().size(15.0));
                            ui.label("Lightweight, click-through desktop security watermark layer designed natively for Linux & Pop!_OS COSMIC.");
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("Version: 0.1.0").weak());
                        });
                    }
                });
            });
        });

        if changed {
            self.notify_daemon();
        }
    }
}

pub fn run_gui() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = egui::ViewportBuilder::default()
        .with_inner_size([640.0, 560.0])
        .with_min_inner_size([540.0, 480.0])
        .with_title("Deskstamp Settings");

    if let Some(icon) = load_app_icon() {
        builder = builder.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport: builder,
        ..Default::default()
    };

    eframe::run_native(
        "Deskstamp Settings",
        native_options,
        Box::new(|cc| Ok(Box::new(SettingsApp::new(cc)))),
    ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    Ok(())
}

pub fn run_quick_menu() -> Result<(), Box<dyn std::error::Error>> {
    run_gui()
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
