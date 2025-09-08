//! Axis-aligned bounding box implementation using glam
//!
//! This module provides a simple AABB (Axis-Aligned Bounding Box) implementation
//! for 2D rectangles. Since Luna doesn't support rotation, all our bounds remain
//! axis-aligned, making calculations much simpler.

use glam::Vec2;

/// An axis-aligned bounding box represented by minimum and maximum points
///
/// This representation is efficient for intersection tests and unions.
/// All bounds in Luna are axis-aligned since we don't support rotation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Bounds {
    /// The minimum point (top-left in screen coordinates)
    pub min: Vec2,
    /// The maximum point (bottom-right in screen coordinates)
    pub max: Vec2,
}

impl Bounds {
    /// Creates a new bounds from minimum and maximum points
    ///
    /// Note: This doesn't validate that min is actually less than max.
    /// Use `from_corners` if you need automatic ordering.
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Creates bounds from an origin point and size
    pub fn from_origin_size(origin: Vec2, size: Vec2) -> Self {
        Self {
            min: origin,
            max: origin + size,
        }
    }

    /// Creates bounds from center point and half-extents (half width/height)
    pub fn from_center_half_size(center: Vec2, half_size: Vec2) -> Self {
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    /// Creates bounds from center point and full size
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half_size = size * 0.5;
        Self::from_center_half_size(center, half_size)
    }

    /// Creates bounds from two corner points, automatically ordering them
    pub fn from_corners(a: Vec2, b: Vec2) -> Self {
        Self {
            min: a.min(b),
            max: a.max(b),
        }
    }

    /// Creates an empty bounds at the origin
    pub fn zero() -> Self {
        Self {
            min: Vec2::ZERO,
            max: Vec2::ZERO,
        }
    }

    /// Returns the origin (minimum point) of the bounds
    pub fn origin(&self) -> Vec2 {
        self.min
    }

    /// Returns the size of the bounds
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    /// Returns the center point of the bounds
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    /// Returns the half-size (half width and height) of the bounds
    pub fn half_size(&self) -> Vec2 {
        self.size() * 0.5
    }

    /// Returns the width of the bounds
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// Returns the height of the bounds
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// Returns the area of the bounds
    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }

    /// Checks if the bounds are valid (min <= max)
    pub fn is_valid(&self) -> bool {
        self.min.x <= self.max.x && self.min.y <= self.max.y
    }

    /// Checks if the bounds are empty (zero size)
    pub fn is_empty(&self) -> bool {
        self.min.x >= self.max.x || self.min.y >= self.max.y
    }

    /// Tests if this bounds intersects with another
    ///
    /// Two bounds intersect if they overlap in both X and Y axes
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    /// Computes the intersection of two bounds
    ///
    /// Returns None if the bounds don't intersect
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let min = self.min.max(other.min);
        let max = self.max.min(other.max);

        if min.x <= max.x && min.y <= max.y {
            Some(Self { min, max })
        } else {
            None
        }
    }

    /// Computes the union of two bounds
    ///
    /// The union is the smallest bounds that contains both input bounds
    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Tests if a point is contained within the bounds
    ///
    /// Points on the boundary are considered contained
    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Tests if another bounds is entirely contained within this bounds
    pub fn contains_bounds(&self, other: &Self) -> bool {
        other.min.x >= self.min.x
            && other.max.x <= self.max.x
            && other.min.y >= self.min.y
            && other.max.y <= self.max.y
    }

    /// Expands the bounds by a given amount in all directions
    pub fn expand(&self, amount: f32) -> Self {
        Self {
            min: self.min - Vec2::splat(amount),
            max: self.max + Vec2::splat(amount),
        }
    }

    /// Expands the bounds by a given vector amount
    pub fn expand_vec(&self, amount: Vec2) -> Self {
        Self {
            min: self.min - amount,
            max: self.max + amount,
        }
    }

    /// Contracts the bounds by a given amount in all directions
    ///
    /// If the contraction would make the bounds invalid, returns a zero-sized bounds at the center
    pub fn contract(&self, amount: f32) -> Self {
        let new_min = self.min + Vec2::splat(amount);
        let new_max = self.max - Vec2::splat(amount);

        if new_min.x <= new_max.x && new_min.y <= new_max.y {
            Self {
                min: new_min,
                max: new_max,
            }
        } else {
            let center = self.center();
            Self {
                min: center,
                max: center,
            }
        }
    }

    /// Translates the bounds by a given offset
    pub fn translate(&self, offset: Vec2) -> Self {
        Self {
            min: self.min + offset,
            max: self.max + offset,
        }
    }

    /// Scales the bounds by a given factor from its center
    pub fn scale_from_center(&self, scale: f32) -> Self {
        let center = self.center();
        let half_size = self.half_size() * scale;
        Self::from_center_half_size(center, half_size)
    }

    /// Scales the bounds by a given factor from its origin
    pub fn scale_from_origin(&self, scale: f32) -> Self {
        Self {
            min: self.min,
            max: self.min + (self.size() * scale),
        }
    }

    /// Scales the bounds by a given factor from a specific point
    pub fn scale_from_point(&self, scale: f32, point: Vec2) -> Self {
        Self {
            min: point + (self.min - point) * scale,
            max: point + (self.max - point) * scale,
        }
    }

    /// Clamps a point to be within the bounds
    pub fn clamp_point(&self, point: Vec2) -> Vec2 {
        point.clamp(self.min, self.max)
    }

    /// Returns the four corner points of the bounds
    pub fn corners(&self) -> [Vec2; 4] {
        [
            self.min,                          // top-left
            Vec2::new(self.max.x, self.min.y), // top-right
            self.max,                          // bottom-right
            Vec2::new(self.min.x, self.max.y), // bottom-left
        ]
    }

    /// Computes the distance from a point to the bounds
    ///
    /// Returns 0 if the point is inside the bounds
    pub fn distance_to_point(&self, point: Vec2) -> f32 {
        let clamped = self.clamp_point(point);
        (point - clamped).length()
    }

    /// Computes the squared distance from a point to the bounds
    ///
    /// More efficient than `distance_to_point` when you don't need the exact distance
    pub fn distance_squared_to_point(&self, point: Vec2) -> f32 {
        let clamped = self.clamp_point(point);
        (point - clamped).length_squared()
    }
}

impl Default for Bounds {
    fn default() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_creation() {
        let bounds = Bounds::from_origin_size(Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0));
        assert_eq!(bounds.min, Vec2::new(10.0, 20.0));
        assert_eq!(bounds.max, Vec2::new(110.0, 70.0));
        assert_eq!(bounds.size(), Vec2::new(100.0, 50.0));
        assert_eq!(bounds.center(), Vec2::new(60.0, 45.0));
    }

    #[test]
    fn test_bounds_intersection() {
        let a = Bounds::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let b = Bounds::from_origin_size(Vec2::new(50.0, 50.0), Vec2::new(100.0, 100.0));

        assert!(a.intersects(&b));

        let intersection = a.intersection(&b).unwrap();
        assert_eq!(intersection.min, Vec2::new(50.0, 50.0));
        assert_eq!(intersection.max, Vec2::new(100.0, 100.0));
    }

    #[test]
    fn test_bounds_union() {
        let a = Bounds::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let b = Bounds::from_origin_size(Vec2::new(50.0, 50.0), Vec2::new(100.0, 100.0));

        let union = a.union(&b);
        assert_eq!(union.min, Vec2::new(0.0, 0.0));
        assert_eq!(union.max, Vec2::new(150.0, 150.0));
    }

    #[test]
    fn test_bounds_contains() {
        let bounds = Bounds::from_origin_size(Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0));

        assert!(bounds.contains_point(Vec2::new(50.0, 40.0)));
        assert!(bounds.contains_point(Vec2::new(10.0, 20.0))); // edge case: minimum point
        assert!(bounds.contains_point(Vec2::new(110.0, 70.0))); // edge case: maximum point
        assert!(!bounds.contains_point(Vec2::new(5.0, 40.0)));
        assert!(!bounds.contains_point(Vec2::new(120.0, 40.0)));
    }

    #[test]
    fn test_bounds_expand_contract() {
        let bounds = Bounds::from_origin_size(Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0));

        let expanded = bounds.expand(10.0);
        assert_eq!(expanded.min, Vec2::new(0.0, 10.0));
        assert_eq!(expanded.max, Vec2::new(120.0, 80.0));

        let contracted = bounds.contract(5.0);
        assert_eq!(contracted.min, Vec2::new(15.0, 25.0));
        assert_eq!(contracted.max, Vec2::new(105.0, 65.0));
    }

    #[test]
    fn test_bounds_scale() {
        let bounds = Bounds::from_origin_size(Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0));

        let scaled = bounds.scale_from_center(2.0);
        assert_eq!(scaled.center(), bounds.center()); // center should remain the same
        assert_eq!(scaled.size(), Vec2::new(200.0, 100.0));

        let scaled_origin = bounds.scale_from_origin(2.0);
        assert_eq!(scaled_origin.min, bounds.min); // origin should remain the same
        assert_eq!(scaled_origin.size(), Vec2::new(200.0, 100.0));
    }

    #[test]
    fn test_bounds_distance() {
        let bounds = Bounds::from_origin_size(Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0));

        // Point inside bounds
        assert_eq!(bounds.distance_to_point(Vec2::new(50.0, 40.0)), 0.0);

        // Point directly above bounds
        assert_eq!(bounds.distance_to_point(Vec2::new(50.0, 10.0)), 10.0);

        // Point directly to the right of bounds
        assert_eq!(bounds.distance_to_point(Vec2::new(120.0, 40.0)), 10.0);

        // Point at diagonal
        let diagonal_point = Vec2::new(120.0, 80.0);
        let expected = Vec2::new(10.0, 10.0).length();
        assert!((bounds.distance_to_point(diagonal_point) - expected).abs() < 0.001);
    }
}
