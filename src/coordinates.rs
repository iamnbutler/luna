//! # Coordinate System and Position Management
//!
//! This module provides:
//! 1. Type-safe coordinate representations for different coordinate systems
//! 2. Global position data management for coordinate transformations
//! 3. Conversion utilities between coordinate spaces
//!
//! By using distinct types for each coordinate space and centralized position management,
//! we prevent accidental mixing of coordinates from different spaces and ensure consistent
//! transformations throughout the application.
//!
//! ## Coordinate Spaces
//!
//! - **World (Canvas) Space**: Centered at (0,0) in the middle of the canvas
//! - **Window (UI) Space**: Origin at top-left of the window (0,0)
//! - **Local (Node/Child) Space**: Position relative to parent element

use gpui::{App, Bounds, Context, Global, Point, Size};
use std::ops::{Add, Div, Mul, Sub};
use std::sync::{Arc, RwLock};

/// Canvas coordinates with origin (0,0) at the center of the canvas
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldPoint {
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
pub struct LocalPoint {
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
    pub origin: WorldPoint,
    pub size: CanvasSize,
}

impl WorldPoint {
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
    pub fn to_local(&self, parent_pos: WorldPoint) -> LocalPoint {
        LocalPoint {
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

impl LocalPoint {
    /// Create a new parent-relative point at the specified coordinates
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Convert to canvas coordinates by applying the parent's position
    pub fn to_canvas(&self, parent_pos: WorldPoint) -> WorldPoint {
        WorldPoint {
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
    pub fn new(origin: WorldPoint, size: CanvasSize) -> Self {
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
            origin: WorldPoint::from_point(bounds.origin),
            size: CanvasSize::from_size(bounds.size),
        }
    }

    /// Check if this bounds contains a point
    pub fn contains(&self, point: WorldPoint) -> bool {
        point.x >= self.origin.x
            && point.y >= self.origin.y
            && point.x <= self.origin.x + self.size.width
            && point.y <= self.origin.y + self.size.height
    }
}

// Implementation for Add, Sub, Mul, Div operations
impl Add for WorldPoint {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for WorldPoint {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for WorldPoint {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Div<f32> for WorldPoint {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

/// Stores position-related data for coordinate transformations
pub struct PositionData {
    /// Delta between canvas origin and top-left point in window
    pub canvas_offset: WorldPoint,
    
    /// Current window dimensions
    pub window_dimensions: Size<f32>,
    
    /// Flag for when positions need recalculating
    pub dirty: bool,
}

impl PositionData {
    /// Create a new PositionData with initial values
    pub fn new(canvas_offset: WorldPoint, window_dimensions: Size<f32>) -> Self {
        Self {
            canvas_offset,
            window_dimensions,
            dirty: false,
        }
    }
    
    /// Mark position data as dirty, requiring recalculation
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    
    /// Update canvas offset
    pub fn update_offset(&mut self, new_offset: WorldPoint) {
        self.canvas_offset = new_offset;
        self.mark_dirty();
    }
    
    /// Update window dimensions
    pub fn update_dimensions(&mut self, new_dimensions: Size<f32>) {
        self.window_dimensions = new_dimensions;
        self.mark_dirty();
    }
    
    /// Convert a world point to window space
    pub fn world_to_window(&self, world_point: WorldPoint, zoom: f32) -> WindowPoint {
        // Center of viewport in window space
        let center_x = self.window_dimensions.width / 2.0;
        let center_y = self.window_dimensions.height / 2.0;
        
        // Convert from canvas to window space with center origin
        let window_x = ((world_point.x - self.canvas_offset.x) * zoom) + center_x;
        let window_y = ((world_point.y - self.canvas_offset.y) * zoom) + center_y;
        
        WindowPoint::new(window_x, window_y)
    }
    
    /// Convert a window point to world space
    pub fn window_to_world(&self, window_point: WindowPoint, zoom: f32) -> WorldPoint {
        // Center of viewport in window space
        let center_x = self.window_dimensions.width / 2.0;
        let center_y = self.window_dimensions.height / 2.0;
        
        // Convert from window to canvas space with center origin
        let world_x = ((window_point.x - center_x) / zoom) + self.canvas_offset.x;
        let world_y = ((window_point.y - center_y) / zoom) + self.canvas_offset.y;
        
        WorldPoint::new(world_x, world_y)
    }
}

/// Global access to position data
#[derive(Clone)]
pub struct GlobalPosition(pub Arc<RwLock<PositionData>>);

// Implement Global trait to allow use with cx.global() and cx.set_global()
impl Global for GlobalPosition {}

/// Trait for accessing position data from context
pub trait PositionStore {
    fn position_data(&self) -> Arc<RwLock<PositionData>>;
}

// Implement for App and Context types
impl PositionStore for App {
    fn position_data(&self) -> Arc<RwLock<PositionData>> {
        self.global::<GlobalPosition>().0.clone()
    }
}

impl<'a, T> PositionStore for Context<'a, T> {
    fn position_data(&self) -> Arc<RwLock<PositionData>> {
        self.global::<GlobalPosition>().0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_point_conversion() {
        let canvas_point = WorldPoint::new(10.0, 20.0);
        let parent_pos = WorldPoint::new(5.0, 8.0);

        let parent_relative = canvas_point.to_local(parent_pos);
        assert_eq!(parent_relative.x, 5.0);
        assert_eq!(parent_relative.y, 12.0);

        let canvas_point2 = parent_relative.to_canvas(parent_pos);
        assert_eq!(canvas_point, canvas_point2);
    }

    #[test]
    fn test_canvas_bounds_contains() {
        let bounds = CanvasBounds::new(WorldPoint::new(10.0, 10.0), CanvasSize::new(20.0, 30.0));

        // Points inside
        assert!(bounds.contains(WorldPoint::new(15.0, 15.0)));
        assert!(bounds.contains(WorldPoint::new(10.0, 10.0))); // On edge
        assert!(bounds.contains(WorldPoint::new(30.0, 40.0))); // Bottom right

        // Points outside
        assert!(!bounds.contains(WorldPoint::new(5.0, 15.0)));
        assert!(!bounds.contains(WorldPoint::new(15.0, 5.0)));
        assert!(!bounds.contains(WorldPoint::new(35.0, 15.0)));
        assert!(!bounds.contains(WorldPoint::new(15.0, 45.0)));
    }
    
    #[test]
    fn test_position_data_conversions() {
        let position_data = PositionData::new(
            WorldPoint::new(0.0, 0.0),  // No offset
            Size::new(1000.0, 800.0),   // Window size
        );
        
        let zoom = 1.0;
        
        // Test world to window conversion
        let world_point = WorldPoint::new(0.0, 0.0);
        let window_point = position_data.world_to_window(world_point, zoom);
        
        // Origin (0,0) in world space should map to center of window
        assert_eq!(window_point.x, 500.0);
        assert_eq!(window_point.y, 400.0);
        
        // Test window to world conversion
        let window_point = WindowPoint::new(500.0, 400.0);
        let world_point = position_data.window_to_world(window_point, zoom);
        
        // Center of window should map to origin (0,0) in world space
        assert_eq!(world_point.x, 0.0);
        assert_eq!(world_point.y, 0.0);
    }
}
