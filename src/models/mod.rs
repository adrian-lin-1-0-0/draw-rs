pub mod arrow;
pub mod circle;
pub mod point;
pub mod stroke;

#[allow(unused_imports)]
pub use arrow::ArrowShape;
#[allow(unused_imports)]
pub use circle::CircleShape;
#[allow(unused_imports)]
pub use point::Point2D;
#[allow(unused_imports)]
pub use stroke::StrokeShape;

/// Core abstraction for any drawable element on the canvas (OCP & LSP).
///
/// Any new shape or annotation (e.g., Grid, Text, Polygon) can be added simply
/// by implementing this trait, without modifying existing canvas or rendering code.
pub trait Drawable: Send + Sync {
    fn draw(&self, pixmap: &mut tiny_skia::Pixmap);

    /// Checks if this shape intersects with a circular eraser region centered at `point` with `radius`.
    fn intersects(&self, point: Point2D, radius: f32) -> bool;
}
