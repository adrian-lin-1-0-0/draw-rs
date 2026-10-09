use crate::models::Drawable;
use tiny_skia::Pixmap;

/// Manages the history of drawn objects on the canvas, providing undo,
/// clear, and unified rendering functionality.
///
/// Adheres to:
/// - SRP: Manages only drawing history and canvas composition.
/// - OCP & LSP: Treats all shapes uniformly via `Box<dyn Drawable>`.
/// - DIP: Depends on the `Drawable` abstraction rather than concrete shapes.
pub struct CanvasController {
    history: Vec<Box<dyn Drawable>>,
}

impl Default for CanvasController {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasController {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    /// Add a new completed shape to the canvas history.
    pub fn push_shape(&mut self, shape: Box<dyn Drawable>) {
        self.history.push(shape);
    }

    /// Undo the last drawn shape. Returns true if a shape was removed.
    pub fn undo(&mut self) -> bool {
        self.history.pop().is_some()
    }

    /// Clear all shapes from canvas history.
    pub fn clear(&mut self) {
        self.history.clear();
    }

    /// Returns the number of shapes currently in history.
    pub fn shape_count(&self) -> usize {
        self.history.len()
    }

    /// Returns whether the canvas history is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// Render all shapes in history, plus an optional in-progress preview shape.
    pub fn draw_all(&self, pixmap: &mut Pixmap, preview: Option<&dyn Drawable>) {
        for shape in &self.history {
            shape.draw(pixmap);
        }

        if let Some(preview_shape) = preview {
            preview_shape.draw(pixmap);
        }
    }
}
