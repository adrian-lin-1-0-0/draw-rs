use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Transform};

use super::{
    CornerStyle, Drawable, Point2D, Rect2D, RoughGenerator, Sloppiness, StrokeStyle,
    generate_shape_seed,
};

/// A directional arrow with support for straight or polyline/elbow shafts,
/// rough sketching, and dynamic arrowhead calculation.
#[derive(Debug, Clone)]
pub struct ArrowShape {
    pub start: Point2D,
    pub end: Point2D,
    pub points: Vec<Point2D>,
    pub color: Color,
    pub stroke_width: f32,
    pub head_length: f32,
    pub head_angle: f32,
    pub stroke_style: StrokeStyle,
    pub sloppiness: Sloppiness,
    pub opacity: f32,
    pub seed: u64,
}

impl ArrowShape {
    pub fn new(start: Point2D, end: Point2D, color: Color, stroke_width: f32) -> Self {
        let head_length = (stroke_width * 4.5).clamp(14.0, 28.0);
        let head_angle = std::f32::consts::FRAC_PI_6; // 30 degrees
        Self {
            start,
            end,
            points: vec![start, end],
            color,
            stroke_width,
            head_length,
            head_angle,
            stroke_style: StrokeStyle::Solid,
            sloppiness: Sloppiness::Artist,
            opacity: 1.0,
            seed: generate_shape_seed(),
        }
    }

    /// Arrow with polyline waypoints (e.g., elbow connectors).
    pub fn with_waypoints(points: Vec<Point2D>, color: Color, stroke_width: f32) -> Self {
        let start = points.first().copied().unwrap_or(Point2D::ZERO);
        let end = points.last().copied().unwrap_or(Point2D::ZERO);
        let head_length = (stroke_width * 4.5).clamp(14.0, 28.0);
        let head_angle = std::f32::consts::FRAC_PI_6;
        Self {
            start,
            end,
            points,
            color,
            stroke_width,
            head_length,
            head_angle,
            stroke_style: StrokeStyle::Solid,
            sloppiness: Sloppiness::Artist,
            opacity: 1.0,
            seed: generate_shape_seed(),
        }
    }

    /// Effective list of points forming the shaft.
    pub fn effective_points(&self) -> Vec<Point2D> {
        if self.points.len() >= 2 {
            self.points.clone()
        } else {
            vec![self.start, self.end]
        }
    }
}

impl Drawable for ArrowShape {
    fn draw(&self, pixmap: &mut Pixmap) {
        let pts = self.effective_points();
        if pts.len() < 2 {
            return;
        }

        let start = pts[0];
        let end = pts[pts.len() - 1];
        let total_dist = start.distance(&end);
        if total_dist < 2.0 && pts.len() == 2 {
            return;
        }

        let prev_pt = pts[pts.len() - 2];
        let angle = prev_pt.angle_to(&end);
        let head_len = self.head_length.min(prev_pt.distance(&end) * 0.7);

        // Arrowhead vertices
        let tip = end;
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

        // 1. Draw shaft
        for i in 0..pts.len() - 1 {
            let p1 = pts[i];
            let p2 = if i == pts.len() - 2 {
                base_center
            } else {
                pts[i + 1]
            };
            let seg_seed = self.seed.wrapping_add((i as u64) * 43);

            RoughGenerator::draw_rough_line(
                pixmap,
                p1,
                p2,
                self.color,
                self.stroke_width,
                self.stroke_style,
                self.sloppiness,
                seg_seed,
                self.opacity,
            );
        }

        // 2. Draw solid arrowhead
        let mut head_builder = PathBuilder::new();
        head_builder.move_to(tip.x, tip.y);
        head_builder.line_to(left_wing.x, left_wing.y);
        head_builder.line_to(base_center.x, base_center.y);
        head_builder.line_to(right_wing.x, right_wing.y);
        head_builder.close();

        if let Some(head_path) = head_builder.finish() {
            let mut paint = Paint::default();
            paint.set_color(super::rough::safe_color_with_alpha(
                self.color,
                self.opacity,
            ));
            paint.anti_alias = true;
            pixmap.fill_path(
                &head_path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    fn intersects(&self, point: Point2D, radius: f32) -> bool {
        let pts = self.effective_points();
        let hit_radius = radius + (self.stroke_width * 0.5);

        for i in 0..pts.len() - 1 {
            if point.distance_to_segment(&pts[i], &pts[i + 1]) <= hit_radius {
                return true;
            }
        }

        if point.distance(&self.end) <= radius + self.head_length {
            return true;
        }

        false
    }

    fn hit_test(&self, point: Point2D) -> bool {
        let tolerance = (self.stroke_width * 0.5).max(6.0);
        self.intersects(point, tolerance)
    }

    fn bounding_box(&self) -> Option<Rect2D> {
        let pts = self.effective_points();
        if pts.is_empty() {
            return None;
        }

        let mut min_x = pts[0].x;
        let mut min_y = pts[0].y;
        let mut max_x = pts[0].x;
        let mut max_y = pts[0].y;

        for p in &pts {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        let margin = (self.stroke_width * 0.5 + self.head_length).max(8.0);
        Some(Rect2D::new(min_x, min_y, max_x, max_y).expand(margin))
    }

    fn translate(&mut self, offset: Point2D) {
        self.start = self.start + offset;
        self.end = self.end + offset;
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

        if let Some(first) = self.points.first() {
            self.start = *first;
        }
        if let Some(last) = self.points.last() {
            self.end = *last;
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
        self.stroke_width
    }

    fn set_stroke_width(&mut self, width: f32) {
        self.stroke_width = width;
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
