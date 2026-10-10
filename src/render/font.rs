use std::sync::OnceLock;
use tiny_skia::{Color, Paint, Pixmap, PremultipliedColorU8, Rect, Transform};

/// Static cache for system CJK/Unicode font loaded via fontdue.
static SYSTEM_FONT: OnceLock<Option<fontdue::Font>> = OnceLock::new();

fn get_system_font() -> Option<&'static fontdue::Font> {
    SYSTEM_FONT
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

/// High-performance font renderer supporting both Unicode (Chinese/CJK) and ASCII,
/// with automatic antialiasing and fallback to a zero-dependency 8x8 bitmap font table.
pub struct BitmapFont;

impl BitmapFont {
    pub const CHAR_WIDTH: f32 = 8.0;
    pub const CHAR_HEIGHT: f32 = 8.0;

    /// Renders a single ASCII character onto the pixmap at (x, y) with a given scale and color.
    pub fn draw_char(pixmap: &mut Pixmap, x: f32, y: f32, c: char, color: Color, scale: f32) {
        let glyph = get_glyph(c);
        let mut paint = Paint::default();
        paint.set_color(color);

        for (row, &row_byte) in glyph.iter().enumerate() {
            for col in 0..8 {
                if (row_byte & (0x80 >> col)) != 0 {
                    let px = x + (col as f32) * scale;
                    let py = y + (row as f32) * scale;
                    if let Some(rect) = Rect::from_xywh(px, py, scale, scale) {
                        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                    }
                }
            }
        }
    }

    /// Renders a text string with TrueType antialiasing (supports both Chinese & ASCII),
    /// falling back to the built-in 8x8 bitmap font if system fonts are unavailable.
    pub fn draw_text(pixmap: &mut Pixmap, x: f32, y: f32, text: &str, color: Color, scale: f32) {
        if let Some(font) = get_system_font() {
            let font_size = (11.5 * scale).max(9.0);
            let mut cursor_x = x;
            let mut cursor_y = y;
            let line_step_y = font_size * 1.35;

            let p_width = pixmap.width() as i32;
            let p_height = pixmap.height() as i32;
            let pixels = pixmap.pixels_mut();

            let cr = color.red();
            let cg = color.green();
            let cb = color.blue();
            let ca = color.alpha();

            for c in text.chars() {
                if c == '\n' {
                    cursor_x = x;
                    cursor_y += line_step_y;
                    continue;
                }

                let (metrics, bitmap) = font.rasterize(c, font_size);
                if metrics.width > 0 && metrics.height > 0 {
                    let baseline_y = cursor_y + font_size * 0.82;
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

                            let alpha = (alpha_byte as f32 / 255.0) * ca;
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
        } else {
            Self::draw_bitmap_text(pixmap, x, y, text, color, scale);
        }
    }

    /// Fallback bitmap text rendering.
    fn draw_bitmap_text(pixmap: &mut Pixmap, x: f32, y: f32, text: &str, color: Color, scale: f32) {
        let mut cursor_x = x;
        let mut cursor_y = y;
        let char_step_x = (Self::CHAR_WIDTH + 1.0) * scale;
        let line_step_y = (Self::CHAR_HEIGHT + 4.0) * scale;

        for c in text.chars() {
            if c == '\n' {
                cursor_x = x;
                cursor_y += line_step_y;
                continue;
            }
            Self::draw_char(pixmap, cursor_x, cursor_y, c, color, scale);
            cursor_x += char_step_x;
        }
    }

    /// Returns the bounding dimensions (width, height) of a rendered string.
    pub fn measure_text(text: &str, scale: f32) -> (f32, f32) {
        if let Some(font) = get_system_font() {
            let font_size = (11.5 * scale).max(9.0);
            let mut width: f32 = 0.0;
            for c in text.chars() {
                if c == '\n' {
                    continue;
                }
                let metrics = font.metrics(c, font_size);
                width += metrics.advance_width;
            }
            (width, font_size * 1.3)
        } else {
            Self::measure_bitmap_text(text, scale)
        }
    }

    fn measure_bitmap_text(text: &str, scale: f32) -> (f32, f32) {
        let mut max_width: f32 = 0.0;
        let mut current_width: f32 = 0.0;
        let mut lines = 1;

        let char_step_x = (Self::CHAR_WIDTH + 1.0) * scale;
        let line_step_y = (Self::CHAR_HEIGHT + 4.0) * scale;

        for c in text.chars() {
            if c == '\n' {
                if current_width > max_width {
                    max_width = current_width;
                }
                current_width = 0.0;
                lines += 1;
            } else {
                current_width += char_step_x;
            }
        }

        if current_width > max_width {
            max_width = current_width;
        }

        let height = (lines as f32) * line_step_y;
        (max_width, height)
    }
}

/// 8x8 bitmap font table for ASCII characters.
fn get_glyph(c: char) -> [u8; 8] {
    match c {
        '0' => [0x3C, 0x66, 0x6E, 0x76, 0x66, 0x66, 0x3C, 0x00],
        '1' => [0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00],
        '2' => [0x3C, 0x66, 0x06, 0x1C, 0x30, 0x60, 0x7E, 0x00],
        '3' => [0x3C, 0x66, 0x06, 0x1C, 0x06, 0x66, 0x3C, 0x00],
        '4' => [0x0C, 0x1C, 0x3C, 0x6C, 0xFE, 0x0C, 0x0C, 0x00],
        '5' => [0x7E, 0x60, 0x7C, 0x06, 0x06, 0x66, 0x3C, 0x00],
        '6' => [0x1C, 0x30, 0x60, 0x7C, 0x66, 0x66, 0x3C, 0x00],
        '7' => [0x7E, 0x06, 0x0C, 0x18, 0x30, 0x30, 0x30, 0x00],
        '8' => [0x3C, 0x66, 0x66, 0x3C, 0x66, 0x66, 0x3C, 0x00],
        '9' => [0x3C, 0x66, 0x66, 0x3E, 0x06, 0x0C, 0x38, 0x00],

        'A' => [0x18, 0x3C, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00],
        'B' => [0x7C, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x7C, 0x00],
        'C' => [0x3C, 0x66, 0x60, 0x60, 0x60, 0x66, 0x3C, 0x00],
        'D' => [0x78, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0x78, 0x00],
        'E' => [0x7E, 0x60, 0x60, 0x78, 0x60, 0x60, 0x7E, 0x00],
        'F' => [0x7E, 0x60, 0x60, 0x78, 0x60, 0x60, 0x60, 0x00],
        'G' => [0x3C, 0x66, 0x60, 0x6E, 0x66, 0x66, 0x3A, 0x00],
        'H' => [0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00],
        'I' => [0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00],
        'J' => [0x0E, 0x06, 0x06, 0x06, 0x06, 0x66, 0x3C, 0x00],
        'K' => [0x66, 0x6C, 0x78, 0x70, 0x78, 0x6C, 0x66, 0x00],
        'L' => [0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x7E, 0x00],
        'M' => [0x63, 0x77, 0x7F, 0x6B, 0x63, 0x63, 0x63, 0x00],
        'N' => [0x66, 0x76, 0x7E, 0x7E, 0x6E, 0x66, 0x66, 0x00],
        'O' => [0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00],
        'P' => [0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0x00],
        'Q' => [0x3C, 0x66, 0x66, 0x66, 0x6A, 0x6C, 0x36, 0x00],
        'R' => [0x7C, 0x66, 0x66, 0x7C, 0x6C, 0x66, 0x66, 0x00],
        'S' => [0x3C, 0x66, 0x60, 0x3C, 0x06, 0x66, 0x3C, 0x00],
        'T' => [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00],
        'U' => [0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00],
        'V' => [0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00],
        'W' => [0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0x00],
        'X' => [0x66, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x66, 0x00],
        'Y' => [0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x18, 0x00],
        'Z' => [0x7E, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x7E, 0x00],

        'a' => [0x00, 0x00, 0x3C, 0x06, 0x3E, 0x66, 0x3E, 0x00],
        'b' => [0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x7C, 0x00],
        'c' => [0x00, 0x00, 0x3C, 0x66, 0x60, 0x66, 0x3C, 0x00],
        'd' => [0x06, 0x06, 0x3E, 0x66, 0x66, 0x66, 0x3E, 0x00],
        'e' => [0x00, 0x00, 0x3C, 0x66, 0x7E, 0x60, 0x3C, 0x00],
        'f' => [0x0C, 0x18, 0x7E, 0x18, 0x18, 0x18, 0x18, 0x00],
        'g' => [0x00, 0x00, 0x3E, 0x66, 0x66, 0x3E, 0x06, 0x3C],
        'h' => [0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x00],
        'i' => [0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x3C, 0x00],
        'j' => [0x06, 0x00, 0x06, 0x06, 0x06, 0x66, 0x3C, 0x00],
        'k' => [0x60, 0x60, 0x66, 0x6C, 0x78, 0x6C, 0x66, 0x00],
        'l' => [0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00],
        'm' => [0x00, 0x00, 0x66, 0x7F, 0x6B, 0x63, 0x63, 0x00],
        'n' => [0x00, 0x00, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x00],
        'o' => [0x00, 0x00, 0x3C, 0x66, 0x66, 0x66, 0x3C, 0x00],
        'p' => [0x00, 0x00, 0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60],
        'q' => [0x00, 0x00, 0x3E, 0x66, 0x66, 0x3E, 0x06, 0x06],
        'r' => [0x00, 0x00, 0x5C, 0x66, 0x60, 0x60, 0x60, 0x00],
        's' => [0x00, 0x00, 0x3E, 0x60, 0x3C, 0x06, 0x7C, 0x00],
        't' => [0x18, 0x18, 0x7E, 0x18, 0x18, 0x18, 0x0E, 0x00],
        'u' => [0x00, 0x00, 0x66, 0x66, 0x66, 0x66, 0x3E, 0x00],
        'v' => [0x00, 0x00, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00],
        'w' => [0x00, 0x00, 0x63, 0x6B, 0x6B, 0x7F, 0x36, 0x00],
        'x' => [0x00, 0x00, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x00],
        'y' => [0x00, 0x00, 0x66, 0x66, 0x66, 0x3E, 0x06, 0x3C],
        'z' => [0x00, 0x00, 0x7E, 0x0C, 0x18, 0x30, 0x7E, 0x00],

        ':' => [0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00, 0x00],
        '-' => [0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00],
        '>' => [0x60, 0x30, 0x18, 0x0C, 0x18, 0x30, 0x60, 0x00],
        '<' => [0x06, 0x0C, 0x18, 0x30, 0x18, 0x0C, 0x06, 0x00],
        '[' => [0x3C, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3C, 0x00],
        ']' => [0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3C, 0x00],
        '(' => [0x0C, 0x18, 0x30, 0x30, 0x30, 0x18, 0x0C, 0x00],
        ')' => [0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x18, 0x30, 0x00],
        '/' => [0x02, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x40, 0x00],
        '|' => [0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30],
        '!' => [0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00],
        '?' => [0x3C, 0x66, 0x0C, 0x18, 0x18, 0x00, 0x18, 0x00],
        '+' => [0x00, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x00, 0x00],
        '=' => [0x00, 0x7E, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00],
        '_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00],
        _ => [0x00; 8], // space and fallback
    }
}
