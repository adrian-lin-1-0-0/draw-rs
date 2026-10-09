use std::ops::{Add, Sub};

/// Represents a 2D point / vector in canvas coordinates (physical pixels).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Euclidean distance to another point.
    #[inline]
    pub fn distance(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Angle in radians towards another point (from self to other).
    #[inline]
    pub fn angle_to(&self, other: &Self) -> f32 {
        (other.y - self.y).atan2(other.x - self.x)
    }

    /// Linear interpolation between self and other.
    #[inline]
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    /// Midpoint between two points.
    #[inline]
    pub fn midpoint(&self, other: &Self) -> Self {
        self.lerp(other, 0.5)
    }

    /// Calculate the minimum distance from `self` to the line segment between `a` and `b`.
    pub fn distance_to_segment(&self, a: &Self, b: &Self) -> f32 {
        let ab = *b - *a;
        let ap = *self - *a;
        let len_sq = ab.x * ab.x + ab.y * ab.y;
        if len_sq == 0.0 {
            return self.distance(a);
        }
        let t = ((ap.x * ab.x + ap.y * ab.y) / len_sq).clamp(0.0, 1.0);
        let projection = Self::new(a.x + t * ab.x, a.y + t * ab.y);
        self.distance(&projection)
    }
}

impl Add for Point2D {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Point2D {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
