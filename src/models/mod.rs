pub mod arrow;
pub mod circle;
pub mod diamond;
pub mod line;
pub mod point;
pub mod rect;
pub mod rectangle;
pub mod rough;
pub mod stroke;
pub mod text;

#[allow(unused_imports)]
pub use arrow::ArrowShape;
#[allow(unused_imports)]
pub use circle::CircleShape;
#[allow(unused_imports)]
pub use diamond::DiamondShape;
#[allow(unused_imports)]
pub use line::LineShape;
#[allow(unused_imports)]
pub use point::Point2D;
#[allow(unused_imports)]
pub use rect::{Rect2D, TransformHandle};
#[allow(unused_imports)]
pub use rectangle::RectangleShape;
#[allow(unused_imports)]
pub use rough::{CornerStyle, RoughGenerator, Sloppiness, StrokeStyle, generate_shape_seed};
#[allow(unused_imports)]
pub use stroke::StrokeShape;
#[allow(unused_imports)]
pub use text::TextShape;

use tiny_skia::{Color, Pixmap};

/// Core abstraction for any drawable element on the canvas (OCP & LSP).
///
/// Any new shape or annotation can be added simply by implementing this trait,
/// without modifying existing canvas or rendering code.
pub trait Drawable: Send + Sync {
    /// Render this shape onto the target tiny-skia Pixmap.
    fn draw(&self, pixmap: &mut Pixmap);

    /// Checks if this shape intersects with a circular eraser region centered at `point` with `radius`.
    fn intersects(&self, point: Point2D, radius: f32) -> bool;

    /// Hit-tests whether a click at `point` selects this shape.
    fn hit_test(&self, point: Point2D) -> bool {
        self.intersects(point, 6.0)
    }

    /// Calculates axis-aligned bounding box enclosing this shape.
    fn bounding_box(&self) -> Option<Rect2D>;

    /// Translates this shape by a 2D offset vector.
    fn translate(&mut self, offset: Point2D);

    /// Resizes / stretches this shape from `old_bounds` to `new_bounds`.
    fn resize(&mut self, old_bounds: Rect2D, new_bounds: Rect2D);

    /// Deep-clones this trait object into a new heap allocation.
    fn clone_box(&self) -> Box<dyn Drawable>;

    // --- Styling attributes ---
    fn stroke_color(&self) -> Color;
    fn set_stroke_color(&mut self, color: Color);

    fn fill_color(&self) -> Option<Color> {
        None
    }
    fn set_fill_color(&mut self, _color: Option<Color>) {}

    fn stroke_width(&self) -> f32 {
        3.5
    }
    fn set_stroke_width(&mut self, _width: f32) {}

    fn stroke_style(&self) -> StrokeStyle {
        StrokeStyle::Solid
    }
    fn set_stroke_style(&mut self, _style: StrokeStyle) {}

    fn sloppiness(&self) -> Sloppiness {
        Sloppiness::Architect
    }
    fn set_sloppiness(&mut self, _sloppiness: Sloppiness) {}

    fn corner_style(&self) -> CornerStyle {
        CornerStyle::Sharp
    }
    fn set_corner_style(&mut self, _corner: CornerStyle) {}

    fn opacity(&self) -> f32 {
        1.0
    }
    fn set_opacity(&mut self, _opacity: f32) {}

    fn seed(&self) -> u64 {
        0
    }
    fn set_seed(&mut self, _seed: u64) {}

    fn text_content(&self) -> Option<&str> {
        None
    }
    fn set_text_content(&mut self, _text: String) {}
}

impl Clone for Box<dyn Drawable> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
