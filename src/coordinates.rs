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

use gpui::{Bounds, Point, Size};
use std::ops::{Add, Div, Mul, Sub};

/// Canvas coordinates with origin (0,0) at the center of the canvas
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasPoint {
    pub x: f32,
    pub y: f32,
}

/// Window coordinates with origin (0,0) at the top-left of the window
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowPoint {
    pub x: f32,
    pub y: f32,
}

/// Parent-relative coordinates, position relative to parent element's origin
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParentRelativePoint {
    pub x: f32,
    pub y: f32,
}

/// Canvas size (width, height) in canvas space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasSize {
    pub width: f32,
    pub height: f32,
}

/// Canvas bounds in canvas coordinate space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasBounds {
    pub origin: CanvasPoint,
    pub size: CanvasSize,
}

impl CanvasPoint {
    /// Create a new canvas point at the specified coordinates
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Convert to a GPUI Point
    pub fn to_point(&self) -> Point<f32> {
        Point::new(self.x, self.y)
    }

    /// Create from a GPUI Point
    pub fn from_point(point: Point<f32>) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }

    /// Convert to parent-relative coordinates by applying the parent's position
    pub fn to_parent_relative(&self, parent_pos: CanvasPoint) -> ParentRelativePoint {
        ParentRelativePoint {
            x: self.x - parent_pos.x,
            y: self.y - parent_pos.y,
        }
    }
}

impl WindowPoint {
    /// Create a new window point at the specified coordinates
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Convert to a GPUI Point
    pub fn to_point(&self) -> Point<f32> {
        Point::new(self.x, self.y)
    }

    /// Create from a GPUI Point
    pub fn from_point(point: Point<f32>) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

impl ParentRelativePoint {
    /// Create a new parent-relative point at the specified coordinates
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Convert to canvas coordinates by applying the parent's position
    pub fn to_canvas(&self, parent_pos: CanvasPoint) -> CanvasPoint {
        CanvasPoint {
            x: self.x + parent_pos.x,
            y: self.y + parent_pos.y,
        }
    }

    /// Convert to a GPUI Point
    pub fn to_point(&self) -> Point<f32> {
        Point::new(self.x, self.y)
    }

    /// Create from a GPUI Point
    pub fn from_point(point: Point<f32>) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

impl CanvasSize {
    /// Create a new canvas size with the specified dimensions
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Convert to a GPUI Size
    pub fn to_size(&self) -> Size<f32> {
        Size::new(self.width, self.height)
    }

    /// Create from a GPUI Size
    pub fn from_size(size: Size<f32>) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}

impl CanvasBounds {
    /// Create new canvas bounds with the specified origin and size
    pub fn new(origin: CanvasPoint, size: CanvasSize) -> Self {
        Self { origin, size }
    }

    /// Convert to a GPUI Bounds
    pub fn to_bounds(&self) -> Bounds<f32> {
        Bounds {
            origin: self.origin.to_point(),
            size: self.size.to_size(),
        }
    }

    /// Create from a GPUI Bounds
    pub fn from_bounds(bounds: Bounds<f32>) -> Self {
        Self {
            origin: CanvasPoint::from_point(bounds.origin),
            size: CanvasSize::from_size(bounds.size),
        }
    }

    /// Check if this bounds contains a point
    pub fn contains(&self, point: CanvasPoint) -> bool {
        point.x >= self.origin.x
            && point.y >= self.origin.y
            && point.x <= self.origin.x + self.size.width
            && point.y <= self.origin.y + self.size.height
    }
}

// Implementation for Add, Sub, Mul, Div operations
impl Add for CanvasPoint {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for CanvasPoint {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for CanvasPoint {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Div<f32> for CanvasPoint {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
        }
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
        assert_eq!(parent_relative.x, 5.0);
        assert_eq!(parent_relative.y, 12.0);

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
}
