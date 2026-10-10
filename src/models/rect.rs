use super::point::Point2D;

/// Resize and rotate handles on a selected bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformHandle {
    Nw,
    N,
    Ne,
    E,
    Se,
    S,
    Sw,
    W,
    Rotate,
}

/// Axis-Aligned Bounding Box (AABB) in canvas coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect2D {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl Rect2D {
    pub const ZERO: Self = Self {
        min_x: 0.0,
        min_y: 0.0,
        max_x: 0.0,
        max_y: 0.0,
    };

    #[inline]
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min_x: min_x.min(max_x),
            min_y: min_y.min(max_y),
            max_x: min_x.max(max_x),
            max_y: min_y.max(max_y),
        }
    }

    #[inline]
    pub fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::new(x, y, x + width, y + height)
    }

    #[inline]
    pub fn from_points(p1: Point2D, p2: Point2D) -> Self {
        Self::new(p1.x, p1.y, p2.x, p2.y)
    }

    #[inline]
    pub fn width(&self) -> f32 {
        (self.max_x - self.min_x).max(0.0)
    }

    #[inline]
    pub fn height(&self) -> f32 {
        (self.max_y - self.min_y).max(0.0)
    }

    #[inline]
    pub fn center(&self) -> Point2D {
        Point2D::new(
            (self.min_x + self.max_x) * 0.5,
            (self.min_y + self.max_y) * 0.5,
        )
    }

    #[inline]
    pub fn contains(&self, p: Point2D) -> bool {
        p.x >= self.min_x && p.x <= self.max_x && p.y >= self.min_y && p.y <= self.max_y
    }

    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }

    #[inline]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }

    #[inline]
    pub fn expand(&self, margin: f32) -> Self {
        Self {
            min_x: self.min_x - margin,
            min_y: self.min_y - margin,
            max_x: self.max_x + margin,
            max_y: self.max_y + margin,
        }
    }

    /// Returns the positions of all 8 resize handles and 1 rotation handle.
    pub fn handles(&self) -> [(TransformHandle, Point2D); 9] {
        let mid_x = (self.min_x + self.max_x) * 0.5;
        let mid_y = (self.min_y + self.max_y) * 0.5;
        let rot_y = self.min_y - 24.0;

        [
            (TransformHandle::Nw, Point2D::new(self.min_x, self.min_y)),
            (TransformHandle::N, Point2D::new(mid_x, self.min_y)),
            (TransformHandle::Ne, Point2D::new(self.max_x, self.min_y)),
            (TransformHandle::E, Point2D::new(self.max_x, mid_y)),
            (TransformHandle::Se, Point2D::new(self.max_x, self.max_y)),
            (TransformHandle::S, Point2D::new(mid_x, self.max_y)),
            (TransformHandle::Sw, Point2D::new(self.min_x, self.max_y)),
            (TransformHandle::W, Point2D::new(self.min_x, mid_y)),
            (TransformHandle::Rotate, Point2D::new(mid_x, rot_y)),
        ]
    }

    /// Hit-tests all transform handles given a cursor point and tolerance radius.
    pub fn hit_test_handle(&self, point: Point2D, tolerance: f32) -> Option<TransformHandle> {
        for (handle, pos) in self.handles() {
            if point.distance(&pos) <= tolerance {
                return Some(handle);
            }
        }
        None
    }
}
