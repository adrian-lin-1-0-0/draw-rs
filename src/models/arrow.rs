use tiny_skia::{Color, FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

use super::{point::Point2D, Drawable};

/// A directional arrow, perfect for Two Pointers, Linked List `next` pointers,
/// and Graph directed edges.
#[derive(Debug, Clone)]
pub struct ArrowShape {
    pub start: Point2D,
    pub end: Point2D,
    pub color: Color,
    pub stroke_width: f32,
    pub head_length: f32,
    pub head_angle: f32,
}

impl ArrowShape {
    pub fn new(start: Point2D, end: Point2D, color: Color, stroke_width: f32) -> Self {
        // Dynamic arrowhead sizing proportional to stroke width
        let head_length = (stroke_width * 4.5).clamp(14.0, 28.0);
        let head_angle = std::f32::consts::FRAC_PI_6; // 30 degrees
        Self {
            start,
            end,
            color,
            stroke_width,
            head_length,
            head_angle,
        }
    }
}

impl Drawable for ArrowShape {
    fn draw(&self, pixmap: &mut Pixmap) {
        let length = self.start.distance(&self.end);
        if length < 2.0 {
            return;
        }

        let angle = self.start.angle_to(&self.end);
        let head_len = self.head_length.min(length * 0.6);

        // Arrowhead vertices
        let tip = self.end;
        let left_wing = Point2D::new(
            tip.x - head_len * (angle - self.head_angle).cos(),
            tip.y - head_len * (angle - self.head_angle).sin(),
        );
        let right_wing = Point2D::new(
            tip.x - head_len * (angle + self.head_angle).cos(),
            tip.y - head_len * (angle + self.head_angle).sin(),
        );
        let base_center = Point2D::new(
            tip.x - (head_len * 0.7) * angle.cos(),
            tip.y - (head_len * 0.7) * angle.sin(),
        );

        let mut paint = Paint::default();
        paint.set_color(self.color);
        paint.anti_alias = true;

        // 1. Draw arrow shaft
        let mut shaft_builder = PathBuilder::new();
        shaft_builder.move_to(self.start.x, self.start.y);
        shaft_builder.line_to(base_center.x, base_center.y);

        if let Some(shaft_path) = shaft_builder.finish() {
            let stroke = Stroke {
                width: self.stroke_width,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&shaft_path, &paint, &stroke, Transform::identity(), None);
        }

        // 2. Draw solid arrowhead (triangle)
        let mut head_builder = PathBuilder::new();
        head_builder.move_to(tip.x, tip.y);
        head_builder.line_to(left_wing.x, left_wing.y);
        head_builder.line_to(base_center.x, base_center.y);
        head_builder.line_to(right_wing.x, right_wing.y);
        head_builder.close();

        if let Some(head_path) = head_builder.finish() {
            pixmap.fill_path(&head_path, &paint, FillRule::Winding, Transform::identity(), None);
        }
    }
}
