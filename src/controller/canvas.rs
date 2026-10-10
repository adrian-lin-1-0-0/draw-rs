use crate::models::{Drawable, Point2D, Rect2D};
use tiny_skia::Pixmap;

/// Represents an undoable canvas action with full forward/reverse semantics.
enum CanvasAction {
    Draw(Box<dyn Drawable>),
    DrawBatch(Vec<Box<dyn Drawable>>),
    Erase(Vec<(usize, Box<dyn Drawable>)>),
    Clear(Vec<Box<dyn Drawable>>),
    Transform {
        original: Vec<(usize, Box<dyn Drawable>)>,
        modified: Vec<(usize, Box<dyn Drawable>)>,
    },
    Reorder {
        old_shapes: Vec<Box<dyn Drawable>>,
        new_shapes: Vec<Box<dyn Drawable>>,
    },
}

/// Manages the history of drawn objects on the canvas, providing undo, redo,
/// eraser, selection hit-testing, layer reordering, and unified rendering.
///
/// Adheres to:
/// - SRP: Manages only drawing history and canvas composition.
/// - OCP & LSP: Treats all shapes uniformly via `Box<dyn Drawable>`.
/// - DIP: Depends on the `Drawable` abstraction rather than concrete shapes.
pub struct CanvasController {
    shapes: Vec<Box<dyn Drawable>>,
    action_history: Vec<CanvasAction>,
    redo_history: Vec<CanvasAction>,
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
            redo_history: Vec::new(),
        }
    }

    /// Add a new completed shape to the canvas.
    pub fn push_shape(&mut self, shape: Box<dyn Drawable>) {
        self.action_history
            .push(CanvasAction::Draw(shape.clone_box()));
        self.shapes.push(shape);
        self.redo_history.clear();
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
            self.redo_history.clear();
        }
    }

    /// Commit a transformation (move, resize, property change) to the undo stack.
    pub fn commit_transform(
        &mut self,
        original: Vec<(usize, Box<dyn Drawable>)>,
        modified: Vec<(usize, Box<dyn Drawable>)>,
    ) {
        if !original.is_empty() {
            self.action_history
                .push(CanvasAction::Transform { original, modified });
            self.redo_history.clear();
        }
    }

    /// Undo the last action (Draw, Erase, Clear, Transform, Reorder).
    pub fn undo(&mut self) -> bool {
        let Some(last_action) = self.action_history.pop() else {
            return false;
        };

        match last_action {
            CanvasAction::Draw(shape) => {
                self.shapes.pop();
                self.redo_history.push(CanvasAction::Draw(shape));
            }
            CanvasAction::DrawBatch(batch) => {
                let count = batch.len();
                self.shapes
                    .truncate(self.shapes.len().saturating_sub(count));
                self.redo_history.push(CanvasAction::DrawBatch(batch));
            }
            CanvasAction::Erase(removed) => {
                // Restore erased shapes in reverse removal order to maintain indices
                for (index, shape) in removed.iter().rev() {
                    let insert_pos = (*index).min(self.shapes.len());
                    self.shapes.insert(insert_pos, shape.clone_box());
                }
                self.redo_history.push(CanvasAction::Erase(removed));
            }
            CanvasAction::Clear(old_shapes) => {
                let current_shapes = std::mem::replace(&mut self.shapes, old_shapes);
                self.redo_history.push(CanvasAction::Clear(current_shapes));
            }
            CanvasAction::Transform { original, modified } => {
                for (idx, shape) in &original {
                    if let Some(target) = self.shapes.get_mut(*idx) {
                        *target = shape.clone_box();
                    }
                }
                self.redo_history
                    .push(CanvasAction::Transform { original, modified });
            }
            CanvasAction::Reorder {
                old_shapes,
                new_shapes,
            } => {
                self.shapes = old_shapes.clone();
                self.redo_history.push(CanvasAction::Reorder {
                    old_shapes,
                    new_shapes,
                });
            }
        }

        true
    }

    /// Redo the last undone action.
    pub fn redo(&mut self) -> bool {
        let Some(next_action) = self.redo_history.pop() else {
            return false;
        };

        match next_action {
            CanvasAction::Draw(shape) => {
                self.shapes.push(shape.clone_box());
                self.action_history.push(CanvasAction::Draw(shape));
            }
            CanvasAction::DrawBatch(batch) => {
                for shape in &batch {
                    self.shapes.push(shape.clone_box());
                }
                self.action_history.push(CanvasAction::DrawBatch(batch));
            }
            CanvasAction::Erase(removed) => {
                // Erase again in descending order
                let mut sorted_indices: Vec<usize> = removed.iter().map(|(idx, _)| *idx).collect();
                sorted_indices.sort_unstable_by(|a, b| b.cmp(a));
                for idx in sorted_indices {
                    if idx < self.shapes.len() {
                        self.shapes.remove(idx);
                    }
                }
                self.action_history.push(CanvasAction::Erase(removed));
            }
            CanvasAction::Clear(_) => {
                let current = std::mem::take(&mut self.shapes);
                self.action_history.push(CanvasAction::Clear(current));
            }
            CanvasAction::Transform { original, modified } => {
                for (idx, shape) in &modified {
                    if let Some(target) = self.shapes.get_mut(*idx) {
                        *target = shape.clone_box();
                    }
                }
                self.action_history
                    .push(CanvasAction::Transform { original, modified });
            }
            CanvasAction::Reorder {
                old_shapes,
                new_shapes,
            } => {
                self.shapes = new_shapes.clone();
                self.action_history.push(CanvasAction::Reorder {
                    old_shapes,
                    new_shapes,
                });
            }
        }

        true
    }

    /// Clear all shapes from canvas history. Undoable!
    pub fn clear(&mut self) {
        if !self.shapes.is_empty() {
            let old_shapes = std::mem::take(&mut self.shapes);
            self.action_history.push(CanvasAction::Clear(old_shapes));
            self.redo_history.clear();
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

    /// Hit-tests all shapes from top to bottom, returning the index of the topmost clicked shape.
    pub fn hit_test_all(&self, point: Point2D) -> Option<usize> {
        for (i, shape) in self.shapes.iter().enumerate().rev() {
            if shape.hit_test(point) {
                return Some(i);
            }
        }
        None
    }

    /// Selects all shapes whose bounding box intersects with the given rectangle.
    pub fn box_select(&self, rect: Rect2D) -> Vec<usize> {
        let mut selected = Vec::new();
        for (i, shape) in self.shapes.iter().enumerate() {
            if let Some(bb) = shape.bounding_box()
                && rect.intersects(&bb)
            {
                selected.push(i);
            }
        }
        selected
    }

    /// Computes the combined bounding box enclosing all shapes at `indices`.
    pub fn combined_bounds(&self, indices: &[usize]) -> Option<Rect2D> {
        let mut combined: Option<Rect2D> = None;
        for &idx in indices {
            if let Some(shape) = self.shapes.get(idx)
                && let Some(bb) = shape.bounding_box()
            {
                combined = Some(match combined {
                    Some(c) => c.union(&bb),
                    None => bb,
                });
            }
        }
        combined
    }

    /// Translates all shapes by `offset` (e.g. for canvas panning).
    pub fn translate_all(&mut self, offset: Point2D) {
        for shape in &mut self.shapes {
            shape.translate(offset);
        }
    }

    /// Clones selected shapes and offsets them by `offset`, returning their newly added indices.
    pub fn duplicate(&mut self, indices: &[usize], offset: Point2D) -> Vec<usize> {
        if indices.is_empty() {
            return Vec::new();
        }

        let mut cloned_shapes = Vec::new();
        let mut new_indices = Vec::new();

        for &idx in indices {
            if let Some(shape) = self.shapes.get(idx) {
                let mut cloned = shape.clone_box();
                cloned.translate(offset);
                new_indices.push(self.shapes.len() + cloned_shapes.len());
                cloned_shapes.push(cloned);
            }
        }

        for s in &cloned_shapes {
            self.shapes.push(s.clone_box());
        }

        self.action_history
            .push(CanvasAction::DrawBatch(cloned_shapes));
        self.redo_history.clear();

        new_indices
    }

    /// Deletes shapes at `indices` and commits to undo history.
    pub fn delete_selected(&mut self, indices: &[usize]) {
        if indices.is_empty() {
            return;
        }

        let mut sorted: Vec<usize> = indices.to_vec();
        sorted.sort_unstable();
        sorted.dedup();

        let mut removed = Vec::new();
        // Remove from highest index to lowest so preceding indices remain valid
        for &idx in sorted.iter().rev() {
            if idx < self.shapes.len() {
                let shape = self.shapes.remove(idx);
                removed.push((idx, shape));
            }
        }

        // Store in forward order for commit_erase
        removed.reverse();
        self.commit_erase(removed);
    }

    /// Brings selected shapes to the very front (topmost drawing layer).
    pub fn bring_to_front(&mut self, indices: &[usize]) -> Vec<usize> {
        if indices.is_empty() || self.shapes.len() <= 1 {
            return indices.to_vec();
        }

        let old_shapes = self.shapes.clone();
        let set: std::collections::HashSet<usize> = indices.iter().copied().collect();

        let mut unselected = Vec::new();
        let mut selected = Vec::new();

        for (i, shape) in self.shapes.drain(..).enumerate() {
            if set.contains(&i) {
                selected.push(shape);
            } else {
                unselected.push(shape);
            }
        }

        let start_new_idx = unselected.len();
        let new_indices: Vec<usize> = (start_new_idx..start_new_idx + selected.len()).collect();

        self.shapes = unselected;
        self.shapes.extend(selected);

        self.action_history.push(CanvasAction::Reorder {
            old_shapes,
            new_shapes: self.shapes.clone(),
        });
        self.redo_history.clear();

        new_indices
    }

    /// Sends selected shapes to the very back (bottommost drawing layer).
    pub fn send_to_back(&mut self, indices: &[usize]) -> Vec<usize> {
        if indices.is_empty() || self.shapes.len() <= 1 {
            return indices.to_vec();
        }

        let old_shapes = self.shapes.clone();
        let set: std::collections::HashSet<usize> = indices.iter().copied().collect();

        let mut unselected = Vec::new();
        let mut selected = Vec::new();

        for (i, shape) in self.shapes.drain(..).enumerate() {
            if set.contains(&i) {
                selected.push(shape);
            } else {
                unselected.push(shape);
            }
        }

        let new_indices: Vec<usize> = (0..selected.len()).collect();
        selected.extend(unselected);
        self.shapes = selected;

        self.action_history.push(CanvasAction::Reorder {
            old_shapes,
            new_shapes: self.shapes.clone(),
        });
        self.redo_history.clear();

        new_indices
    }

    /// Moves selected shapes forward by 1 layer.
    pub fn bring_forward(&mut self, indices: &[usize]) -> Vec<usize> {
        if indices.is_empty() || self.shapes.len() <= 1 {
            return indices.to_vec();
        }

        let old_shapes = self.shapes.clone();
        let mut sorted: Vec<usize> = indices.to_vec();
        sorted.sort_unstable();

        let mut new_indices = Vec::new();
        // Move from right to left
        for &idx in sorted.iter().rev() {
            if idx + 1 < self.shapes.len() && !sorted.contains(&(idx + 1)) {
                self.shapes.swap(idx, idx + 1);
                new_indices.push(idx + 1);
            } else {
                new_indices.push(idx);
            }
        }
        new_indices.reverse();

        self.action_history.push(CanvasAction::Reorder {
            old_shapes,
            new_shapes: self.shapes.clone(),
        });
        self.redo_history.clear();

        new_indices
    }

    /// Moves selected shapes backward by 1 layer.
    pub fn send_backward(&mut self, indices: &[usize]) -> Vec<usize> {
        if indices.is_empty() || self.shapes.len() <= 1 {
            return indices.to_vec();
        }

        let old_shapes = self.shapes.clone();
        let mut sorted: Vec<usize> = indices.to_vec();
        sorted.sort_unstable();

        let mut new_indices = Vec::new();
        // Move from left to right
        for &idx in &sorted {
            if idx > 0 && !sorted.contains(&(idx - 1)) {
                self.shapes.swap(idx, idx - 1);
                new_indices.push(idx - 1);
            } else {
                new_indices.push(idx);
            }
        }

        self.action_history.push(CanvasAction::Reorder {
            old_shapes,
            new_shapes: self.shapes.clone(),
        });
        self.redo_history.clear();

        new_indices
    }

    /// Access immutable reference to shape at `index`.
    pub fn shape(&self, index: usize) -> Option<&dyn Drawable> {
        self.shapes.get(index).map(|b| b.as_ref())
    }

    /// Access mutable reference to shape at `index`.
    pub fn shape_mut(&mut self, index: usize) -> Option<&mut Box<dyn Drawable>> {
        self.shapes.get_mut(index)
    }

    /// Slice of all shapes on canvas.
    pub fn shapes(&self) -> &[Box<dyn Drawable>] {
        &self.shapes
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
