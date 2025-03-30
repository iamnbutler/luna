#![allow(unused, dead_code)]

use gpui::Pixels;
use gpui::Point;

/// Represents a drag operation in progress with start and current points
#[derive(Clone, Debug)]
pub struct ActiveDrag {
    pub start_position: Point<Pixels>,
    pub current_position: Point<Pixels>,
}
