use std::sync::OnceLock;
use tiny_skia::{Color, Pixmap, PremultipliedColorU8};

use super::{CornerStyle, Drawable, Point2D, Rect2D, Sloppiness, StrokeStyle, generate_shape_seed};

static TEXT_SYSTEM_FONT: OnceLock<Option<fontdue::Font>> = OnceLock::new();

fn get_text_font() -> Option<&'static fontdue::Font> {
    TEXT_SYSTEM_FONT
        .get_or_init(|| {
            let candidates = [
                // macOS
                "/System/Library/Fonts/Hiragino Sans GB.ttc",
                "/System/Library/Fonts/STHeiti Light.ttc",
                "/System/Library/Fonts/STHeiti Medium.ttc",
                "/System/Library/Fonts/Supplemental/Songti.ttc",
                "/System/Library/Fonts/Helvetica.ttc",
                // Linux / Ubuntu
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
                "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
                "/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf",
                "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            ];
            for path in candidates {
                if let Ok(bytes) = std::fs::read(path)
                    && let Ok(font) =
                        fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
                {
                    return Some(font);
                }
            }
            None
        })
        .as_ref()
}

/// A rich vector typography text shape powered by `fontdue` for CJK and Unicode typography.
#[derive(Debug, Clone)]
pub struct TextShape {
    pub position: Point2D,
    pub text: String,
    pub font_size: f32,
    pub color: Color,
    pub opacity: f32,
    pub seed: u64,
}

impl TextShape {
    pub fn new(position: Point2D, text: impl Into<String>, font_size: f32, color: Color) -> Self {
        Self {
            position,
            text: text.into(),
            font_size: font_size.clamp(10.0, 120.0),
            color,
            opacity: 1.0,
            seed: generate_shape_seed(),
        }
    }

    /// Measures text dimensions (width, height) using fontdue metrics.
    pub fn measure(&self) -> (f32, f32) {
        if let Some(font) = get_text_font() {
            let line_height = self.font_size * 1.35;
            let mut max_w: f32 = 0.0;
            let mut current_w: f32 = 0.0;
            let mut line_count = 1;

            for c in self.text.chars() {
                if c == '\n' {
                    max_w = max_w.max(current_w);
                    current_w = 0.0;
                    line_count += 1;
                    continue;
                }
                let (metrics, _) = font.rasterize(c, self.font_size);
                current_w += metrics.advance_width;
            }
            max_w = max_w.max(current_w);
            (max_w.max(12.0), (line_count as f32) * line_height)
        } else {
            // Fallback estimation
            let lines: Vec<&str> = self.text.lines().collect();
            let max_len = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1);
            let w = (max_len as f32) * self.font_size * 0.6;
            let h = (lines.len().max(1) as f32) * self.font_size * 1.3;
            (w.max(12.0), h.max(self.font_size))
        }
    }
}

impl Drawable for TextShape {
    fn draw(&self, pixmap: &mut Pixmap) {
        if self.text.is_empty() {
            return;
        }

        let Some(font) = get_text_font() else {
            // Fallback to built-in bitmap font when TrueType fonts are unavailable
            let scale = (self.font_size / 8.0).max(1.0);
            crate::render::BitmapFont::draw_text(
                pixmap,
                self.position.x,
                self.position.y,
                &self.text,
                self.color,
                scale,
            );
            return;
        };

        let font_size = self.font_size;
        let line_step_y = font_size * 1.35;
        let mut cursor_x = self.position.x;
        let mut cursor_y = self.position.y;

        let p_width = pixmap.width() as i32;
        let p_height = pixmap.height() as i32;
        let pixels = pixmap.pixels_mut();

        let cr = self.color.red();
        let cg = self.color.green();
        let cb = self.color.blue();
        let eff_alpha = (self.color.alpha() * self.opacity).clamp(0.0, 1.0);

        for c in self.text.chars() {
            if c == '\n' {
                cursor_x = self.position.x;
                cursor_y += line_step_y;
                continue;
            }

            let (metrics, bitmap) = font.rasterize(c, font_size);
            if metrics.width > 0 && metrics.height > 0 {
                let baseline_y = cursor_y + font_size * 0.85;
                let glyph_x = cursor_x + metrics.xmin as f32;
                let glyph_y = baseline_y - (metrics.height as f32 + metrics.ymin as f32);

                for row in 0..metrics.height {
                    let py = glyph_y as i32 + row as i32;
                    if py < 0 || py >= p_height {
                        continue;
                    }
                    let row_offset = (py as usize) * (p_width as usize);

                    for col in 0..metrics.width {
                        let px = glyph_x as i32 + col as i32;
                        if px < 0 || px >= p_width {
                            continue;
                        }

                        let alpha_byte = bitmap[row * metrics.width + col];
                        if alpha_byte == 0 {
                            continue;
                        }

                        let alpha = (alpha_byte as f32 / 255.0) * eff_alpha;
                        let r = (cr * alpha * 255.0).round() as u8;
                        let g = (cg * alpha * 255.0).round() as u8;
                        let b = (cb * alpha * 255.0).round() as u8;
                        let a_u8 = (alpha * 255.0).round() as u8;

                        let idx = row_offset + (px as usize);
                        let dst = &mut pixels[idx];
                        let inv_a = 255 - a_u8 as u32;
                        let dst_r = (dst.red() as u32 * inv_a) / 255 + r as u32;
                        let dst_g = (dst.green() as u32 * inv_a) / 255 + g as u32;
                        let dst_b = (dst.blue() as u32 * inv_a) / 255 + b as u32;
                        let dst_a = (dst.alpha() as u32 * inv_a) / 255 + a_u8 as u32;

                        if let Some(blended) = PremultipliedColorU8::from_rgba(
                            dst_r.min(255) as u8,
                            dst_g.min(255) as u8,
                            dst_b.min(255) as u8,
                            dst_a.min(255) as u8,
                        ) {
                            *dst = blended;
                        }
                    }
                }
            }
            cursor_x += metrics.advance_width;
        }
    }

    fn intersects(&self, point: Point2D, radius: f32) -> bool {
        let (w, h) = self.measure();
        let bounds = Rect2D::from_xywh(self.position.x, self.position.y, w, h).expand(radius);
        bounds.contains(point)
    }

    fn hit_test(&self, point: Point2D) -> bool {
        let (w, h) = self.measure();
        let bounds = Rect2D::from_xywh(self.position.x, self.position.y, w, h).expand(6.0);
        bounds.contains(point)
    }

    fn bounding_box(&self) -> Option<Rect2D> {
        let (w, h) = self.measure();
        Some(Rect2D::from_xywh(self.position.x, self.position.y, w, h).expand(4.0))
    }

    fn translate(&mut self, offset: Point2D) {
        self.position = self.position + offset;
    }

    fn resize(&mut self, old_bounds: Rect2D, new_bounds: Rect2D) {
        if old_bounds.width() <= 0.0 || old_bounds.height() <= 0.0 {
            return;
        }
        let tx = (self.position.x - old_bounds.min_x) / old_bounds.width();
        let ty = (self.position.y - old_bounds.min_y) / old_bounds.height();
        self.position.x = new_bounds.min_x + tx * new_bounds.width();
        self.position.y = new_bounds.min_y + ty * new_bounds.height();

        let scale = new_bounds.height() / old_bounds.height();
        self.font_size = (self.font_size * scale).clamp(10.0, 120.0);
    }

    fn clone_box(&self) -> Box<dyn Drawable> {
        Box::new(self.clone())
    }

    fn stroke_color(&self) -> Color {
        self.color
    }

    fn set_stroke_color(&mut self, color: Color) {
        self.color = color;
    }

    fn stroke_width(&self) -> f32 {
        self.font_size
    }

    fn set_stroke_width(&mut self, width: f32) {
        self.font_size = width.clamp(10.0, 120.0);
    }

    fn stroke_style(&self) -> StrokeStyle {
        StrokeStyle::Solid
    }

    fn sloppiness(&self) -> Sloppiness {
        Sloppiness::Architect
    }

    fn corner_style(&self) -> CornerStyle {
        CornerStyle::Sharp
    }

    fn opacity(&self) -> f32 {
        self.opacity
    }

    fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
    }

    fn seed(&self) -> u64 {
        self.seed
    }

    fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
    }

    fn text_content(&self) -> Option<&str> {
        Some(&self.text)
    }

    fn set_text_content(&mut self, text: String) {
        self.text = text;
    }
}
