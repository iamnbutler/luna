#![allow(dead_code, unused)]

//! The Canvas, the heart of Luna.
use gpui::*;

use schemars_derive::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use uuid::Uuid;

use gpui::{div, impl_actions, px, Hsla, ParentElement, Pixels, Point, Size};

impl_actions!(element, [SelectElement]);

pub enum Event {
    ElementSelected { id: LunaElementId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct SelectElement {
    pub id: LunaElementId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct LunaElementId(usize);

impl LunaElementId {
    pub fn element_id(&self) -> ElementId {
        ElementId::Integer(self.0)
    }
}

impl Into<ElementId> for LunaElementId {
    fn into(self) -> ElementId {
        ElementId::Integer(self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SerializableElement {
    id: LunaElementId,
    name: String,
    style: ElementStyle,
}

impl SerializableElement {
    pub fn new(id: LunaElementId, style: ElementStyle) -> Self {
        Self {
            id,
            name: "Untitled".to_string(),
            style,
        }
    }

    pub fn position(&mut self, position: Point<Pixels>) -> &mut Self {
        self.style.position = Some(position);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LunaElement {
    id: LunaElementId,
    name: SharedString,
    style: ElementStyle,
    focus_handle: FocusHandle,
}

impl LunaElement {
    pub fn new(
        id: LunaElementId,
        name: Option<impl Into<SharedString>>,
        style: ElementStyle,
        cx: &mut App,
    ) -> Entity<Self> {
        let focus_handle = cx.focus_handle();
        cx.new(|cx| Self {
            id,
            name: name
                .map(Into::into)
                .unwrap_or_else(|| SharedString::from("Untitled")),
            style,
            focus_handle,
        })
    }

    pub fn handle_press(
        &mut self,
        e: &ClickEvent,
        _window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        println!("handle_press: Clicked on element with ID: {:?}", self.id);
        if e.down.button == MouseButton::Right {
            // just return for now
            return;
        } else {
            cx.dispatch_action(&SelectElement { id: self.id });
        }

        cx.notify();
    }
}

impl Render for LunaElement {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let style = self.style.clone();
        let id = self.id.clone();
        let position = self.style.position.expect("Canvas must have a position");

        div()
            .id(self.id.element_id())
            .on_click(cx.listener(move |this, e, window, cx| {
                println!("Clicked on element with ID: {:?}", id);
                this.handle_press(e, window, cx);
                cx.stop_propagation();
            }))
            .on_hover(cx.listener(move |this, e, window, cx| {
                println!("Hovered over element with ID: {:?}", id);
                cx.stop_propagation();
            }))
            .track_focus(&self.focus_handle.clone())
            .occlude()
            .absolute()
            .top(position.y)
            .left(position.x)
            .w(style.size.width)
            .h(style.size.height)
            .bg(style.background_color)
            .border(style.border_width)
            .border_color(style.border_color)
    }
}

impl Focusable for LunaElement {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<Event> for LunaElement {}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ElementStyle {
    size: Size<Pixels>,
    border_width: Pixels,
    border_color: Hsla,
    background_color: Hsla,
    position: Option<Point<Pixels>>,
}

impl ElementStyle {
    pub fn new(cx: &mut App) -> Self {
        Self {
            size: Size::new(px(48.), px(48.)),
            border_width: px(1.),
            border_color: rgb(0x3F434C).into(),
            background_color: rgb(0x292C32).into(),
            position: None,
        }
    }

    pub fn size(mut self, size: Size<Pixels>) -> Self {
        self.size = size;
        self
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

pub struct CanvasElements(HashMap<LunaElementId, SerializableElement>);

impl CanvasElements {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&LunaElementId, &SerializableElement)> {
        self.0.iter()
    }

    pub fn get(&self, id: &LunaElementId) -> Option<&SerializableElement> {
        self.0.get(id)
    }

    pub fn insert(&mut self, id: LunaElementId, element: SerializableElement) {
        self.0.insert(id, element);
    }
}

impl CanvasId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Into<ElementId> for CanvasId {
    fn into(self) -> ElementId {
        ElementId::Uuid(self.as_uuid())
    }
}

pub struct Canvas {
    id: CanvasId,
    element_positions: HashMap<LunaElementId, Point<Pixels>>,
    elements: CanvasElements,
    focus_handle: FocusHandle,
    initial_size: Size<Pixels>,
    next_id: usize,
    selected_ids: Vec<LunaElementId>,
    dragged_element: Option<(LunaElementId, Point<Pixels>)>,
}

impl Canvas {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self {
            id: CanvasId::new(),
            element_positions: HashMap::new(),
            elements: CanvasElements::new(),
            focus_handle: cx.focus_handle(),
            initial_size: Size {
                width: px(2000.),
                height: px(2000.),
            },
            next_id: 0,
            selected_ids: Vec::new(),
            dragged_element: None,
        })
    }

    fn select_element(&mut self, element: &SelectElement, _: &mut Window, cx: &mut Context<Self>) {
        self.selected_ids.push(element.id);
        cx.notify();
    }

    pub fn deselect_element(&mut self, id: LunaElementId) {
        self.selected_ids.retain(|&selected_id| selected_id != id);
    }

    pub fn deselect_all(&mut self) {
        self.selected_ids.clear();
    }

    pub fn add_element(
        &mut self,
        element: ElementStyle,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) -> LunaElementId {
        let id = LunaElementId(self.next_id);
        self.next_id += 1;

        let element = SerializableElement::new(id, element);

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

    fn handle_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let position = event.position;
        if let Some((id, _)) = self.element_at_position(position) {
            self.dragged_element = Some((id, position));
            cx.notify();
        }
    }

    fn handle_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some((id, start_pos)) = self.dragged_element {
            let delta = event.position - start_pos;
            if let Some(old_pos) = self.element_positions.get(&id) {
                let new_pos = *old_pos + delta;
                self.move_element(id, new_pos);
                self.dragged_element = Some((id, event.position));
                cx.notify();
            }
        }
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.dragged_element = None;
        cx.notify();
    }

    fn element_at_position(
        &self,
        position: Point<Pixels>,
    ) -> Option<(LunaElementId, &SerializableElement)> {
        self.elements.iter().find_map(|(id, element)| {
            let el_pos = self.element_positions.get(id).unwrap();
            let el_bounds = Bounds {
                origin: *el_pos,
                size: element.style.size,
            };
            if el_bounds.contains(&position) {
                Some((*id, element))
            } else {
                None
            }
        })
    }

    pub fn render_elements(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> Vec<Entity<LunaElement>> {
        let elements = self
            .elements
            .iter()
            .map(|(id, element)| {
                let position = self
                    .element_positions
                    .get(id)
                    .expect("Element position not found");
                let mut element = element.clone();
                element.position(position.clone());

                LunaElement::new(*id, element.name.into(), element.style.clone(), cx)
            })
            .collect();

        elements
    }
}

impl Render for Canvas {
    fn render(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let id: ElementId = self.id.clone().into();
        let focus_handle = self.focus_handle.clone();

        div()
            .id(id)
            .track_focus(&focus_handle)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_mouse_down))
            .on_mouse_move(cx.listener(Self::handle_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_mouse_up))
            .on_action(cx.listener(Self::select_element))
            .relative()
            .w(self.initial_size.width)
            .h(self.initial_size.height)
            .children(self.render_elements(window, cx))
    }
}

impl Focusable for Canvas {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

struct Luna {
    canvas: Entity<Canvas>,
}

impl Render for Luna {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .bg(rgb(0x1B1D22))
            .size_full()
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(self.canvas.clone())
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |window, cx| {
            let canvas = Canvas::new(window, cx);
            canvas.update(cx, |canvas, cx| {
                let element_1 = ElementStyle::new(cx).size(size(px(32.), px(128.)));
                let element_2 = ElementStyle::new(cx);
                let element_3 = ElementStyle::new(cx).size(size(px(64.), px(64.)));
                let element_4 = ElementStyle::new(cx).size(size(px(128.), px(128.)));

                canvas.add_element(element_1, point(px(0.), px(0.)), cx);
                canvas.add_element(element_2, point(px(300.), px(300.)), cx);
                canvas.add_element(element_3, point(px(600.), px(150.)), cx);
                canvas.add_element(element_4, point(px(240.), px(550.)), cx);
            });

            cx.new(|_cx| Luna { canvas })
        })
        .unwrap();
    });
}
