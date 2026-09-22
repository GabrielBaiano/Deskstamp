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

    /// Loads or scales a logo / image tile
    fn load_image_tile(&self, path: Option<&str>, scale: f32) -> Pixmap {
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

        let s = scale.clamp(0.2, 5.0);
        if (s - 1.0).abs() > 0.05 {
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
        }
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

        // 3. Prepare Tile (Text, Image, or Both)
        let resolved_text = resolve_tokens(&cfg.text);
        let has_stroke = cfg.stroke_width > 0.1;

        let (tile_pixmap, tile_w, tile_h) = if cfg.mode == "image" {
            let img = self.load_image_tile(cfg.image_path.as_deref(), cfg.image_scale);
            let w = img.width() as f32;
            let h = img.height() as f32;
            (img, w, h)
        } else if cfg.mode == "both" {
            let img = self.load_image_tile(cfg.image_path.as_deref(), cfg.image_scale);
            let (txt_tile, txt_w, txt_h) = self.render_text_tile(
                &resolved_text,
                cfg.font_size,
                cfg.color_rgba,
                cfg.stroke_color_rgba,
                has_stroke,
            );
            let combined_w = (img.width() as f32).max(txt_w).ceil() as u32;
            let combined_h = (img.height() as f32 + txt_h + 8.0).ceil() as u32;
            let mut comb = Pixmap::new(combined_w.max(1), combined_h.max(1)).unwrap();
            comb.fill(Color::TRANSPARENT);

            let img_x = (combined_w as f32 - img.width() as f32) * 0.5;
            comb.draw_pixmap(img_x as i32, 0, img.as_ref(), &PixmapPaint::default(), Transform::identity(), None);

            let txt_x = (combined_w as f32 - txt_w) * 0.5;
            let txt_y = img.height() as f32 + 8.0;
            comb.draw_pixmap(txt_x as i32, txt_y as i32, txt_tile.as_ref(), &PixmapPaint::default(), Transform::identity(), None);

            (comb, combined_w as f32, combined_h as f32)
        } else {
            // Text mode (default)
            self.render_text_tile(
                &resolved_text,
                cfg.font_size,
                cfg.color_rgba,
                cfg.stroke_color_rgba,
                has_stroke,
            )
        };

        let mut paint = PixmapPaint::default();
        paint.opacity = cfg.opacity.clamp(0.0, 1.0);
        paint.quality = FilterQuality::Bilinear;

        let diag = ((width * width + height * height) as f32).sqrt();

        // If angle is 0, align directly with the vertical columns
        if cfg.angle_deg.abs() < 0.1 {
            let mut x = (step_x * 0.5) as f32;
            let mut col = 0;
            while x < width as f32 + 100.0 {
                let col_offset = if col % 2 == 1 { cfg.stagger_offset } else { 0.0 };
                let mut y = 40.0 + col_offset;
                while y < height as f32 + 100.0 {
                    let transform = Transform::from_translate(x - tile_w * 0.5, y - tile_h * 0.5);
                    pixmap.draw_pixmap(0, 0, tile_pixmap.as_ref(), &paint, transform, None);
                    y += step_y;
                }
                x += step_x;
                col += 1;
            }
        } else {
            // Diagonal grid iteration
            let mut row_idx = 0;
            let mut y = -diag * 0.3;
            while y < height as f32 + diag * 0.3 {
                let row_offset = if row_idx % 2 == 1 { cfg.stagger_offset } else { 0.0 };
                let mut x = -diag * 0.3 + row_offset;

                while x < width as f32 + diag * 0.3 {
                    let transform = Transform::from_translate(x, y)
                        .post_rotate(cfg.angle_deg)
                        .post_translate(-tile_w * 0.5, -tile_h * 0.5);

                    pixmap.draw_pixmap(0, 0, tile_pixmap.as_ref(), &paint, transform, None);
                    x += step_x;
                }
                y += step_y;
                row_idx += 1;
            }
        }
    }
}
