use gpui::{prelude::*, Pixels, Point, Size, Hsla};
use std::ops::{Add, Sub, Mul, Div};

/// Window coordinates - positions in the application window
/// Origin (0,0) is at top-left of window
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowPoint {
    pub x: Pixels,
    pub y: Pixels,
}

/// Window size with pixel dimensions
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowSize {
    pub width: Pixels,
    pub height: Pixels,
}

/// Window rectangle with position and size
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowRect {
    pub origin: WindowPoint,
    pub size: WindowSize,
}

/// Intermediate canvas point that requires context to be fully resolved
/// This prevents premature conversion without proper context
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnresolvedCanvasPoint {
    pub x: f32,
    pub y: f32,
    /// Tracks where a point originated from (for debugging/conversion logic)
    pub source: PointSource,
}

/// Fully resolved canvas coordinates
/// Origin (0,0) is at center of canvas
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasPoint {
    pub x: f32,
    pub y: f32,
}

/// Canvas size in canvas units
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasSize {
    pub width: f32,
    pub height: f32,
}

/// Canvas rectangle with position and size
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasRect {
    pub origin: CanvasPoint,
    pub size: CanvasSize,
}

/// Tracks where a point originated from (for debugging/conversion logic)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointSource {
    Window,
    PartiallyResolved,
}

impl WindowPoint {
    /// Create a new WindowPoint
    pub fn new(x: Pixels, y: Pixels) -> Self {
        Self { x, y }
    }

    /// Convert from gpui::Point<Pixels>
    pub fn from_gpui(point: Point<Pixels>) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }

    /// Convert to gpui::Point<Pixels>
    pub fn to_gpui(&self) -> Point<Pixels> {
        Point::new(self.x, self.y)
    }

    /// Convert to an unresolved canvas point (partial conversion)
    pub fn to_unresolved_canvas(&self) -> UnresolvedCanvasPoint {
        UnresolvedCanvasPoint {
            x: self.x.0,
            y: self.y.0,
            source: PointSource::Window,
        }
    }
}

impl WindowSize {
    /// Create a new WindowSize
    pub fn new(width: Pixels, height: Pixels) -> Self {
        Self { width, height }
    }

    /// Convert from gpui::Size<Pixels>
    pub fn from_gpui(size: Size<Pixels>) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }

    /// Convert to gpui::Size<Pixels>
    pub fn to_gpui(&self) -> Size<Pixels> {
        Size::new(self.width, self.height)
    }
}

impl WindowRect {
    /// Create a new WindowRect
    pub fn new(origin: WindowPoint, size: WindowSize) -> Self {
        Self { origin, size }
    }

    /// Create a WindowRect from gpui::Bounds<Pixels>
    pub fn from_gpui_bounds(bounds: gpui::Bounds<Pixels>) -> Self {
        Self {
            origin: WindowPoint::from_gpui(bounds.origin),
            size: WindowSize::from_gpui(bounds.size),
        }
    }

    /// Convert to gpui::Bounds<Pixels>
    pub fn to_gpui_bounds(&self) -> gpui::Bounds<Pixels> {
        gpui::Bounds {
            origin: self.origin.to_gpui(),
            size: self.size.to_gpui(),
        }
    }
}

impl UnresolvedCanvasPoint {
    /// Creates an unresolved point directly
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            source: PointSource::PartiallyResolved,
        }
    }

    /// Convert to gpui::Point<f32> (for calculations)
    pub fn to_gpui_f32(&self) -> Point<f32> {
        Point::new(self.x, self.y)
    }
}

impl CanvasPoint {
    /// Create a new CanvasPoint
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Convert from gpui::Point<f32>
    pub fn from_gpui(point: Point<f32>) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }

    /// Convert to gpui::Point<f32>
    pub fn to_gpui(&self) -> Point<f32> {
        Point::new(self.x, self.y)
    }
}

impl CanvasSize {
    /// Create a new CanvasSize
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Convert from gpui::Size<f32>
    pub fn from_gpui(size: Size<f32>) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }

    /// Convert to gpui::Size<f32>
    pub fn to_gpui(&self) -> Size<f32> {
        Size::new(self.width, self.height)
    }
}

impl CanvasRect {
    /// Create a new CanvasRect
    pub fn new(origin: CanvasPoint, size: CanvasSize) -> Self {
        Self { origin, size }
    }

    /// Convert from gpui::Bounds<f32>
    pub fn from_gpui_bounds(bounds: gpui::Bounds<f32>) -> Self {
        Self {
            origin: CanvasPoint::from_gpui(bounds.origin),
            size: CanvasSize::from_gpui(bounds.size),
        }
    }

    /// Convert to gpui::Bounds<f32>
    pub fn to_gpui_bounds(&self) -> gpui::Bounds<f32> {
        gpui::Bounds {
            origin: self.origin.to_gpui(),
            size: self.size.to_gpui(),
        }
    }

    /// Check if this rect contains a point
    pub fn contains(&self, point: &CanvasPoint) -> bool {
        point.x >= self.origin.x &&
        point.x <= self.origin.x + self.size.width &&
        point.y >= self.origin.y &&
        point.y <= self.origin.y + self.size.height
    }

    /// Check if this rect intersects with another rect
    pub fn intersects(&self, other: &CanvasRect) -> bool {
        // Check if one rectangle is to the left of the other
        if self.origin.x + self.size.width < other.origin.x || 
           other.origin.x + other.size.width < self.origin.x {
            return false;
        }

        // Check if one rectangle is above the other
        if self.origin.y + self.size.height < other.origin.y || 
           other.origin.y + other.size.height < self.origin.y {
            return false;
        }

        true
    }
}

// Implement math operations for CanvasPoint
impl Add for CanvasPoint {
    type Output = CanvasPoint;

    fn add(self, other: CanvasPoint) -> CanvasPoint {
        CanvasPoint {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for CanvasPoint {
    type Output = CanvasPoint;

    fn sub(self, other: CanvasPoint) -> CanvasPoint {
        CanvasPoint {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for CanvasPoint {
    type Output = CanvasPoint;

    fn mul(self, scalar: f32) -> CanvasPoint {
        CanvasPoint {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Div<f32> for CanvasPoint {
    type Output = CanvasPoint;

    fn div(self, scalar: f32) -> CanvasPoint {
        CanvasPoint {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::px;

    #[test]
    fn test_window_point_conversion() {
        let gpui_point = Point::new(px(10.0), px(20.0));
        let window_point = WindowPoint::from_gpui(gpui_point);
        
        assert_eq!(window_point.x, px(10.0));
        assert_eq!(window_point.y, px(20.0));
        
        let converted_back = window_point.to_gpui();
        assert_eq!(converted_back, gpui_point);
    }

    #[test]
    fn test_canvas_point_math() {
        let p1 = CanvasPoint::new(10.0, 20.0);
        let p2 = CanvasPoint::new(5.0, 8.0);
        
        let sum = p1 + p2;
        assert_eq!(sum, CanvasPoint::new(15.0, 28.0));
        
        let diff = p1 - p2;
        assert_eq!(diff, CanvasPoint::new(5.0, 12.0));
        
        let scaled = p1 * 2.0;
        assert_eq!(scaled, CanvasPoint::new(20.0, 40.0));
        
        let divided = p1 / 2.0;
        assert_eq!(divided, CanvasPoint::new(5.0, 10.0));
    }

    #[test]
    fn test_canvas_rect_contains() {
        let rect = CanvasRect::new(
            CanvasPoint::new(10.0, 10.0),
            CanvasSize::new(20.0, 20.0)
        );
        
        // Point inside
        let inside = CanvasPoint::new(15.0, 15.0);
        assert!(rect.contains(&inside));
        
        // Point on edge
        let on_edge = CanvasPoint::new(10.0, 15.0);
        assert!(rect.contains(&on_edge));
        
        // Point outside
        let outside = CanvasPoint::new(5.0, 5.0);
        assert!(!rect.contains(&outside));
    }

    #[test]
    fn test_canvas_rect_intersects() {
        let rect1 = CanvasRect::new(
            CanvasPoint::new(10.0, 10.0),
            CanvasSize::new(20.0, 20.0)
        );
        
        // Overlapping rect
        let rect2 = CanvasRect::new(
            CanvasPoint::new(20.0, 20.0),
            CanvasSize::new(20.0, 20.0)
        );
        assert!(rect1.intersects(&rect2));
        
        // Adjacent rect (touching)
        let rect3 = CanvasRect::new(
            CanvasPoint::new(30.0, 10.0),
            CanvasSize::new(20.0, 20.0)
        );
        assert!(rect1.intersects(&rect3));
        
        // Non-intersecting rect
        let rect4 = CanvasRect::new(
            CanvasPoint::new(50.0, 50.0),
            CanvasSize::new(20.0, 20.0)
        );
        assert!(!rect1.intersects(&rect4));
    }
}