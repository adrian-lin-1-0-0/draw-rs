use super::font::BitmapFont;
use crate::app::i18n::AppLanguage;
use crate::app::state::PaletteColor;
use crate::models::{Point2D, StrokeStyle};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

/// Hit targets for the Shape Properties Inspector HUD.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InspectorHitTarget {
    None,
    Close,
    DragHeader,
    StrokeColor(PaletteColor),
    FillColor(Option<PaletteColor>),
    StrokeWidth(f32),
    StrokeStyle(StrokeStyle),
    LayerFront,
    LayerForward,
    LayerBackward,
    LayerBack,
    Duplicate,
    Delete,
}

/// Parameters passed to the Inspector HUD for rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InspectorRenderParams {
    pub selected_count: usize,
    pub stroke_color: Color,
    pub fill_color: Option<Color>,
    pub stroke_width: f32,
    pub stroke_style: StrokeStyle,
    pub language: AppLanguage,
}

/// Left-side floating Inspector HUD panel for real-time shape attribute customization.
pub struct InspectorOverlay;

impl InspectorOverlay {
    pub const CARD_BASE_WIDTH: f32 = 236.0;
    pub const CARD_BASE_HEIGHT: f32 = 330.0;

    /// Calculate physical pixel dimensions of the inspector card.
    pub fn get_card_size(scale_factor: f32) -> (f32, f32) {
        let ui_scale = (scale_factor.max(1.0) * 0.9).clamp(1.0, 2.5);
        (
            Self::CARD_BASE_WIDTH * ui_scale,
            Self::CARD_BASE_HEIGHT * ui_scale,
        )
    }

    /// Check if cursor coordinate lies within the inspector card bounds.
    pub fn contains_point(inspector_pos: Point2D, point: Point2D, scale_factor: f32) -> bool {
        let (w, h) = Self::get_card_size(scale_factor);
        point.x >= inspector_pos.x
            && point.x <= inspector_pos.x + w
            && point.y >= inspector_pos.y
            && point.y <= inspector_pos.y + h
    }

    /// Hit-tests all interactive controls inside the inspector card.
    pub fn hit_test(
        inspector_pos: Point2D,
        point: Point2D,
        scale_factor: f32,
    ) -> InspectorHitTarget {
        let (w, h) = Self::get_card_size(scale_factor);
        if point.x < inspector_pos.x
            || point.x > inspector_pos.x + w
            || point.y < inspector_pos.y
            || point.y > inspector_pos.y + h
        {
            return InspectorHitTarget::None;
        }

        let ui_scale = (scale_factor.max(1.0) * 0.9).clamp(1.0, 2.5);
        let rx = (point.x - inspector_pos.x) / ui_scale;
        let ry = (point.y - inspector_pos.y) / ui_scale;

        // Header / drag bar (0..34) or Close button
        if ry <= 34.0 {
            if (204.0..=230.0).contains(&rx) && (6.0..=28.0).contains(&ry) {
                return InspectorHitTarget::Close;
            }
            return InspectorHitTarget::DragHeader;
        }

        // Section 1: Stroke Color swatches (ry: 52..76)
        if (52.0..=76.0).contains(&ry) && rx >= 14.0 {
            let col = ((rx - 14.0) / 42.0) as usize;
            if col < PaletteColor::ALL.len() {
                return InspectorHitTarget::StrokeColor(PaletteColor::ALL[col]);
            }
        }

        // Section 2: Fill Color swatches (ry: 98..122)
        if (98.0..=122.0).contains(&ry) && rx >= 14.0 {
            let col = ((rx - 14.0) / 35.0) as usize;
            if col == 0 {
                return InspectorHitTarget::FillColor(None);
            } else if col <= PaletteColor::ALL.len() {
                return InspectorHitTarget::FillColor(Some(PaletteColor::ALL[col - 1]));
            }
        }

        // Section 3: Stroke Width (ry: 144..170)
        if (144.0..=170.0).contains(&ry) && (14.0..=222.0).contains(&rx) {
            let btn_w = (222.0 - 14.0) / 3.0;
            let idx = ((rx - 14.0) / btn_w) as usize;
            match idx {
                0 => return InspectorHitTarget::StrokeWidth(1.5),
                1 => return InspectorHitTarget::StrokeWidth(3.5),
                _ => return InspectorHitTarget::StrokeWidth(6.0),
            }
        }

        // Section 4: Stroke Style (ry: 192..218)
        if (192.0..=218.0).contains(&ry) && (14.0..=222.0).contains(&rx) {
            let btn_w = (222.0 - 14.0) / 3.0;
            let idx = ((rx - 14.0) / btn_w) as usize;
            match idx {
                0 => return InspectorHitTarget::StrokeStyle(StrokeStyle::Solid),
                1 => return InspectorHitTarget::StrokeStyle(StrokeStyle::Dashed),
                _ => return InspectorHitTarget::StrokeStyle(StrokeStyle::Dotted),
            }
        }

        // Section 5: Layers (ry: 240..266)
        if (240.0..=266.0).contains(&ry) && (14.0..=222.0).contains(&rx) {
            let btn_w = (222.0 - 14.0) / 4.0;
            let idx = ((rx - 14.0) / btn_w) as usize;
            match idx {
                0 => return InspectorHitTarget::LayerFront,
                1 => return InspectorHitTarget::LayerForward,
                2 => return InspectorHitTarget::LayerBackward,
                _ => return InspectorHitTarget::LayerBack,
            }
        }

        // Section 6: Actions (ry: 288..318)
        if (288.0..=318.0).contains(&ry) && (14.0..=222.0).contains(&rx) {
            let btn_w = (222.0 - 14.0) / 2.0;
            let idx = ((rx - 14.0) / btn_w) as usize;
            match idx {
                0 => return InspectorHitTarget::Duplicate,
                _ => return InspectorHitTarget::Delete,
            }
        }

        InspectorHitTarget::None
    }

    /// Renders the complete Inspector panel onto the pixmap.
    pub fn render(
        pixmap: &mut Pixmap,
        params: &InspectorRenderParams,
        scale_factor: f32,
        pos: Point2D,
    ) {
        let ui_scale = (scale_factor.max(1.0) * 0.9).clamp(1.0, 2.5);
        let (card_w, card_h) = Self::get_card_size(scale_factor);

        // 1. Frosted dark glass background
        let mut card_pb = PathBuilder::new();
        if let Some(rect) = tiny_skia::Rect::from_xywh(pos.x, pos.y, card_w, card_h) {
            card_pb.push_rect(rect);
            if let Some(card_path) = card_pb.finish() {
                let mut bg_paint = Paint::default();
                bg_paint.set_color(Color::from_rgba8(24, 24, 27, 240));
                bg_paint.anti_alias = true;
                pixmap.fill_path(
                    &card_path,
                    &bg_paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );

                // Subtle border
                let mut border_paint = Paint::default();
                border_paint.set_color(Color::from_rgba8(63, 63, 70, 200));
                let border_stroke = Stroke {
                    width: 1.0 * ui_scale,
                    ..Default::default()
                };
                pixmap.stroke_path(
                    &card_path,
                    &border_paint,
                    &border_stroke,
                    Transform::identity(),
                    None,
                );
            }
        }

        let props = &params.language.strings().properties;

        // 2. Header
        let header_text = params.language.properties_header(params.selected_count);
        BitmapFont::draw_text(
            pixmap,
            pos.x + 14.0 * ui_scale,
            pos.y + 12.0 * ui_scale,
            &header_text,
            Color::from_rgba8(244, 244, 245, 255),
            ui_scale * 0.88,
        );

        // Header close button [✕]
        let close_x = pos.x + card_w - 28.0 * ui_scale;
        let close_y = pos.y + 8.0 * ui_scale;
        let close_s = 18.0 * ui_scale;
        if let Some(r) = tiny_skia::Rect::from_xywh(close_x, close_y, close_s, close_s) {
            let mut close_pb = PathBuilder::new();
            close_pb.push_rect(r);
            if let Some(path) = close_pb.finish() {
                let mut p = Paint::default();
                p.set_color(Color::from_rgba8(39, 39, 42, 200));
                pixmap.fill_path(&path, &p, FillRule::Winding, Transform::identity(), None);
                let mut bp = Paint::default();
                bp.set_color(Color::from_rgba8(63, 63, 70, 160));
                let s = Stroke {
                    width: 1.0 * ui_scale,
                    ..Default::default()
                };
                pixmap.stroke_path(&path, &bp, &s, Transform::identity(), None);
            }
        }
        let mut x_pb = PathBuilder::new();
        x_pb.move_to(close_x + 5.0 * ui_scale, close_y + 5.0 * ui_scale);
        x_pb.line_to(
            close_x + close_s - 5.0 * ui_scale,
            close_y + close_s - 5.0 * ui_scale,
        );
        x_pb.move_to(close_x + close_s - 5.0 * ui_scale, close_y + 5.0 * ui_scale);
        x_pb.line_to(close_x + 5.0 * ui_scale, close_y + close_s - 5.0 * ui_scale);
        if let Some(x_path) = x_pb.finish() {
            let mut p = Paint::default();
            p.set_color(Color::from_rgba8(161, 161, 170, 240));
            let s = Stroke {
                width: 1.5 * ui_scale,
                ..Default::default()
            };
            pixmap.stroke_path(&x_path, &p, &s, Transform::identity(), None);
        }

        // Header separator line
        let mut sep_pb = PathBuilder::new();
        sep_pb.move_to(pos.x + 12.0 * ui_scale, pos.y + 34.0 * ui_scale);
        sep_pb.line_to(pos.x + card_w - 12.0 * ui_scale, pos.y + 34.0 * ui_scale);
        if let Some(sep_path) = sep_pb.finish() {
            let mut sep_paint = Paint::default();
            sep_paint.set_color(Color::from_rgba8(63, 63, 70, 150));
            let s = Stroke {
                width: 1.0 * ui_scale,
                ..Default::default()
            };
            pixmap.stroke_path(&sep_path, &sep_paint, &s, Transform::identity(), None);
        }

        // Section 1: STROKE
        BitmapFont::draw_text(
            pixmap,
            pos.x + 14.0 * ui_scale,
            pos.y + 40.0 * ui_scale,
            &props.stroke_color,
            Color::from_rgba8(161, 161, 170, 255),
            ui_scale * 0.75,
        );
        for (i, pal) in PaletteColor::ALL.iter().enumerate() {
            let cx = pos.x + (14.0 + (i as f32) * 42.0 + 12.0) * ui_scale;
            let cy = pos.y + (52.0 + 12.0) * ui_scale;
            let radius = 10.0 * ui_scale;

            let mut circle_pb = PathBuilder::new();
            circle_pb.push_circle(cx, cy, radius);
            if let Some(path) = circle_pb.finish() {
                let mut p = Paint::default();
                p.set_color(pal.to_color());
                pixmap.fill_path(&path, &p, FillRule::Winding, Transform::identity(), None);

                // Highlight active
                if pal.to_color().red() == params.stroke_color.red()
                    && pal.to_color().green() == params.stroke_color.green()
                    && pal.to_color().blue() == params.stroke_color.blue()
                {
                    let mut ring_paint = Paint::default();
                    ring_paint.set_color(Color::from_rgba8(255, 255, 255, 255));
                    let s = Stroke {
                        width: 2.0 * ui_scale,
                        ..Default::default()
                    };
                    pixmap.stroke_path(&path, &ring_paint, &s, Transform::identity(), None);
                }
            }
        }

        // Section 2: BACKGROUND (FILL)
        BitmapFont::draw_text(
            pixmap,
            pos.x + 14.0 * ui_scale,
            pos.y + 84.0 * ui_scale,
            &props.background_fill,
            Color::from_rgba8(161, 161, 170, 255),
            ui_scale * 0.75,
        );
        // None swatch
        let none_cx = pos.x + (14.0 + 10.0) * ui_scale;
        let none_cy = pos.y + (98.0 + 10.0) * ui_scale;
        let mut none_pb = PathBuilder::new();
        none_pb.push_circle(none_cx, none_cy, 9.0 * ui_scale);
        if let Some(path) = none_pb.finish() {
            let mut p = Paint::default();
            p.set_color(Color::from_rgba8(39, 39, 42, 255));
            pixmap.fill_path(&path, &p, FillRule::Winding, Transform::identity(), None);

            let mut border_p = Paint::default();
            border_p.set_color(if params.fill_color.is_none() {
                Color::from_rgba8(255, 255, 255, 255)
            } else {
                Color::from_rgba8(113, 113, 122, 200)
            });
            let s = Stroke {
                width: 1.5 * ui_scale,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &border_p, &s, Transform::identity(), None);
        }

        // Color swatches for fill
        for (i, pal) in PaletteColor::ALL.iter().enumerate() {
            let cx = pos.x + (14.0 + 35.0 + (i as f32) * 35.0 + 10.0) * ui_scale;
            let cy = pos.y + (98.0 + 10.0) * ui_scale;
            let radius = 9.0 * ui_scale;

            let mut circle_pb = PathBuilder::new();
            circle_pb.push_circle(cx, cy, radius);
            if let Some(path) = circle_pb.finish() {
                let mut p = Paint::default();
                p.set_color(pal.to_color());
                pixmap.fill_path(&path, &p, FillRule::Winding, Transform::identity(), None);

                if let Some(fill) = params.fill_color
                    && pal.to_color().red() == fill.red()
                    && pal.to_color().green() == fill.green()
                    && pal.to_color().blue() == fill.blue()
                {
                    let mut ring = Paint::default();
                    ring.set_color(Color::from_rgba8(255, 255, 255, 255));
                    let s = Stroke {
                        width: 2.0 * ui_scale,
                        ..Default::default()
                    };
                    pixmap.stroke_path(&path, &ring, &s, Transform::identity(), None);
                }
            }
        }

        // Section 3: STROKE WIDTH (S, M, L)
        BitmapFont::draw_text(
            pixmap,
            pos.x + 14.0 * ui_scale,
            pos.y + 130.0 * ui_scale,
            &props.stroke_width,
            Color::from_rgba8(161, 161, 170, 255),
            ui_scale * 0.75,
        );
        let widths = [
            (&props.width_thin, 1.5f32),
            (&props.width_med, 3.5f32),
            (&props.width_thick, 6.0f32),
        ];
        let btn_w = (card_w - 28.0 * ui_scale) / 3.0;
        for (i, (label, val)) in widths.iter().enumerate() {
            let bx = pos.x + (14.0 * ui_scale) + (i as f32) * btn_w;
            let by = pos.y + 144.0 * ui_scale;
            let bh = 24.0 * ui_scale;
            let is_active = (params.stroke_width - val).abs() < 1.0;
            render_segment_button(
                pixmap,
                bx,
                by,
                btn_w - 4.0 * ui_scale,
                bh,
                label,
                is_active,
                ui_scale,
            );
        }

        // Section 4: STROKE STYLE (Solid, Dashed, Dotted)
        BitmapFont::draw_text(
            pixmap,
            pos.x + 14.0 * ui_scale,
            pos.y + 178.0 * ui_scale,
            &props.stroke_style,
            Color::from_rgba8(161, 161, 170, 255),
            ui_scale * 0.75,
        );
        let styles = [
            (&props.style_solid, StrokeStyle::Solid),
            (&props.style_dashed, StrokeStyle::Dashed),
            (&props.style_dotted, StrokeStyle::Dotted),
        ];
        for (i, (label, val)) in styles.iter().enumerate() {
            let bx = pos.x + (14.0 * ui_scale) + (i as f32) * btn_w;
            let by = pos.y + 192.0 * ui_scale;
            let bh = 24.0 * ui_scale;
            let is_active = params.stroke_style == *val;
            render_segment_button(
                pixmap,
                bx,
                by,
                btn_w - 4.0 * ui_scale,
                bh,
                label,
                is_active,
                ui_scale,
            );
        }

        // Section 5: LAYERS (Front, Forward, Backward, Back)
        BitmapFont::draw_text(
            pixmap,
            pos.x + 14.0 * ui_scale,
            pos.y + 226.0 * ui_scale,
            &props.layers,
            Color::from_rgba8(161, 161, 170, 255),
            ui_scale * 0.75,
        );
        let layers = [
            &props.layer_front,
            &props.layer_forward,
            &props.layer_backward,
            &props.layer_back,
        ];
        let op_btn_w = (card_w - 28.0 * ui_scale) / 4.0;
        for (i, label) in layers.iter().enumerate() {
            let bx = pos.x + (14.0 * ui_scale) + (i as f32) * op_btn_w;
            let by = pos.y + 240.0 * ui_scale;
            let bh = 24.0 * ui_scale;
            render_segment_button(
                pixmap,
                bx,
                by,
                op_btn_w - 4.0 * ui_scale,
                bh,
                label,
                false,
                ui_scale,
            );
        }

        // Section 6: ACTIONS (Duplicate, Delete)
        BitmapFont::draw_text(
            pixmap,
            pos.x + 14.0 * ui_scale,
            pos.y + 274.0 * ui_scale,
            &props.actions,
            Color::from_rgba8(161, 161, 170, 255),
            ui_scale * 0.75,
        );
        let act_btn_w = (card_w - 28.0 * ui_scale) / 2.0;
        render_segment_button(
            pixmap,
            pos.x + 14.0 * ui_scale,
            pos.y + 288.0 * ui_scale,
            act_btn_w - 4.0 * ui_scale,
            26.0 * ui_scale,
            &props.action_duplicate,
            false,
            ui_scale,
        );
        render_segment_button(
            pixmap,
            pos.x + 14.0 * ui_scale + act_btn_w,
            pos.y + 288.0 * ui_scale,
            act_btn_w - 4.0 * ui_scale,
            26.0 * ui_scale,
            &props.action_delete,
            false,
            ui_scale,
        );
    }
}

/// Helper function to draw a sleek, rounded segmented button.
#[allow(clippy::too_many_arguments)]
fn render_segment_button(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    label: &str,
    is_active: bool,
    scale: f32,
) {
    let mut pb = PathBuilder::new();
    if let Some(rect) = tiny_skia::Rect::from_xywh(x, y, width, height) {
        pb.push_rect(rect);
        if let Some(path) = pb.finish() {
            let mut bg_paint = Paint::default();
            if is_active {
                // Vibrant active blue
                bg_paint.set_color(Color::from_rgba8(59, 130, 246, 230));
            } else {
                // Translucent slate
                bg_paint.set_color(Color::from_rgba8(39, 39, 42, 210));
            }
            pixmap.fill_path(
                &path,
                &bg_paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );

            // Subtle border
            let mut border_paint = Paint::default();
            border_paint.set_color(if is_active {
                Color::from_rgba8(147, 197, 253, 240)
            } else {
                Color::from_rgba8(63, 63, 70, 160)
            });
            let stroke = Stroke {
                width: 1.0 * scale,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);

            // Centered Label
            let text_color = if is_active {
                Color::from_rgba8(255, 255, 255, 255)
            } else {
                Color::from_rgba8(212, 212, 216, 240)
            };
            let text_scale = scale * 0.72;
            let (tw, th) = BitmapFont::measure_text(label, text_scale);
            let tx = x + (width - tw).max(0.0) / 2.0;
            let ty = y + (height - th).max(0.0) / 2.0;
            BitmapFont::draw_text(pixmap, tx, ty, label, text_color, text_scale);
        }
    }
}
