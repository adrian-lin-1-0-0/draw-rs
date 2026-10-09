use super::font::BitmapFont;
use crate::app::state::{AppMode, DrawingTool, PaletteColor};
use crate::models::point::Point2D;
use tiny_skia::{Color, FillRule, Paint, Path, PathBuilder, Pixmap, Stroke, Transform};

/// Targets on the HUD overlay that can be clicked or dragged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudHitTarget {
    None,
    DragHeader,
    Tool(DrawingTool),
    Color(PaletteColor),
    ToggleMode,
    Undo,
    Clear,
}

/// Bounding rectangle for an interactive HUD button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HudButtonRect {
    pub target: HudHitTarget,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl HudButtonRect {
    pub fn contains(&self, point: Point2D) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }
}

/// Renders a sleek, translucent floating HUD card displaying the operational
/// state, tools menu, palette colors, and quick actions.
/// Supports clicking any menu button as well as dragging the card across the screen.
pub struct HudOverlay;

impl HudOverlay {
    pub const CARD_BASE_WIDTH: f32 = 510.0;
    pub const CARD_BASE_HEIGHT: f32 = 116.0;

    /// Calculate the physical pixel size of the HUD card based on display scale factor.
    pub fn get_card_size(scale_factor: f32) -> (f32, f32) {
        let ui_scale = (scale_factor.max(1.0) * 0.9).clamp(1.0, 2.5);
        (
            Self::CARD_BASE_WIDTH * ui_scale,
            Self::CARD_BASE_HEIGHT * ui_scale,
        )
    }

    /// Check if a physical cursor coordinate lies within the HUD card bounding box.
    pub fn contains_point(hud_pos: Point2D, point: Point2D, scale_factor: f32) -> bool {
        let (width, height) = Self::get_card_size(scale_factor);
        point.x >= hud_pos.x
            && point.x <= hud_pos.x + width
            && point.y >= hud_pos.y
            && point.y <= hud_pos.y + height
    }

    /// Computes the exact bounding rectangles of all interactive buttons on the HUD.
    pub fn get_buttons(hud_pos: Point2D, scale_factor: f32) -> [HudButtonRect; 12] {
        let ui_scale = (scale_factor.max(1.0) * 0.9).clamp(1.0, 2.5);
        let card_x = hud_pos.x;
        let card_y = hud_pos.y;

        [
            // Row 1: Mode Toggle button
            HudButtonRect {
                target: HudHitTarget::ToggleMode,
                x: card_x + 288.0 * ui_scale,
                y: card_y + 8.0 * ui_scale,
                width: 122.0 * ui_scale,
                height: 22.0 * ui_scale,
            },
            // Row 2: Tool Pen
            HudButtonRect {
                target: HudHitTarget::Tool(DrawingTool::Pen),
                x: card_x + 68.0 * ui_scale,
                y: card_y + 42.0 * ui_scale,
                width: 56.0 * ui_scale,
                height: 24.0 * ui_scale,
            },
            // Row 2: Tool Circle
            HudButtonRect {
                target: HudHitTarget::Tool(DrawingTool::Circle),
                x: card_x + 130.0 * ui_scale,
                y: card_y + 42.0 * ui_scale,
                width: 68.0 * ui_scale,
                height: 24.0 * ui_scale,
            },
            // Row 2: Tool Arrow
            HudButtonRect {
                target: HudHitTarget::Tool(DrawingTool::Arrow),
                x: card_x + 204.0 * ui_scale,
                y: card_y + 42.0 * ui_scale,
                width: 64.0 * ui_scale,
                height: 24.0 * ui_scale,
            },
            // Row 2: Tool Eraser
            HudButtonRect {
                target: HudHitTarget::Tool(DrawingTool::Eraser),
                x: card_x + 274.0 * ui_scale,
                y: card_y + 42.0 * ui_scale,
                width: 76.0 * ui_scale,
                height: 24.0 * ui_scale,
            },
            // Row 2: Action Undo
            HudButtonRect {
                target: HudHitTarget::Undo,
                x: card_x + 365.0 * ui_scale,
                y: card_y + 42.0 * ui_scale,
                width: 64.0 * ui_scale,
                height: 24.0 * ui_scale,
            },
            // Row 2: Action Clear
            HudButtonRect {
                target: HudHitTarget::Clear,
                x: card_x + 435.0 * ui_scale,
                y: card_y + 42.0 * ui_scale,
                width: 64.0 * ui_scale,
                height: 24.0 * ui_scale,
            },
            // Row 3: Color 1 (Cyan)
            HudButtonRect {
                target: HudHitTarget::Color(PaletteColor::Cyan),
                x: card_x + 68.0 * ui_scale,
                y: card_y + 79.0 * ui_scale,
                width: 78.0 * ui_scale,
                height: 24.0 * ui_scale,
            },
            // Row 3: Color 2 (Emerald)
            HudButtonRect {
                target: HudHitTarget::Color(PaletteColor::Emerald),
                x: card_x + 152.0 * ui_scale,
                y: card_y + 79.0 * ui_scale,
                width: 90.0 * ui_scale,
                height: 24.0 * ui_scale,
            },
            // Row 3: Color 3 (Coral)
            HudButtonRect {
                target: HudHitTarget::Color(PaletteColor::Coral),
                x: card_x + 248.0 * ui_scale,
                y: card_y + 79.0 * ui_scale,
                width: 78.0 * ui_scale,
                height: 24.0 * ui_scale,
            },
            // Row 3: Color 4 (Amber)
            HudButtonRect {
                target: HudHitTarget::Color(PaletteColor::Amber),
                x: card_x + 332.0 * ui_scale,
                y: card_y + 79.0 * ui_scale,
                width: 78.0 * ui_scale,
                height: 24.0 * ui_scale,
            },
            // Row 3: Color 5 (Violet)
            HudButtonRect {
                target: HudHitTarget::Color(PaletteColor::Violet),
                x: card_x + 416.0 * ui_scale,
                y: card_y + 79.0 * ui_scale,
                width: 82.0 * ui_scale,
                height: 24.0 * ui_scale,
            },
        ]
    }

    /// Performs hit-testing against the HUD.
    /// Returns the specific button hit, or `DragHeader` if clicking elsewhere on the card,
    /// or `None` if completely outside.
    pub fn hit_test(hud_pos: Point2D, point: Point2D, scale_factor: f32) -> HudHitTarget {
        let buttons = Self::get_buttons(hud_pos, scale_factor);
        for btn in buttons {
            if btn.contains(point) {
                return btn.target;
            }
        }

        if Self::contains_point(hud_pos, point, scale_factor) {
            HudHitTarget::DragHeader
        } else {
            HudHitTarget::None
        }
    }

    /// Renders the entire interactive HUD onto the pixmap.
    pub fn render(
        pixmap: &mut Pixmap,
        mode: AppMode,
        tool: DrawingTool,
        color: PaletteColor,
        shape_count: usize,
        scale_factor: f32,
        hud_pos: Point2D,
    ) {
        let ui_scale = (scale_factor.max(1.0) * 0.9).clamp(1.0, 2.5);
        let card_x = hud_pos.x;
        let card_y = hud_pos.y;
        let (card_width, card_height) = Self::get_card_size(scale_factor);
        let corner_radius = 12.0 * ui_scale;

        // 1. Draw Glassmorphic Card Background
        if let Some(bg_path) =
            Self::rounded_rect_path(card_x, card_y, card_width, card_height, corner_radius)
        {
            // Dark translucent slate fill (rgba(15, 23, 42, 0.90))
            let mut bg_paint = Paint::default();
            bg_paint.set_color(Color::from_rgba8(15, 23, 42, 230));
            bg_paint.anti_alias = true;
            pixmap.fill_path(
                &bg_path,
                &bg_paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );

            // Subtle luminous border
            let mut border_paint = Paint::default();
            let border_color = match mode {
                AppMode::Drawing => Color::from_rgba8(34, 197, 94, 180), // Emerald accent
                AppMode::ClickThrough => Color::from_rgba8(56, 189, 248, 160), // Sky blue accent
            };
            border_paint.set_color(border_color);
            border_paint.anti_alias = true;

            let stroke = Stroke {
                width: 1.5 * ui_scale,
                ..Default::default()
            };
            pixmap.stroke_path(
                &bg_path,
                &border_paint,
                &stroke,
                Transform::identity(),
                None,
            );
        }

        // 2. Draw Dividers
        Self::draw_horizontal_line(
            pixmap,
            card_x + 12.0 * ui_scale,
            card_y + 36.0 * ui_scale,
            card_width - 24.0 * ui_scale,
            Color::from_rgba8(255, 255, 255, 20),
            1.0 * ui_scale,
        );
        Self::draw_horizontal_line(
            pixmap,
            card_x + 12.0 * ui_scale,
            card_y + 72.0 * ui_scale,
            card_width - 24.0 * ui_scale,
            Color::from_rgba8(255, 255, 255, 20),
            1.0 * ui_scale,
        );

        // 3. Row 1: Header (Status Dot + Title + Shapes Count + Mode Toggle + Drag Grip)
        let dot_x = card_x + 18.0 * ui_scale;
        let dot_y = card_y + 19.0 * ui_scale;
        let dot_radius = 4.5 * ui_scale;

        let dot_color = match mode {
            AppMode::Drawing => Color::from_rgba8(34, 197, 94, 255),
            AppMode::ClickThrough => Color::from_rgba8(56, 189, 248, 255),
        };
        let mut dot_paint = Paint::default();
        dot_paint.set_color(dot_color);
        dot_paint.anti_alias = true;

        let mut dot_pb = PathBuilder::new();
        dot_pb.push_circle(dot_x, dot_y, dot_radius);
        if let Some(dot_path) = dot_pb.finish() {
            pixmap.fill_path(
                &dot_path,
                &dot_paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }

        let mode_title = match mode {
            AppMode::Drawing => "DRAWING",
            AppMode::ClickThrough => "CLICK-THRU",
        };
        let title_color = match mode {
            AppMode::Drawing => Color::from_rgba8(74, 222, 128, 255), // Green 400
            AppMode::ClickThrough => Color::from_rgba8(56, 189, 248, 255), // Sky 400
        };
        BitmapFont::draw_text(
            pixmap,
            dot_x + 10.0 * ui_scale,
            card_y + 13.0 * ui_scale,
            mode_title,
            title_color,
            1.15 * ui_scale,
        );

        let shape_str = format!("({shape_count} shapes)");
        BitmapFont::draw_text(
            pixmap,
            card_x + 135.0 * ui_scale,
            card_y + 15.0 * ui_scale,
            &shape_str,
            Color::from_rgba8(148, 163, 184, 220),
            0.88 * ui_scale,
        );

        // Mode Toggle Button (Row 1)
        let mode_btn_text = match mode {
            AppMode::Drawing => "[Pass-Thru F1]",
            AppMode::ClickThrough => "[Draw Mode F1]",
        };
        let buttons = Self::get_buttons(hud_pos, scale_factor);
        let mode_btn = buttons[0];
        Self::draw_button_pill(
            pixmap,
            &mode_btn,
            4.0 * ui_scale,
            Color::from_rgba8(56, 189, 248, 30),
            Color::from_rgba8(56, 189, 248, 120),
            ui_scale,
        );
        Self::draw_centered_text(
            pixmap,
            &mode_btn,
            mode_btn_text,
            Color::from_rgba8(186, 230, 253, 255),
            0.85 * ui_scale,
        );

        // Drag grip hint (Row 1 right)
        BitmapFont::draw_text(
            pixmap,
            card_x + 424.0 * ui_scale,
            card_y + 15.0 * ui_scale,
            "::: Drag",
            Color::from_rgba8(148, 163, 184, 180),
            0.88 * ui_scale,
        );

        // 4. Row 2: Tools (Label + Pen + Circle + Arrow + Eraser + Undo + Clear)
        BitmapFont::draw_text(
            pixmap,
            card_x + 14.0 * ui_scale,
            card_y + 48.0 * ui_scale,
            "TOOLS:",
            Color::from_rgba8(148, 163, 184, 220),
            0.92 * ui_scale,
        );

        let tool_items = [
            (buttons[1], DrawingTool::Pen, "Pen[P]"),
            (buttons[2], DrawingTool::Circle, "Circle"),
            (buttons[3], DrawingTool::Arrow, "Arrow"),
            (buttons[4], DrawingTool::Eraser, "Eraser[E]"),
        ];

        for (btn, t, label) in tool_items {
            let is_active = tool == t;
            let (bg, border, text_col) = if is_active {
                (
                    Color::from_rgba8(56, 189, 248, 60),
                    Color::from_rgba8(56, 189, 248, 255),
                    Color::from_rgba8(255, 255, 255, 255),
                )
            } else {
                (
                    Color::from_rgba8(255, 255, 255, 12),
                    Color::from_rgba8(255, 255, 255, 30),
                    Color::from_rgba8(203, 213, 225, 200),
                )
            };

            Self::draw_button_pill(
                pixmap,
                &btn,
                4.0 * ui_scale,
                bg,
                border,
                if is_active {
                    1.6 * ui_scale
                } else {
                    1.0 * ui_scale
                },
            );
            Self::draw_centered_text(pixmap, &btn, label, text_col, 0.86 * ui_scale);
        }

        // Action: Undo
        let undo_btn = buttons[5];
        Self::draw_button_pill(
            pixmap,
            &undo_btn,
            4.0 * ui_scale,
            Color::from_rgba8(251, 191, 36, 20),
            Color::from_rgba8(251, 191, 36, 100),
            1.0 * ui_scale,
        );
        Self::draw_centered_text(
            pixmap,
            &undo_btn,
            "Undo[Z]",
            Color::from_rgba8(253, 230, 138, 240),
            0.86 * ui_scale,
        );

        // Action: Clear
        let clear_btn = buttons[6];
        Self::draw_button_pill(
            pixmap,
            &clear_btn,
            4.0 * ui_scale,
            Color::from_rgba8(248, 113, 113, 20),
            Color::from_rgba8(248, 113, 113, 100),
            1.0 * ui_scale,
        );
        Self::draw_centered_text(
            pixmap,
            &clear_btn,
            "Clear[C]",
            Color::from_rgba8(254, 202, 202, 240),
            0.86 * ui_scale,
        );

        // 5. Row 3: Colors (Label + 5 Interactive Color Swatch Pills)
        BitmapFont::draw_text(
            pixmap,
            card_x + 14.0 * ui_scale,
            card_y + 85.0 * ui_scale,
            "COLOR:",
            Color::from_rgba8(148, 163, 184, 220),
            0.92 * ui_scale,
        );

        let color_items = [
            (buttons[7], PaletteColor::Cyan, "1:Cyan"),
            (buttons[8], PaletteColor::Emerald, "2:Emerald"),
            (buttons[9], PaletteColor::Coral, "3:Coral"),
            (buttons[10], PaletteColor::Amber, "4:Amber"),
            (buttons[11], PaletteColor::Violet, "5:Violet"),
        ];

        for (btn, col, name) in color_items {
            let is_active = color == col;
            let c = col.to_color();

            let (bg, border, text_col) = if is_active {
                (
                    Color::from_rgba(c.red(), c.green(), c.blue(), 0.22).unwrap_or(c),
                    c,
                    Color::from_rgba8(255, 255, 255, 255),
                )
            } else {
                (
                    Color::from_rgba8(255, 255, 255, 10),
                    Color::from_rgba8(255, 255, 255, 28),
                    Color::from_rgba8(203, 213, 225, 200),
                )
            };

            Self::draw_button_pill(
                pixmap,
                &btn,
                4.0 * ui_scale,
                bg,
                border,
                if is_active {
                    1.6 * ui_scale
                } else {
                    1.0 * ui_scale
                },
            );

            // Draw circular swatch dot inside pill
            let swatch_x = btn.x + 9.0 * ui_scale;
            let swatch_y = btn.y + btn.height / 2.0;
            let swatch_r = 4.0 * ui_scale;
            let mut swatch_paint = Paint::default();
            swatch_paint.set_color(c);
            swatch_paint.anti_alias = true;

            let mut swatch_pb = PathBuilder::new();
            swatch_pb.push_circle(swatch_x, swatch_y, swatch_r);
            if let Some(swatch_path) = swatch_pb.finish() {
                pixmap.fill_path(
                    &swatch_path,
                    &swatch_paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }

            // Draw color label
            let font_scale = 0.82 * ui_scale;
            let (_, text_h) = BitmapFont::measure_text(name, font_scale);
            let text_y = btn.y + (btn.height - text_h) / 2.0;
            BitmapFont::draw_text(
                pixmap,
                btn.x + 18.0 * ui_scale,
                text_y,
                name,
                text_col,
                font_scale,
            );
        }
    }

    /// Helper to render a rounded rectangle button pill.
    fn draw_button_pill(
        pixmap: &mut Pixmap,
        btn: &HudButtonRect,
        radius: f32,
        bg_color: Color,
        border_color: Color,
        stroke_width: f32,
    ) {
        if let Some(path) = Self::rounded_rect_path(btn.x, btn.y, btn.width, btn.height, radius) {
            let mut bg_paint = Paint::default();
            bg_paint.set_color(bg_color);
            bg_paint.anti_alias = true;
            pixmap.fill_path(
                &path,
                &bg_paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );

            let mut border_paint = Paint::default();
            border_paint.set_color(border_color);
            border_paint.anti_alias = true;
            let stroke = Stroke {
                width: stroke_width,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
        }
    }

    /// Helper to render horizontally and vertically centered text.
    fn draw_centered_text(
        pixmap: &mut Pixmap,
        btn: &HudButtonRect,
        text: &str,
        color: Color,
        scale: f32,
    ) {
        let (text_w, text_h) = BitmapFont::measure_text(text, scale);
        let tx = btn.x + (btn.width - text_w).max(0.0) / 2.0;
        let ty = btn.y + (btn.height - text_h).max(0.0) / 2.0;
        BitmapFont::draw_text(pixmap, tx, ty, text, color, scale);
    }

    /// Helper to draw a horizontal dividing line.
    fn draw_horizontal_line(
        pixmap: &mut Pixmap,
        x: f32,
        y: f32,
        len: f32,
        color: Color,
        thickness: f32,
    ) {
        let mut pb = PathBuilder::new();
        pb.move_to(x, y);
        pb.line_to(x + len, y);
        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(color);
            let stroke = Stroke {
                width: thickness,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    /// Generates a smooth rounded rectangle path.
    fn rounded_rect_path(x: f32, y: f32, width: f32, height: f32, radius: f32) -> Option<Path> {
        let r = radius.min(width / 2.0).min(height / 2.0);
        let mut pb = PathBuilder::new();

        pb.move_to(x + r, y);
        pb.line_to(x + width - r, y);
        pb.quad_to(x + width, y, x + width, y + r);
        pb.line_to(x + width, y + height - r);
        pb.quad_to(x + width, y + height, x + width - r, y + height);
        pb.line_to(x + r, y + height);
        pb.quad_to(x, y + height, x, y + height - r);
        pb.line_to(x, y + r);
        pb.quad_to(x, y, x + r, y);
        pb.close();

        pb.finish()
    }
}
