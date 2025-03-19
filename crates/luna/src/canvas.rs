use gpui::{prelude::FluentBuilder as _, *};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use uuid::Uuid;

use crate::Corner;

use crate::{element::*, ResizeDirection, CORNER_HANDLE_SIZE, EDGE_HITBOX_PADDING, THEME_SELECTED};

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
    pub(crate) selected_ids: Vec<LunaElementId>,
    pub(crate) dragging: Option<Point<Pixels>>,
    canvas_offset: Point<Pixels>,
    is_dragging_canvas: bool,
    drag_start: Option<Point<Pixels>>,
    current_resize_direction: Option<ResizeDirection>,
    resize_start: Option<(Point<Pixels>, Size<Pixels>)>,
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
            resize_start: None,
        })
    }

    pub fn elements(&self) -> &HashMap<LunaElementId, Entity<LunaElement>> {
        &self.elements
    }

    pub fn selected_ids(&self) -> &Vec<LunaElementId> {
        &self.selected_ids
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
        let resize_from_center = current_modifiers.alt;
        let element_id = element_at_position.map(|element| element.0);

        match event.button {
            MouseButton::Left => {
                // Handle resizing first
                if let Some(direction) = self.current_resize_direction {
                    if self.selected_ids.len() == 1 {
                        let id = self.selected_ids[0];
                        if let Some(element) = self.elements.get(&id) {
                            self.resize_start = Some((position, element.read(cx).style.size));
                            self.drag_start = Some(position);
                            cx.notify();
                            return;
                        }
                    }
                }

                if current_modifiers.alt {
                    self.is_dragging_canvas = true;
                    self.drag_start = Some(position);
                    cx.notify();
                    return;
                }

                // Handle element selection if not resizing
                if let Some(element_id) = element_id {
                    if current_modifiers.shift {
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
            if let Some(direction) = self.current_resize_direction {
                if direction.is_corner() && self.selected_ids.len() == 1 {
                    let id = self.selected_ids[0];
                    self.resize_element(id, direction, event.position, cx);
                    self.dragging = Some(event.position);
                } else if let Some(start_pos) = self.dragging {
                    let delta = event.position - start_pos;
                    self.move_selected_elements(delta, cx);
                    self.dragging = Some(event.position);
                }
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

        if !self.selected_ids.is_empty() && self.selected_ids.len() == 1 {
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

                    let corner_size = px(CORNER_HANDLE_SIZE * 2.0);
                    let top_left = Bounds {
                        origin: expanded_bounds.origin,
                        size: Size::new(corner_size, corner_size),
                    };
                    let top_right = Bounds {
                        origin: Point::new(
                            expanded_bounds.origin.x + expanded_bounds.size.width - corner_size,
                            expanded_bounds.origin.y,
                        ),
                        size: Size::new(corner_size, corner_size),
                    };
                    let bottom_left = Bounds {
                        origin: Point::new(
                            expanded_bounds.origin.x,
                            expanded_bounds.origin.y + expanded_bounds.size.height - corner_size,
                        ),
                        size: Size::new(corner_size, corner_size),
                    };
                    let bottom_right = Bounds {
                        origin: Point::new(
                            expanded_bounds.origin.x + expanded_bounds.size.width - corner_size,
                            expanded_bounds.origin.y + expanded_bounds.size.height - corner_size,
                        ),
                        size: Size::new(corner_size, corner_size),
                    };

                    if top_left.contains(&adjusted_position) {
                        self.current_resize_direction = Some(ResizeDirection::TopLeft);
                    } else if top_right.contains(&adjusted_position) {
                        self.current_resize_direction = Some(ResizeDirection::TopRight);
                    } else if bottom_left.contains(&adjusted_position) {
                        self.current_resize_direction = Some(ResizeDirection::BottomLeft);
                    } else if bottom_right.contains(&adjusted_position) {
                        self.current_resize_direction = Some(ResizeDirection::BottomRight);
                    } else if left_hitbox.contains(&adjusted_position) {
                        self.current_resize_direction = Some(ResizeDirection::Left);
                    } else if right_hitbox.contains(&adjusted_position) {
                        self.current_resize_direction = Some(ResizeDirection::Right);
                    } else if top_hitbox.contains(&adjusted_position) {
                        self.current_resize_direction = Some(ResizeDirection::Top);
                    } else if bottom_hitbox.contains(&adjusted_position) {
                        self.current_resize_direction = Some(ResizeDirection::Bottom);
                    }
                }
            }
        } else {
            self.current_resize_direction = None;
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

    fn handle_mouse_up(&mut self, event: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.dragging = None;
        self.is_dragging_canvas = false;
        self.drag_start = None;
        self.resize_start = None;
        cx.notify();
    }

    fn resize_element(
        &mut self,
        id: LunaElementId,
        direction: ResizeDirection,
        current_pos: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        if let (Some(element), Some((start_pos, start_size))) =
            (self.elements.get(&id), self.resize_start)
        {
            let delta = current_pos - start_pos;
            let mut new_size = start_size;
            let mut new_pos = element.read(cx).style.position.unwrap();

            match direction {
                ResizeDirection::TopLeft => {
                    new_size.width = (start_size.width - delta.x).max(px(10.0));
                    new_size.height = (start_size.height - delta.y).max(px(10.0));
                    new_pos.x = start_pos.x + start_size.width - new_size.width;
                    new_pos.y = start_pos.y + start_size.height - new_size.height;
                }
                ResizeDirection::TopRight => {
                    new_size.width = (start_size.width + delta.x).max(px(10.0));
                    new_size.height = (start_size.height - delta.y).max(px(10.0));
                    new_pos.y = start_pos.y + start_size.height - new_size.height;
                }
                ResizeDirection::BottomLeft => {
                    new_size.width = (start_size.width - delta.x).max(px(10.0));
                    new_size.height = (start_size.height + delta.y).max(px(10.0));
                    new_pos.x = start_pos.x + start_size.width - new_size.width;
                }
                ResizeDirection::BottomRight => {
                    new_size.width = (start_size.width + delta.x).max(px(10.0));
                    new_size.height = (start_size.height + delta.y).max(px(10.0));
                }
                _ => {}
            }

            element.update(cx, |element, _cx| {
                element.style.size = new_size;
                element.style.position = Some(new_pos);
            });
        }
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
                el = el
                    .h(px(EDGE_HITBOX_PADDING))
                    .top(px(CORNER_HANDLE_SIZE))
                    .bottom(px(CORNER_HANDLE_SIZE))
                    .h_full();
            } else {
                el = el
                    .w(px(EDGE_HITBOX_PADDING))
                    .left(px(CORNER_HANDLE_SIZE))
                    .right(px(CORNER_HANDLE_SIZE))
                    .w_full();
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
            let multiple_selection = self.selected_ids.len() > 1;
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
                .when(!multiple_selection, |this| {
                    this.child(
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
