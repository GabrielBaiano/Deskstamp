use crate::config::WatermarkConfig;
use crate::tokens::resolve_tokens;
use fontdue::{Font, FontSettings};
use tiny_skia::*;

pub struct WatermarkRenderer {
    font: Font,
}

impl WatermarkRenderer {
    pub fn new(custom_font: Option<&str>) -> Result<Self, String> {
        let font_candidates = [
            custom_font,
            Some("/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf"),
            Some("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf"),
            Some("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"),
            Some("/usr/share/fonts/truetype/roboto-slab/RobotoSlab-Bold.ttf"),
        ];

        for candidate in font_candidates.into_iter().flatten() {
            if let Ok(data) = std::fs::read(candidate) {
                if let Ok(font) = Font::from_bytes(data, FontSettings::default()) {
                    return Ok(Self { font });
                }
            }
        }

        Err("No suitable TTF font found on system".to_string())
    }

    pub fn update_font(&mut self, custom_font: Option<&str>, font_family: &str) {
        let mut candidates = Vec::new();
        if let Some(p) = custom_font {
            candidates.push(p.to_string());
        }
        match font_family.to_lowercase().as_str() {
            "roboto" => {
                candidates.push("/usr/share/fonts/truetype/roboto/unhinted/RobotoTTF/Roboto-Bold.ttf".to_string());
                candidates.push("/usr/share/fonts/truetype/roboto/unhinted/RobotoTTF/Roboto-Regular.ttf".to_string());
            }
            "roboto slab" => {
                candidates.push("/usr/share/fonts/truetype/roboto-slab/RobotoSlab-Bold.ttf".to_string());
                candidates.push("/usr/share/fonts/truetype/roboto-slab/RobotoSlab-Regular.ttf".to_string());
            }
            "monospace" | "liberation mono" => {
                candidates.push("/usr/share/fonts/truetype/liberation/LiberationMono-Bold.ttf".to_string());
                candidates.push("/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf".to_string());
            }
            "liberation serif" => {
                candidates.push("/usr/share/fonts/truetype/liberation/LiberationSerif-Bold.ttf".to_string());
                candidates.push("/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf".to_string());
            }
            _ => {
                candidates.push("/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf".to_string());
                candidates.push("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf".to_string());
                candidates.push("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf".to_string());
            }
        }
        candidates.push("/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf".to_string());

        for path in candidates {
            if let Ok(data) = std::fs::read(&path) {
                if let Ok(f) = Font::from_bytes(data, FontSettings::default()) {
                    self.font = f;
                    return;
                }
            }
        }
    }

    /// Loads or scales a logo / image tile, optionally tinting it monochrome to match text color
    fn load_image_tile(&self, path: Option<&str>, scale: f32, monochrome: bool, tint_color: [u8; 4]) -> Pixmap {
        let base_pix = if let Some(p) = path {
            std::fs::read(p)
                .ok()
                .and_then(|data| Pixmap::decode_png(&data).ok())
                .unwrap_or_else(|| {
                    static LOGO: &[u8] = include_bytes!("../data/icons/hicolor/128x128/apps/io.github.gabrielbaiano.Deskstamp.png");
                    Pixmap::decode_png(LOGO).unwrap()
                })
        } else {
            static LOGO: &[u8] = include_bytes!("../data/icons/hicolor/128x128/apps/io.github.gabrielbaiano.Deskstamp.png");
            Pixmap::decode_png(LOGO).unwrap()
        };

        let s = scale.clamp(0.1, 5.0);
        let mut pix = if (s - 1.0).abs() > 0.05 {
            let nw = ((base_pix.width() as f32 * s).round() as u32).max(12);
            let nh = ((base_pix.height() as f32 * s).round() as u32).max(12);
            let mut scaled = Pixmap::new(nw, nh).unwrap_or_else(|| Pixmap::new(1, 1).unwrap());
            scaled.draw_pixmap(
                0,
                0,
                base_pix.as_ref(),
                &PixmapPaint::default(),
                Transform::from_scale(s, s),
                None,
            );
            scaled
        } else {
            base_pix
        };

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

        pix
    }

    /// Renders text glyphs onto an RGBA Pixmap tile
    fn render_text_tile(&self, text: &str, size: f32, color_rgba: [u8; 4], stroke_color: [u8; 4], stroke: bool) -> (Pixmap, f32, f32) {
        let mut glyphs = Vec::new();
        let mut total_width = 0.0f32;
        let mut max_ascent = 0.0f32;
        let mut max_descent = 0.0f32;

        for ch in text.chars() {
            let (metrics, bitmap) = self.font.rasterize(ch, size);
            total_width += metrics.advance_width;
            if metrics.bounds.ymin.abs() > max_descent {
                max_descent = metrics.bounds.ymin.abs();
            }
            if metrics.bounds.height as f32 > max_ascent {
                max_ascent = metrics.bounds.height as f32;
            }
            glyphs.push((metrics, bitmap));
        }

        let pad = 12.0f32;
        let tile_w = (total_width + pad * 2.0).ceil() as u32;
        let tile_h = (max_ascent + max_descent + pad * 2.0).ceil().max(size + pad * 2.0) as u32;

        let mut pixmap = Pixmap::new(tile_w.max(1), tile_h.max(1)).unwrap_or_else(|| Pixmap::new(1, 1).unwrap());
        let baseline = pad + max_ascent;

        let mut current_x = pad;
        for (metrics, bitmap) in glyphs {
            let gx = (current_x + metrics.bounds.xmin) as i32;
            let gy = (baseline - metrics.bounds.height as f32 - metrics.bounds.ymin) as i32;

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

            current_x += metrics.advance_width;
        }

        (pixmap, tile_w as f32, tile_h as f32)
    }

    /// Renders repeated watermark grid and vertical lines over the output buffer
    pub fn render_to_buffer(
        &self,
        buffer: &mut [u8],
        width: u32,
        height: u32,
        _stride: u32,
        cfg: &WatermarkConfig,
    ) {
        if !cfg.active || cfg.opacity <= 0.001 {
            buffer.fill(0);
            return;
        }

        let mut pixmap = match PixmapMut::from_bytes(buffer, width, height) {
            Some(p) => p,
            None => return,
        };
        pixmap.fill(Color::TRANSPARENT);

        let step_x = cfg.spacing_x.max(100.0);
        let step_y = cfg.spacing_y.max(50.0);

        // 1. Draw Vertical Lines directly into pixel buffer (blazing fast and crash-proof)
        if cfg.show_vertical_lines && cfg.line_width >= 0.5 {
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

        // 2. Draw Diagonal Lines
        if cfg.show_diagonal_lines && cfg.diagonal_line_width >= 0.5 {
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

        // 3. Prepare Tile (Text + Icon side-by-side + Lines alongside with spacing)
        let resolved_text = resolve_tokens(&cfg.text);
        let has_stroke = cfg.stroke_width > 0.1;

        let should_show_text = cfg.show_text && !resolved_text.is_empty();
        let should_show_icon = cfg.show_icon || cfg.mode == "image" || cfg.mode == "both";

        let (txt_tile, txt_w, txt_h) = if should_show_text {
            self.render_text_tile(
                &resolved_text,
                cfg.font_size,
                cfg.color_rgba,
                cfg.stroke_color_rgba,
                has_stroke,
            )
        } else {
            (Pixmap::new(1, 1).unwrap(), 0.0, 0.0)
        };

        let (icon_tile, icon_w, icon_h) = if should_show_icon {
            // Scale icon proportionally to match font size and image_scale
            let base_scale = (cfg.font_size / 24.0).clamp(0.2, 4.0) * cfg.image_scale;
            let img = self.load_image_tile(
                cfg.image_path.as_deref(),
                base_scale,
                cfg.monochrome_icon,
                cfg.color_rgba,
            );
            let w = img.width() as f32;
            let h = img.height() as f32;
            (Some(img), w, h)
        } else {
            (None, 0.0, 0.0)
        };

        let icon_gap = if should_show_icon && should_show_text && txt_w > 0.0 { 12.0 } else { 0.0 };
        let content_w = icon_w + icon_gap + txt_w;
        let content_h = icon_h.max(txt_h).max(12.0);

        let (tile_pixmap, tile_w, tile_h) = if cfg.show_lines_next_to_text && (content_w > 0.0) {
            let line_len = cfg.line_length.max(20.0);
            let line_gap = cfg.line_gap.max(10.0);
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

            // Left line (stops before content with clear gap)
            let mut pb_left = PathBuilder::new();
            pb_left.move_to(0.0, line_y);
            pb_left.line_to(line_len, line_y);
            if let Some(path) = pb_left.finish() {
                comb.stroke_path(&path, &Paint { shader: Shader::SolidColor(line_color), ..Default::default() }, &line_stroke, Transform::identity(), None);
            }

            // Right line (starts after content with clear gap)
            let right_start = total_w as f32 - line_len;
            let mut pb_right = PathBuilder::new();
            pb_right.move_to(right_start, line_y);
            pb_right.line_to(total_w as f32, line_y);
            if let Some(path) = pb_right.finish() {
                comb.stroke_path(&path, &Paint { shader: Shader::SolidColor(line_color), ..Default::default() }, &line_stroke, Transform::identity(), None);
            }

            // Draw Icon + Text in center
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
        } else {
            let total_w = content_w.max(1.0).ceil() as u32;
            let total_h = content_h.max(1.0).ceil() as u32;
            let mut comb = Pixmap::new(total_w, total_h).unwrap();
            comb.fill(Color::TRANSPARENT);

            let mut curr_x = 0.0;
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

        let mut paint = PixmapPaint::default();
        let eff_opacity = if cfg.stealth_mode {
            0.015 // Stealth mode: invisible to naked eye, present in recorded videos
        } else {
            cfg.opacity
        };
        paint.opacity = eff_opacity.clamp(0.0, 1.0);
        paint.quality = FilterQuality::Bilinear;

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
    }
}
