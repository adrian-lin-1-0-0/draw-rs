use tiny_skia::{Color, Pixmap};

use super::{
    CornerStyle, Drawable, Point2D, Rect2D, RoughGenerator, Sloppiness, StrokeStyle,
    generate_shape_seed,
};

/// A diamond flowchart node with hand-drawn rough styling.
#[derive(Debug, Clone)]
pub struct DiamondShape {
    pub center: Point2D,
    pub width: f32,
    pub height: f32,
    pub stroke_color: Color,
    pub fill_color: Option<Color>,
    pub stroke_width: f32,
    pub stroke_style: StrokeStyle,
    pub sloppiness: Sloppiness,
    pub opacity: f32,
    pub seed: u64,
}

impl DiamondShape {
    pub fn new(
        center: Point2D,
        width: f32,
        height: f32,
        stroke_color: Color,
        fill_color: Option<Color>,
        stroke_width: f32,
    ) -> Self {
        Self {
            center,
            width,
            height,
            stroke_color,
            fill_color,
            stroke_width,
            stroke_style: StrokeStyle::Solid,
            sloppiness: Sloppiness::Artist,
            opacity: 1.0,
            seed: generate_shape_seed(),
        }
    }

    /// 4 diamond vertices: top, right, bottom, left.
    pub fn vertices(&self) -> [Point2D; 4] {
        let hw = (self.width * 0.5).abs();
        let hh = (self.height * 0.5).abs();
        [
            Point2D::new(self.center.x, self.center.y - hh),
            Point2D::new(self.center.x + hw, self.center.y),
            Point2D::new(self.center.x, self.center.y + hh),
            Point2D::new(self.center.x - hw, self.center.y),
        ]
    }

    /// Checks if a point is inside the diamond boundary via Manhattan normalization.
    fn contains_point(&self, point: Point2D) -> bool {
        let hw = (self.width * 0.5).abs();
        let hh = (self.height * 0.5).abs();
        if hw <= 0.0 || hh <= 0.0 {
            return false;
        }
        let dx = (point.x - self.center.x).abs() / hw;
        let dy = (point.y - self.center.y).abs() / hh;
        dx + dy <= 1.0
    }
}

impl Drawable for DiamondShape {
    fn draw(&self, pixmap: &mut Pixmap) {
        if self.width.abs() < 1.0 || self.height.abs() < 1.0 {
            return;
        }

        let verts = self.vertices();
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

    fn intersects(&self, point: Point2D, radius: f32) -> bool {
        let verts = self.vertices();
        let hit_radius = radius + self.stroke_width * 0.5;

        for i in 0..4 {
            let next_i = (i + 1) % 4;
            if point.distance_to_segment(&verts[i], &verts[next_i]) <= hit_radius {
                return true;
            }
        }

        if self.fill_color.is_some() && self.contains_point(point) {
            return true;
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

        if self.fill_color.is_some() && self.contains_point(point) {
            return true;
        }

        false
    }

    fn bounding_box(&self) -> Option<Rect2D> {
        let hw = (self.width * 0.5).abs();
        let hh = (self.height * 0.5).abs();
        Some(Rect2D::new(
            self.center.x - hw,
            self.center.y - hh,
            self.center.x + hw,
            self.center.y + hh,
        ))
    }

    fn translate(&mut self, offset: Point2D) {
        self.center = self.center + offset;
    }

    fn resize(&mut self, old_bounds: Rect2D, new_bounds: Rect2D) {
        if old_bounds.width() <= 0.0 || old_bounds.height() <= 0.0 {
            return;
        }
        let tc_x = (self.center.x - old_bounds.min_x) / old_bounds.width();
        let tc_y = (self.center.y - old_bounds.min_y) / old_bounds.height();
        let tw = self.width / old_bounds.width();
        let th = self.height / old_bounds.height();

        self.center.x = new_bounds.min_x + tc_x * new_bounds.width();
        self.center.y = new_bounds.min_y + tc_y * new_bounds.height();
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
