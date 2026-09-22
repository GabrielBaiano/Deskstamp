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
                                            &Paint { shader: Shader::SolidColor(c), anti_alias: true, ..Default::default() },
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
                                &Paint { shader: Shader::SolidColor(c), anti_alias: true, ..Default::default() },
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

    /// Renders repeated watermark grid over the output buffer
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

        let resolved_text = resolve_tokens(&cfg.text);
        let has_stroke = cfg.stroke_width > 0.1;
        let (tile_pixmap, tile_w, tile_h) = self.render_text_tile(
            &resolved_text,
            cfg.font_size,
            cfg.color_rgba,
            cfg.stroke_color_rgba,
            has_stroke,
        );

        let mut paint = PixmapPaint::default();
        paint.opacity = cfg.opacity.clamp(0.0, 1.0);
        paint.quality = FilterQuality::Bilinear;

        let diag = ((width * width + height * height) as f32).sqrt();

        // Staggered grid iteration
        let step_x = cfg.spacing_x.max(80.0);
        let step_y = cfg.spacing_y.max(40.0);

        let mut row_idx = 0;
        let mut y = -diag * 0.5;
        while y < height as f32 + diag * 0.5 {
            let row_offset = if row_idx % 2 == 1 { cfg.stagger_offset } else { 0.0 };
            let mut x = -diag * 0.5 + row_offset;

            while x < width as f32 + diag * 0.5 {
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
