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

#[derive(Debug, Clone, PartialEq)]
pub struct LunaElement {
    id: LunaElementId,
    name: SharedString,
    style: ElementStyle,
    focus_handle: FocusHandle,
    selected: bool,
}

impl LunaElement {
    pub fn new(
        id: LunaElementId,
        name: Option<SharedString>,
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
            selected: false,
        })
    }

    pub fn selected(&mut self, selected: bool) -> &mut Self {
        self.selected = selected;
        self
    }
}

impl Render for LunaElement {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let style = self.style.clone();
        let id = self.id.clone();
        let position = self.style.position.expect("Canvas must have a position");

        div()
            .id(self.id.element_id())
            .track_focus(&self.focus_handle.clone())
            .absolute()
            .top(position.y)
            .left(position.x)
            .w(style.size.width)
            .h(style.size.height)
            .border_1()
            .border_color(if self.selected {
                rgb(0x0C8CE9).into()
            } else {
                gpui::transparent_black()
            })
            // todo: not when dragging an element
            .hover(|this| this.border_color(rgb(0x0C8CE9)))
            .child(
                div()
                    .size_full()
                    .bg(style.background_color)
                    .border(style.border_width)
                    .border_color(style.border_color),
            )
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
    elements: HashMap<LunaElementId, Entity<LunaElement>>,
    element_positions: HashMap<LunaElementId, Point<Pixels>>,
    focus_handle: FocusHandle,
    initial_size: Size<Pixels>,
    next_id: usize,
    selected_ids: Vec<LunaElementId>,
    dragged_element: Option<(LunaElementId, Point<Pixels>)>,
    canvas_offset: Point<Pixels>,
    is_dragging_canvas: bool,
    drag_start: Option<Point<Pixels>>,
}

impl Canvas {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self {
            id: CanvasId::new(),
            element_positions: HashMap::new(),
            elements: HashMap::new(),
            focus_handle: cx.focus_handle(),
            initial_size: Size {
                width: px(2000.),
                height: px(2000.),
            },
            next_id: 0,
            selected_ids: Vec::new(),
            dragged_element: None,
            canvas_offset: Point::default(),
            is_dragging_canvas: false,
            drag_start: None,
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
        style: ElementStyle,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) -> LunaElementId {
        let id = LunaElementId(self.next_id);
        self.next_id += 1;

        let mut style = style;
        style.position = Some(position);

        let element = LunaElement::new(id, None, style, cx);
        self.elements.insert(id, element);
        self.element_positions.insert(id, position);
        id
    }

    pub fn move_element(
        &mut self,
        id: LunaElementId,
        new_position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) -> bool {
        if let Some(element) = self.elements.get(&id) {
            element.update(cx, |element, _cx| {
                element.style.position = Some(new_position);
            });
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
        if let Some((id, _)) = self.element_at_position(position, cx) {
            self.dragged_element = Some((id, position));
        } else {
            self.is_dragging_canvas = true;
            self.drag_start = Some(position);
        }
        cx.notify();
    }

    fn handle_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some((id, start_pos)) = self.dragged_element {
            let delta = event.position - start_pos;
            if let Some(old_pos) = self.element_positions.get(&id) {
                let new_pos = self.clamp_element_position(*old_pos + delta, id, cx);
                self.move_element(id, new_pos, cx);
                self.dragged_element = Some((id, event.position));
            }
        } else if self.is_dragging_canvas {
            if let Some(start_pos) = self.drag_start {
                let delta = event.position - start_pos;
                self.canvas_offset = self.clamp_canvas_offset(self.canvas_offset + delta, window);
                self.drag_start = Some(event.position);
            }
        }
        cx.notify();
    }

    fn clamp_element_position(
        &self,
        pos: Point<Pixels>,
        id: LunaElementId,
        cx: &mut Context<Self>,
    ) -> Point<Pixels> {
        let element = self.elements.get(&id).unwrap();
        let element_size = element.read(cx).style.size;

        let max_x = self.initial_size.width - element_size.width;
        let max_y = self.initial_size.height - element_size.height;

        Point::new(pos.x.clamp(px(0.), max_x), pos.y.clamp(px(0.), max_y))
    }

    fn clamp_canvas_offset(&self, offset: Point<Pixels>, window: &Window) -> Point<Pixels> {
        let viewport_size = window.bounds();
        let max_x = (self.initial_size.width - viewport_size.size.width).max(px(0.));
        let max_y = (self.initial_size.height - viewport_size.size.height).max(px(0.));

        Point::new(
            offset.x.clamp(-max_x, px(0.)),
            offset.y.clamp(-max_y, px(0.)),
        )
    }

    fn find_element_by_id(&self, id: LunaElementId) -> Option<&Entity<LunaElement>> {
        self.elements.get(&id)
    }

    fn select_element_by_id(&mut self, id: LunaElementId, cx: &mut Context<Self>) {
        if let Some(element) = self.find_element_by_id(id) {
            if let Some(index) = self.selected_ids.iter().position(|&i| i == id) {
                self.selected_ids.remove(index);
            } else {
                element.update(cx, |element, cx| {
                    element.selected(true);
                });
                self.selected_ids.push(id);
            }
        }
    }

    fn handle_mouse_up(&mut self, event: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.dragged_element.is_none() {
            if let Some((id, _)) = self.element_at_position(event.position, cx) {
                self.select_element_by_id(id, cx);
            } else {
                self.deselect_all();
            }
        }
        self.dragged_element = None;
        self.is_dragging_canvas = false;
        self.drag_start = None;
        cx.notify();
    }

    fn element_at_position(
        &self,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) -> Option<(LunaElementId, &Entity<LunaElement>)> {
        let adjusted_position = position - self.canvas_offset;
        self.element_positions.iter().find_map(|(&id, &pos)| {
            if let Some(element) = self.elements.get(&id) {
                let el_bounds = Bounds {
                    origin: pos,
                    size: element.read(cx).style.size,
                };
                if el_bounds.contains(&adjusted_position) {
                    Some((id, element))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    pub fn render_elements(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> Vec<Entity<LunaElement>> {
        self.elements.values().cloned().collect()
    }
}

impl Render for Canvas {
    fn render(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let id: ElementId = self.id.clone().into();
        let focus_handle = self.focus_handle.clone();
        let clamped_offset = self.clamp_canvas_offset(self.canvas_offset, window);

        div()
            .id(id)
            .track_focus(&focus_handle)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_mouse_down))
            .on_mouse_move(cx.listener(Self::handle_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_mouse_up))
            .on_action(cx.listener(Self::select_element))
            .absolute()
            .w(self.initial_size.width)
            .h(self.initial_size.height)
            .left(clamped_offset.x)
            .top(clamped_offset.y)
            .bg(rgb(0x1B1D22))
            .children(self.render_elements(window, cx))
            .child(
                div()
                    .absolute()
                    .text_xs()
                    .text_color(gpui::red())
                    .top_2()
                    .left_2()
                    .child(format!("{:?}", self.selected_ids)),
            )
    }
}

impl Focusable for Canvas {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

struct Luna {
    titlebar: Entity<Titlebar>,
    canvas: Entity<Canvas>,
}

impl Render for Luna {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .relative()
            .bg(rgb(0x3B414D))
            .size_full()
            .text_color(rgb(0xffffff))
            .child(self.titlebar.clone())
            .child(div().size_full().flex_1().child(self.canvas.clone()))
    }
}

struct Titlebar {
    title: SharedString,
}

impl Titlebar {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let title = "Untitled".into();
        Titlebar { title }
    }
}

impl Render for Titlebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .w_full()
            .h(px(28.))
            .border_b_1()
            .border_color(rgb(0x3F434C))
            .bg(rgb(0x2A2C31))
            .text_xs()
            .text_color(rgb(0xA9AFBC))
            .font_family("Berkeley Mono")
            .child(div().flex().items_center().h_full().px_2().child("Luna"))
        // .child(
        //     div()
        //         .flex()
        //         .flex_1()
        //         .items_center()
        //         .h_full()
        //         .w_full()
        //         .px_2()
        //         .text_center()
        //         .child(self.title.clone()),
        // )
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

            let titlebar = cx.new(|cx| Titlebar::new(window, cx));

            cx.new(|_cx| Luna { titlebar, canvas })
        })
        .unwrap();

        cx.activate(true)
    });
}
