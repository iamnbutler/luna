#![allow(unused, dead_code)]
use crate::canvas_element::CanvasElement;
use crate::{canvas::LunaCanvas, theme::Theme};
use gpui::{
    actions, div, hsla, point, prelude::*, px, svg, App, Application, AssetSource, BoxShadow,
    ElementId, Entity, FocusHandle, Focusable, Global, Hsla, IntoElement, Keystroke, Menu,
    MenuItem, Modifiers, Pixels, Point, SharedString, TitlebarOptions, UpdateGlobal, WeakEntity,
    Window, WindowBackgroundAppearance, WindowOptions,
};
use std::{fs, path::PathBuf};
use strum::Display;

pub mod inspector;
pub mod layer_list;
mod property;
pub mod sidebar;

const TITLEBAR_HEIGHT: f32 = 31.;

#[derive(Debug, Display, Clone, PartialEq)]
pub enum Icon {
    ArrowCounterClockwise,
    ArrowDownRight,
    Frame,
    Image,
    Path,
    Square,
    Text,
}

impl Icon {
    pub fn src(self) -> SharedString {
        match self {
            Icon::ArrowCounterClockwise => "svg/arrow_counter_clockwise.svg".into(),
            Icon::ArrowDownRight => "svg/arrow_down_right.svg".into(),
            Icon::Frame => "svg/frame.svg".into(),
            Icon::Image => "svg/image.svg".into(),
            Icon::Path => "svg/pen_tool.svg".into(),
            Icon::Square => "svg/square.svg".into(),
            Icon::Text => "svg/text_cursor.svg".into(),
        }
    }
}
