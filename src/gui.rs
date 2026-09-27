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
    new_ws_name: String,
    new_ws_text: String,
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
            new_ws_name: String::new(),
            new_ws_text: String::new(),
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
        if self.last_check_frame.is_multiple_of(60) {
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
                    });


                    // Card 2: Appearance & Geometry
                    let card2 = egui::Frame::new()
                        .fill(card_bg)
                        .stroke(card_stroke)
                        .corner_radius(egui::CornerRadius::same(10))
                        .inner_margin(egui::Margin::symmetric(16, 12));

                    card2.show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 8.0);

                        // Layout Mode (Grid vs Corner)
                        cosmic_row(ui, "Layout Style", Some("Watermark placement across your screen"), |ui| {
                            ui.horizontal(|ui| {
                                let is_grid = self.config.layout == "grid";
                                let is_corner = self.config.layout == "corner";
                                if ui.selectable_label(is_grid, "Full Screen Grid").clicked() {
                                    self.config.layout = "grid".to_string();
                                    changed = true;
                                }
                                if ui.selectable_label(is_corner, "Discreet Corner (Windows Style)").clicked() {
                                    self.config.layout = "corner".to_string();
                                    changed = true;
                                }
                            });
                        });

                        if self.config.is_corner_mode() {
                            ui.separator();
                            cosmic_row(ui, "Corner Position", Some("Anchor position on display"), |ui| {
                                ui.horizontal(|ui| {
                                    for (pos_id, label) in [
                                        ("bottom_right", "Bottom-Right"),
                                        ("bottom_left", "Bottom-Left"),
                                        ("top_right", "Top-Right"),
                                        ("top_left", "Top-Left"),
                                    ] {
                                        if ui.selectable_label(self.config.corner_position == pos_id, label).clicked() {
                                            self.config.corner_position = pos_id.to_string();
                                            changed = true;
                                        }
                                    }
                                });
                            });

                            ui.separator();
                            cosmic_row(ui, "Corner Margins", Some("Distance from screen edges"), |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("X:");
                                    if ui.add_sized([90.0, 24.0], egui::Slider::new(&mut self.config.corner_margin_x, 0.0..=250.0).suffix(" px")).changed() {
                                        changed = true;
                                    }
                                    ui.label("Y:");
                                    if ui.add_sized([90.0, 24.0], egui::Slider::new(&mut self.config.corner_margin_y, 0.0..=250.0).suffix(" px")).changed() {
                                        changed = true;
                                    }
                                });
                            });

                            ui.separator();
                            cosmic_row(ui, "Secondary Bottom Text", Some("Optional subtitle (e.g. 'Go to Settings to activate.')"), |ui| {
                                if ui.add_sized([260.0, 26.0], egui::TextEdit::singleline(&mut self.config.corner_secondary_text).hint_text("Secondary text...")).changed() {
                                    changed = true;
                                }
                            });

                            if !self.config.corner_secondary_text.trim().is_empty() {
                                ui.separator();
                                cosmic_row(ui, "Secondary Font Size", Some("Size of bottom subtitle line"), |ui| {
                                    if ui.add_sized([220.0, 24.0], egui::Slider::new(&mut self.config.corner_secondary_font_size, 8.0..=36.0).suffix(" px")).changed() {
                                        changed = true;
                                    }
                                });
                            }
                        }

                        ui.separator();

                        // Rotation
                        cosmic_row(ui, "Rotation", Some("Angle of watermark text"), |ui| {
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

                        // Font Family
                        cosmic_row(ui, "Font Family", Some("Typography style for watermark text"), |ui| {
                            ui.horizontal(|ui| {
                                egui::ComboBox::from_id_salt("font_family_combo")
                                    .selected_text(&self.config.font_family)
                                    .width(160.0)
                                    .show_ui(ui, |ui| {
                                        for font in [
                                            "Liberation Sans",
                                            "Fira Sans",
                                            "FiraCode Nerd Font",
                                            "Ubuntu",
                                            "Roboto",
                                            "Roboto Slab",
                                            "Liberation Serif",
                                            "Liberation Mono",
                                            "DejaVu Sans",
                                        ] {
                                            if ui.selectable_label(self.config.font_family == font, font).clicked() {
                                                self.config.font_family = font.to_string();
                                                changed = true;
                                            }
                                        }
                                    });

                                if ui.button("Custom TTF...").on_hover_text("Pick custom TTF/OTF font file from disk").clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Fonts", &["ttf", "otf"])
                                        .pick_file()
                                    {
                                        self.config.font_path = Some(path.to_string_lossy().to_string());
                                        if let Some(file_stem) = path.file_stem() {
                                            self.config.font_family = file_stem.to_string_lossy().to_string();
                                        }
                                        changed = true;
                                    }
                                }
                                if self.config.font_path.is_some()
                                    && ui.button("Reset").on_hover_text("Reset to system font").clicked()
                                {
                                    self.config.font_path = None;
                                    self.config.font_family = "Liberation Sans".to_string();
                                    changed = true;
                                }
                            });
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
                                self.config.mode = match (self.config.show_text, self.config.show_icon) {
                                    (true, true) => "both".to_string(),
                                    (true, false) => "text".to_string(),
                                    (false, true) => "image".to_string(),
                                    (false, false) => "none".to_string(),
                                };
                                changed = true;
                            }
                        });

                        if self.config.show_text {
                            ui.separator();
                            cosmic_row(ui, "Watermark Text", Some("Supports newlines & tokens ({workspace}, {user}, {hostname}, {time})"), |ui| {
                                if ui.add_sized([260.0, 52.0], egui::TextEdit::multiline(&mut self.config.text)).changed() {
                                    changed = true;
                                }
                            });
                        }

                        ui.separator();

                        // Show Icon
                        cosmic_row(ui, "Show Icon / Image", None, |ui| {
                            if cosmic_switch(ui, &mut self.config.show_icon).changed() {
                                self.config.mode = match (self.config.show_text, self.config.show_icon) {
                                    (true, true) => "both".to_string(),
                                    (true, false) => "text".to_string(),
                                    (false, true) => "image".to_string(),
                                    (false, false) => "none".to_string(),
                                };
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
                                    if self.config.image_path.is_some() && ui.button("Reset").clicked() {
                                        self.config.image_path = None;
                                        changed = true;
                                    }
                                    let mut path_str = self.config.image_path.clone().unwrap_or_default();
                                    if ui.add_sized([160.0, 26.0], egui::TextEdit::singleline(&mut path_str).hint_text("Default 3D icon")).changed() {
                                        self.config.image_path = if path_str.trim().is_empty() { None } else { Some(path_str.trim().to_string()) };
                                        changed = true;
                                    }
                                });
                            });

                            ui.separator();
                            cosmic_row(ui, "Icon Size", Some("Size in pixels (bounding box)"), |ui| {
                                if ui.add_sized([220.0, 24.0], egui::Slider::new(&mut self.config.image_size, 12.0..=256.0).suffix(" px")).changed() {
                                    self.config.image_scale = self.config.image_size / 28.0;
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

                    // Card 4: Per-Workspace Stamps (Feature for Issue #1)
                    let card_ws = egui::Frame::new()
                        .fill(card_bg)
                        .stroke(card_stroke)
                        .corner_radius(egui::CornerRadius::same(10))
                        .inner_margin(egui::Margin::symmetric(16, 12));

                    card_ws.show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 8.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Workspace-Specific Stamps").strong().size(13.5).color(egui::Color32::from_rgb(235, 238, 245)));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new("Unique stamp per desktop").size(11.0).color(egui::Color32::from_rgb(155, 160, 175)));
                            });
                        });
                        ui.label(egui::RichText::new("Assign custom watermark stamps for individual workspaces, or use the {workspace} dynamic token.").size(11.0).color(egui::Color32::from_rgb(155, 160, 175)));

                        let mut to_remove = None;
                        for (ws_name, stamp_text) in &mut self.config.workspace_stamps {
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("Workspace \"{}\":", ws_name)).strong().color(egui::Color32::from_rgb(233, 84, 32)));
                                if ui.add_sized([ui.available_width() - 36.0, 26.0], egui::TextEdit::singleline(stamp_text)).changed() {
                                    changed = true;
                                }
                                if ui.button("✕").on_hover_text("Remove custom stamp for this workspace").clicked() {
                                    to_remove = Some(ws_name.clone());
                                }
                            });
                        }
                        if let Some(r) = to_remove {
                            self.config.workspace_stamps.remove(&r);
                            changed = true;
                        }

                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.add_sized([100.0, 26.0], egui::TextEdit::singleline(&mut self.new_ws_name).hint_text("Name (e.g. 1, 2, Dev)"));
                            ui.add_sized([220.0, 26.0], egui::TextEdit::singleline(&mut self.new_ws_text).hint_text("Custom stamp text"));
                            if ui.button("+ Add Workspace Stamp").clicked() {
                                let key = self.new_ws_name.trim().to_string();
                                let val = self.new_ws_text.trim().to_string();
                                if !key.is_empty() && !val.is_empty() {
                                    self.config.workspace_stamps.insert(key, val);
                                    self.new_ws_name.clear();
                                    self.new_ws_text.clear();
                                    changed = true;
                                }
                            }
                        });
                    });

                    // Card 5: Live Desktop Preview
                    let card5 = egui::Frame::new()
                        .fill(card_bg)
                        .stroke(card_stroke)
                        .corner_radius(egui::CornerRadius::same(10))
                        .inner_margin(egui::Margin::symmetric(16, 12));

                    card5.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Live Pattern Preview").strong().size(12.5).color(egui::Color32::from_rgb(233, 84, 32)));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let mode_str = if self.config.is_corner_mode() {
                                    format!("Corner: {}", self.config.corner_position)
                                } else {
                                    format!("Grid: {}° rotation", self.config.angle_deg as i32)
                                };
                                ui.label(egui::RichText::new(mode_str).size(11.0).color(egui::Color32::from_rgb(155, 160, 175)));
                            });
                        });
                        ui.add_space(4.0);

                        let preview_height = 120.0;
                        let (response, painter) = ui.allocate_painter(egui::vec2(ui.available_width(), preview_height), egui::Sense::hover());
                        let rect = response.rect;

                        // Desktop wallpaper canvas with subtle gradient border
                        painter.rect_filled(rect, 8.0, egui::Color32::from_rgb(18, 20, 26));
                        painter.rect_stroke(rect, 8.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 44, 54)), egui::StrokeKind::Inside);

                        // Draw subtle desktop dock or top bar hint for realism
                        let bar_rect = egui::Rect::from_min_size(
                            egui::pos2(rect.left() + 10.0, rect.top() + 6.0),
                            egui::vec2(rect.width() - 20.0, 10.0),
                        );
                        painter.rect_filled(bar_rect, 3.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 12));

                        let sample_text = if self.config.show_text && !self.config.text.is_empty() {
                            crate::tokens::resolve_tokens(&self.config.text)
                        } else {
                            String::new()
                        };

                        let sample_sec_text = if self.config.is_corner_mode() && !self.config.corner_secondary_text.trim().is_empty() {
                            crate::tokens::resolve_tokens(&self.config.corner_secondary_text)
                        } else {
                            String::new()
                        };

                        let alpha = ((self.config.opacity * 255.0) as u8).max(60);
                        let main_col = egui::Color32::from_rgba_unmultiplied(
                            self.config.color_rgba[0],
                            self.config.color_rgba[1],
                            self.config.color_rgba[2],
                            alpha,
                        );
                        let sub_col = egui::Color32::from_rgba_unmultiplied(
                            self.config.color_rgba[0],
                            self.config.color_rgba[1],
                            self.config.color_rgba[2],
                            (alpha as f32 * 0.75) as u8,
                        );

                        let font_id = egui::FontId::proportional((self.config.font_size * 0.42).clamp(9.0, 15.0));
                        let sec_font_id = egui::FontId::proportional((self.config.corner_secondary_font_size * 0.42).clamp(8.0, 12.0));

                        let lines: Vec<&str> = sample_text.lines().collect();
                        let max_chars = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
                        let char_w = font_id.size * 0.58;
                        let text_w = if self.config.show_text { (max_chars as f32 * char_w).max(4.0) } else { 0.0 };
                        let text_h = if self.config.show_text { lines.len().max(1) as f32 * (font_id.size * 1.25) } else { 0.0 };

                        let icon_size = if self.config.show_icon { (self.config.image_size * 0.35).clamp(10.0, 22.0) } else { 0.0 };
                        let icon_gap = if self.config.show_icon && self.config.show_text && !sample_text.is_empty() { 6.0 } else { 0.0 };
                        let top_w = icon_size + icon_gap + text_w;

                        let sec_chars = sample_sec_text.chars().count();
                        let sec_w = if !sample_sec_text.is_empty() { sec_chars as f32 * sec_font_id.size * 0.58 } else { 0.0 };

                        let block_w = top_w.max(sec_w);

                        if self.config.is_corner_mode() {
                            // Realistic Corner Preview
                            let (pos_x, pos_y) = match self.config.corner_position.as_str() {
                                "bottom_left" => (rect.left() + 20.0, rect.bottom() - 36.0),
                                "top_right" => (rect.right() - block_w - 20.0, rect.top() + 24.0),
                                "top_left" => (rect.left() + 20.0, rect.top() + 24.0),
                                _ => (rect.right() - block_w - 20.0, rect.bottom() - 36.0),
                            };

                            let align_right = self.config.corner_position == "bottom_right" || self.config.corner_position == "top_right";
                            let mut start_x = if align_right { pos_x + (block_w - top_w) } else { pos_x };

                            // Draw Icon
                            if self.config.show_icon {
                                let icon_center = egui::pos2(start_x + icon_size * 0.5, pos_y + text_h * 0.45);
                                if self.config.monochrome_icon {
                                    painter.circle_filled(icon_center, icon_size * 0.45, main_col);
                                } else {
                                    // 3D icon accent
                                    painter.circle_filled(icon_center, icon_size * 0.45, egui::Color32::from_rgb(233, 84, 32));
                                }
                                start_x += icon_size + icon_gap;
                            }

                            // Draw Main Text
                            if self.config.show_text && !lines.is_empty() {
                                for (idx, line) in lines.iter().enumerate() {
                                    let ly = pos_y + idx as f32 * (font_id.size * 1.25);
                                    let tx = if align_right { start_x + text_w } else { start_x };
                                    let align = if align_right { egui::Align2::RIGHT_TOP } else { egui::Align2::LEFT_TOP };
                                    painter.text(egui::pos2(tx, ly), align, line, font_id.clone(), main_col);
                                }
                            }

                            // Draw Secondary Subtitle Text
                            if !sample_sec_text.is_empty() {
                                let sec_y = pos_y + text_h + 3.0;
                                let sx = if align_right { pos_x + block_w } else { pos_x };
                                let align = if align_right { egui::Align2::RIGHT_TOP } else { egui::Align2::LEFT_TOP };
                                painter.text(egui::pos2(sx, sec_y), align, &sample_sec_text, sec_font_id, sub_col);
                            }
                        } else {
                            // Realistic Full Screen Grid Preview (repeated stamps)
                            let spacing_x = (self.config.spacing_x * 0.32).clamp(90.0, 220.0);
                            let spacing_y = (self.config.spacing_y * 0.32).clamp(40.0, 90.0);

                            // Vertical decorative lines in grid mode
                            if self.config.show_vertical_lines {
                                let line_alpha = ((self.config.line_color_rgba[3] as f32 / 255.0) * self.config.opacity * 255.0) as u8;
                                let lcol = egui::Color32::from_rgba_unmultiplied(
                                    self.config.line_color_rgba[0],
                                    self.config.line_color_rgba[1],
                                    self.config.line_color_rgba[2],
                                    line_alpha.max(30),
                                );
                                let mut lx = rect.left() + 25.0;
                                while lx < rect.right() {
                                    painter.line_segment([egui::pos2(lx, rect.top() + 8.0), egui::pos2(lx, rect.bottom() - 8.0)], egui::Stroke::new(1.0, lcol));
                                    lx += spacing_x;
                                }
                            }

                            // Grid pattern repetitions
                            let mut row = 0;
                            let mut gy = rect.top() + 20.0;
                            while gy < rect.bottom() - 15.0 {
                                let stagger = if row % 2 == 1 { spacing_x * 0.4 } else { 0.0 };
                                let mut gx = rect.left() + 15.0 + stagger;
                                while gx < rect.right() + spacing_x {
                                    let mut cur_x = gx;
                                    if self.config.show_icon {
                                        let icon_c = egui::pos2(cur_x + icon_size * 0.4, gy + 4.0);
                                        painter.circle_filled(icon_c, (icon_size * 0.35).max(3.0), main_col);
                                        cur_x += icon_size * 0.8 + 4.0;
                                    }

                                    if self.config.show_text && !sample_text.is_empty() {
                                        let preview_str = lines.first().copied().unwrap_or("");
                                        painter.text(egui::pos2(cur_x, gy), egui::Align2::LEFT_TOP, preview_str, font_id.clone(), main_col);
                                    }

                                    gx += spacing_x;
                                }
                                gy += spacing_y;
                                row += 1;
                            }
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
                                ui.image(egui::load::SizedTexture::new(logo.id(), egui::vec2(44.0, 44.0)));
                            }
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Deskstamp").strong().size(18.0).color(egui::Color32::WHITE));
                                ui.label(egui::RichText::new("Desktop Watermark Layer for Linux & Pop!_OS COSMIC").weak().size(12.0));
                            });
                        });

                        ui.separator();
                        ui.label("Lightweight, click-through desktop security watermark overlay designed natively for Wayland and Pop!_OS COSMIC.");
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Version:").strong());
                            ui.label(egui::RichText::new("0.1.0").weak());
                        });

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("License:").strong());
                            ui.label(egui::RichText::new("GPL-3.0").weak());
                        });

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(4.0);

                        ui.label(egui::RichText::new("Community & Support").strong().size(13.0).color(egui::Color32::WHITE));

                        ui.horizontal(|ui| {
                            let issue_btn = egui::Button::new(
                                egui::RichText::new("🐛 Report an Issue / Bug")
                                    .color(egui::Color32::WHITE)
                                    .size(13.0),
                            )
                            .fill(egui::Color32::from_rgb(233, 84, 32))
                            .corner_radius(egui::CornerRadius::same(6));

                            if ui.add(issue_btn).on_hover_text("Open issue tracker on GitHub").clicked() {
                                ui.ctx().open_url(egui::OpenUrl::new_tab("https://github.com/GabrielBaiano/Deskstamp/issues/new/choose"));
                                open_browser("https://github.com/GabrielBaiano/Deskstamp/issues/new/choose");
                            }

                            let repo_btn = egui::Button::new(
                                egui::RichText::new("⭐ GitHub Repository")
                                    .color(egui::Color32::from_rgb(225, 230, 240))
                                    .size(13.0),
                            )
                            .fill(egui::Color32::from_rgb(45, 48, 58))
                            .corner_radius(egui::CornerRadius::same(6));

                            if ui.add(repo_btn).on_hover_text("Visit Deskstamp repository").clicked() {
                                ui.ctx().open_url(egui::OpenUrl::new_tab("https://github.com/GabrielBaiano/Deskstamp"));
                                open_browser("https://github.com/GabrielBaiano/Deskstamp");
                            }
                        });
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
    ).map_err(|e| std::io::Error::other(e.to_string()))?;

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

fn open_browser(url: &str) {
    let _ = std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .or_else(|_| std::process::Command::new("gio").args(["open", url]).spawn());
}

