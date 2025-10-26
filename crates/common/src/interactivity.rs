use glam::Vec2;
use gpui::{px, Pixels, Point};

use crate::util::PixelsExt as _;

/// Position in drag coordinates (screen pixels)
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct DragPosition(Vec2);

impl DragPosition {
    /// Create a new drag position
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    /// Create from a Vec2
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

    /// Convert to a GPUI Point with Pixels
    pub fn to_point_pixels(&self) -> Point<Pixels> {
        Point::new(px(self.0.x), px(self.0.y))
    }

    /// Create from a GPUI Point with Pixels
    pub fn from_point_pixels(point: Point<Pixels>) -> Self {
        Self(Vec2::new(point.x.f32(), point.y.f32()))
    }

    /// Convert to a regular Point<f32>
    pub fn to_point(&self) -> Point<f32> {
        Point::new(self.0.x, self.0.y)
    }

    /// Create from a regular Point<f32>
    pub fn from_point(point: Point<f32>) -> Self {
        Self(Vec2::new(point.x, point.y))
    }

    /// Calculate distance to another position
    pub fn distance(&self, other: DragPosition) -> f32 {
        self.0.distance(other.0)
    }

    /// Calculate the delta (difference) to another position
    pub fn delta_to(&self, other: DragPosition) -> Vec2 {
        other.0 - self.0
    }
}

/// The type of dragging operation being performed
#[derive(Clone, Debug, PartialEq)]
pub enum DragType {
    /// Dragging to create a selection box
    Selection,
    /// Dragging to move selected elements
    MoveElements,
    /// Dragging to create a new element
    CreateElement,
    /// Dragging to resize an element
    Resize(ResizeOperation),
}

/// Represents a drag operation in progress with start and current points
#[derive(Clone, Debug)]
pub struct ActiveDrag {
    pub start_position: DragPosition,
    pub current_position: DragPosition,
    /// The type of drag operation being performed
    pub drag_type: DragType,
    /// Track the last position where we checked for a potential parent frame
    /// Used to throttle expensive hit testing during drags
    pub last_parent_check_position: Option<gpui::Point<f32>>,
}

impl ActiveDrag {
    /// Creates a new selection drag operation
    pub fn new_selection(start: Point<Pixels>) -> Self {
        let position = DragPosition::from_point_pixels(start);
        Self {
            start_position: position,
            current_position: position,
            drag_type: DragType::Selection,
            last_parent_check_position: None,
        }
    }

    /// Creates a new move elements drag operation
    pub fn new_move(start: Point<Pixels>) -> Self {
        let position = DragPosition::from_point_pixels(start);
        Self {
            start_position: position,
            current_position: position,
            drag_type: DragType::MoveElements,
            last_parent_check_position: None,
        }
    }

    /// Creates a new create element drag operation
    pub fn new_create(start: Point<Pixels>) -> Self {
        let position = DragPosition::from_point_pixels(start);
        Self {
            start_position: position,
            current_position: position,
            drag_type: DragType::CreateElement,
            last_parent_check_position: None,
        }
    }

    /// Creates a new resize element drag operation
    pub fn new_resize(start: Point<Pixels>, operation: ResizeOperation) -> Self {
        let position = DragPosition::from_point_pixels(start);
        Self {
            start_position: position,
            current_position: position,
            drag_type: DragType::Resize(operation),
            last_parent_check_position: None,
        }
    }

    /// Updates the current position from a Point<Pixels>
    pub fn update_position(&mut self, position: Point<Pixels>) {
        self.current_position = DragPosition::from_point_pixels(position);
    }

    /// Gets the delta (change) between the current position and the start position
    pub fn delta(&self) -> Point<f32> {
        let delta_vec = self.start_position.delta_to(self.current_position);
        Point::new(delta_vec.x, delta_vec.y)
    }

    /// Gets the delta as a Vec2
    pub fn delta_vec2(&self) -> Vec2 {
        self.start_position.delta_to(self.current_position)
    }

    /// Gets the distance dragged from the start position
    pub fn distance(&self) -> f32 {
        self.start_position.distance(self.current_position)
    }

    /// Checks if the drag has moved beyond a threshold
    pub fn has_moved(&self, threshold: f32) -> bool {
        self.distance() > threshold
    }
}

/// Represents a resize handle position on a node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl ResizeHandle {
    /// Returns the opposite corner handle
    pub fn opposite(&self) -> Self {
        match self {
            ResizeHandle::TopLeft => ResizeHandle::BottomRight,
            ResizeHandle::TopRight => ResizeHandle::BottomLeft,
            ResizeHandle::BottomLeft => ResizeHandle::TopRight,
            ResizeHandle::BottomRight => ResizeHandle::TopLeft,
        }
    }

    /// Returns true if this handle is on the left side
    pub fn is_left(&self) -> bool {
        matches!(self, ResizeHandle::TopLeft | ResizeHandle::BottomLeft)
    }

    /// Returns true if this handle is on the right side
    pub fn is_right(&self) -> bool {
        matches!(self, ResizeHandle::TopRight | ResizeHandle::BottomRight)
    }

    /// Returns true if this handle is on the top side
    pub fn is_top(&self) -> bool {
        matches!(self, ResizeHandle::TopLeft | ResizeHandle::TopRight)
    }

    /// Returns true if this handle is on the bottom side
    pub fn is_bottom(&self) -> bool {
        matches!(self, ResizeHandle::BottomLeft | ResizeHandle::BottomRight)
    }

    /// Get the scale factor for this handle when resizing
    /// Returns (x_scale, y_scale) where -1 means invert, 0 means no change, 1 means normal
    pub fn scale_factor(&self) -> (f32, f32) {
        match self {
            ResizeHandle::TopLeft => (-1.0, -1.0),
            ResizeHandle::TopRight => (1.0, -1.0),
            ResizeHandle::BottomLeft => (-1.0, 1.0),
            ResizeHandle::BottomRight => (1.0, 1.0),
        }
    }
}

/// Configuration for resize operations
#[derive(Debug, Clone, PartialEq)]
pub struct ResizeConfig {
    /// Whether to preserve aspect ratio during resize
    pub preserve_aspect_ratio: bool,
    /// Whether to resize from center instead of opposite corner
    pub resize_from_center: bool,
    /// Minimum width allowed
    pub min_width: Option<f32>,
    /// Minimum height allowed
    pub min_height: Option<f32>,
    /// Maximum width allowed
    pub max_width: Option<f32>,
    /// Maximum height allowed
    pub max_height: Option<f32>,
}

impl Default for ResizeConfig {
    fn default() -> Self {
        Self {
            preserve_aspect_ratio: false,
            resize_from_center: false,
            min_width: Some(1.0),
            min_height: Some(1.0),
            max_width: None,
            max_height: None,
        }
    }
}

/// Contains data for tracking a resize operation
#[derive(Debug, Clone, PartialEq)]
pub struct ResizeOperation {
    /// The handle being dragged
    pub handle: ResizeHandle,
    /// The original bounds stored as Vec2s for position and size
    original_bounds: (Vec2, Vec2), // (position, size)
    /// Configuration for the resize operation
    pub config: ResizeConfig,
}

impl ResizeOperation {
    /// Creates a new resize operation
    pub fn new(handle: ResizeHandle, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            handle,
            original_bounds: (Vec2::new(x, y), Vec2::new(width, height)),
            config: ResizeConfig::default(),
        }
    }

    /// Gets the original position
    pub fn original_position(&self) -> Vec2 {
        self.original_bounds.0
    }

    /// Gets the original size
    pub fn original_size(&self) -> Vec2 {
        self.original_bounds.1
    }

    /// Gets the original x position
    pub fn original_x(&self) -> f32 {
        self.original_bounds.0.x
    }

    /// Gets the original y position
    pub fn original_y(&self) -> f32 {
        self.original_bounds.0.y
    }

    /// Gets the original width
    pub fn original_width(&self) -> f32 {
        self.original_bounds.1.x
    }

    /// Gets the original height
    pub fn original_height(&self) -> f32 {
        self.original_bounds.1.y
    }

    /// Sets whether to preserve aspect ratio
    pub fn with_preserve_aspect_ratio(mut self, preserve: bool) -> Self {
        self.config.preserve_aspect_ratio = preserve;
        self
    }

    /// Sets whether to resize from center
    pub fn with_resize_from_center(mut self, from_center: bool) -> Self {
        self.config.resize_from_center = from_center;
        self
    }

    /// Sets the minimum size constraints
    pub fn with_min_size(mut self, min_width: Option<f32>, min_height: Option<f32>) -> Self {
        self.config.min_width = min_width;
        self.config.min_height = min_height;
        self
    }

    /// Sets the maximum size constraints
    pub fn with_max_size(mut self, max_width: Option<f32>, max_height: Option<f32>) -> Self {
        self.config.max_width = max_width;
        self.config.max_height = max_height;
        self
    }

    /// Calculate new bounds based on a drag delta
    pub fn calculate_new_bounds(&self, delta: Vec2) -> (Vec2, Vec2) {
        let (scale_x, scale_y) = self.handle.scale_factor();
        let scaled_delta = Vec2::new(delta.x * scale_x, delta.y * scale_y);

        let mut new_size = self.original_bounds.1 + scaled_delta;
        let mut new_position = self.original_bounds.0;

        // Apply size constraints
        if let Some(min_w) = self.config.min_width {
            new_size.x = new_size.x.max(min_w);
        }
        if let Some(min_h) = self.config.min_height {
            new_size.y = new_size.y.max(min_h);
        }
        if let Some(max_w) = self.config.max_width {
            new_size.x = new_size.x.min(max_w);
        }
        if let Some(max_h) = self.config.max_height {
            new_size.y = new_size.y.min(max_h);
        }

        // Preserve aspect ratio if needed
        if self.config.preserve_aspect_ratio {
            let original_aspect = self.original_bounds.1.x / self.original_bounds.1.y;
            let new_aspect = new_size.x / new_size.y;

            if new_aspect > original_aspect {
                new_size.x = new_size.y * original_aspect;
            } else {
                new_size.y = new_size.x / original_aspect;
            }
        }

        // Adjust position based on resize handle
        if self.config.resize_from_center {
            let size_delta = new_size - self.original_bounds.1;
            new_position = self.original_bounds.0 - size_delta * 0.5;
        } else {
            // Adjust position for handles that affect the origin
            if self.handle.is_left() {
                new_position.x = self.original_bounds.0.x + self.original_bounds.1.x - new_size.x;
            }
            if self.handle.is_top() {
                new_position.y = self.original_bounds.0.y + self.original_bounds.1.y - new_size.y;
            }
        }

        (new_position, new_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drag_position() {
        let pos1 = DragPosition::new(10.0, 20.0);
        let pos2 = DragPosition::new(15.0, 25.0);

        assert_eq!(pos1.x(), 10.0);
        assert_eq!(pos1.y(), 20.0);

        let delta = pos1.delta_to(pos2);
        assert_eq!(delta.x, 5.0);
        assert_eq!(delta.y, 5.0);

        let distance = pos1.distance(pos2);
        assert!((distance - 7.071).abs() < 0.01); // sqrt(25 + 25)
    }

    #[test]
    fn test_active_drag() {
        let start = Point::new(px(10.0), px(20.0));
        let mut drag = ActiveDrag::new_selection(start);

        assert_eq!(drag.start_position.x(), 10.0);
        assert_eq!(drag.start_position.y(), 20.0);

        drag.update_position(Point::new(px(15.0), px(30.0)));

        let delta = drag.delta();
        assert_eq!(delta.x, 5.0);
        assert_eq!(delta.y, 10.0);

        assert!(drag.has_moved(5.0));
        assert!(!drag.has_moved(15.0));
    }

    #[test]
    fn test_resize_operation() {
        let resize = ResizeOperation::new(ResizeHandle::BottomRight, 10.0, 10.0, 100.0, 100.0);

        let (new_pos, new_size) = resize.calculate_new_bounds(Vec2::new(20.0, 30.0));

        assert_eq!(new_pos.x, 10.0);
        assert_eq!(new_pos.y, 10.0);
        assert_eq!(new_size.x, 120.0);
        assert_eq!(new_size.y, 130.0);
    }

    #[test]
    fn test_resize_with_constraints() {
        let resize = ResizeOperation::new(ResizeHandle::BottomRight, 10.0, 10.0, 100.0, 100.0)
            .with_min_size(Some(50.0), Some(50.0))
            .with_max_size(Some(150.0), Some(150.0));

        // Test min constraint
        let (_, new_size) = resize.calculate_new_bounds(Vec2::new(-60.0, -60.0));
        assert_eq!(new_size.x, 50.0);
        assert_eq!(new_size.y, 50.0);

        // Test max constraint
        let (_, new_size) = resize.calculate_new_bounds(Vec2::new(100.0, 100.0));
        assert_eq!(new_size.x, 150.0);
        assert_eq!(new_size.y, 150.0);
    }
}
