#![allow(unused, dead_code)]
use gpui::{
    hsla, point, prelude::*, px, relative, solid_background, App, ContentMask, DispatchPhase,
    ElementId, ElementInputHandler, Entity, Focusable, Hitbox, Hsla, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Point, Style, TextStyle, TextStyleRefinement, Window,
};

use crate::{
    util::{round_to_pixel, rounded_point},
    GlobalState, Theme,
};

#[derive(Clone, Debug)]
pub struct ActiveDrag {
    pub start_position: Point<Pixels>,
    pub current_position: Point<Pixels>,
}
