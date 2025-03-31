#![allow(unused, dead_code)]

use gpui::Pixels;
use gpui::Point;

/// The type of dragging operation being performed
#[derive(Clone, Debug, PartialEq)]
pub enum DragType {
    /// Dragging to create a selection box
    Selection,
    /// Dragging to move selected elements
    MoveElements,
    /// Dragging to create a new element
    CreateElement,
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
    
    /// Gets the delta (change) between the current position and the start position
    pub fn delta(&self) -> Point<f32> {
        Point::new(
            self.current_position.x.0 - self.start_position.x.0,
            self.current_position.y.0 - self.start_position.y.0,
        )
    }
}
