//! # Coordinate System Types
//!
//! This module provides type-safe coordinate representations for different coordinate systems
//! used in the Luna application. By using distinct types for each coordinate space, we prevent
//! accidental mixing of coordinates from different spaces.
//!
//! ## Coordinate Spaces
//!
//! - **Canvas Coordinates**: Centered at (0,0) in the middle of the canvas
//! - **Window Coordinates**: Origin at top-left of the window (0,0)
//! - **Parent-Relative Coordinates**: Position relative to parent element

use glam::Vec2;
use gpui::{Bounds, Point, Size};
use std::ops::{Add, Div, Mul, Sub};

/// Canvas coordinates with origin (0,0) at the center of the canvas
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasPoint(Vec2);

/// Window coordinates with origin (0,0) at the top-left of the window
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowPoint(Vec2);

/// Parent-relative coordinates, position relative to parent element's origin
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParentRelativePoint(Vec2);

/// Canvas size (width, height) in canvas space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasSize(Vec2);

/// Canvas bounds in canvas coordinate space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasBounds {
    pub origin: CanvasPoint,
    pub size: CanvasSize,
}

impl CanvasPoint {
    /// Create a new canvas point at the specified coordinates
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    /// Create from a glam Vec2
    pub fn from_vec2(vec: Vec2) -> Self {
        Self(vec)
    }

    /// Get the underlying Vec2
    pub fn as_vec2(&self) -> Vec2 {
        self.0
    }

    /// Get x coordinate
    pub fn x(&self) -> f32 {
        self.0.x
    }

    /// Get y coordinate
    pub fn y(&self) -> f32 {
        self.0.y
    }

    /// Set x coordinate
    pub fn set_x(&mut self, x: f32) {
        self.0.x = x;
    }

    /// Set y coordinate
    pub fn set_y(&mut self, y: f32) {
        self.0.y = y;
    }

    /// Convert to a GPUI Point
    pub fn to_gpui(&self) -> Point<f32> {
        Point::new(self.0.x, self.0.y)
    }

    /// Create from a GPUI Point
    pub fn from_gpui(point: Point<f32>) -> Self {
        Self(Vec2::new(point.x, point.y))
    }

    /// Convert to parent-relative coordinates by applying the parent's position
    pub fn to_parent_relative(&self, parent_pos: CanvasPoint) -> ParentRelativePoint {
        ParentRelativePoint(self.0 - parent_pos.0)
    }

    /// Calculate distance to another point
    pub fn distance(&self, other: CanvasPoint) -> f32 {
        self.0.distance(other.0)
    }

    /// Calculate squared distance to another point (more efficient for comparisons)
    pub fn distance_squared(&self, other: CanvasPoint) -> f32 {
        self.0.distance_squared(other.0)
    }

    /// Linear interpolation to another point
    pub fn lerp(&self, other: CanvasPoint, t: f32) -> CanvasPoint {
        CanvasPoint(self.0.lerp(other.0, t))
    }

    /// Get the length (distance from origin)
    pub fn length(&self) -> f32 {
        self.0.length()
    }

    /// Normalize the vector (make it unit length)
    pub fn normalize(&self) -> CanvasPoint {
        CanvasPoint(self.0.normalize())
    }

    /// Dot product with another point (treated as vector)
    pub fn dot(&self, other: CanvasPoint) -> f32 {
        self.0.dot(other.0)
    }
}

impl WindowPoint {
    /// Create a new window point at the specified coordinates
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    /// Create from a glam Vec2
    pub fn from_vec2(vec: Vec2) -> Self {
        Self(vec)
    }

    /// Get the underlying Vec2
    pub fn as_vec2(&self) -> Vec2 {
        self.0
    }

    /// Get x coordinate
    pub fn x(&self) -> f32 {
        self.0.x
    }

    /// Get y coordinate
    pub fn y(&self) -> f32 {
        self.0.y
    }

    /// Set x coordinate
    pub fn set_x(&mut self, x: f32) {
        self.0.x = x;
    }

    /// Set y coordinate
    pub fn set_y(&mut self, y: f32) {
        self.0.y = y;
    }

    /// Convert to a GPUI Point
    pub fn to_point(&self) -> Point<f32> {
        Point::new(self.0.x, self.0.y)
    }

    /// Create from a GPUI Point
    pub fn from_point(point: Point<f32>) -> Self {
        Self(Vec2::new(point.x, point.y))
    }

    /// Calculate distance to another point
    pub fn distance(&self, other: WindowPoint) -> f32 {
        self.0.distance(other.0)
    }

    /// Linear interpolation to another point
    pub fn lerp(&self, other: WindowPoint, t: f32) -> WindowPoint {
        WindowPoint(self.0.lerp(other.0, t))
    }
}

impl ParentRelativePoint {
    /// Create a new parent-relative point at the specified coordinates
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    /// Create from a glam Vec2
    pub fn from_vec2(vec: Vec2) -> Self {
        Self(vec)
    }

    /// Get the underlying Vec2
    pub fn as_vec2(&self) -> Vec2 {
        self.0
    }

    /// Get x coordinate
    pub fn x(&self) -> f32 {
        self.0.x
    }

    /// Get y coordinate
    pub fn y(&self) -> f32 {
        self.0.y
    }

    /// Set x coordinate
    pub fn set_x(&mut self, x: f32) {
        self.0.x = x;
    }

    /// Set y coordinate
    pub fn set_y(&mut self, y: f32) {
        self.0.y = y;
    }

    /// Convert to canvas coordinates by applying the parent's position
    pub fn to_canvas(&self, parent_pos: CanvasPoint) -> CanvasPoint {
        CanvasPoint(self.0 + parent_pos.0)
    }

    /// Convert to a GPUI Point
    pub fn to_point(&self) -> Point<f32> {
        Point::new(self.0.x, self.0.y)
    }

    /// Create from a GPUI Point
    pub fn from_point(point: Point<f32>) -> Self {
        Self(Vec2::new(point.x, point.y))
    }
}

impl CanvasSize {
    /// Create a new canvas size with the specified dimensions
    pub fn new(width: f32, height: f32) -> Self {
        Self(Vec2::new(width, height))
    }

    /// Create from a glam Vec2
    pub fn from_vec2(vec: Vec2) -> Self {
        Self(vec)
    }

    /// Get the underlying Vec2
    pub fn as_vec2(&self) -> Vec2 {
        self.0
    }

    /// Get width
    pub fn width(&self) -> f32 {
        self.0.x
    }

    /// Get height
    pub fn height(&self) -> f32 {
        self.0.y
    }

    /// Set width
    pub fn set_width(&mut self, width: f32) {
        self.0.x = width;
    }

    /// Set height
    pub fn set_height(&mut self, height: f32) {
        self.0.y = height;
    }

    /// Get the area (width * height)
    pub fn area(&self) -> f32 {
        self.0.x * self.0.y
    }

    /// Convert to a GPUI Size
    pub fn to_size(&self) -> Size<f32> {
        Size::new(self.0.x, self.0.y)
    }

    /// Create from a GPUI Size
    pub fn from_size(size: Size<f32>) -> Self {
        Self(Vec2::new(size.width, size.height))
    }

    /// Check if this size contains another size
    pub fn contains(&self, other: &CanvasSize) -> bool {
        self.0.x >= other.0.x && self.0.y >= other.0.y
    }
}

impl CanvasBounds {
    /// Create new canvas bounds with the specified origin and size
    pub fn new(origin: CanvasPoint, size: CanvasSize) -> Self {
        Self { origin, size }
    }

    /// Create from center point and size
    pub fn from_center_size(center: CanvasPoint, size: CanvasSize) -> Self {
        let half_size = size.0 * 0.5;
        Self {
            origin: CanvasPoint(center.0 - half_size),
            size,
        }
    }

    /// Get the center point of the bounds
    pub fn center(&self) -> CanvasPoint {
        CanvasPoint(self.origin.0 + self.size.0 * 0.5)
    }

    /// Get the minimum point (top-left)
    pub fn min(&self) -> CanvasPoint {
        self.origin
    }

    /// Get the maximum point (bottom-right)
    pub fn max(&self) -> CanvasPoint {
        CanvasPoint(self.origin.0 + self.size.0)
    }

    /// Convert to a GPUI Bounds
    pub fn to_bounds(&self) -> Bounds<f32> {
        Bounds {
            origin: self.origin.to_gpui(),
            size: self.size.to_size(),
        }
    }

    /// Create from a GPUI Bounds
    pub fn from_bounds(bounds: Bounds<f32>) -> Self {
        Self {
            origin: CanvasPoint::from_gpui(bounds.origin),
            size: CanvasSize::from_size(bounds.size),
        }
    }

    /// Check if this bounds contains a point
    pub fn contains(&self, point: CanvasPoint) -> bool {
        let min = self.origin.0;
        let max = self.origin.0 + self.size.0;
        point.0.x >= min.x && point.0.y >= min.y && point.0.x <= max.x && point.0.y <= max.y
    }

    /// Check if this bounds intersects with another bounds
    pub fn intersects(&self, other: &CanvasBounds) -> bool {
        let self_min = self.origin.0;
        let self_max = self.origin.0 + self.size.0;
        let other_min = other.origin.0;
        let other_max = other.origin.0 + other.size.0;

        self_min.x < other_max.x
            && self_max.x > other_min.x
            && self_min.y < other_max.y
            && self_max.y > other_min.y
    }

    /// Get the intersection of two bounds, if any
    pub fn intersection(&self, other: &CanvasBounds) -> Option<CanvasBounds> {
        if !self.intersects(other) {
            return None;
        }

        let self_min = self.origin.0;
        let self_max = self.origin.0 + self.size.0;
        let other_min = other.origin.0;
        let other_max = other.origin.0 + other.size.0;

        let min = Vec2::new(self_min.x.max(other_min.x), self_min.y.max(other_min.y));
        let max = Vec2::new(self_max.x.min(other_max.x), self_max.y.min(other_max.y));

        Some(CanvasBounds {
            origin: CanvasPoint(min),
            size: CanvasSize(max - min),
        })
    }

    /// Get the union of two bounds
    pub fn union(&self, other: &CanvasBounds) -> CanvasBounds {
        let self_min = self.origin.0;
        let self_max = self.origin.0 + self.size.0;
        let other_min = other.origin.0;
        let other_max = other.origin.0 + other.size.0;

        let min = Vec2::new(self_min.x.min(other_min.x), self_min.y.min(other_min.y));
        let max = Vec2::new(self_max.x.max(other_max.x), self_max.y.max(other_max.y));

        CanvasBounds {
            origin: CanvasPoint(min),
            size: CanvasSize(max - min),
        }
    }

    /// Expand the bounds by a given amount in all directions
    pub fn expand(&self, amount: f32) -> CanvasBounds {
        let expand_vec = Vec2::splat(amount);
        CanvasBounds {
            origin: CanvasPoint(self.origin.0 - expand_vec),
            size: CanvasSize(self.size.0 + expand_vec * 2.0),
        }
    }

    /// Contract the bounds by a given amount in all directions
    pub fn contract(&self, amount: f32) -> CanvasBounds {
        self.expand(-amount)
    }
}

// Implementation for Add, Sub, Mul, Div operations
impl Add for CanvasPoint {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for CanvasPoint {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl Mul<f32> for CanvasPoint {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self(self.0 * scalar)
    }
}

impl Div<f32> for CanvasPoint {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self(self.0 / scalar)
    }
}

// Similar operations for WindowPoint
impl Add for WindowPoint {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for WindowPoint {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl Mul<f32> for WindowPoint {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self(self.0 * scalar)
    }
}

impl Div<f32> for WindowPoint {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self(self.0 / scalar)
    }
}

// Similar operations for ParentRelativePoint
impl Add for ParentRelativePoint {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for ParentRelativePoint {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl Mul<f32> for ParentRelativePoint {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self(self.0 * scalar)
    }
}

impl Div<f32> for ParentRelativePoint {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self(self.0 / scalar)
    }
}

// Operations for CanvasSize
impl Add for CanvasSize {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for CanvasSize {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl Mul<f32> for CanvasSize {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self(self.0 * scalar)
    }
}

impl Div<f32> for CanvasSize {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self(self.0 / scalar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_point_conversion() {
        let canvas_point = CanvasPoint::new(10.0, 20.0);
        let parent_pos = CanvasPoint::new(5.0, 8.0);

        let parent_relative = canvas_point.to_parent_relative(parent_pos);
        assert_eq!(parent_relative.x(), 5.0);
        assert_eq!(parent_relative.y(), 12.0);

        let canvas_point2 = parent_relative.to_canvas(parent_pos);
        assert_eq!(canvas_point, canvas_point2);
    }

    #[test]
    fn test_canvas_bounds_contains() {
        let bounds = CanvasBounds::new(CanvasPoint::new(10.0, 10.0), CanvasSize::new(20.0, 30.0));

        // Points inside
        assert!(bounds.contains(CanvasPoint::new(15.0, 15.0)));
        assert!(bounds.contains(CanvasPoint::new(10.0, 10.0))); // On edge
        assert!(bounds.contains(CanvasPoint::new(30.0, 40.0))); // Bottom right

        // Points outside
        assert!(!bounds.contains(CanvasPoint::new(5.0, 15.0)));
        assert!(!bounds.contains(CanvasPoint::new(15.0, 5.0)));
        assert!(!bounds.contains(CanvasPoint::new(35.0, 15.0)));
        assert!(!bounds.contains(CanvasPoint::new(15.0, 45.0)));
    }

    #[test]
    fn test_bounds_intersection() {
        let bounds1 = CanvasBounds::new(CanvasPoint::new(0.0, 0.0), CanvasSize::new(10.0, 10.0));
        let bounds2 = CanvasBounds::new(CanvasPoint::new(5.0, 5.0), CanvasSize::new(10.0, 10.0));

        let intersection = bounds1.intersection(&bounds2).unwrap();
        assert_eq!(intersection.origin.x(), 5.0);
        assert_eq!(intersection.origin.y(), 5.0);
        assert_eq!(intersection.size.width(), 5.0);
        assert_eq!(intersection.size.height(), 5.0);
    }

    #[test]
    fn test_vec2_operations() {
        let p1 = CanvasPoint::new(10.0, 20.0);
        let p2 = CanvasPoint::new(5.0, 10.0);

        let sum = p1 + p2;
        assert_eq!(sum.x(), 15.0);
        assert_eq!(sum.y(), 30.0);

        let diff = p1 - p2;
        assert_eq!(diff.x(), 5.0);
        assert_eq!(diff.y(), 10.0);

        let scaled = p1 * 2.0;
        assert_eq!(scaled.x(), 20.0);
        assert_eq!(scaled.y(), 40.0);

        let divided = p1 / 2.0;
        assert_eq!(divided.x(), 5.0);
        assert_eq!(divided.y(), 10.0);
    }

    #[test]
    fn test_distance_and_lerp() {
        let p1 = CanvasPoint::new(0.0, 0.0);
        let p2 = CanvasPoint::new(3.0, 4.0);

        assert_eq!(p1.distance(p2), 5.0);

        let mid = p1.lerp(p2, 0.5);
        assert_eq!(mid.x(), 1.5);
        assert_eq!(mid.y(), 2.0);
    }
}
