use super::{SectionHeader, ITEM_HEIGHT, ROW_GAP};

use std::collections::HashMap;

use crate::geometry::LocalPoint;
use crate::input::{InputMap, InputMapKey, TextInput};
use crate::Luna;
use gpui::{
    actions, div, hsla, point, prelude::*, px, rgba, App, AppContext, Application, ElementId,
    Entity, EventEmitter, FocusHandle, Focusable, KeyBinding, Keystroke, Menu, MenuItem,
    MouseButton, MouseUpEvent, Rgba, SharedString, TitlebarOptions, WeakEntity, Window,
    WindowOptions,
};

pub struct SidebarInspector {
    app: Entity<Luna>,
    input_map: InputMap,
    focus_handle: FocusHandle,
}

impl SidebarInspector {
    pub fn new(app: Entity<Luna>, cx: &mut Context<Self>) -> Self {
        let input_map = InputMap::new()
            .new_input(InputMapKey::PositionX, cx, |input, cx| {})
            .new_input(InputMapKey::PositionY, cx, |input, cx| {})
            .new_input(InputMapKey::Width, cx, |input, cx| {})
            .new_input(InputMapKey::Height, cx, |input, cx| {})
            .new_input(InputMapKey::Rotation, cx, |input, cx| {})
            .new_input(InputMapKey::CornerRadius, cx, |input, cx| {})
            .new_input(InputMapKey::Opacity, cx, |input, cx| {});

        Self {
            app,
            input_map,
            focus_handle: cx.focus_handle(),
        }
    }

    fn focus_handle(&self, _cx: &mut Context<Self>) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SidebarInspector {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let x_input = self.input_map.get_input(InputMapKey::PositionX);
        let y_input = self.input_map.get_input(InputMapKey::PositionY);
        let width_input = self.input_map.get_input(InputMapKey::Width);
        let height_input = self.input_map.get_input(InputMapKey::Height);
        let rotation_input = self.input_map.get_input(InputMapKey::Rotation);
        let opacity_input = self.input_map.get_input(InputMapKey::Opacity);
        let corner_radius_input = self.input_map.get_input(InputMapKey::CornerRadius);

        div()
            .absolute()
            .top(px(0.))
            .right(px(0.))
            .flex()
            .flex_col()
            .w(px(240.))
            .h_full()
            .track_focus(&self.focus_handle(cx))
            .gap(px(4.))
            .overflow_hidden()
            .py_1()
            .px_1p5()
            .child(
                div()
                    .flex()
                    .w_full()
                    .overflow_hidden()
                    .gap(px(6.))
                    .children(x_input)
                    .children(y_input)
                    .children(rotation_input),
            )
            .child(
                div()
                    .flex()
                    .w_full()
                    .overflow_hidden()
                    .gap(px(6.))
                    .children(width_input)
                    .children(height_input)
                    .child(div().flex_1()),
            )
            .child(SectionHeader::with_title("Corners"))
            .child(
                div()
                    .flex()
                    .w_full()
                    .overflow_hidden()
                    .gap(px(6.))
                    .children(corner_radius_input),
            )
            .child(SectionHeader::with_title("Opacity"))
            .child(
                div()
                    .flex()
                    .w_full()
                    .overflow_hidden()
                    .gap(px(6.))
                    .children(opacity_input),
            )
    }
}

impl Focusable for SidebarInspector {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<InspectorEvent> for SidebarInspector {}

pub enum UpdatePropertyEvent {
    Position(LocalPoint),
    Width(f32),
    Height(f32),
}

pub enum InspectorEvent {
    UpdateProperty(UpdatePropertyEvent),
}
