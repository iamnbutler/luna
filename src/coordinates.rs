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

use gpui::{App, Bounds, Context, Global, Point, Size, DevicePixels};
use std::ops::{Add, Div, Mul, Sub};
use std::sync::{Arc, RwLock};

/// Snaps a float value to device-appropriate pixels
/// 
/// When snap_to_pixel_grid is true, rounds to whole pixels
/// When false, rounds to the nearest device pixel based on the scale factor
fn snap_value(value: f32, snap_to_pixel_grid: bool, scale_factor: f32) -> f32 {
    if snap_to_pixel_grid {
        // Snap to nearest whole pixel
        value.round()
    } else if scale_factor <= 1.0 {
        // At 1x density or lower, still snap to whole pixels
        value.round()
    } else {
        // Convert to device pixels, round, then convert back to logical pixels
        // This ensures we're snapping to actual device pixel boundaries
        let device_pixels = value * scale_factor;
        let rounded_device_pixels = device_pixels.round();
        rounded_device_pixels / scale_factor
    }
}

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

    /// Creates a rendering-friendly version of this point with values snapped to the specified grid
    ///
    /// If snap_to_pixel_grid is true, coordinates will be rounded to whole pixels
    /// If false, coordinates will be rounded to the nearest device pixel based on the scale factor
    pub fn snapped(&self, snap_to_pixel_grid: bool, scale_factor: f32) -> Self {
        Self {
            x: snap_value(self.x, snap_to_pixel_grid, scale_factor),
            y: snap_value(self.y, snap_to_pixel_grid, scale_factor),
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

    /// Creates a rendering-friendly version of this point with values snapped to the specified grid
    ///
    /// If snap_to_pixel_grid is true, coordinates will be rounded to whole pixels
    /// If false, coordinates will be rounded to the nearest device pixel based on the scale factor
    pub fn snapped(&self, snap_to_pixel_grid: bool, scale_factor: f32) -> Self {
        Self {
            x: snap_value(self.x, snap_to_pixel_grid, scale_factor),
            y: snap_value(self.y, snap_to_pixel_grid, scale_factor),
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

    /// Creates a rendering-friendly version of this point with values snapped to the specified grid
    ///
    /// If snap_to_pixel_grid is true, coordinates will be rounded to whole pixels
    /// If false, coordinates will be rounded to the nearest device pixel based on the scale factor
    pub fn snapped(&self, snap_to_pixel_grid: bool, scale_factor: f32) -> Self {
        Self {
            x: snap_value(self.x, snap_to_pixel_grid, scale_factor),
            y: snap_value(self.y, snap_to_pixel_grid, scale_factor),
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
    fn test_render_value() {
        // Test with various scale factors
        let scale_1x = 1.0;
        let scale_2x = 2.0;
        let scale_3x = 3.0;
        
        // Test with whole pixel snapping (snap_to_pixel_grid = true)
        let world_point = WorldPoint::new(10.3, 20.7);
        let snapped_whole = world_point.snapped(true, scale_2x);
        assert_eq!(snapped_whole.x, 10.0);
        assert_eq!(snapped_whole.y, 21.0);
        
        // Test with device pixel snapping at 1x (should round to whole pixels)
        let window_point = WindowPoint::new(15.12, 25.62);
        let snapped_1x = window_point.snapped(false, scale_1x);
        assert_eq!(snapped_1x.x, 15.0);  // 15.12 rounds to 15.0
        assert_eq!(snapped_1x.y, 26.0);  // 25.62 rounds to 26.0 at 1x
        
        // Test with device pixel snapping at 2x
        let snapped_2x = window_point.snapped(false, scale_2x);
        assert_eq!(snapped_2x.x, 15.0);  // At 2x, 15.12*2=30.24 rounds to 30/2=15.0
        assert_eq!(snapped_2x.y, 25.5);  // At 2x, 25.62*2=51.24 rounds to 51/2=25.5
        
        // Test with device pixel snapping at 3x
        let local_point = LocalPoint::new(5.25, 8.75);
        let snapped_3x = local_point.snapped(false, scale_3x);
        // 5.25*3=15.75 rounds to 16/3≈5.33
        // 8.75*3=26.25 rounds to 26/3≈8.67
        // But we'll use more exact values for the comparison
        let expected_x = (5.25 * scale_3x).round() / scale_3x;
        let expected_y = (8.75 * scale_3x).round() / scale_3x;
        assert_eq!(snapped_3x.x, expected_x);
        assert_eq!(snapped_3x.y, expected_y);
        
        // Test with values that need rounding
        let edge_point = LocalPoint::new(7.24, 7.26);
        let snapped_edge = edge_point.snapped(false, scale_2x);
        // At 2x, 7.24*2=14.48 rounds to 14/2=7.0
        // At 2x, 7.26*2=14.52 rounds to 15/2=7.5
        assert_eq!(snapped_edge.x, 7.0);
        assert_eq!(snapped_edge.y, 7.5);
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
            WorldPoint::new(0.0, 0.0), // No offset
            Size::new(1000.0, 800.0),  // Window size
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

    #[test]
    fn test_roundtrip_coordinate_conversions() {
        // Create test data: parent position, position data, and zoom level
        let parent_pos = WorldPoint::new(100.0, 150.0);
        let position_data = PositionData::new(
            WorldPoint::new(50.0, 25.0), // Canvas offset
            Size::new(1000.0, 800.0),    // Window size
        );
        let zoom = 1.5; // Non-1.0 zoom to ensure scaling works properly

        // Start with local coordinates relative to parent
        let original_local = LocalPoint::new(25.0, 35.0);

        // Convert: Local → World
        let world = original_local.to_canvas(parent_pos);
        assert_eq!(world.x, 125.0); // 100 + 25
        assert_eq!(world.y, 185.0); // 150 + 35

        // Convert: World → Window
        let window = position_data.world_to_window(world, zoom);
        // Expected: ((125 - 50) * 1.5) + 500 = 112.5 + 500 = 612.5
        // Expected: ((185 - 25) * 1.5) + 400 = 240 + 400 = 640
        assert_eq!(window.x, 612.5);
        assert_eq!(window.y, 640.0);

        // Convert: Window → World
        let world_again = position_data.window_to_world(window, zoom);
        // Should be same as original world coordinates
        assert_eq!(world_again.x, world.x);
        assert_eq!(world_again.y, world.y);

        // Convert: World → Local
        let local_again = world_again.to_local(parent_pos);
        // Should be same as original local coordinates
        assert_eq!(local_again.x, original_local.x);
        assert_eq!(local_again.y, original_local.y);
    }
}
