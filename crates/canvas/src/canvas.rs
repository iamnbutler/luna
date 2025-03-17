#![allow(unused)]

//! The Canvas, the heart of Luna.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use uuid::Uuid;

use gpui::{div, prelude::*, px, Hsla, ParentElement, Pixels, Point, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LunaElementId(pub u32);

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LunaElement {
    size: Size<Pixels>,
    border_width: Pixels,
    border_color: Hsla,
    background_color: Hsla,
}

impl Default for LunaElement {
    fn default() -> Self {
        Self {
            size: Size::new(px(48.), px(48.)),
            border_width: px(0.),
            border_color: gpui::transparent_black(),
            background_color: gpui::red(),
        }
    }
}

impl Render for LunaElement {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut Context<'_, Self>,
    ) -> impl IntoElement {
        div()
            .absolute()
            .w(self.size.width)
            .h(self.size.height)
            .bg(self.background_color)
            .border(self.border_width)
            .border_color(self.background_color)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CanvasId(Uuid);

impl Default for CanvasId {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<CanvasId> for Uuid {
    fn from(id: CanvasId) -> Self {
        id.0
    }
}

pub struct CanvasElements(HashMap<LunaElementId, LunaElement>);

impl CanvasElements {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&LunaElementId, &LunaElement)> {
        self.0.iter()
    }

    pub fn get(&self, id: &LunaElementId) -> Option<&LunaElement> {
        self.0.get(id)
    }

    pub fn insert(&mut self, id: LunaElementId, element: LunaElement) {
        self.0.insert(id, element);
    }
}

pub struct Canvas {
    initial_size: Size<Pixels>,
    elements: CanvasElements,
    element_positions: HashMap<LunaElementId, Point<Pixels>>,
    next_id: u32,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            initial_size: Size {
                width: px(512.),
                height: px(512.),
            },
            elements: CanvasElements::new(),
            element_positions: HashMap::new(),
            next_id: 0,
        }
    }
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            initial_size: Size {
                width: px(512.),
                height: px(512.),
            },
            elements: CanvasElements::new(),
            element_positions: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_element(&mut self, element: LunaElement, position: Point<Pixels>) -> LunaElementId {
        let id = LunaElementId(self.next_id);
        self.next_id += 1;

        self.elements.insert(id, element);
        self.element_positions.insert(id, position);
        id
    }

    pub fn move_element(&mut self, id: LunaElementId, new_position: Point<Pixels>) -> bool {
        if self.elements.get(&id).is_some() {
            self.element_positions.insert(id, new_position);
            true
        } else {
            false
        }
    }
}

impl Render for Canvas {
    fn render(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let elements = self.elements.iter().map(|(id, element)| {
            let position = self
                .element_positions
                .get(id)
                .expect("failed to unwrap element position");

            div()
                .absolute()
                .left(position.x)
                .top(position.y)
                .w(element.size.width)
                .h(element.size.height)
                .bg(element.background_color)
        });

        div()
            .relative()
            .w(self.initial_size.width)
            .h(self.initial_size.height)
            .children(elements)
    }
}
