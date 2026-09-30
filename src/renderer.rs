use crate::config::WatermarkConfig;
use fontdue::{Font, FontSettings};
use tiny_skia::*;

/// Mathematical cubic bezier approximation for smooth continuous rounded corners (squircles / capsules)
fn add_rounded_rect(pb: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32, r: f32) {
    let r = r.min(w * 0.5).min(h * 0.5);
    let k = r * 0.55228475;
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    pb.close();
}

pub struct WatermarkRenderer {
    font: Font,
}

impl WatermarkRenderer {
    pub fn new(custom_font: Option<&str>) -> Result<Self, String> {
        Self::new_with_family(custom_font, "Liberation Sans")
    }

    pub fn new_with_family(custom_font: Option<&str>, font_family: &str) -> Result<Self, String> {
        let mut candidates = Vec::new();
        if let Some(p) = custom_font {
            candidates.push(p.to_string());
        }

        // Query system fontconfig via fc-match if font_family is requested
        if !font_family.trim().is_empty() {
            if let Ok(output) = std::process::Command::new("fc-match")
                .arg(font_family)
                .arg("-f")
                .arg("%{file}")
                .output()
            {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        candidates.push(path);
                    }
                }
            }
        }

        // Apple & Modern UI system font fallbacks
        let home = std::env::var("HOME").unwrap_or_default();
        candidates.push(format!("{}/.local/share/fonts/SF-Pro-Display-Medium.otf", home));
        candidates.push(format!("{}/.local/share/fonts/SFProDisplay-Medium.ttf", home));
        candidates.push(format!("{}/.local/share/fonts/Inter-Medium.otf", home));
        candidates.push(format!("{}/.local/share/fonts/Inter-Regular.ttf", home));
        candidates.push("/usr/share/fonts/opentype/fira/FiraSans-Regular.otf".to_string());
        candidates.push("/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf".to_string());
        candidates.push("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf".to_string());
        candidates.push("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf".to_string());
        candidates.push("/usr/share/fonts/truetype/roboto-slab/RobotoSlab-Bold.ttf".to_string());

        for path in candidates {
            if let Ok(data) = std::fs::read(&path) {
                if let Ok(font) = Font::from_bytes(data, FontSettings::default()) {
                    return Ok(Self { font });
                }
            }
        }

        Err("No suitable TTF font found on system".to_string())
    }

    pub fn update_font(&mut self, custom_font: Option<&str>, font_family: &str) {
        if let Ok(new_renderer) = Self::new_with_family(custom_font, font_family) {
            self.font = new_renderer.font;
        }
    }

    /// Loads or scales a logo / image tile, optionally tinting it monochrome to match text color.
    /// Returns None if the image cannot be loaded or is invalid (does not force Pop!_OS logo if custom path failed).
    fn load_image_tile(&self, path: Option<&str>, target_pixel_size: f32, monochrome: bool, tint_color: [u8; 4]) -> Option<Pixmap> {
        let base_pix = if let Some(p) = path {
            let data = std::fs::read(p).ok()?;
            let img = image::load_from_memory(&data).ok()?;
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            if w == 0 || h == 0 { return None; }
            let mut raw = rgba.into_raw();
            // Premultiply alpha for tiny-skia
            for chunk in raw.chunks_exact_mut(4) {
                let a = chunk[3] as u32;
                chunk[0] = ((chunk[0] as u32 * a) / 255) as u8;
                chunk[1] = ((chunk[1] as u32 * a) / 255) as u8;
                chunk[2] = ((chunk[2] as u32 * a) / 255) as u8;
            }
            tiny_skia::Pixmap::from_vec(raw, tiny_skia::IntSize::from_wh(w, h)?)?
        } else {
            static LOGO: &[u8] = include_bytes!("../data/icons/hicolor/128x128/apps/io.github.gabrielbaiano.Deskstamp.png");
            Pixmap::decode_png(LOGO).ok()?
        };

        let target = target_pixel_size.clamp(8.0, 512.0);
        let max_dim = (base_pix.width().max(base_pix.height()) as f32).max(1.0);
        let s = target / max_dim;
        let nw = ((base_pix.width() as f32 * s).round() as u32).max(4);
        let nh = ((base_pix.height() as f32 * s).round() as u32).max(4);

        let mut pix = Pixmap::new(nw, nh)?;
        pix.draw_pixmap(
            0,
            0,
            base_pix.as_ref(),
            &PixmapPaint { quality: FilterQuality::Bilinear, ..Default::default() },
            Transform::from_scale(s, s),
            None,
        );

        if monochrome {
            for pixel in pix.pixels_mut() {
                let a = pixel.alpha();
                if a > 0 {
                    let r = ((tint_color[0] as u32 * a as u32) / 255) as u8;
                    let g = ((tint_color[1] as u32 * a as u32) / 255) as u8;
                    let b = ((tint_color[2] as u32 * a as u32) / 255) as u8;
                    *pixel = PremultipliedColorU8::from_rgba(r, g, b, a).unwrap_or(PremultipliedColorU8::TRANSPARENT);
                }
            }
        }

        Some(pix)
    }

    /// Renders text glyphs onto an RGBA Pixmap tile, supporting multiline text with newlines,
    /// tabular figures for numerals, and customizable letter spacing.
    fn render_text_tile(
        &self,
        text: &str,
        size: f32,
        color_rgba: [u8; 4],
        stroke_color: [u8; 4],
        stroke: bool,
        letter_spacing: f32,
    ) -> (Pixmap, f32, f32) {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return (Pixmap::new(1, 1).unwrap(), 0.0, 0.0);
        }

        // Apple typography detail: Tabular numbers (digits 0..=9 share the same advance width)
        let max_digit_advance = ('0'..='9')
            .map(|d| self.font.rasterize(d, size).0.advance_width)
            .fold(0.0f32, f32::max);

        let mut lines_data = Vec::new();
        let mut max_line_width = 0.0f32;
        let mut max_ascent = 0.0f32;
        let mut max_descent = 0.0f32;

        for line in &lines {
            let mut glyphs = Vec::new();
            let mut total_width = 0.0f32;
            for ch in line.chars() {
                let (metrics, bitmap) = self.font.rasterize(ch, size);
                let adv = if ch.is_ascii_digit() && max_digit_advance > 0.0 {
                    max_digit_advance + letter_spacing
                } else {
                    metrics.advance_width + letter_spacing
                };
                total_width += adv;
                if metrics.bounds.ymin.abs() > max_descent {
                    max_descent = metrics.bounds.ymin.abs();
                }
                if metrics.bounds.height > max_ascent {
                    max_ascent = metrics.bounds.height;
                }
                glyphs.push((ch, metrics, bitmap, adv));
            }
            if total_width > max_line_width {
                max_line_width = total_width;
            }
            lines_data.push((total_width, glyphs));
        }

        let pad = 8.0f32;
        let line_height = (max_ascent + max_descent).max(size * 1.25);
        let tile_w = (max_line_width + pad * 2.0).ceil() as u32;
        let tile_h = (line_height * lines_data.len() as f32 + pad * 2.0).ceil() as u32;

        let mut pixmap = Pixmap::new(tile_w.max(1), tile_h.max(1)).unwrap_or_else(|| Pixmap::new(1, 1).unwrap());

        for (line_idx, (_line_w, glyphs)) in lines_data.into_iter().enumerate() {
            let baseline = pad + max_ascent + line_idx as f32 * line_height;
            let mut current_x = pad;

            for (ch, metrics, bitmap, adv) in glyphs {
                // If tabular digit, center it inside its advance slot
                let offset_x = if ch.is_ascii_digit() && max_digit_advance > metrics.advance_width {
                    (max_digit_advance - metrics.advance_width) * 0.5
                } else {
                    0.0
                };

                let gx = (current_x + metrics.bounds.xmin + offset_x) as i32;
                let gy = (baseline - metrics.bounds.height - metrics.bounds.ymin) as i32;

                if stroke {
                    // Draw outline stroke
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 { continue; }
                            for row in 0..metrics.height {
                                for col in 0..metrics.width {
                                    let alpha = bitmap[row * metrics.width + col];
                                    if alpha > 30 {
                                        let px = gx + dx + col as i32;
                                        let py = gy + dy + row as i32;
                                        if px >= 0 && px < pixmap.width() as i32 && py >= 0 && py < pixmap.height() as i32 {
                                            let a = ((alpha as u32 * stroke_color[3] as u32) / 255) as u8;
                                            let c = Color::from_rgba8(stroke_color[0], stroke_color[1], stroke_color[2], a);
                                            pixmap.fill_rect(
                                                Rect::from_xywh(px as f32, py as f32, 1.0, 1.0).unwrap(),
                                                &Paint { shader: Shader::SolidColor(c), anti_alias: false, ..Default::default() },
                                                Transform::identity(),
                                                None,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Draw glyph core
                for row in 0..metrics.height {
                    for col in 0..metrics.width {
                        let alpha = bitmap[row * metrics.width + col];
                        if alpha > 0 {
                            let px = gx + col as i32;
                            let py = gy + row as i32;
                            if px >= 0 && px < pixmap.width() as i32 && py >= 0 && py < pixmap.height() as i32 {
                                let a = ((alpha as u32 * color_rgba[3] as u32) / 255) as u8;
                                let c = Color::from_rgba8(color_rgba[0], color_rgba[1], color_rgba[2], a);
                                pixmap.fill_rect(
                                    Rect::from_xywh(px as f32, py as f32, 1.0, 1.0).unwrap(),
                                    &Paint { shader: Shader::SolidColor(c), anti_alias: false, ..Default::default() },
                                    Transform::identity(),
                                    None,
                                );
                            }
                        }
                    }
                }

                current_x += adv;
            }
        }

        (pixmap, tile_w as f32, tile_h as f32)
    }

    /// Renders watermark (grid or corner) and optional vertical/diagonal lines over the output buffer.
    /// Returns the dirty bounding rect `Some([x, y, width, height])` if anything was drawn.
    pub fn render_to_buffer(
        &self,
        buffer: &mut [u8],
        width: u32,
        height: u32,
        _stride: u32,
        cfg: &WatermarkConfig,
        workspace: Option<&str>,
        screen_idx: usize,
        screen_name: Option<&str>,
    ) -> Option<[i32; 4]> {
        if !cfg.active || cfg.opacity <= 0.001 {
            buffer.fill(0);
            return None;
        }

        let mut pixmap = match PixmapMut::from_bytes(buffer, width, height) {
            Some(p) => p,
            None => return None,
        };
        pixmap.fill(Color::TRANSPARENT);

        self.draw_watermark_elements(&mut pixmap, width, height, cfg, workspace, screen_idx, screen_name)
    }

    fn draw_watermark_elements(
        &self,
        pixmap: &mut PixmapMut,
        width: u32,
        height: u32,
        cfg: &WatermarkConfig,
        workspace: Option<&str>,
        screen_idx: usize,
        screen_name: Option<&str>,
    ) -> Option<[i32; 4]> {
        let is_corner = cfg.is_corner_mode();
        let step_x = cfg.spacing_x.max(100.0);
        let step_y = cfg.spacing_y.max(50.0);

        // 1. Draw Vertical Lines directly into pixel buffer (only in grid mode)
        if !is_corner && cfg.show_vertical_lines && cfg.line_width >= 0.5 {
            let l_alpha = ((cfg.line_color_rgba[3] as f32 / 255.0) * cfg.opacity.clamp(0.0, 1.0) * 255.0) as u8;
            let shadow_alpha = (l_alpha / 2).max(1);
            let shadow_color = PremultipliedColorU8::from_rgba(0, 0, 0, shadow_alpha).unwrap_or(PremultipliedColorU8::TRANSPARENT);
            let pm_color = PremultipliedColorU8::from_rgba(
                ((cfg.line_color_rgba[0] as u32 * l_alpha as u32) / 255) as u8,
                ((cfg.line_color_rgba[1] as u32 * l_alpha as u32) / 255) as u8,
                ((cfg.line_color_rgba[2] as u32 * l_alpha as u32) / 255) as u8,
                l_alpha,
            ).unwrap_or(PremultipliedColorU8::TRANSPARENT);

            let pixels = pixmap.pixels_mut();
            let lw = (cfg.line_width.round() as i32).max(1);
            let dash_len = 16i32;
            let gap_len = 10i32;
            let period = dash_len + gap_len;

            // Draw vertical column lines spaced by step_x
            let mut col_x = 0i32;
            while col_x < width as i32 + 50 {
                for y in 0..height as i32 {
                    if !cfg.line_dashed || (y % period < dash_len) {
                        // Outline border for high contrast on light backgrounds
                        let sl = col_x - lw / 2 - 1;
                        let sr = col_x - lw / 2 + lw;
                        if sl >= 0 && sl < width as i32 {
                            let idx = (y * width as i32 + sl) as usize;
                            if idx < pixels.len() {
                                pixels[idx] = shadow_color;
                            }
                        }
                        if sr >= 0 && sr < width as i32 {
                            let idx = (y * width as i32 + sr) as usize;
                            if idx < pixels.len() {
                                pixels[idx] = shadow_color;
                            }
                        }

                        // Core line
                        for offset in 0..lw {
                            let px = col_x - lw / 2 + offset;
                            if px >= 0 && px < width as i32 {
                                let idx = (y * width as i32 + px) as usize;
                                if idx < pixels.len() {
                                    pixels[idx] = pm_color;
                                }
                            }
                        }
                    }
                }
                col_x += step_x as i32;
            }
        }

        // 2. Draw Diagonal Lines (only in grid mode)
        if !is_corner && cfg.show_diagonal_lines && cfg.diagonal_line_width >= 0.5 {
            let l_alpha = ((cfg.diagonal_line_color_rgba[3] as f32 / 255.0) * cfg.opacity.clamp(0.0, 1.0) * 255.0) as u8;
            if l_alpha > 0 {
                let color = Color::from_rgba8(
                    cfg.diagonal_line_color_rgba[0],
                    cfg.diagonal_line_color_rgba[1],
                    cfg.diagonal_line_color_rgba[2],
                    l_alpha,
                );
                let stroke_color = Color::from_rgba8(0, 0, 0, (l_alpha / 2).max(1));
                let step = cfg.diagonal_line_spacing.max(40.0);
                let diag = ((width * width + height * height) as f32).sqrt();

                let mut pb = PathBuilder::new();
                let mut x = -diag;
                while x < diag + width as f32 {
                    pb.move_to(x, -diag);
                    pb.line_to(x, height as f32 + diag);
                    x += step;
                }

                if let Some(path) = pb.finish() {
                    let transform = Transform::from_rotate_at(
                        cfg.diagonal_line_angle,
                        width as f32 * 0.5,
                        height as f32 * 0.5,
                    );
                    let dash = if cfg.diagonal_line_dashed {
                        StrokeDash::new(vec![16.0, 10.0], 0.0)
                    } else {
                        None
                    };
                    let stroke = Stroke {
                        width: cfg.diagonal_line_width.max(0.5),
                        dash,
                        ..Default::default()
                    };

                    let outline_dash = if cfg.diagonal_line_dashed {
                        StrokeDash::new(vec![16.0, 10.0], 0.0)
                    } else {
                        None
                    };
                    let outline_stroke = Stroke {
                        width: cfg.diagonal_line_width.max(0.5) + 1.2,
                        dash: outline_dash,
                        ..Default::default()
                    };
                    pixmap.stroke_path(&path, &Paint { shader: Shader::SolidColor(stroke_color), ..Default::default() }, &outline_stroke, transform, None);
                    pixmap.stroke_path(&path, &Paint { shader: Shader::SolidColor(color), ..Default::default() }, &stroke, transform, None);
                }
            }
        }

        // 3. Prepare Content
        let raw_text = cfg.get_text_for_workspace(workspace);
        let mut resolved_text = crate::tokens::resolve_tokens_with_context(&raw_text, workspace, screen_idx, screen_name);

        // Apple Capsule Corner Integration: Single-line flowing metadata with subtle Apple middle-dot separators
        if is_corner && cfg.corner_style == "capsule" {
            if cfg.show_workspace_indicator && !raw_text.contains("{workspace") {
                let ws_val = workspace.unwrap_or("1");
                if !resolved_text.is_empty() {
                    resolved_text.push_str(&format!("  ·  Área {}", ws_val));
                } else {
                    resolved_text = format!("Área {}", ws_val);
                }
            }
            if !cfg.corner_secondary_text.trim().is_empty() {
                let sec_text = crate::tokens::resolve_tokens_with_context(&cfg.corner_secondary_text, workspace, screen_idx, screen_name);
                if !sec_text.is_empty() {
                    resolved_text.push_str(&format!("  ·  {}", sec_text));
                }
            }
        }

        let has_stroke = cfg.stroke_width > 0.1;
        let should_show_text = cfg.show_text && !resolved_text.is_empty();
        let should_show_icon = cfg.show_icon && (cfg.mode != "text") && (cfg.mode != "none");

        let effective_font_size = if is_corner && cfg.corner_font_size > 0.0 {
            cfg.corner_font_size
        } else {
            cfg.font_size
        };

        let (txt_tile, txt_w, txt_h) = if should_show_text {
            self.render_text_tile(
                &resolved_text,
                effective_font_size,
                cfg.color_rgba,
                cfg.stroke_color_rgba,
                has_stroke,
                cfg.letter_spacing,
            )
        } else {
            (Pixmap::new(1, 1).unwrap(), 0.0, 0.0)
        };

        // Render secondary text for corner layout if configured in minimal mode
        let raw_sec_text = if is_corner && cfg.corner_style != "capsule" && !cfg.corner_secondary_text.trim().is_empty() {
            crate::tokens::resolve_tokens_with_context(&cfg.corner_secondary_text, workspace, screen_idx, screen_name)
        } else {
            String::new()
        };
        let (sec_tile, sec_w, sec_h) = if !raw_sec_text.is_empty() {
            let sec_size = if cfg.corner_secondary_font_size > 0.0 {
                cfg.corner_secondary_font_size
            } else {
                (cfg.font_size * 0.75).max(10.0)
            };
            self.render_text_tile(
                &raw_sec_text,
                sec_size,
                cfg.color_rgba,
                cfg.stroke_color_rgba,
                has_stroke,
                cfg.letter_spacing,
            )
        } else {
            (Pixmap::new(1, 1).unwrap(), 0.0, 0.0)
        };

        // Screen indicator badge (Apple status pill style [ 1 ])
        let (screen_badge, screen_badge_w, screen_badge_h) = if is_corner && cfg.show_screen_indicator {
            let badge_label = format!("{}", screen_idx);
            let (btxt_tile, btxt_w, btxt_h) = self.render_text_tile(
                &badge_label,
                (cfg.font_size * 0.72).clamp(10.0, 16.0),
                [255, 255, 255, 240],
                [0, 0, 0, 180],
                true,
                0.0,
            );
            let bw = (btxt_w + 12.0).max(20.0);
            let bh = (btxt_h + 4.0).max(18.0);
            let mut bpix = Pixmap::new(bw.ceil() as u32, bh.ceil() as u32).unwrap();
            bpix.fill(Color::TRANSPARENT);

            let mut pb = PathBuilder::new();
            add_rounded_rect(&mut pb, 0.5, 0.5, bw - 1.0, bh - 1.0, bh * 0.4);
            if let Some(path) = pb.finish() {
                bpix.fill_path(
                    &path,
                    &Paint { shader: Shader::SolidColor(Color::from_rgba8(255, 255, 255, 36)), ..Default::default() },
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
                bpix.stroke_path(
                    &path,
                    &Paint { shader: Shader::SolidColor(Color::from_rgba8(255, 255, 255, 60)), ..Default::default() },
                    &Stroke { width: 1.0, ..Default::default() },
                    Transform::identity(),
                    None,
                );
            }
            let tx = (bw - btxt_w) * 0.5;
            let ty = (bh - btxt_h) * 0.5;
            bpix.draw_pixmap(tx.round() as i32, ty.round() as i32, btxt_tile.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
            (Some(bpix), bw, bh)
        } else {
            (None, 0.0, 0.0)
        };

        let (icon_tile, icon_w, icon_h) = if should_show_icon {
            let target_pixels = if cfg.image_size > 0.0 {
                cfg.image_size
            } else {
                (cfg.font_size * 1.5).max(16.0) * cfg.image_scale
            };
            if let Some(img) = self.load_image_tile(
                cfg.image_path.as_deref(),
                target_pixels,
                cfg.monochrome_icon,
                cfg.color_rgba,
            ) {
                let w = img.width() as f32;
                let h = img.height() as f32;
                (Some(img), w, h)
            } else {
                (None, 0.0, 0.0)
            }
        } else {
            (None, 0.0, 0.0)
        };

        let badge_gap = if screen_badge.is_some() && (should_show_icon || should_show_text) { 8.0 } else { 0.0 };
        let icon_gap = if should_show_icon && should_show_text && txt_w > 0.0 { 10.0 } else { 0.0 };
        let content_w = screen_badge_w + badge_gap + icon_w + icon_gap + txt_w;
        let content_h = screen_badge_h.max(icon_h).max(txt_h);

        let (tile_pixmap, tile_w, tile_h) = if is_corner {
            if cfg.corner_style == "capsule" {
                // Apple Glass Capsule HUD: Translucent squircle pill with hairline border and soft drop shadow
                let pad_x = 16.0f32;
                let pad_y = 8.0f32;
                let capsule_w = (content_w + pad_x * 2.0).max(40.0);
                let capsule_h = (content_h + pad_y * 2.0).max(32.0);
                let radius = capsule_h * 0.5;

                let shadow_pad = 6.0f32;
                let total_w = (capsule_w + shadow_pad * 2.0).ceil() as u32;
                let total_h = (capsule_h + shadow_pad * 2.0 + 4.0).ceil() as u32;

                let mut comb = Pixmap::new(total_w.max(1), total_h.max(1)).unwrap();
                comb.fill(Color::TRANSPARENT);

                // 1 & 2. Translucent Squircle Glass Background & Soft Drop Shadow (if enabled)
                if cfg.corner_show_background {
                    // Soft Drop Shadow
                    let mut pb_shadow = PathBuilder::new();
                    add_rounded_rect(&mut pb_shadow, shadow_pad, shadow_pad + 3.0, capsule_w, capsule_h, radius);
                    if let Some(path) = pb_shadow.finish() {
                        let shadow_alpha = (70.0 * cfg.corner_bg_opacity.clamp(0.0, 1.0) * cfg.opacity.clamp(0.2, 1.0)) as u8;
                        comb.fill_path(
                            &path,
                            &Paint { shader: Shader::SolidColor(Color::from_rgba8(0, 0, 0, shadow_alpha)), ..Default::default() },
                            FillRule::Winding,
                            Transform::identity(),
                            None,
                        );
                    }

                    // Translucent Glass Background (Apple Dark Vibrancy)
                    let mut pb_bg = PathBuilder::new();
                    add_rounded_rect(&mut pb_bg, shadow_pad, shadow_pad, capsule_w, capsule_h, radius);
                    if let Some(path) = pb_bg.finish() {
                        let bg_alpha = (255.0 * cfg.corner_bg_opacity.clamp(0.0, 1.0) * cfg.opacity.clamp(0.15, 1.0)) as u8;
                        comb.fill_path(
                            &path,
                            &Paint { shader: Shader::SolidColor(Color::from_rgba8(18, 20, 26, bg_alpha)), ..Default::default() },
                            FillRule::Winding,
                            Transform::identity(),
                            None,
                        );
                    }
                }

                // 3. Hairline Stroke Border (1px)
                if cfg.corner_show_border {
                    let mut pb_stroke = PathBuilder::new();
                    add_rounded_rect(&mut pb_stroke, shadow_pad + 0.5, shadow_pad + 0.5, capsule_w - 1.0, capsule_h - 1.0, radius - 0.5);
                    if let Some(path) = pb_stroke.finish() {
                        let stroke_alpha = (50.0 * cfg.opacity.clamp(0.25, 1.0)) as u8;
                        comb.stroke_path(
                            &path,
                            &Paint { shader: Shader::SolidColor(Color::from_rgba8(255, 255, 255, stroke_alpha)), ..Default::default() },
                            &Stroke { width: 1.0, ..Default::default() },
                            Transform::identity(),
                            None,
                        );
                    }
                }

                // 4. Centered Inline Elements
                let mut curr_x = shadow_pad + pad_x;
                if let Some(badge) = &screen_badge {
                    let badge_y = shadow_pad + (capsule_h - screen_badge_h) * 0.5;
                    comb.draw_pixmap(curr_x.round() as i32, badge_y.round() as i32, badge.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
                    curr_x += screen_badge_w + badge_gap;
                }

                if let Some(img) = &icon_tile {
                    let img_y = shadow_pad + (capsule_h - icon_h) * 0.5;
                    comb.draw_pixmap(curr_x.round() as i32, img_y.round() as i32, img.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
                    curr_x += icon_w + icon_gap;
                }

                if should_show_text && txt_w > 0.0 {
                    let txt_y = shadow_pad + (capsule_h - txt_h) * 0.5;
                    comb.draw_pixmap(curr_x.round() as i32, txt_y.round() as i32, txt_tile.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
                }

                (comb, total_w as f32, total_h as f32)
            } else {
                // Minimal Float: Floating text and optional subtitle
                let has_secondary = sec_w > 0.0 && sec_h > 0.0;
                let sec_gap = if has_secondary { 4.0 } else { 0.0 };
                let align_right = cfg.corner_position == "bottom_right" || cfg.corner_position == "top_right";

                let block_w = content_w.max(sec_w).max(1.0);
                let block_h = (content_h + sec_gap + sec_h).max(1.0);

                let mut comb = Pixmap::new(block_w.ceil() as u32, block_h.ceil() as u32).unwrap();
                comb.fill(Color::TRANSPARENT);

                let top_start_x = if align_right { block_w - content_w } else { 0.0 };
                let mut curr_x = top_start_x;

                if let Some(badge) = &screen_badge {
                    let badge_y = (content_h - screen_badge_h) * 0.5;
                    comb.draw_pixmap(curr_x.round() as i32, badge_y.round() as i32, badge.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
                    curr_x += screen_badge_w + badge_gap;
                }

                if let Some(img) = &icon_tile {
                    let img_y = (content_h - icon_h) * 0.5;
                    comb.draw_pixmap(curr_x.round() as i32, img_y.round() as i32, img.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
                    curr_x += icon_w + icon_gap;
                }

                if should_show_text && txt_w > 0.0 {
                    let txt_y = (content_h - txt_h) * 0.5;
                    comb.draw_pixmap(curr_x.round() as i32, txt_y.round() as i32, txt_tile.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
                }

                if has_secondary {
                    let sec_x = if align_right { block_w - sec_w } else { 0.0 };
                    let sec_y = content_h + sec_gap;
                    comb.draw_pixmap(sec_x.round() as i32, sec_y.round() as i32, sec_tile.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
                }

                (comb, block_w, block_h)
            }
        } else if cfg.show_lines_next_to_text && (content_w > 0.0) {
            let line_gap = cfg.line_gap.clamp(6.0, 40.0);

            if cfg.single_line_connector {
                // 1 Single Continuous Dynamic Line connecting adjacent repeated texts across the screen
                let total_w = step_x.ceil() as u32;
                let total_h = (content_h + 8.0).ceil() as u32;
                let mut comb = Pixmap::new(total_w.max(1), total_h.max(1)).unwrap();
                comb.fill(Color::TRANSPARENT);

                let line_y = total_h as f32 * 0.5;
                let line_color = Color::from_rgba8(
                    cfg.line_color_rgba[0],
                    cfg.line_color_rgba[1],
                    cfg.line_color_rgba[2],
                    cfg.line_color_rgba[3],
                );
                let line_stroke = Stroke {
                    width: cfg.line_width.max(1.0),
                    dash: if cfg.line_dashed { StrokeDash::new(vec![8.0, 6.0], 0.0) } else { None },
                    ..Default::default()
                };

                let mut curr_x = 0.0f32;
                if let Some(img) = &icon_tile {
                    let img_y = (total_h as f32 - icon_h) * 0.5;
                    comb.draw_pixmap(curr_x.round() as i32, img_y.round() as i32, img.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
                    curr_x += icon_w + icon_gap;
                }

                if should_show_text && txt_w > 0.0 {
                    let txt_y = (total_h as f32 - txt_h) * 0.5;
                    comb.draw_pixmap(curr_x.round() as i32, txt_y.round() as i32, txt_tile.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
                    curr_x += txt_w;
                }

                // Dynamic single line bridging the space between text blocks
                let line_start_x = curr_x + line_gap;
                let line_end_x = total_w as f32 - line_gap;
                if line_end_x > line_start_x + 8.0 {
                    let mut pb = PathBuilder::new();
                    pb.move_to(line_start_x, line_y);
                    pb.line_to(line_end_x, line_y);
                    if let Some(path) = pb.finish() {
                        comb.stroke_path(&path, &Paint { shader: Shader::SolidColor(line_color), ..Default::default() }, &line_stroke, Transform::identity(), None);
                    }
                }

                (comb, total_w as f32, total_h as f32)
            } else {
                // Classic double stubs mode
                let line_len = cfg.line_length.max(20.0);
                let total_w = ((line_len + line_gap) * 2.0 + content_w).ceil() as u32;
                let total_h = (content_h + 8.0).ceil() as u32;

                let mut comb = Pixmap::new(total_w.max(1), total_h.max(1)).unwrap();
                comb.fill(Color::TRANSPARENT);

                let line_y = total_h as f32 * 0.5;
                let line_color = Color::from_rgba8(
                    cfg.line_color_rgba[0],
                    cfg.line_color_rgba[1],
                    cfg.line_color_rgba[2],
                    cfg.line_color_rgba[3],
                );
                let line_stroke = Stroke {
                    width: cfg.line_width.max(1.0),
                    dash: if cfg.line_dashed { StrokeDash::new(vec![8.0, 6.0], 0.0) } else { None },
                    ..Default::default()
                };

                let mut pb_left = PathBuilder::new();
                pb_left.move_to(0.0, line_y);
                pb_left.line_to(line_len, line_y);
                if let Some(path) = pb_left.finish() {
                    comb.stroke_path(&path, &Paint { shader: Shader::SolidColor(line_color), ..Default::default() }, &line_stroke, Transform::identity(), None);
                }

                let right_start = total_w as f32 - line_len;
                let mut pb_right = PathBuilder::new();
                pb_right.move_to(right_start, line_y);
                pb_right.line_to(total_w as f32, line_y);
                if let Some(path) = pb_right.finish() {
                    comb.stroke_path(&path, &Paint { shader: Shader::SolidColor(line_color), ..Default::default() }, &line_stroke, Transform::identity(), None);
                }

                let mut curr_x = line_len + line_gap;
                if let Some(img) = &icon_tile {
                    let img_y = (total_h as f32 - icon_h) * 0.5;
                    comb.draw_pixmap(curr_x.round() as i32, img_y.round() as i32, img.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
                    curr_x += icon_w + icon_gap;
                }

                if should_show_text && txt_w > 0.0 {
                    let txt_y = (total_h as f32 - txt_h) * 0.5;
                    comb.draw_pixmap(curr_x.round() as i32, txt_y.round() as i32, txt_tile.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
                }

                (comb, total_w as f32, total_h as f32)
            }
        } else {
            let total_w = content_w.max(1.0).ceil() as u32;
            let total_h = content_h.max(1.0).ceil() as u32;
            let mut comb = Pixmap::new(total_w, total_h).unwrap();
            comb.fill(Color::TRANSPARENT);

            let mut curr_x = 0.0f32;
            if let Some(img) = &icon_tile {
                let img_y = (total_h as f32 - icon_h) * 0.5;
                comb.draw_pixmap(curr_x as i32, img_y as i32, img.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
                curr_x += icon_w + icon_gap;
            }
            if should_show_text && txt_w > 0.0 {
                let txt_y = (total_h as f32 - txt_h) * 0.5;
                comb.draw_pixmap(curr_x as i32, txt_y as i32, txt_tile.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
            }

            (comb, total_w as f32, total_h as f32)
        };

        let paint = PixmapPaint {
            opacity: cfg.opacity.clamp(0.0, 1.0),
            quality: FilterQuality::Bilinear,
            ..Default::default()
        };

        if is_corner {
            let margin_x = cfg.corner_margin_x.max(0.0);
            let margin_y = cfg.corner_margin_y.max(0.0);
            let (pos_x, pos_y) = match cfg.corner_position.as_str() {
                "bottom_left" => (
                    margin_x,
                    (height as f32 - tile_h - margin_y).max(0.0),
                ),
                "top_right" => (
                    (width as f32 - tile_w - margin_x).max(0.0),
                    margin_y,
                ),
                "top_left" => (
                    margin_x,
                    margin_y,
                ),
                _ => {
                    // "bottom_right" default (Windows activation style)
                    (
                        (width as f32 - tile_w - margin_x).max(0.0),
                        (height as f32 - tile_h - margin_y).max(0.0),
                    )
                }
            };

            let transform = if cfg.angle_deg.abs() > 0.01 {
                Transform::from_translate(pos_x + tile_w * 0.5, pos_y + tile_h * 0.5)
                    .pre_rotate(cfg.angle_deg)
                    .pre_translate(-tile_w * 0.5, -tile_h * 0.5)
            } else {
                Transform::from_translate(pos_x, pos_y)
            };

            pixmap.draw_pixmap(0, 0, tile_pixmap.as_ref(), &paint, transform, None);
            let rx = pos_x.floor() as i32;
            let ry = pos_y.floor() as i32;
            let rw = (tile_w.ceil() as i32 + 4).min(width as i32 - rx);
            let rh = (tile_h.ceil() as i32 + 4).min(height as i32 - ry);
            return Some([rx.max(0), ry.max(0), rw.max(1), rh.max(1)]);
        }

        let cx = width as f32 * 0.5;
        let cy = height as f32 * 0.5;
        let diag = ((width * width + height * height) as f32).sqrt();

        let rad = cfg.angle_deg.to_radians();
        let cos = rad.cos();
        let sin = rad.sin();

        // Infinite rotated grid iteration covering the full screen bounds at any angle
        let mut v = -diag;
        let mut row_idx = 0;
        while v <= diag + step_y {
            let u_offset = if row_idx % 2 == 1 { cfg.stagger_offset } else { 0.0 };
            let mut u = -diag + u_offset;
            while u <= diag + step_x {
                let sx = cx + u * cos - v * sin;
                let sy = cy + u * sin + v * cos;

                let transform = Transform::from_translate(sx, sy)
                    .pre_rotate(cfg.angle_deg)
                    .pre_translate(-tile_w * 0.5, -tile_h * 0.5);

                pixmap.draw_pixmap(0, 0, tile_pixmap.as_ref(), &paint, transform, None);
                u += step_x;
            }
            v += step_y;
            row_idx += 1;
        }

        Some([0, 0, width as i32, height as i32])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_corner_capsule_mode() {
        let renderer = WatermarkRenderer::new(None);
        if let Ok(r) = renderer {
            let mut cfg = WatermarkConfig::default();
            cfg.layout = "corner".to_string();
            cfg.corner_style = "capsule".to_string();
            cfg.corner_position = "bottom_right".to_string();
            cfg.text = "CONFIDENTIAL".to_string();
            cfg.show_screen_indicator = true;
            cfg.show_workspace_indicator = true;
            cfg.active = true;
            cfg.opacity = 0.5;

            let width = 800;
            let height = 600;
            let mut buf = vec![0u8; (width * height * 4) as usize];
            r.render_to_buffer(&mut buf, width, height, width * 4, &cfg, Some("Workspace 1"), 1, Some("DP-1"));

            // Ensure pixels were drawn
            assert!(buf.iter().any(|&b| b > 0));
        }
    }

    #[test]
    fn test_renderer_without_image() {
        let renderer = WatermarkRenderer::new(None);
        if let Ok(r) = renderer {
            let mut cfg = WatermarkConfig::default();
            cfg.show_icon = false; // Pure typography mode
            cfg.mode = "text".to_string();
            cfg.text = "Deskstamp Pure Typography".to_string();
            cfg.active = true;
            cfg.opacity = 0.5;

            let width = 800;
            let height = 600;
            let mut buf = vec![0u8; (width * height * 4) as usize];
            r.render_to_buffer(&mut buf, width, height, width * 4, &cfg, None, 1, None);

            assert!(buf.iter().any(|&b| b > 0));
        }
    }

    #[test]
    fn test_renderer_single_line_connector() {
        let renderer = WatermarkRenderer::new(None);
        if let Ok(r) = renderer {
            let mut cfg = WatermarkConfig::default();
            cfg.layout = "grid".to_string();
            cfg.show_lines_next_to_text = true;
            cfg.single_line_connector = true;
            cfg.spacing_x = 400.0;
            cfg.text = "CONFIDENTIAL".to_string();
            cfg.active = true;
            cfg.opacity = 0.5;

            let width = 800;
            let height = 600;
            let mut buf = vec![0u8; (width * height * 4) as usize];
            r.render_to_buffer(&mut buf, width, height, width * 4, &cfg, None, 1, None);

            assert!(buf.iter().any(|&b| b > 0));
        }
    }

    #[test]
    fn test_renderer_corner_mode_and_multiline() {
        let renderer = WatermarkRenderer::new(None);
        if let Ok(r) = renderer {
            let mut cfg = WatermarkConfig::default();
            cfg.layout = "corner".to_string();
            cfg.corner_position = "bottom_right".to_string();
            cfg.text = "Activate Pop!_OS\nGo to Settings to activate.".to_string();
            cfg.active = true;
            cfg.opacity = 0.5;

            let width = 800;
            let height = 600;
            let mut buf = vec![0u8; (width * height * 4) as usize];
            r.render_to_buffer(&mut buf, width, height, width * 4, &cfg, Some("Workspace 1"), 1, None);

            // Ensure pixels were drawn
            assert!(buf.iter().any(|&b| b > 0));
        }
    }

    #[test]
    fn test_renderer_corner_secondary_text_and_no_icon() {
        let renderer = WatermarkRenderer::new(None);
        if let Ok(r) = renderer {
            let mut cfg = WatermarkConfig::default();
            cfg.layout = "corner".to_string();
            cfg.corner_position = "bottom_right".to_string();
            cfg.text = "Activate Pop!_OS".to_string();
            cfg.corner_secondary_text = "Go to Settings to activate.".to_string();
            cfg.show_icon = false; // user disabled icon
            cfg.active = true;
            cfg.opacity = 0.5;

            let width = 800;
            let height = 600;
            let mut buf = vec![0u8; (width * height * 4) as usize];
            let rect = r.render_to_buffer(&mut buf, width, height, width * 4, &cfg, None, 1, None);

            assert!(buf.iter().any(|&b| b > 0));
            assert!(rect.is_some());
        }
    }

    #[test]
    fn test_renderer_corner_without_background_and_border() {
        let renderer = WatermarkRenderer::new(None);
        if let Ok(r) = renderer {
            let mut cfg = WatermarkConfig::default();
            cfg.layout = "corner".to_string();
            cfg.corner_style = "capsule".to_string();
            cfg.corner_show_background = false; // Background removed!
            cfg.corner_show_border = false; // Border removed!
            cfg.corner_position = "top_left".to_string();
            cfg.text = "Floating Clean Watermark".to_string();
            cfg.active = true;
            cfg.opacity = 0.8;

            let width = 800;
            let height = 600;
            let mut buf = vec![0u8; (width * height * 4) as usize];
            let dirty = r.render_to_buffer(&mut buf, width, height, width * 4, &cfg, None, 1, None);

            assert!(buf.iter().any(|&b| b > 0));
            let rect = dirty.expect("should return dirty rect");
            assert_eq!(rect[0], cfg.corner_margin_x as i32); // top_left margin_x
            assert_eq!(rect[1], cfg.corner_margin_y as i32); // top_left margin_y
        }
    }
}
