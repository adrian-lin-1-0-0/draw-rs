use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Transform};

use super::{
    CornerStyle, Drawable, Point2D, Rect2D, RoughGenerator, Sloppiness, StrokeStyle,
    generate_shape_seed,
};

/// A rectangle shape with support for sharp/rounded corners, hand-drawn rough styling,
/// and solid/translucent fills.
#[derive(Debug, Clone)]
pub struct RectangleShape {
    pub top_left: Point2D,
    pub width: f32,
    pub height: f32,
    pub corner_radius: f32,
    pub stroke_color: Color,
    pub fill_color: Option<Color>,
    pub stroke_width: f32,
    pub stroke_style: StrokeStyle,
    pub sloppiness: Sloppiness,
    pub corner_style: CornerStyle,
    pub opacity: f32,
    pub seed: u64,
}

impl RectangleShape {
    pub fn new(
        top_left: Point2D,
        width: f32,
        height: f32,
        stroke_color: Color,
        fill_color: Option<Color>,
        stroke_width: f32,
    ) -> Self {
        Self {
            top_left,
            width,
            height,
            corner_radius: 12.0,
            stroke_color,
            fill_color,
            stroke_width,
            stroke_style: StrokeStyle::Solid,
            sloppiness: Sloppiness::Artist,
            corner_style: CornerStyle::Sharp,
            opacity: 1.0,
            seed: generate_shape_seed(),
        }
    }

    /// Normalized rectangle vertices (top-left, top-right, bottom-right, bottom-left).
    pub fn vertices(&self) -> [Point2D; 4] {
        let x1 = self.top_left.x;
        let y1 = self.top_left.y;
        let x2 = x1 + self.width;
        let y2 = y1 + self.height;

        let min_x = x1.min(x2);
        let max_x = x1.max(x2);
        let min_y = y1.min(y2);
        let max_y = y1.max(y2);

        [
            Point2D::new(min_x, min_y),
            Point2D::new(max_x, min_y),
            Point2D::new(max_x, max_y),
            Point2D::new(min_x, max_y),
        ]
    }
}

impl Drawable for RectangleShape {
    fn draw(&self, pixmap: &mut Pixmap) {
        if self.width.abs() < 1.0 || self.height.abs() < 1.0 {
            return;
        }

        let verts = self.vertices();
        let min_x = verts[0].x;
        let min_y = verts[0].y;
        let max_x = verts[2].x;
        let max_y = verts[2].y;
        let w = max_x - min_x;
        let h = max_y - min_y;

        if self.corner_style == CornerStyle::Round {
            let r = self.corner_radius.min(w * 0.4).min(h * 0.4).max(2.0);

            // 1. Fill if configured
            if let Some(fill) = self.fill_color {
                let mut pb = PathBuilder::new();
                if let Some(rect) = tiny_skia::Rect::from_xywh(min_x, min_y, w, h) {
                    pb.push_rect(rect);
                    if let Some(path) = pb.finish() {
                        let mut fill_paint = Paint::default();
                        fill_paint
                            .set_color(super::rough::safe_color_with_alpha(fill, self.opacity));
                        fill_paint.anti_alias = true;
                        pixmap.fill_path(
                            &path,
                            &fill_paint,
                            FillRule::Winding,
                            Transform::identity(),
                            None,
                        );
                    }
                }
            }

            // 2. Stroke 4 straight edges
            let top_left = Point2D::new(min_x + r, min_y);
            let top_right = Point2D::new(max_x - r, min_y);
            let right_top = Point2D::new(max_x, min_y + r);
            let right_bottom = Point2D::new(max_x, max_y - r);
            let bottom_right = Point2D::new(max_x - r, max_y);
            let bottom_left = Point2D::new(min_x + r, max_y);
            let left_bottom = Point2D::new(min_x, max_y - r);
            let left_top = Point2D::new(min_x, min_y + r);

            RoughGenerator::draw_rough_line(
                pixmap,
                top_left,
                top_right,
                self.stroke_color,
                self.stroke_width,
                self.stroke_style,
                self.sloppiness,
                self.seed.wrapping_add(11),
                self.opacity,
            );
            RoughGenerator::draw_rough_line(
                pixmap,
                right_top,
                right_bottom,
                self.stroke_color,
                self.stroke_width,
                self.stroke_style,
                self.sloppiness,
                self.seed.wrapping_add(23),
                self.opacity,
            );
            RoughGenerator::draw_rough_line(
                pixmap,
                bottom_right,
                bottom_left,
                self.stroke_color,
                self.stroke_width,
                self.stroke_style,
                self.sloppiness,
                self.seed.wrapping_add(37),
                self.opacity,
            );
            RoughGenerator::draw_rough_line(
                pixmap,
                left_bottom,
                left_top,
                self.stroke_color,
                self.stroke_width,
                self.stroke_style,
                self.sloppiness,
                self.seed.wrapping_add(53),
                self.opacity,
            );

            // Connect corner arcs with small rough curves
            RoughGenerator::draw_rough_line(
                pixmap,
                top_right,
                right_top,
                self.stroke_color,
                self.stroke_width,
                self.stroke_style,
                self.sloppiness,
                self.seed.wrapping_add(71),
                self.opacity,
            );
            RoughGenerator::draw_rough_line(
                pixmap,
                right_bottom,
                bottom_right,
                self.stroke_color,
                self.stroke_width,
                self.stroke_style,
                self.sloppiness,
                self.seed.wrapping_add(89),
                self.opacity,
            );
            RoughGenerator::draw_rough_line(
                pixmap,
                bottom_left,
                left_bottom,
                self.stroke_color,
                self.stroke_width,
                self.stroke_style,
                self.sloppiness,
                self.seed.wrapping_add(101),
                self.opacity,
            );
            RoughGenerator::draw_rough_line(
                pixmap,
                left_top,
                top_left,
                self.stroke_color,
                self.stroke_width,
                self.stroke_style,
                self.sloppiness,
                self.seed.wrapping_add(113),
                self.opacity,
            );
        } else {
            RoughGenerator::draw_rough_polygon(
                pixmap,
                &verts,
                self.stroke_color,
                self.fill_color,
                self.stroke_width,
                self.stroke_style,
                self.sloppiness,
                self.seed,
                self.opacity,
            );
        }
    }

    fn intersects(&self, point: Point2D, radius: f32) -> bool {
        let verts = self.vertices();
        let hit_radius = radius + self.stroke_width * 0.5;

        // Check edges
        for i in 0..4 {
            let next_i = (i + 1) % 4;
            if point.distance_to_segment(&verts[i], &verts[next_i]) <= hit_radius {
                return true;
            }
        }

        // If filled, check inside
        if self.fill_color.is_some() {
            let min_x = verts[0].x;
            let min_y = verts[0].y;
            let max_x = verts[2].x;
            let max_y = verts[2].y;
            if point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y {
                return true;
            }
        }

        false
    }

    fn hit_test(&self, point: Point2D) -> bool {
        let verts = self.vertices();
        let tolerance = (self.stroke_width * 0.5).max(6.0);

        for i in 0..4 {
            let next_i = (i + 1) % 4;
            if point.distance_to_segment(&verts[i], &verts[next_i]) <= tolerance {
                return true;
            }
        }

        if self.fill_color.is_some() {
            let min_x = verts[0].x;
            let min_y = verts[0].y;
            let max_x = verts[2].x;
            let max_y = verts[2].y;
            if point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y {
                return true;
            }
        }

        false
    }

    fn bounding_box(&self) -> Option<Rect2D> {
        let verts = self.vertices();
        Some(Rect2D::new(verts[0].x, verts[0].y, verts[2].x, verts[2].y))
    }

    fn translate(&mut self, offset: Point2D) {
        self.top_left = self.top_left + offset;
    }

    fn resize(&mut self, old_bounds: Rect2D, new_bounds: Rect2D) {
        if old_bounds.width() <= 0.0 || old_bounds.height() <= 0.0 {
            return;
        }
        let tx = (self.top_left.x - old_bounds.min_x) / old_bounds.width();
        let ty = (self.top_left.y - old_bounds.min_y) / old_bounds.height();
        let tw = self.width / old_bounds.width();
        let th = self.height / old_bounds.height();

        self.top_left.x = new_bounds.min_x + tx * new_bounds.width();
        self.top_left.y = new_bounds.min_y + ty * new_bounds.height();
        self.width = tw * new_bounds.width();
        self.height = th * new_bounds.height();
    }

    fn clone_box(&self) -> Box<dyn Drawable> {
        Box::new(self.clone())
    }

    fn stroke_color(&self) -> Color {
        self.stroke_color
    }

    fn set_stroke_color(&mut self, color: Color) {
        self.stroke_color = color;
    }

    fn fill_color(&self) -> Option<Color> {
        self.fill_color
    }

    fn set_fill_color(&mut self, color: Option<Color>) {
        self.fill_color = color;
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
        self.corner_style
    }

    fn set_corner_style(&mut self, corner: CornerStyle) {
        self.corner_style = corner;
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
