use tiny_skia::{
    Color, FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, StrokeDash, Transform,
};

use crate::models::{Point2D, Rect2D, TransformHandle};

/// Visual renderer for selection bounding boxes, resize handles, and box-select marquee.
pub struct SelectionRenderer;

impl SelectionRenderer {
    pub const HANDLE_SIZE: f32 = 8.0;

    /// Renders selection bounding box, 8 resize handles, and rotation handle.
    pub fn render_selection_box(pixmap: &mut Pixmap, bounds: Rect2D, scale: f32) {
        let pad = 4.0 * scale;
        let expanded = bounds.expand(pad);
        let min_x = expanded.min_x;
        let min_y = expanded.min_y;
        let max_x = expanded.max_x;
        let max_y = expanded.max_y;
        let w = max_x - min_x;
        let h = max_y - min_y;

        let primary_blue = Color::from_rgba8(59, 130, 246, 240); // #3B82F6
        let handle_fill = Color::from_rgba8(255, 255, 255, 255);

        // 1. Bounding box outline
        let mut pb = PathBuilder::new();
        if let Some(r) = tiny_skia::Rect::from_xywh(min_x, min_y, w, h) {
            pb.push_rect(r);
            if let Some(path) = pb.finish() {
                let mut stroke_paint = Paint::default();
                stroke_paint.set_color(primary_blue);
                stroke_paint.anti_alias = true;

                let stroke = Stroke {
                    width: 1.5 * scale,
                    line_cap: LineCap::Round,
                    line_join: LineJoin::Round,
                    ..Default::default()
                };
                pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }

        // 2. Rotation connector stem (vertical line above top-center)
        let mid_x = (min_x + max_x) * 0.5;
        let rot_y = min_y - 20.0 * scale;
        let mut stem_pb = PathBuilder::new();
        stem_pb.move_to(mid_x, min_y);
        stem_pb.line_to(mid_x, rot_y);
        if let Some(stem_path) = stem_pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(primary_blue);
            let stroke = Stroke {
                width: 1.5 * scale,
                ..Default::default()
            };
            pixmap.stroke_path(&stem_path, &paint, &stroke, Transform::identity(), None);
        }

        // 3. Render 8 resize handles + 1 rotation knob
        let handle_radius = (Self::HANDLE_SIZE * 0.5 * scale).max(3.5);
        let handles = expanded.handles();

        for (handle_type, pos) in handles {
            let mut handle_pb = PathBuilder::new();
            if handle_type == TransformHandle::Rotate {
                // Circular rotation knob
                handle_pb.push_circle(pos.x, pos.y, handle_radius * 1.1);
            } else {
                // Square handle
                if let Some(rect) = tiny_skia::Rect::from_xywh(
                    pos.x - handle_radius,
                    pos.y - handle_radius,
                    handle_radius * 2.0,
                    handle_radius * 2.0,
                ) {
                    handle_pb.push_rect(rect);
                }
            }

            if let Some(path) = handle_pb.finish() {
                // White fill
                let mut fill_paint = Paint::default();
                fill_paint.set_color(handle_fill);
                pixmap.fill_path(
                    &path,
                    &fill_paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );

                // Blue border
                let mut border_paint = Paint::default();
                border_paint.set_color(primary_blue);
                let border_stroke = Stroke {
                    width: 1.5 * scale,
                    ..Default::default()
                };
                pixmap.stroke_path(
                    &path,
                    &border_paint,
                    &border_stroke,
                    Transform::identity(),
                    None,
                );
            }
        }
    }

    /// Renders live rectangular marquee during box selection drag.
    pub fn render_marquee(pixmap: &mut Pixmap, start: Point2D, current: Point2D, scale: f32) {
        let rect = Rect2D::from_points(start, current);
        if rect.width() < 1.0 || rect.height() < 1.0 {
            return;
        }

        let mut pb = PathBuilder::new();
        if let Some(r) =
            tiny_skia::Rect::from_xywh(rect.min_x, rect.min_y, rect.width(), rect.height())
        {
            pb.push_rect(r);
            if let Some(path) = pb.finish() {
                // Subtle blue translucent fill
                let mut fill_paint = Paint::default();
                fill_paint.set_color(Color::from_rgba8(59, 130, 246, 35));
                pixmap.fill_path(
                    &path,
                    &fill_paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );

                // Dashed blue outline
                let mut stroke_paint = Paint::default();
                stroke_paint.set_color(Color::from_rgba8(59, 130, 246, 210));
                let stroke = Stroke {
                    width: 1.5 * scale,
                    dash: StrokeDash::new(vec![5.0 * scale, 4.0 * scale], 0.0),
                    ..Default::default()
                };
                pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
    }
}
