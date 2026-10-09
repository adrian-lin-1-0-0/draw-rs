use super::font::BitmapFont;
use crate::app::state::{AppMode, DrawingTool, PaletteColor};
use tiny_skia::{Color, FillRule, Paint, Path, PathBuilder, Pixmap, Stroke, Transform};

/// Renders a sleek, translucent floating HUD card in the top-left corner
/// displaying the current operational state, active tool, shape count, and keybindings.
pub struct HudOverlay;

impl HudOverlay {
    pub fn render(
        pixmap: &mut Pixmap,
        mode: AppMode,
        tool: DrawingTool,
        color: PaletteColor,
        shape_count: usize,
        scale_factor: f32,
    ) {
        let ui_scale = (scale_factor.max(1.0) * 0.9).clamp(1.0, 2.5);
        let padding = 16.0 * ui_scale;
        let card_x = 24.0 * ui_scale;
        let card_y = 24.0 * ui_scale;
        let card_width = 380.0 * ui_scale;
        let card_height = 84.0 * ui_scale;
        let corner_radius = 12.0 * ui_scale;

        // 1. Draw Glassmorphic Card Background
        if let Some(bg_path) = Self::rounded_rect_path(
            card_x,
            card_y,
            card_width,
            card_height,
            corner_radius,
        ) {
            // Dark translucent slate fill (rgba(15, 23, 42, 0.88))
            let mut bg_paint = Paint::default();
            bg_paint.set_color(Color::from_rgba8(15, 23, 42, 225));
            bg_paint.anti_alias = true;
            pixmap.fill_path(&bg_path, &bg_paint, FillRule::Winding, Transform::identity(), None);

            // Subtle luminous border
            let mut border_paint = Paint::default();
            let border_color = match mode {
                AppMode::Drawing => Color::from_rgba8(34, 197, 94, 180), // emerald green accent
                AppMode::ClickThrough => Color::from_rgba8(56, 189, 248, 140), // sky blue accent
            };
            border_paint.set_color(border_color);
            border_paint.anti_alias = true;

            let stroke = Stroke {
                width: 1.5 * ui_scale,
                ..Default::default()
            };
            pixmap.stroke_path(&bg_path, &border_paint, &stroke, Transform::identity(), None);
        }

        // 2. Draw Status Indicator Dot
        let dot_x = card_x + padding + 6.0 * ui_scale;
        let dot_y = card_y + padding + 6.0 * ui_scale;
        let dot_radius = 5.0 * ui_scale;

        let dot_color = match mode {
            AppMode::Drawing => Color::from_rgba8(34, 197, 94, 255), // Green (Active draw)
            AppMode::ClickThrough => Color::from_rgba8(56, 189, 248, 255), // Cyan (Passive click-through)
        };
        let mut dot_paint = Paint::default();
        dot_paint.set_color(dot_color);
        dot_paint.anti_alias = true;

        let mut dot_pb = PathBuilder::new();
        dot_pb.push_circle(dot_x, dot_y, dot_radius);
        if let Some(dot_path) = dot_pb.finish() {
            pixmap.fill_path(&dot_path, &dot_paint, FillRule::Winding, Transform::identity(), None);
        }

        // 3. Render Header Row (Mode | Tool | Shape count)
        let mode_text = match mode {
            AppMode::Drawing => "DRAWING",
            AppMode::ClickThrough => "CLICK-THROUGH",
        };
        let tool_name = tool.display_name();

        let header_str = format!("{mode_text}  |  TOOL: {tool_name}  |  SHAPES: {shape_count}");
        let header_color = Color::from_rgba8(248, 250, 252, 255); // White / Slate 50
        let text_scale = 1.15 * ui_scale;

        BitmapFont::draw_text(
            pixmap,
            dot_x + 12.0 * ui_scale,
            card_y + padding,
            &header_str,
            header_color,
            text_scale,
        );

        // 4. Render Action Hints Row
        let sub_hint = match mode {
            AppMode::Drawing => {
                format!(
                    "[F1] Pass-thru  [F2] Tool  [1-5] Color: {}  [Z] Undo  [C] Clear",
                    color.name()
                )
            }
            AppMode::ClickThrough => {
                "[F1] Resume Drawing  |  Clicks pass through to IDE / Browser".to_string()
            }
        };

        let hint_color = match mode {
            AppMode::Drawing => Color::from_rgba8(148, 163, 184, 255), // Slate 400
            AppMode::ClickThrough => Color::from_rgba8(125, 211, 252, 255), // Sky 300
        };
        let hint_scale = 0.95 * ui_scale;

        BitmapFont::draw_text(
            pixmap,
            card_x + padding,
            card_y + padding + 22.0 * ui_scale,
            &sub_hint,
            hint_color,
            hint_scale,
        );
    }

    /// Generates a smooth rounded rectangle path.
    fn rounded_rect_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<Path> {
        let r = r.min(w / 2.0).min(h / 2.0);
        let mut pb = PathBuilder::new();
        pb.move_to(x + r, y);
        pb.line_to(x + w - r, y);
        pb.quad_to(x + w, y, x + w, y + r);
        pb.line_to(x + w, y + h - r);
        pb.quad_to(x + w, y + h, x + w - r, y + h);
        pb.line_to(x + r, y + h);
        pb.quad_to(x, y + h, x, y + h - r);
        pb.line_to(x, y + r);
        pb.quad_to(x, y, x + r, y);
        pb.close();
        pb.finish()
    }
}
