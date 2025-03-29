#![allow(unused_variables)]
use gpui::{Pixels, Point};

#[derive(Clone, Debug)]
pub struct ActiveDrag {
    pub start_position: Point<Pixels>,
    pub current_position: Point<Pixels>,
}
