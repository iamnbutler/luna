#![allow(dead_code, unused)]

//! The Canvas, the heart of Luna.
use gpui::{prelude::FluentBuilder as _, *};

use schemars_derive::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use uuid::Uuid;

use gpui::{div, impl_actions, px, Hsla, ParentElement, Pixels, Point, Size};

const EDGE_HITBOX_PADDING: f32 = 12.0;
const CORNER_HANDLE_SIZE: f32 = 7.0;

const THEME_SELECTED: Hsla = Hsla {
    h: 205.0 / 360.0,
    s: 0.9,
    l: 0.48,
    a: 1.0,
};

// TODO: Go update gpui::Corner to derive display/EnumString
/// Identifies a corner of a 2d box.
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum Corner {
    /// The top left corner
    TopLeft,
    /// The top right corner
    TopRight,
    /// The bottom left corner
    BottomLeft,
    /// The bottom right corner
    BottomRight,
}

impl std::fmt::Display for Corner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Corner::TopLeft => write!(f, "TopLeft"),
            Corner::TopRight => write!(f, "TopRight"),
            Corner::BottomLeft => write!(f, "BottomLeft"),
            Corner::BottomRight => write!(f, "BottomRight"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum ResizeDirection {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl std::fmt::Display for ResizeDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResizeDirection::Left => write!(f, "Left"),
            ResizeDirection::Right => write!(f, "Right"),
            ResizeDirection::Top => write!(f, "Up"),
            ResizeDirection::Bottom => write!(f, "Down"),
            ResizeDirection::TopLeft => write!(f, "TopLeft"),
            ResizeDirection::TopRight => write!(f, "TopRight"),
            ResizeDirection::BottomLeft => write!(f, "BottomLeft"),
            ResizeDirection::BottomRight => write!(f, "BottomRight"),
        }
    }
}

impl ResizeDirection {
    pub fn is_edge(&self) -> bool {
        match self {
            ResizeDirection::Left
            | ResizeDirection::Right
            | ResizeDirection::Top
            | ResizeDirection::Bottom => true,
            _ => false,
        }
    }

    pub fn is_corner(&self) -> bool {
        match self {
            ResizeDirection::TopLeft
            | ResizeDirection::TopRight
            | ResizeDirection::BottomLeft
            | ResizeDirection::BottomRight => true,
            _ => false,
        }
    }

    pub fn cursor(&self) -> CursorStyle {
        match self {
            ResizeDirection::Left => CursorStyle::ResizeLeft,
            ResizeDirection::Right => CursorStyle::ResizeRight,
            ResizeDirection::Top => CursorStyle::ResizeUp,
            ResizeDirection::Bottom => CursorStyle::ResizeDown,
            ResizeDirection::TopLeft => CursorStyle::ResizeUpRightDownLeft,
            ResizeDirection::TopRight => CursorStyle::ResizeUpLeftDownRight,
            ResizeDirection::BottomLeft => CursorStyle::ResizeUpLeftDownRight,
            ResizeDirection::BottomRight => CursorStyle::ResizeUpRightDownLeft,
        }
    }
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
    canvas: WeakEntity<Canvas>,
}

impl LunaElement {
    pub fn new(
        id: LunaElementId,
        name: Option<SharedString>,
        style: ElementStyle,
        canvas: WeakEntity<Canvas>,
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
            canvas,
        })
    }

    pub fn selected(&self, cx: &mut Context<Self>) -> bool {
        self.canvas
            .upgrade()
            .map(|canvas| canvas.read(cx).selected_ids.contains(&self.id))
            .unwrap_or(false)
    }
}

impl Render for LunaElement {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let style = self.style.clone();
        let id = self.id.clone();
        let position = self.style.position.expect("Canvas must have a position");
        let dragging = if let Some(canvas) = self.canvas.upgrade() {
            canvas.read(cx).dragging.is_some()
        } else {
            false
        };
        let corner_handle_offset = px(CORNER_HANDLE_SIZE / 2.0 - 1.0);

        div()
            .id(self.id.element_id())
            .track_focus(&self.focus_handle.clone())
            .absolute()
            .top(position.y)
            .left(position.x)
            .w(style.size.width)
            .h(style.size.height)
            .border_1()
            .border_color(gpui::transparent_black())
            .hover(|this| {
                if !dragging {
                    this.border_color(rgb(0x0C8CE9))
                } else {
                    this
                }
            })
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

struct SelectionContainer {
    bounds: Bounds<Pixels>,
}

impl SelectionContainer {
    fn new(bounds: Bounds<Pixels>) -> Self {
        Self { bounds }
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
    dragging: Option<Point<Pixels>>,
    canvas_offset: Point<Pixels>,
    is_dragging_canvas: bool,
    drag_start: Option<Point<Pixels>>,
    current_resize_direction: Option<ResizeDirection>,
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
            dragging: None,
            canvas_offset: Point::default(),
            is_dragging_canvas: false,
            drag_start: None,
            current_resize_direction: None,
        })
    }

    pub fn select_element(&mut self, id: LunaElementId, cx: &mut Context<Self>) {
        self.selected_ids.push(id);
        cx.notify();
    }

    pub fn deselect_element(&mut self, id: LunaElementId, cx: &mut Context<Self>) {
        self.selected_ids.retain(|&selected_id| selected_id != id);
        cx.notify();
    }

    pub fn clear_selection(&mut self, cx: &mut Context<Self>) {
        self.selected_ids.clear();
        cx.notify();
    }

    pub fn add_element(
        &mut self,
        style: ElementStyle,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) -> LunaElementId {
        let weak_self = cx.weak_entity();
        let id = LunaElementId(self.next_id);
        self.next_id += 1;

        let mut style = style;
        style.position = Some(position);

        let element = LunaElement::new(id, None, style, weak_self, cx);
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
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let current_modifiers = window.modifiers();
        let position = event.position;
        let element_at_position = self.element_at_position(position, cx);

        match (event.button, current_modifiers) {
            (MouseButton::Left, modifiers) if modifiers.alt => {
                if element_at_position.is_none() {
                    self.is_dragging_canvas = true;
                    self.drag_start = Some(position);
                }
                cx.notify();
            }
            (MouseButton::Left, modifiers) => {
                if let Some(element) = element_at_position {
                    let element_id = element.0;
                    if modifiers.shift {
                        // Toggle selection when shift is pressed
                        if self.selected_ids.contains(&element_id) {
                            self.deselect_element(element_id, cx);
                        } else {
                            self.select_element(element_id, cx);
                        }
                    } else {
                        // Clear selection only if clicking on an unselected element
                        if !self.selected_ids.contains(&element_id) {
                            self.clear_selection(cx);
                            self.select_element(element_id, cx);
                        }
                    }
                    self.dragging = Some(position);
                } else {
                    self.clear_selection(cx);
                }
                cx.notify();
            }
            _ => {}
        }
    }

    fn handle_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(left_button) = event.pressed_button {
            if let Some(start_pos) = self.dragging {
                let delta = event.position - start_pos;
                self.move_selected_elements(delta, cx);
                self.dragging = Some(event.position);
            } else if self.is_dragging_canvas {
                if let Some(start_pos) = self.drag_start {
                    let delta = event.position - start_pos;
                    self.canvas_offset =
                        self.clamp_canvas_offset(self.canvas_offset + delta, window);
                    self.drag_start = Some(event.position);
                }
            }
            cx.notify();
        }

        if !self.selected_ids.is_empty() {
            if let Some(bounds) = self.get_selection_bounds(cx) {
                let adjusted_position = event.position - self.canvas_offset;
                self.current_resize_direction = None;

                // ensure that the bounds match doesn't drop off the pixel you
                // move your mouse beyond the selection's edge
                let wiggle_room = px(4.0);
                let expanded_bounds = Bounds {
                    origin: Point::new(
                        bounds.origin.x - wiggle_room,
                        bounds.origin.y - wiggle_room,
                    ),
                    size: Size::new(
                        bounds.size.width + wiggle_room * 2.0,
                        bounds.size.height + wiggle_room * 2.0,
                    ),
                };

                if expanded_bounds.contains(&adjusted_position) {
                    let edge_hitbox = px(EDGE_HITBOX_PADDING);
                    let left_hitbox = Bounds {
                        origin: expanded_bounds.origin,
                        size: Size::new(edge_hitbox, expanded_bounds.size.height),
                    };
                    let right_hitbox = Bounds {
                        origin: Point::new(
                            expanded_bounds.origin.x + expanded_bounds.size.width - edge_hitbox,
                            expanded_bounds.origin.y,
                        ),
                        size: Size::new(edge_hitbox, expanded_bounds.size.height),
                    };
                    let top_hitbox = Bounds {
                        origin: expanded_bounds.origin,
                        size: Size::new(expanded_bounds.size.width, edge_hitbox),
                    };
                    let bottom_hitbox = Bounds {
                        origin: Point::new(
                            expanded_bounds.origin.x,
                            expanded_bounds.origin.y + expanded_bounds.size.height - edge_hitbox,
                        ),
                        size: Size::new(expanded_bounds.size.width, edge_hitbox),
                    };

                    if left_hitbox.contains(&adjusted_position) {
                        self.current_resize_direction = Some(ResizeDirection::Left);
                    } else if right_hitbox.contains(&adjusted_position) {
                        self.current_resize_direction = Some(ResizeDirection::Right);
                    } else if top_hitbox.contains(&adjusted_position) {
                        self.current_resize_direction = Some(ResizeDirection::Top);
                    } else if bottom_hitbox.contains(&adjusted_position) {
                        self.current_resize_direction = Some(ResizeDirection::Bottom);
                    }
                }
                cx.notify();
            }
        }
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

    fn handle_mouse_up(&mut self, event: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.dragging = None;
        self.is_dragging_canvas = false;
        self.drag_start = None;
        cx.notify();
    }

    fn move_selected_elements(&mut self, delta: Point<Pixels>, cx: &mut Context<Self>) {
        let selected_ids = self.selected_ids.clone();

        for &id in &selected_ids {
            if let Some(old_pos) = self.element_positions.get(&id) {
                let new_pos = self.clamp_element_position(*old_pos + delta, id, cx);
                self.move_element(id, new_pos, cx);
            }
        }
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

    fn render_resize_control(&self, corner: Corner) -> Stateful<Div> {
        let id = ElementId::Name(format!("resize-control-{}", corner).into());
        let corner_handle_offset = px(CORNER_HANDLE_SIZE / 2.0 - 1.0);

        let mut div = div()
            .absolute()
            .id(id)
            .size(px(CORNER_HANDLE_SIZE))
            .border_1()
            .border_color(THEME_SELECTED)
            .bg(gpui::white());

        match corner {
            Corner::TopLeft => {
                div = div.top(-corner_handle_offset).left(-corner_handle_offset);
            }
            Corner::TopRight => {
                div = div.top(-corner_handle_offset).right(-corner_handle_offset);
            }
            Corner::BottomLeft => {
                div = div
                    .bottom(-corner_handle_offset)
                    .left(-corner_handle_offset);
            }
            Corner::BottomRight => {
                div = div
                    .bottom(-corner_handle_offset)
                    .right(-corner_handle_offset);
            }
        }

        div
    }

    fn get_selection_bounds(&self, cx: &Context<Self>) -> Option<Bounds<Pixels>> {
        if self.selected_ids.is_empty() {
            return None;
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for &id in &self.selected_ids {
            if let Some(element) = self.elements.get(&id) {
                let style = &element.read(cx).style;
                let position = style.position.expect("Element must have a position");
                let size = style.size;

                min_x = min_x.min(position.x.0);
                min_y = min_y.min(position.y.0);
                max_x = max_x.max(position.x.0 + size.width.0);
                max_y = max_y.max(position.y.0 + size.height.0);
            }
        }

        Some(Bounds {
            origin: Point::new(px(min_x), px(min_y)),
            size: Size::new(px(max_x - min_x), px(max_y - min_y)),
        })
    }

    fn render_resize_edge_control(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        if let Some(direction) = self.current_resize_direction {
            if !direction.is_edge() {
                return None;
            }

            let is_horizontal = matches!(direction, ResizeDirection::Left | ResizeDirection::Right);

            let mut el = div()
                .id("resize-edge-control")
                .flex()
                .flex_none()
                .absolute();
            // need to do this in the expanded hitbox
            // .cursor(direction.cursor());

            if is_horizontal {
                el = el.h(px(EDGE_HITBOX_PADDING)).top_0().bottom_0().h_full();
            } else {
                el = el.w(px(EDGE_HITBOX_PADDING)).left_0().right_0().w_full();
            }

            el = match direction {
                ResizeDirection::Left => el.border_l_1().border_color(THEME_SELECTED).left_0(),
                ResizeDirection::Right => el.border_r_1().border_color(THEME_SELECTED).right_0(),
                ResizeDirection::Top => el.border_t_1().border_color(THEME_SELECTED).top_0(),
                ResizeDirection::Bottom => el.border_b_1().border_color(THEME_SELECTED).bottom_0(),
                _ => el,
            };

            Some(el)
        } else {
            None
        }
    }

    fn render_selection_container(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        self.get_selection_bounds(cx).map(|bounds| {
            let container = SelectionContainer::new(bounds);
            let empty_selection = self.selected_ids.is_empty();
            let current_resize_direction = self.current_resize_direction;

            div()
                .id("selection-container")
                .absolute()
                .left(container.bounds.origin.x - self.canvas_offset.x)
                .top(container.bounds.origin.y - self.canvas_offset.y)
                .w(container.bounds.size.width)
                .h(container.bounds.size.height)
                .child(
                    div()
                        .id("selection-container-border")
                        .absolute()
                        // .occlude()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .w(container.bounds.size.width)
                        .h(container.bounds.size.height)
                        .border_1()
                        .border_color(if current_resize_direction.is_some() {
                            THEME_SELECTED.alpha(0.4)
                        } else {
                            THEME_SELECTED
                        })
                        .when(self.selected_ids.is_empty(), |this| {
                            this.bg(THEME_SELECTED.alpha(0.12))
                        }),
                )
                .child(
                    self.render_resize_control(Corner::TopLeft)
                        .cursor(CursorStyle::ResizeUpLeftDownRight),
                )
                .child(
                    self.render_resize_control(Corner::TopRight)
                        .cursor(CursorStyle::ResizeUpRightDownLeft),
                )
                .child(
                    self.render_resize_control(Corner::BottomLeft)
                        .cursor(CursorStyle::ResizeUpRightDownLeft),
                )
                .child(
                    self.render_resize_control(Corner::BottomRight)
                        .cursor(CursorStyle::ResizeUpLeftDownRight),
                )
                .children(self.render_resize_edge_control(cx))
        })
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
            .absolute()
            .w(self.initial_size.width)
            .h(self.initial_size.height)
            .left(clamped_offset.x)
            .top(clamped_offset.y)
            .bg(rgb(0x1B1D22))
            .children(self.render_elements(window, cx))
            .when_some(self.render_selection_container(cx), |this, container| {
                this.child(container)
            })
            .child(
                div()
                    .absolute()
                    .text_xs()
                    .text_color(gpui::red())
                    .top_16()
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

const TITLEBAR_HEIGHT: f32 = 24.0;

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
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .size_full()
                    .overflow_hidden()
                    .child(self.canvas.clone()),
            )
            .child(self.titlebar.clone())
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
            .h(px(TITLEBAR_HEIGHT))
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
