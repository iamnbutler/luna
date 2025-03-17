#![allow(unused)]

//! The Canvas, the heart of Luna.

use serde::{ser::SerializeStruct, Deserialize};
use std::collections::HashMap;

use gpui::{div, prelude::*, px, Edges, Hsla, Pixels, Point, Rgba, Size};

pub struct LunaElementId(pub u32);

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LunaElement {
    size: Size<Pixels>,
    border_width: (Pixels, Pixels, Pixels, Pixels),
    border_color: Hsla,
    background_color: Hsla,
}

pub struct Canvas {
    initial_size: Size<Pixels>,
    elements: Vec<LunaElement>,
    element_positions: HashMap<LunaElementId, Point<Pixels>>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            initial_size: Size {
                width: px(512.),
                height: px(512.),
            },
            elements: Vec::new(),
            element_positions: HashMap::new(),
        }
    }
}

impl Render for Canvas {
    fn render(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .relative()
            .w(self.initial_size.width)
            .h(self.initial_size.height)
            .border_1()
            .border_color(gpui::red())
    }
}
