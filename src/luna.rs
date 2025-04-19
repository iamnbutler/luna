#![allow(unused, dead_code)]

//! # Luna: A software design tool without compromises.
//!
//! Luna is local, files on disk first. Own your own data,
//! collaborate, design on the canvas or write code.
//!
//! It's not a design tool, or a code editor, it's a tool
//! for designing software:
//!
//! That means not just pixels, but representative screens and flows
//! using an abstractionless design and editing experience.

use std::collections::HashMap;

use geometry::LocalPoint;
use gpui::{
    actions, div, hsla, point, prelude::*, px, rgba, App, AppContext, Application, ElementId,
    Entity, EventEmitter, FocusHandle, Focusable, KeyBinding, Keystroke, Menu, MenuItem,
    MouseButton, MouseUpEvent, Rgba, SharedString, TitlebarOptions, WeakEntity, Window,
    WindowOptions,
};
use input::{InputMap, InputMapKey, TextInput};
mod geometry;
mod input;

actions!(luna, [Quit]);

pub enum UpdatePropertyEvent {
    Position(LocalPoint),
    Width(f32),
    Height(f32),
}

pub enum InspectorEvent {
    UpdateProperty(UpdatePropertyEvent),
}

struct Sidebar {
    app: Entity<Luna>,
    input_map: InputMap,
    focus_handle: FocusHandle,
}

impl Sidebar {
    fn new(app: Entity<Luna>, cx: &mut Context<Self>) -> Self {
        // Use default values initially to avoid circular dependency
        let input_map = InputMap::new()
            .new_input(InputMapKey::PositionX, cx, |input, cx| {})
            .new_input(InputMapKey::PositionY, cx, |input, cx| {})
            .new_input(InputMapKey::Width, cx, |input, cx| {})
            .new_input(InputMapKey::Height, cx, |input, cx| {});

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

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let x_input = self.input_map.get_input(InputMapKey::PositionX);
        let y_input = self.input_map.get_input(InputMapKey::PositionY);
        let width_input = self.input_map.get_input(InputMapKey::Width);
        let height_input = self.input_map.get_input(InputMapKey::Height);

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
                    .children(y_input),
            )
            .child(
                div()
                    .flex()
                    .w_full()
                    .overflow_hidden()
                    .gap(px(6.))
                    .children(width_input)
                    .children(height_input),
            )
    }
}

impl Focusable for Sidebar {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<InspectorEvent> for Sidebar {}

struct Luna {
    // The main canvas where elements are rendered and manipulated
    // active_canvas: Entity<LunaCanvas>,
    /// Focus handle for keyboard event routing
    focus_handle: FocusHandle,
    sidebar: Entity<Sidebar>,
}

impl Luna {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let view = cx.entity();
        let sidebar = cx.new(|cx| Sidebar::new(view.clone(), cx));
        let view = cx.entity();

        let mut luna = Self {
            focus_handle: cx.focus_handle(),
            sidebar,
        };

        luna
    }

    fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Focusable for Luna {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Luna {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .relative()
            .track_focus(&self.focus_handle())
            .items_center()
            .justify_center()
            .id("scene-graph")
            .key_context("Luna")
            .track_focus(&self.focus_handle())
            .text_xs()
            .font_family("Berkeley Mono")
            .flex()
            .flex_col()
            .relative()
            .bg(hsla(0.0, 0.0, 0.0, 1.0))
            .size_full()
            .text_color(hsla(0.0, 1.0, 1.0, 1.0))
            .child(self.sidebar.clone())
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.on_action(quit);
        cx.set_menus(vec![Menu {
            name: "Luna".into(),
            items: vec![MenuItem::action("Quit", Quit)],
        }]);

        cx.bind_keys([
            KeyBinding::new("backspace", input::Backspace, None),
            KeyBinding::new("delete", input::Delete, None),
            KeyBinding::new("left", input::Left, None),
            KeyBinding::new("right", input::Right, None),
            KeyBinding::new("shift-left", input::SelectLeft, None),
            KeyBinding::new("shift-right", input::SelectRight, None),
            KeyBinding::new("cmd-a", input::SelectAll, None),
            KeyBinding::new("cmd-v", input::Paste, None),
            KeyBinding::new("cmd-c", input::Copy, None),
            KeyBinding::new("cmd-x", input::Cut, None),
            KeyBinding::new("home", input::Home, None),
            KeyBinding::new("end", input::End, None),
            KeyBinding::new("enter", input::Enter, None),
            KeyBinding::new("ctrl-cmd-space", input::ShowCharacterPalette, None),
        ]);

        let window = cx
            .open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("Luna".into()),
                        traffic_light_position: Some(point(px(8.0), px(8.0))),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| Luna::new(window, cx)),
            )
            .unwrap();

        cx.on_keyboard_layout_change({
            move |cx| {
                window.update(cx, |_, _, cx| cx.notify()).ok();
            }
        })
        .detach();

        window
            .update(cx, |view, window, cx| {
                window.focus(&view.focus_handle());
                cx.activate(true);
            })
            .unwrap();
    });
}

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}
