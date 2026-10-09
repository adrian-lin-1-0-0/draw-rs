use crate::models::{Drawable, point::Point2D};
use tiny_skia::Pixmap;

/// Represents an undoable canvas action.
enum CanvasAction {
    Draw,
    Erase(Vec<(usize, Box<dyn Drawable>)>),
    Clear(Vec<Box<dyn Drawable>>),
}

/// Manages the history of drawn objects on the canvas, providing undo,
/// eraser, clear, and unified rendering functionality.
///
/// Adheres to:
/// - SRP: Manages only drawing history and canvas composition.
/// - OCP & LSP: Treats all shapes uniformly via `Box<dyn Drawable>`.
/// - DIP: Depends on the `Drawable` abstraction rather than concrete shapes.
pub struct CanvasController {
    shapes: Vec<Box<dyn Drawable>>,
    action_history: Vec<CanvasAction>,
}

impl Default for CanvasController {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasController {
    pub fn new() -> Self {
        Self {
            shapes: Vec::new(),
            action_history: Vec::new(),
        }
    }

    /// Add a new completed shape to the canvas.
    pub fn push_shape(&mut self, shape: Box<dyn Drawable>) {
        self.shapes.push(shape);
        self.action_history.push(CanvasAction::Draw);
    }

    /// Erase any shapes intersecting with the circular eraser at `point` with `radius`.
    /// Returns the removed shapes and their original indices.
    pub fn erase_at(&mut self, point: Point2D, radius: f32) -> Vec<(usize, Box<dyn Drawable>)> {
        let mut removed = Vec::new();
        let mut i = 0;
        while i < self.shapes.len() {
            if self.shapes[i].intersects(point, radius) {
                let shape = self.shapes.remove(i);
                removed.push((i, shape));
            } else {
                i += 1;
            }
        }
        removed
    }

    /// Commit a batch of erased shapes from a single drag gesture to the undo stack.
    pub fn commit_erase(&mut self, removed: Vec<(usize, Box<dyn Drawable>)>) {
        if !removed.is_empty() {
            self.action_history.push(CanvasAction::Erase(removed));
        }
    }

    /// Undo the last action (Draw, Erase, or Clear). Returns true if an action was undone.
    pub fn undo(&mut self) -> bool {
        let Some(last_action) = self.action_history.pop() else {
            return false;
        };

        match last_action {
            CanvasAction::Draw => {
                self.shapes.pop();
            }
            CanvasAction::Erase(removed) => {
                // Restore erased shapes in reverse removal order to restore original indices
                for (index, shape) in removed.into_iter().rev() {
                    let insert_pos = index.min(self.shapes.len());
                    self.shapes.insert(insert_pos, shape);
                }
            }
            CanvasAction::Clear(old_shapes) => {
                self.shapes = old_shapes;
            }
        }

        true
    }

    /// Clear all shapes from canvas history. Undoable!
    pub fn clear(&mut self) {
        if !self.shapes.is_empty() {
            let old_shapes = std::mem::take(&mut self.shapes);
            self.action_history.push(CanvasAction::Clear(old_shapes));
        }
    }

    /// Returns the number of shapes currently on the canvas.
    pub fn shape_count(&self) -> usize {
        self.shapes.len()
    }

    /// Returns whether the canvas is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }

    /// Render all shapes in history, plus an optional in-progress preview shape.
    pub fn draw_all(&self, pixmap: &mut Pixmap, preview: Option<&dyn Drawable>) {
        for shape in &self.shapes {
            shape.draw(pixmap);
        }

        if let Some(preview_shape) = preview {
            preview_shape.draw(pixmap);
        }
    }
}
