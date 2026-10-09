use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

use super::{Drawable, point::Point2D};

/// A circle node, perfectly tailored for Tree and Graph nodes in algorithm problems.
#[derive(Debug, Clone)]
pub struct CircleShape {
    pub center: Point2D,
    pub radius: f32,
    pub stroke_color: Color,
    pub fill_color: Option<Color>,
    pub stroke_width: f32,
}

impl CircleShape {
    #[allow(dead_code)]
    pub fn new(
        center: Point2D,
        radius: f32,
        stroke_color: Color,
        fill_color: Option<Color>,
        stroke_width: f32,
    ) -> Self {
        Self {
            center,
            radius,
            stroke_color,
            fill_color,
            stroke_width,
        }
    }

    /// Convenience constructor with a semi-transparent fill based on the stroke color.
    pub fn with_translucent_fill(
        center: Point2D,
        radius: f32,
        stroke_color: Color,
        stroke_width: f32,
        fill_alpha: f32,
    ) -> Self {
        let fill_color = Color::from_rgba(
            stroke_color.red(),
            stroke_color.green(),
            stroke_color.blue(),
            (stroke_color.alpha() * fill_alpha).clamp(0.0, 1.0),
        );
        Self {
            center,
            radius,
            stroke_color,
            fill_color,
            stroke_width,
        }
    }
}

impl Drawable for CircleShape {
    fn draw(&self, pixmap: &mut Pixmap) {
        if self.radius <= 0.0 {
            return;
        }

        let mut pb = PathBuilder::new();
        pb.push_circle(self.center.x, self.center.y, self.radius);
        let Some(path) = pb.finish() else {
            return;
        };

        // 1. Fill if configured
        if let Some(fill_color) = self.fill_color {
            let mut fill_paint = Paint::default();
            fill_paint.set_color(fill_color);
            fill_paint.anti_alias = true;
            pixmap.fill_path(
                &path,
                &fill_paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }

        // 2. Stroke outline
        let mut stroke_paint = Paint::default();
        stroke_paint.set_color(self.stroke_color);
        stroke_paint.anti_alias = true;

        let stroke = Stroke {
            width: self.stroke_width,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
    }

    fn intersects(&self, point: Point2D, radius: f32) -> bool {
        let dist = point.distance(&self.center);
        dist <= self.radius + radius
    }
}
