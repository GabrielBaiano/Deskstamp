use crate::config::WatermarkConfig;
use crate::ipc::{send_command, IpcCommand};
use eframe::egui;

/// Native Pop!_OS COSMIC toggle switch with vibrant orange accent (#e95420)
fn cosmic_switch(ui: &mut egui::Ui, value: &mut bool) -> egui::Response {
    let desired_size = egui::vec2(44.0, 24.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *value);
        let off_color = egui::Color32::from_rgb(54, 58, 68);
        let on_color = egui::Color32::from_rgb(233, 84, 32); // Authentic COSMIC Orange
        let bg_color = off_color.lerp_to_gamma(on_color, how_on);

        let radius = rect.height() * 0.5;
        ui.painter().rect_filled(rect, radius, bg_color);

        let knob_radius = radius - 3.0;
        let knob_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let knob_center = egui::pos2(knob_x, rect.center().y);
        ui.painter().circle_filled(knob_center, knob_radius, egui::Color32::WHITE);
    }

    response
}

/// COSMIC Settings row helper: label on left, control on right
fn cosmic_row<R>(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: Option<&str>,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let row_height = if subtitle.is_some() { 36.0 } else { 30.0 };
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), row_height),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(title).size(13.5).strong().color(egui::Color32::from_rgb(235, 238, 245)));
                if let Some(sub) = subtitle {
                    ui.label(egui::RichText::new(sub).size(11.0).color(egui::Color32::from_rgb(155, 160, 175)));
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add_contents).inner
        },
    ).inner
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

        // Pop!_OS COSMIC Theme Colors
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

        let card_bg = egui::Color32::from_rgb(34, 37, 45);
        let card_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(52, 56, 68));

        // -------------------------------------------------------------------
        // Top COSMIC Header
        // -------------------------------------------------------------------
        ui.horizontal(|ui| {
            if let Some(logo) = &self.logo {
                ui.image(egui::load::SizedTexture::new(logo.id(), egui::vec2(32.0, 32.0)));
            }
            ui.label(egui::RichText::new("Deskstamp").strong().size(18.0).color(egui::Color32::WHITE));

            ui.add_space(16.0);

            // Tab Buttons (COSMIC Pill Style)
            let gen_active = self.selected_tab == 0;
            let gen_btn = egui::Button::new(
                egui::RichText::new("General")
                    .size(13.0)
                    .strong()
                    .color(if gen_active { egui::Color32::WHITE } else { egui::Color32::from_rgb(180, 185, 200) }),
            )
            .fill(if gen_active { egui::Color32::from_rgb(233, 84, 32) } else { egui::Color32::from_rgb(42, 45, 55) })
            .corner_radius(egui::CornerRadius::same(14));

            if ui.add_sized([90.0, 28.0], gen_btn).clicked() {
                self.selected_tab = 0;
            }

            let about_active = self.selected_tab == 1;
            let about_btn = egui::Button::new(
                egui::RichText::new("About")
                    .size(13.0)
                    .strong()
                    .color(if about_active { egui::Color32::WHITE } else { egui::Color32::from_rgb(180, 185, 200) }),
            )
            .fill(if about_active { egui::Color32::from_rgb(233, 84, 32) } else { egui::Color32::from_rgb(42, 45, 55) })
            .corner_radius(egui::CornerRadius::same(14));

            if ui.add_sized([80.0, 28.0], about_btn).clicked() {
                self.selected_tab = 1;
            }

            // Right status indicator
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.config.active && self.daemon_running {
                    ui.colored_label(egui::Color32::from_rgb(72, 199, 116), "● Overlay Active");
                } else if !self.config.active {
                    ui.colored_label(egui::Color32::from_rgb(240, 180, 70), "○ Watermark Hidden");
                } else {
                    ui.colored_label(egui::Color32::from_rgb(240, 90, 90), "✕ Daemon Offline");
                }
            });
        });

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);

        // -------------------------------------------------------------------
        // Main Content Area
        // -------------------------------------------------------------------
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 14.0);

                if self.selected_tab == 0 {
                    // Card 1: Master Enable Toggle
                    let card1 = egui::Frame::new()
                        .fill(card_bg)
                        .stroke(card_stroke)
                        .corner_radius(egui::CornerRadius::same(10))
                        .inner_margin(egui::Margin::symmetric(16, 12));

                    card1.show(ui, |ui| {
                        cosmic_row(ui, "Enable Desktop Watermark", Some("Display security watermark overlay"), |ui| {
                            if cosmic_switch(ui, &mut self.config.active).changed() {
                                changed = true;
                                if self.daemon_running {
                                    let _ = send_command(&IpcCommand::Toggle);
                                }
                            }
                        });

                        ui.separator();

                        cosmic_row(ui, "Stealth Mode (Videos & Streams Only)", Some("Nearly invisible to your eyes, but captured in recorded videos"), |ui| {
                            if cosmic_switch(ui, &mut self.config.stealth_mode).changed() {
                                changed = true;
                            }
                        });
                    });

                    // Card 2: Appearance & Geometry
                    let card2 = egui::Frame::new()
                        .fill(card_bg)
                        .stroke(card_stroke)
                        .corner_radius(egui::CornerRadius::same(10))
                        .inner_margin(egui::Margin::symmetric(16, 12));

                    card2.show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 8.0);

                        // Rotation
                        cosmic_row(ui, "Rotation", Some("Angle of watermark repeat grid"), |ui| {
                            ui.horizontal(|ui| {
                                if ui.button("0°").clicked() {
                                    self.config.angle_deg = 0.0;
                                    changed = true;
                                }
                                if ui.button("-20°").clicked() {
                                    self.config.angle_deg = -20.0;
                                    changed = true;
                                }
                                if ui.add_sized([160.0, 24.0], egui::Slider::new(&mut self.config.angle_deg, -90.0..=90.0).suffix("°")).changed() {
                                    changed = true;
                                }
                            });
                        });

                        ui.separator();

                        // Opacity
                        cosmic_row(ui, "Opacity (Transparency)", Some("Subtle watermark visibility"), |ui| {
                            let mut op_percent = self.config.opacity * 100.0;
                            if ui.add_sized([220.0, 24.0], egui::Slider::new(&mut op_percent, 2.0..=100.0).suffix("%")).changed() {
                                self.config.opacity = op_percent / 100.0;
                                changed = true;
                            }
                        });

                        ui.separator();

                        // Font Size / Scale
                        cosmic_row(ui, "Size", Some("Font size of watermark text"), |ui| {
                            if ui.add_sized([220.0, 24.0], egui::Slider::new(&mut self.config.font_size, 10.0..=64.0).suffix(" px")).changed() {
                                changed = true;
                            }
                        });

                        ui.separator();

                        // Spacing
                        cosmic_row(ui, "Spacing", Some("Distance between repeating elements"), |ui| {
                            let mut spacing = self.config.spacing_x;
                            if ui.add_sized([220.0, 24.0], egui::Slider::new(&mut spacing, 120.0..=800.0).suffix(" px")).changed() {
                                self.config.spacing_x = spacing;
                                self.config.spacing_y = (spacing * 0.45).max(70.0);
                                self.config.stagger_offset = spacing * 0.25;
                                changed = true;
                            }
                        });

                        ui.separator();

                        // Text Color
                        cosmic_row(ui, "Color", Some("Watermark text, lines, and monochrome logo color"), |ui| {
                            let mut c = [
                                self.config.color_rgba[0] as f32 / 255.0,
                                self.config.color_rgba[1] as f32 / 255.0,
                                self.config.color_rgba[2] as f32 / 255.0,
                            ];
                            if ui.color_edit_button_rgb(&mut c).changed() {
                                self.config.color_rgba[0] = (c[0] * 255.0) as u8;
                                self.config.color_rgba[1] = (c[1] * 255.0) as u8;
                                self.config.color_rgba[2] = (c[2] * 255.0) as u8;
                                self.config.line_color_rgba = self.config.color_rgba;
                                changed = true;
                            }
                        });
                    });

                    // Card 3: Watermark Content & Framing Lines
                    let card3 = egui::Frame::new()
                        .fill(card_bg)
                        .stroke(card_stroke)
                        .corner_radius(egui::CornerRadius::same(10))
                        .inner_margin(egui::Margin::symmetric(16, 12));

                    card3.show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 8.0);

                        // Show Text
                        cosmic_row(ui, "Show Text", None, |ui| {
                            if cosmic_switch(ui, &mut self.config.show_text).changed() {
                                changed = true;
                            }
                        });

                        if self.config.show_text {
                            ui.separator();
                            cosmic_row(ui, "Watermark Text", None, |ui| {
                                if ui.add_sized([240.0, 26.0], egui::TextEdit::singleline(&mut self.config.text)).changed() {
                                    changed = true;
                                }
                            });
                        }

                        ui.separator();

                        // Show Icon
                        cosmic_row(ui, "Show Icon / Image", None, |ui| {
                            if cosmic_switch(ui, &mut self.config.show_icon).changed() {
                                changed = true;
                            }
                        });

                        if self.config.show_icon {
                            ui.separator();
                            cosmic_row(ui, "Custom Icon / Logo", Some("Select image from disk or use built-in icon"), |ui| {
                                ui.horizontal(|ui| {
                                    if ui.button("Browse...").clicked() {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .add_filter("Images", &["png", "jpg", "jpeg", "svg"])
                                            .pick_file()
                                        {
                                            self.config.image_path = Some(path.to_string_lossy().to_string());
                                            changed = true;
                                        }
                                    }
                                    if self.config.image_path.is_some() {
                                        if ui.button("Reset").clicked() {
                                            self.config.image_path = None;
                                            changed = true;
                                        }
                                    }
                                    let mut path_str = self.config.image_path.clone().unwrap_or_default();
                                    if ui.add_sized([160.0, 26.0], egui::TextEdit::singleline(&mut path_str).hint_text("Default 3D icon")).changed() {
                                        self.config.image_path = if path_str.trim().is_empty() { None } else { Some(path_str.trim().to_string()) };
                                        changed = true;
                                    }
                                });
                            });

                            ui.separator();
                            cosmic_row(ui, "Icon Size", Some("Scale of custom or built-in icon"), |ui| {
                                let mut scale_pct = self.config.image_scale * 100.0;
                                if ui.add_sized([220.0, 24.0], egui::Slider::new(&mut scale_pct, 20.0..=300.0).suffix("%")).changed() {
                                    self.config.image_scale = scale_pct / 100.0;
                                    changed = true;
                                }
                            });

                            ui.separator();
                            cosmic_row(ui, "Monochrome Logo", Some("Tint icon to match watermark text color"), |ui| {
                                if cosmic_switch(ui, &mut self.config.monochrome_icon).changed() {
                                    changed = true;
                                }
                            });
                        }

                        ui.separator();

                        // Lines Alongside Text (with clean gap)
                        cosmic_row(ui, "Lines Alongside Text", Some("Decorative lines framed with space around text"), |ui| {
                            if cosmic_switch(ui, &mut self.config.show_lines_next_to_text).changed() {
                                changed = true;
                            }
                        });

                        if self.config.show_lines_next_to_text {
                            ui.separator();
                            cosmic_row(ui, "Line Length", None, |ui| {
                                if ui.add_sized([220.0, 24.0], egui::Slider::new(&mut self.config.line_length, 20.0..=160.0).suffix(" px")).changed() {
                                    changed = true;
                                }
                            });

                            ui.separator();
                            cosmic_row(ui, "Spacing to Text (Gap)", Some("Prevents lines from cutting through text"), |ui| {
                                if ui.add_sized([220.0, 24.0], egui::Slider::new(&mut self.config.line_gap, 6.0..=40.0).suffix(" px")).changed() {
                                    changed = true;
                                }
                            });

                            ui.separator();
                            cosmic_row(ui, "Dashed Pattern", None, |ui| {
                                if cosmic_switch(ui, &mut self.config.line_dashed).changed() {
                                    changed = true;
                                }
                            });
                        }
                    });

                    // Card 4: Live Desktop Preview
                    let card4 = egui::Frame::new()
                        .fill(card_bg)
                        .stroke(card_stroke)
                        .corner_radius(egui::CornerRadius::same(10))
                        .inner_margin(egui::Margin::symmetric(16, 12));

                    card4.show(ui, |ui| {
                        ui.label(egui::RichText::new("Live Pattern Preview").strong().size(12.5).color(egui::Color32::from_rgb(233, 84, 32)));
                        ui.add_space(4.0);

                        let preview_height = 80.0;
                        let (response, painter) = ui.allocate_painter(egui::vec2(ui.available_width(), preview_height), egui::Sense::hover());
                        let rect = response.rect;
                        let center = rect.center();

                        // Dark wallpaper background
                        painter.rect_filled(rect, 6.0, egui::Color32::from_rgb(18, 20, 24));

                        let sample_text = if self.config.show_text && !self.config.text.is_empty() {
                            crate::tokens::resolve_tokens(&self.config.text)
                        } else {
                            String::new()
                        };

                        let alpha = ((self.config.opacity * 255.0) as u8).max(50);
                        let col = egui::Color32::from_rgba_unmultiplied(
                            self.config.color_rgba[0],
                            self.config.color_rgba[1],
                            self.config.color_rgba[2],
                            alpha,
                        );

                        let text_w = sample_text.len() as f32 * 7.5;
                        let icon_w = if self.config.show_icon { 16.0 } else { 0.0 };
                        let icon_gap = if self.config.show_icon && self.config.show_text && !sample_text.is_empty() { 8.0 } else { 0.0 };
                        let total_content_w = icon_w + icon_gap + text_w;

                        let line_len = self.config.line_length.clamp(20.0, 70.0);
                        let gap = self.config.line_gap.clamp(6.0, 24.0);

                        if self.config.show_lines_next_to_text {
                            let left_end = center.x - total_content_w * 0.5 - gap;
                            let left_start = left_end - line_len;
                            painter.line_segment([egui::pos2(left_start, center.y), egui::pos2(left_end, center.y)], egui::Stroke::new(1.5, col));

                            let right_start = center.x + total_content_w * 0.5 + gap;
                            let right_end = right_start + line_len;
                            painter.line_segment([egui::pos2(right_start, center.y), egui::pos2(right_end, center.y)], egui::Stroke::new(1.5, col));
                        }

                        let mut cx = center.x - total_content_w * 0.5;
                        if self.config.show_icon {
                            painter.circle_filled(egui::pos2(cx + 8.0, center.y), 6.0, col);
                            cx += icon_w + icon_gap;
                        }

                        if self.config.show_text && !sample_text.is_empty() {
                            painter.text(
                                egui::pos2(cx + text_w * 0.5, center.y),
                                egui::Align2::CENTER_CENTER,
                                &sample_text,
                                egui::FontId::proportional(14.0),
                                col,
                            );
                        }
                    });

                } else {
                    // About Tab
                    let about_card = egui::Frame::new()
                        .fill(card_bg)
                        .stroke(card_stroke)
                        .corner_radius(egui::CornerRadius::same(10))
                        .inner_margin(egui::Margin::symmetric(20, 16));

                    about_card.show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 10.0);
                        ui.horizontal(|ui| {
                            if let Some(logo) = &self.logo {
                                ui.image(egui::load::SizedTexture::new(logo.id(), egui::vec2(40.0, 40.0)));
                            }
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Deskstamp").strong().size(18.0).color(egui::Color32::WHITE));
                                ui.label(egui::RichText::new("Desktop Watermark Layer for Linux & Pop!_OS COSMIC").weak().size(12.0));
                            });
                        });

                        ui.separator();
                        ui.label("Lightweight, click-through desktop security watermark overlay designed natively for Wayland and Pop!_OS COSMIC.");
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Version: 0.1.0").weak());
                    });
                }
            });

        if changed {
            self.notify_daemon();
        }
    }
}

pub fn run_gui() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = egui::ViewportBuilder::default()
        .with_inner_size([680.0, 700.0])
        .with_min_inner_size([520.0, 480.0])
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
