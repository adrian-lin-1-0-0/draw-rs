use tiny_skia::{Color, Pixmap};

use super::{
    CornerStyle, Drawable, Point2D, Rect2D, RoughGenerator, Sloppiness, StrokeStyle,
    generate_shape_seed,
};

/// A circle or ellipse node, supporting hand-drawn rough sketching,
/// perfect for Trees, Graphs, and flowchart nodes.
#[derive(Debug, Clone)]
pub struct CircleShape {
    pub center: Point2D,
    pub radius: f32,
    pub radius_y: Option<f32>,
    pub stroke_color: Color,
    pub fill_color: Option<Color>,
    pub stroke_width: f32,
    pub stroke_style: StrokeStyle,
    pub sloppiness: Sloppiness,
    pub opacity: f32,
    pub seed: u64,
}

impl CircleShape {
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
            radius_y: None,
            stroke_color,
            fill_color,
            stroke_width,
            stroke_style: StrokeStyle::Solid,
            sloppiness: Sloppiness::Artist,
            opacity: 1.0,
            seed: generate_shape_seed(),
        }
    }

    /// Convenience constructor with a semi-transparent fill based on stroke color.
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
            radius_y: None,
            stroke_color,
            fill_color,
            stroke_width,
            stroke_style: StrokeStyle::Solid,
            sloppiness: Sloppiness::Artist,
            opacity: 1.0,
            seed: generate_shape_seed(),
        }
    }

    /// Ellipse constructor with independent horizontal and vertical radii.
    pub fn new_ellipse(
        center: Point2D,
        rx: f32,
        ry: f32,
        stroke_color: Color,
        fill_color: Option<Color>,
        stroke_width: f32,
    ) -> Self {
        Self {
            center,
            radius: rx,
            radius_y: Some(ry),
            stroke_color,
            fill_color,
            stroke_width,
            stroke_style: StrokeStyle::Solid,
            sloppiness: Sloppiness::Artist,
            opacity: 1.0,
            seed: generate_shape_seed(),
        }
    }

    #[inline]
    pub fn rx(&self) -> f32 {
        self.radius
    }

    #[inline]
    pub fn ry(&self) -> f32 {
        self.radius_y.unwrap_or(self.radius)
    }

    #[inline]
    pub fn is_inside(&self, point: Point2D) -> bool {
        let rx = self.rx().abs();
        let ry = self.ry().abs();
        if rx <= 0.0 || ry <= 0.0 {
            return false;
        }
        let dx = point.x - self.center.x;
        let dy = point.y - self.center.y;
        (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry) <= 1.0
    }
}

impl Drawable for CircleShape {
    fn draw(&self, pixmap: &mut Pixmap) {
        let rx = self.rx().abs();
        let ry = self.ry().abs();
        if rx <= 0.0 || ry <= 0.0 {
            return;
        }

        RoughGenerator::draw_rough_ellipse(
            pixmap,
            self.center,
            rx,
            ry,
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
        let rx = self.rx().abs();
        let ry = self.ry().abs();
        if rx <= 0.0 || ry <= 0.0 {
            return false;
        }

        // Check if inside or near border
        let dx = point.x - self.center.x;
        let dy = point.y - self.center.y;

        // Expanded ellipse test for intersection with circular radius
        let eff_rx = rx + radius + self.stroke_width * 0.5;
        let eff_ry = ry + radius + self.stroke_width * 0.5;
        (dx * dx) / (eff_rx * eff_rx) + (dy * dy) / (eff_ry * eff_ry) <= 1.0
    }

    fn hit_test(&self, point: Point2D) -> bool {
        let rx = self.rx().abs();
        let ry = self.ry().abs();
        let tolerance = (self.stroke_width * 0.5).max(6.0);

        if self.fill_color.is_some() {
            // Fill is active: entire interior hits
            let eff_rx = rx + tolerance;
            let eff_ry = ry + tolerance;
            let dx = point.x - self.center.x;
            let dy = point.y - self.center.y;
            (dx * dx) / (eff_rx * eff_rx) + (dy * dy) / (eff_ry * eff_ry) <= 1.0
        } else {
            // Outline only: check distance to elliptical rim
            let dx = point.x - self.center.x;
            let dy = point.y - self.center.y;
            let norm_dist = ((dx * dx) / (rx * rx) + (dy * dy) / (ry * ry)).sqrt();
            let avg_r = (rx + ry) * 0.5;
            let dist_to_rim = (norm_dist - 1.0).abs() * avg_r;
            dist_to_rim <= tolerance
        }
    }

    fn bounding_box(&self) -> Option<Rect2D> {
        let rx = self.rx().abs();
        let ry = self.ry().abs();
        Some(Rect2D::new(
            self.center.x - rx,
            self.center.y - ry,
            self.center.x + rx,
            self.center.y + ry,
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
        let scale_x = new_bounds.width() / old_bounds.width();
        let scale_y = new_bounds.height() / old_bounds.height();

        self.center.x = new_bounds.min_x + tc_x * new_bounds.width();
        self.center.y = new_bounds.min_y + tc_y * new_bounds.height();
        self.radius = (self.radius * scale_x).abs();
        if let Some(ry) = &mut self.radius_y {
            *ry = (*ry * scale_y).abs();
        }
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
        CornerStyle::Round
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
