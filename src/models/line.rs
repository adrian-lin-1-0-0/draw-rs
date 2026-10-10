use tiny_skia::{Color, Pixmap};

use super::{
    CornerStyle, Drawable, Point2D, Rect2D, RoughGenerator, Sloppiness, StrokeStyle,
    generate_shape_seed,
};

/// A straight line shape with hand-drawn rough sketching.
#[derive(Debug, Clone)]
pub struct LineShape {
    pub start: Point2D,
    pub end: Point2D,
    pub stroke_color: Color,
    pub stroke_width: f32,
    pub stroke_style: StrokeStyle,
    pub sloppiness: Sloppiness,
    pub opacity: f32,
    pub seed: u64,
}

impl LineShape {
    pub fn new(start: Point2D, end: Point2D, stroke_color: Color, stroke_width: f32) -> Self {
        Self {
            start,
            end,
            stroke_color,
            stroke_width,
            stroke_style: StrokeStyle::Solid,
            sloppiness: Sloppiness::Artist,
            opacity: 1.0,
            seed: generate_shape_seed(),
        }
    }
}

impl Drawable for LineShape {
    fn draw(&self, pixmap: &mut Pixmap) {
        RoughGenerator::draw_rough_line(
            pixmap,
            self.start,
            self.end,
            self.stroke_color,
            self.stroke_width,
            self.stroke_style,
            self.sloppiness,
            self.seed,
            self.opacity,
        );
    }

    fn intersects(&self, point: Point2D, radius: f32) -> bool {
        let hit_radius = radius + self.stroke_width * 0.5;
        point.distance_to_segment(&self.start, &self.end) <= hit_radius
    }

    fn hit_test(&self, point: Point2D) -> bool {
        let tolerance = (self.stroke_width * 0.5).max(6.0);
        point.distance_to_segment(&self.start, &self.end) <= tolerance
    }

    fn bounding_box(&self) -> Option<Rect2D> {
        let margin = (self.stroke_width * 0.5).max(4.0);
        Some(Rect2D::from_points(self.start, self.end).expand(margin))
    }

    fn translate(&mut self, offset: Point2D) {
        self.start = self.start + offset;
        self.end = self.end + offset;
    }

    fn resize(&mut self, old_bounds: Rect2D, new_bounds: Rect2D) {
        if old_bounds.width() <= 0.0 || old_bounds.height() <= 0.0 {
            return;
        }
        let ts_x = (self.start.x - old_bounds.min_x) / old_bounds.width();
        let ts_y = (self.start.y - old_bounds.min_y) / old_bounds.height();
        let te_x = (self.end.x - old_bounds.min_x) / old_bounds.width();
        let te_y = (self.end.y - old_bounds.min_y) / old_bounds.height();

        self.start.x = new_bounds.min_x + ts_x * new_bounds.width();
        self.start.y = new_bounds.min_y + ts_y * new_bounds.height();
        self.end.x = new_bounds.min_x + te_x * new_bounds.width();
        self.end.y = new_bounds.min_y + te_y * new_bounds.height();
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
