use tiny_skia::{
    Color, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, StrokeDash, Transform,
};

use super::point::Point2D;

/// Roughness / sloppiness rendering style for hand-drawn whiteboard sketching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sloppiness {
    /// Low perturbation, clean architectural blueprint aesthetic.
    Architect,
    /// Default hand-drawn feel, double-stroked subtle jitter.
    #[default]
    Artist,
    /// High perturbation, exaggerated sketchy drafts with corner overshoots.
    Cartoonist,
}

impl Sloppiness {
    pub const ALL: [Self; 3] = [Self::Architect, Self::Artist, Self::Cartoonist];

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Architect => "Architect",
            Self::Artist => "Artist",
            Self::Cartoonist => "Cartoonist",
        }
    }
}

/// Stroke line style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StrokeStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

impl StrokeStyle {
    pub const ALL: [Self; 3] = [Self::Solid, Self::Dashed, Self::Dotted];

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Solid => "Solid",
            Self::Dashed => "Dashed",
            Self::Dotted => "Dotted",
        }
    }

    /// Converts stroke style to tiny-skia stroke dash configuration.
    pub fn to_stroke_dash(&self, stroke_width: f32) -> Option<StrokeDash> {
        match self {
            Self::Solid => None,
            Self::Dashed => {
                let dash = (stroke_width * 3.5).max(6.0);
                let gap = (stroke_width * 2.5).max(4.0);
                StrokeDash::new(vec![dash, gap], 0.0)
            }
            Self::Dotted => {
                let dot = (stroke_width * 1.0).max(2.0);
                let gap = (stroke_width * 2.0).max(4.0);
                StrokeDash::new(vec![dot, gap], 0.0)
            }
        }
    }
}

/// Corner curvature style for rectangles and boxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CornerStyle {
    #[default]
    Sharp,
    Round,
}

impl CornerStyle {
    pub const ALL: [Self; 2] = [Self::Sharp, Self::Round];

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Sharp => "Sharp",
            Self::Round => "Round",
        }
    }
}

/// Fast, deterministic 64-bit Linear Congruential Generator (LCG) PRNG.
/// Ensures identical jitter patterns across redraws without flickering.
#[derive(Debug, Clone)]
pub struct RoughRng {
    state: u64,
}

impl RoughRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x853c49e6748fea9b } else { seed },
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    pub fn gen_range(&mut self, min: f32, max: f32) -> f32 {
        let t = (self.next_u32() as f32) / (u32::MAX as f32);
        min + t * (max - min)
    }

    /// Generates a 2D offset vector within a circle of `radius`.
    pub fn offset(&mut self, radius: f32) -> Point2D {
        let angle = self.gen_range(0.0, std::f32::consts::TAU);
        let dist = self.gen_range(0.0, radius);
        Point2D::new(angle.cos() * dist, angle.sin() * dist)
    }
}

/// Global counter for generating distinct seeds across shapes.
static SEED_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1234567);

pub fn generate_shape_seed() -> u64 {
    SEED_COUNTER.fetch_add(7919, std::sync::atomic::Ordering::Relaxed)
}

/// Hand-drawn path generator for sketchy whiteboard aesthetics.
pub struct RoughGenerator;

#[inline]
pub fn safe_color_with_alpha(color: Color, alpha_mult: f32) -> Color {
    let a = (color.alpha() * alpha_mult).clamp(0.0, 1.0);
    Color::from_rgba(color.red(), color.green(), color.blue(), a).unwrap_or(Color::BLACK)
}

impl RoughGenerator {
    /// Draws a rough sketchy line segment from `start` to `end`.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_rough_line(
        pixmap: &mut Pixmap,
        start: Point2D,
        end: Point2D,
        color: Color,
        stroke_width: f32,
        stroke_style: StrokeStyle,
        sloppiness: Sloppiness,
        seed: u64,
        opacity: f32,
    ) {
        let length = start.distance(&end);
        if length < 0.5 {
            return;
        }

        let mut rng = RoughRng::new(seed);
        let mut paint = Paint::default();
        paint.set_color(safe_color_with_alpha(color, opacity));
        paint.anti_alias = true;

        let stroke = Stroke {
            width: stroke_width,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            dash: stroke_style.to_stroke_dash(stroke_width),
            ..Default::default()
        };

        let passes = match sloppiness {
            Sloppiness::Architect => 1,
            Sloppiness::Artist => 2,
            Sloppiness::Cartoonist => 2,
        };

        let max_jitter = match sloppiness {
            Sloppiness::Architect => 0.5,
            Sloppiness::Artist => (stroke_width * 0.45).clamp(1.0, 2.5),
            Sloppiness::Cartoonist => (stroke_width * 0.9).clamp(2.0, 5.0),
        };

        for pass in 0..passes {
            let mut p_start = start;
            let mut p_end = end;

            if sloppiness != Sloppiness::Architect {
                let s_off = rng.offset(max_jitter);
                let e_off = rng.offset(max_jitter);
                p_start = p_start + s_off;
                p_end = p_end + e_off;

                if sloppiness == Sloppiness::Cartoonist && pass == 1 {
                    // Slight overshoot at endpoints
                    let dir = (p_end - p_start).angle_to(&Point2D::ZERO);
                    let overshoot = rng.gen_range(1.5, 4.0);
                    p_start.x -= dir.cos() * overshoot;
                    p_start.y -= dir.sin() * overshoot;
                    p_end.x += dir.cos() * overshoot;
                    p_end.y += dir.sin() * overshoot;
                }
            }

            let mut pb = PathBuilder::new();
            pb.move_to(p_start.x, p_start.y);

            if sloppiness == Sloppiness::Architect {
                pb.line_to(p_end.x, p_end.y);
            } else {
                // Subtle mid-line curve (bowing)
                let mid = p_start.midpoint(&p_end);
                let normal_angle =
                    (p_end - p_start).angle_to(&Point2D::ZERO) + std::f32::consts::FRAC_PI_2;
                let bow = rng.gen_range(-max_jitter * 0.8, max_jitter * 0.8);
                let ctrl = Point2D::new(
                    mid.x + normal_angle.cos() * bow,
                    mid.y + normal_angle.sin() * bow,
                );
                pb.quad_to(ctrl.x, ctrl.y, p_end.x, p_end.y);
            }

            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }
    }

    /// Builds a rough closed path for polygons (diamonds, rectangles, etc.)
    #[allow(clippy::too_many_arguments)]
    pub fn draw_rough_polygon(
        pixmap: &mut Pixmap,
        vertices: &[Point2D],
        stroke_color: Color,
        fill_color: Option<Color>,
        stroke_width: f32,
        stroke_style: StrokeStyle,
        sloppiness: Sloppiness,
        seed: u64,
        opacity: f32,
    ) {
        if vertices.len() < 3 {
            return;
        }

        // 1. Fill if configured
        if let Some(fill) = fill_color {
            let mut pb = PathBuilder::new();
            pb.move_to(vertices[0].x, vertices[0].y);
            for v in &vertices[1..] {
                pb.line_to(v.x, v.y);
            }
            pb.close();

            if let Some(path) = pb.finish() {
                let mut fill_paint = Paint::default();
                fill_paint.set_color(safe_color_with_alpha(fill, opacity));
                fill_paint.anti_alias = true;
                pixmap.fill_path(
                    &path,
                    &fill_paint,
                    tiny_skia::FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
        }

        // 2. Draw rough edge segments
        for i in 0..vertices.len() {
            let next_i = (i + 1) % vertices.len();
            let edge_seed = seed.wrapping_add((i as u64) * 31);
            Self::draw_rough_line(
                pixmap,
                vertices[i],
                vertices[next_i],
                stroke_color,
                stroke_width,
                stroke_style,
                sloppiness,
                edge_seed,
                opacity,
            );
        }
    }

    /// Draws a rough ellipse or circle.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_rough_ellipse(
        pixmap: &mut Pixmap,
        center: Point2D,
        rx: f32,
        ry: f32,
        stroke_color: Color,
        fill_color: Option<Color>,
        stroke_width: f32,
        stroke_style: StrokeStyle,
        sloppiness: Sloppiness,
        seed: u64,
        opacity: f32,
    ) {
        if rx <= 0.0 || ry <= 0.0 {
            return;
        }

        // 1. Fill base ellipse
        if let Some(fill) = fill_color {
            let mut pb = PathBuilder::new();
            build_ellipse_path(&mut pb, center, rx, ry);
            if let Some(path) = pb.finish() {
                let mut fill_paint = Paint::default();
                fill_paint.set_color(safe_color_with_alpha(fill, opacity));
                fill_paint.anti_alias = true;
                pixmap.fill_path(
                    &path,
                    &fill_paint,
                    tiny_skia::FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
        }

        // 2. Stroke outline
        let mut rng = RoughRng::new(seed);
        let mut stroke_paint = Paint::default();
        stroke_paint.set_color(safe_color_with_alpha(stroke_color, opacity));
        stroke_paint.anti_alias = true;

        let stroke = Stroke {
            width: stroke_width,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            dash: stroke_style.to_stroke_dash(stroke_width),
            ..Default::default()
        };

        let passes = match sloppiness {
            Sloppiness::Architect => 1,
            Sloppiness::Artist => 2,
            Sloppiness::Cartoonist => 2,
        };

        let max_jitter = match sloppiness {
            Sloppiness::Architect => 0.0,
            Sloppiness::Artist => (stroke_width * 0.4).clamp(0.8, 2.5),
            Sloppiness::Cartoonist => (stroke_width * 0.8).clamp(2.0, 4.5),
        };

        for _ in 0..passes {
            let mut pb = PathBuilder::new();
            let c = center + rng.offset(max_jitter * 0.5);
            let prx = rx + rng.gen_range(-max_jitter, max_jitter);
            let pry = ry + rng.gen_range(-max_jitter, max_jitter);
            build_ellipse_path(&mut pb, c, prx.max(1.0), pry.max(1.0));

            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
    }
}

/// Approximates an ellipse using 4 cubic Bézier splines (magic constant k ≈ 0.5522848).
fn build_ellipse_path(pb: &mut PathBuilder, center: Point2D, rx: f32, ry: f32) {
    let kx = rx * 0.552_284_8;
    let ky = ry * 0.552_284_8;

    pb.move_to(center.x, center.y - ry);
    pb.cubic_to(
        center.x + kx,
        center.y - ry,
        center.x + rx,
        center.y - ky,
        center.x + rx,
        center.y,
    );
    pb.cubic_to(
        center.x + rx,
        center.y + ky,
        center.x + kx,
        center.y + ry,
        center.x,
        center.y + ry,
    );
    pb.cubic_to(
        center.x - kx,
        center.y + ry,
        center.x - rx,
        center.y + ky,
        center.x - rx,
        center.y,
    );
    pb.cubic_to(
        center.x - rx,
        center.y - ky,
        center.x - kx,
        center.y - ry,
        center.x,
        center.y - ry,
    );
    pb.close();
}
