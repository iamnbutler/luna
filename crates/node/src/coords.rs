//! Type-safe coordinate system for Luna2.
//!
//! Provides distinct types for different coordinate spaces to prevent
//! accidental mixing at compile time.
//!
//! # Coordinate Spaces
//!
//! - **Canvas space**: Where shapes live (infinite, zoomable)
//! - **Screen space**: Pixels relative to canvas element origin (after zoom/pan)
//! - **Local space**: Position relative to parent shape origin

use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

/// Position in canvas space (where shapes live).
///
/// Canvas space is the infinite coordinate system where shapes are defined.
/// The viewport transforms canvas coordinates to screen coordinates for display.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct CanvasPoint(pub Vec2);

/// Position in screen space (pixels relative to canvas element, after zoom/pan).
///
/// Screen space coordinates are what you see on screen after the viewport
/// transformation has been applied.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct ScreenPoint(pub Vec2);

/// Position relative to parent shape origin.
///
/// Used for child shapes within frames. The position is relative to the
/// parent's top-left corner in canvas space.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct LocalPoint(pub Vec2);

/// Size in canvas space.
///
/// Represents width and height of shapes in canvas coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct CanvasSize(pub Vec2);

/// Movement/offset in canvas space (not a position).
///
/// Represents a change or delta, not an absolute position.
/// Used for translations and movements.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct CanvasDelta(pub Vec2);

// === CanvasPoint ===

impl CanvasPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    pub fn x(&self) -> f32 {
        self.0.x
    }

    pub fn y(&self) -> f32 {
        self.0.y
    }
}

impl From<Vec2> for CanvasPoint {
    fn from(v: Vec2) -> Self {
        Self(v)
    }
}

impl From<CanvasPoint> for Vec2 {
    fn from(p: CanvasPoint) -> Self {
        p.0
    }
}

impl Add<CanvasDelta> for CanvasPoint {
    type Output = CanvasPoint;

    fn add(self, delta: CanvasDelta) -> Self::Output {
        CanvasPoint(self.0 + delta.0)
    }
}

impl Sub for CanvasPoint {
    type Output = CanvasDelta;

    /// Subtracting two points gives a delta.
    fn sub(self, other: CanvasPoint) -> Self::Output {
        CanvasDelta(self.0 - other.0)
    }
}

// === ScreenPoint ===

impl ScreenPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    pub fn x(&self) -> f32 {
        self.0.x
    }

    pub fn y(&self) -> f32 {
        self.0.y
    }
}

impl From<Vec2> for ScreenPoint {
    fn from(v: Vec2) -> Self {
        Self(v)
    }
}

impl From<ScreenPoint> for Vec2 {
    fn from(p: ScreenPoint) -> Self {
        p.0
    }
}

// === LocalPoint ===

impl LocalPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    pub fn x(&self) -> f32 {
        self.0.x
    }

    pub fn y(&self) -> f32 {
        self.0.y
    }

    /// Convert to canvas point given parent's world position.
    pub fn to_canvas(&self, parent_world: CanvasPoint) -> CanvasPoint {
        CanvasPoint(self.0 + parent_world.0)
    }
}

impl From<Vec2> for LocalPoint {
    fn from(v: Vec2) -> Self {
        Self(v)
    }
}

impl From<LocalPoint> for Vec2 {
    fn from(p: LocalPoint) -> Self {
        p.0
    }
}

// === CanvasSize ===

impl CanvasSize {
    pub fn new(width: f32, height: f32) -> Self {
        Self(Vec2::new(width, height))
    }

    pub fn width(&self) -> f32 {
        self.0.x
    }

    pub fn height(&self) -> f32 {
        self.0.y
    }
}

impl From<Vec2> for CanvasSize {
    fn from(v: Vec2) -> Self {
        Self(v)
    }
}

impl From<CanvasSize> for Vec2 {
    fn from(s: CanvasSize) -> Self {
        s.0
    }
}

// === CanvasDelta ===

impl CanvasDelta {
    pub fn new(dx: f32, dy: f32) -> Self {
        Self(Vec2::new(dx, dy))
    }

    pub fn dx(&self) -> f32 {
        self.0.x
    }

    pub fn dy(&self) -> f32 {
        self.0.y
    }
}

impl From<Vec2> for CanvasDelta {
    fn from(v: Vec2) -> Self {
        Self(v)
    }
}

impl From<CanvasDelta> for Vec2 {
    fn from(d: CanvasDelta) -> Self {
        d.0
    }
}

impl Add for CanvasDelta {
    type Output = CanvasDelta;

    fn add(self, other: CanvasDelta) -> Self::Output {
        CanvasDelta(self.0 + other.0)
    }
}

impl Sub for CanvasDelta {
    type Output = CanvasDelta;

    fn sub(self, other: CanvasDelta) -> Self::Output {
        CanvasDelta(self.0 - other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_point_add_delta() {
        let point = CanvasPoint::new(10.0, 20.0);
        let delta = CanvasDelta::new(5.0, -3.0);
        let result = point + delta;
        assert_eq!(result.x(), 15.0);
        assert_eq!(result.y(), 17.0);
    }

    #[test]
    fn canvas_point_sub_gives_delta() {
        let p1 = CanvasPoint::new(10.0, 20.0);
        let p2 = CanvasPoint::new(3.0, 5.0);
        let delta = p1 - p2;
        assert_eq!(delta.dx(), 7.0);
        assert_eq!(delta.dy(), 15.0);
    }

    #[test]
    fn local_to_canvas() {
        let local = LocalPoint::new(10.0, 20.0);
        let parent_world = CanvasPoint::new(100.0, 200.0);
        let canvas = local.to_canvas(parent_world);
        assert_eq!(canvas.x(), 110.0);
        assert_eq!(canvas.y(), 220.0);
    }

    #[test]
    fn from_vec2_conversions() {
        let v = Vec2::new(5.0, 10.0);
        let cp: CanvasPoint = v.into();
        let back: Vec2 = cp.into();
        assert_eq!(v, back);
    }
}
