use tiny_skia::{Color, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

use super::{Drawable, point::Point2D};

/// A freehand pen stroke made of sampled 2D points.
/// Implements quadratic Bézier curve smoothing for natural and responsive ink strokes.
#[derive(Debug, Clone)]
pub struct StrokeShape {
    pub points: Vec<Point2D>,
    pub color: Color,
    pub width: f32,
}

impl StrokeShape {
    pub fn new(points: Vec<Point2D>, color: Color, width: f32) -> Self {
        Self {
            points,
            color,
            width,
        }
    }
}

impl Drawable for StrokeShape {
    fn draw(&self, pixmap: &mut Pixmap) {
        if self.points.is_empty() {
            return;
        }

        let mut paint = Paint::default();
        paint.set_color(self.color);
        paint.anti_alias = true;

        let stroke = Stroke {
            width: self.width,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Default::default()
        };

        // If there is only one point, draw a small circle (dot)
        if self.points.len() == 1 {
            let p = self.points[0];
            let radius = (self.width / 2.0).max(1.0);
            let mut pb = PathBuilder::new();
            pb.push_circle(p.x, p.y, radius);
            if let Some(path) = pb.finish() {
                pixmap.fill_path(
                    &path,
                    &paint,
                    tiny_skia::FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
            return;
        }

        let mut pb = PathBuilder::new();
        pb.move_to(self.points[0].x, self.points[0].y);

        if self.points.len() == 2 {
            pb.line_to(self.points[1].x, self.points[1].y);
        } else {
            // Quadratic Bézier curve smoothing through midpoints
            for i in 1..self.points.len() - 1 {
                let p_curr = self.points[i];
                let p_next = self.points[i + 1];
                let mid = p_curr.midpoint(&p_next);
                pb.quad_to(p_curr.x, p_curr.y, mid.x, mid.y);
            }
            // Line to the final point
            let last = self.points[self.points.len() - 1];
            pb.line_to(last.x, last.y);
        }

        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    fn intersects(&self, point: Point2D, radius: f32) -> bool {
        let hit_radius = radius + (self.width * 0.5);
        if self.points.is_empty() {
            return false;
        }
        if self.points.len() == 1 {
            return point.distance(&self.points[0]) <= hit_radius;
        }
        for i in 0..self.points.len() - 1 {
            if point.distance_to_segment(&self.points[i], &self.points[i + 1]) <= hit_radius {
                return true;
            }
        }
        false
    }
}
