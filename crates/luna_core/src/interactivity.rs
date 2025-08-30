use gpui::{Pixels, Point};

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
    pub start_position: Point<Pixels>,
    pub current_position: Point<Pixels>,
    /// The type of drag operation being performed
    pub drag_type: DragType,
}

impl ActiveDrag {
    /// Creates a new selection drag operation
    pub fn new_selection(start: Point<Pixels>) -> Self {
        Self {
            start_position: start,
            current_position: start,
            drag_type: DragType::Selection,
        }
    }

    /// Creates a new move elements drag operation
    pub fn new_move_elements(start: Point<Pixels>) -> Self {
        Self {
            start_position: start,
            current_position: start,
            drag_type: DragType::MoveElements,
        }
    }

    /// Creates a new create element drag operation
    pub fn new_create_element(start: Point<Pixels>) -> Self {
        Self {
            start_position: start,
            current_position: start,
            drag_type: DragType::CreateElement,
        }
    }

    /// Creates a new resize element drag operation
    pub fn new_resize(start: Point<Pixels>, resize_op: ResizeOperation) -> Self {
        Self {
            start_position: start,
            current_position: start,
            drag_type: DragType::Resize(resize_op),
        }
    }

    /// Gets the delta (change) between the current position and the start position
    pub fn delta(&self) -> Point<f32> {
        Point::new(
            self.current_position.x.0 - self.start_position.x.0,
            self.current_position.y.0 - self.start_position.y.0,
        )
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
}

/// Configuration for resize operations
#[derive(Debug, Clone, PartialEq)]
pub struct ResizeConfig {
    /// Whether to preserve aspect ratio during resize
    pub preserve_aspect_ratio: bool,
    /// Whether to resize from center instead of opposite corner
    pub resize_from_center: bool,
}

impl Default for ResizeConfig {
    fn default() -> Self {
        Self {
            preserve_aspect_ratio: false,
            resize_from_center: false,
        }
    }
}

/// Contains data for tracking a resize operation
#[derive(Debug, Clone, PartialEq)]
pub struct ResizeOperation {
    /// The handle being dragged
    pub handle: ResizeHandle,
    /// The original width before resize
    pub original_width: f32,
    /// The original height before resize
    pub original_height: f32,
    /// The original x position before resize
    pub original_x: f32,
    /// The original y position before resize
    pub original_y: f32,
    /// Configuration for the resize operation
    pub config: ResizeConfig,
}

impl ResizeOperation {
    /// Creates a new resize operation
    pub fn new(handle: ResizeHandle, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            handle,
            original_width: width,
            original_height: height,
            original_x: x,
            original_y: y,
            config: ResizeConfig::default(),
        }
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
}
