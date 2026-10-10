use tiny_skia::{Color, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

use super::{CornerStyle, Drawable, Point2D, Rect2D, Sloppiness, StrokeStyle, generate_shape_seed};

/// A freehand pen stroke made of sampled 2D points.
/// Implements quadratic Bézier curve smoothing for natural and responsive ink strokes.
#[derive(Debug, Clone)]
pub struct StrokeShape {
    pub points: Vec<Point2D>,
    pub color: Color,
    pub width: f32,
    pub stroke_style: StrokeStyle,
    pub sloppiness: Sloppiness,
    pub opacity: f32,
    pub seed: u64,
}

impl StrokeShape {
    pub fn new(points: Vec<Point2D>, color: Color, width: f32) -> Self {
        Self {
            points,
            color,
            width,
            stroke_style: StrokeStyle::Solid,
            sloppiness: Sloppiness::Artist,
            opacity: 1.0,
            seed: generate_shape_seed(),
        }
    }
}

impl Drawable for StrokeShape {
    fn draw(&self, pixmap: &mut Pixmap) {
        if self.points.is_empty() {
            return;
        }

        let mut paint = Paint::default();
        paint.set_color(super::rough::safe_color_with_alpha(
            self.color,
            self.opacity,
        ));
        paint.anti_alias = true;

        let stroke = Stroke {
            width: self.width,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            dash: self.stroke_style.to_stroke_dash(self.width),
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

    fn hit_test(&self, point: Point2D) -> bool {
        let tolerance = (self.width * 0.5).max(6.0);
        self.intersects(point, tolerance)
    }

    fn bounding_box(&self) -> Option<Rect2D> {
        if self.points.is_empty() {
            return None;
        }
        let mut min_x = self.points[0].x;
        let mut min_y = self.points[0].y;
        let mut max_x = self.points[0].x;
        let mut max_y = self.points[0].y;

        for p in &self.points {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        let margin = (self.width * 0.5).max(4.0);
        Some(Rect2D::new(min_x, min_y, max_x, max_y).expand(margin))
    }

    fn translate(&mut self, offset: Point2D) {
        for p in &mut self.points {
            *p = *p + offset;
        }
    }

    fn resize(&mut self, old_bounds: Rect2D, new_bounds: Rect2D) {
        if old_bounds.width() <= 0.0 || old_bounds.height() <= 0.0 {
            return;
        }
        for p in &mut self.points {
            let tx = (p.x - old_bounds.min_x) / old_bounds.width();
            let ty = (p.y - old_bounds.min_y) / old_bounds.height();
            p.x = new_bounds.min_x + tx * new_bounds.width();
            p.y = new_bounds.min_y + ty * new_bounds.height();
        }
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
        self.width
    }

    fn set_stroke_width(&mut self, width: f32) {
        self.width = width;
    }

    fn stroke_style(&self) -> StrokeStyle {
        self.stroke_style
    }

    fn set_stroke_style(&mut self, style: StrokeStyle) {
        self.stroke_style = style;
    }

    fn sloppiness(&self) -> Sloppiness {
        self.sloppiness
    }

    fn set_sloppiness(&mut self, sloppiness: Sloppiness) {
        self.sloppiness = sloppiness;
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
}
