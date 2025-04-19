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
use input::text_input::TextInput;
mod geometry;
mod input;

actions!(luna, [Quit]);

pub enum UpdateProperty {
    Position(LocalPoint),
    Width(f32),
    Height(f32),
}

pub enum InspectorEvent {
    UpdateProperty(UpdateProperty),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum InputMapKey {
    PositionX,
    PositionY,
    Width,
    Height,
}

impl InputMapKey {
    pub fn to_element_id(&self) -> ElementId {
        match self {
            InputMapKey::PositionX => ElementId::from("input-x"),
            InputMapKey::PositionY => ElementId::from("input-y"),
            InputMapKey::Width => ElementId::from("input-width"),
            InputMapKey::Height => ElementId::from("input-height"),
        }
    }
}

impl std::fmt::Display for InputMapKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputMapKey::PositionX => write!(f, "X"),
            InputMapKey::PositionY => write!(f, "Y"),
            InputMapKey::Width => write!(f, "Width"),
            InputMapKey::Height => write!(f, "Height"),
        }
    }
}

struct InputMap {
    map: HashMap<InputMapKey, Entity<TextInput>>,
}

impl InputMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn new_input(
        mut self,
        key: InputMapKey,
        cx: &mut App,
        f: impl FnOnce(&mut TextInput, &mut App),
    ) -> Self {
        let element_id = key.to_element_id();
        let placeholder = key.to_string();
        let mut input = TextInput::new(element_id, placeholder, cx);
        f(&mut input, cx);
        self.map.insert(key, cx.new(|cx| input));
        self
    }

    pub fn get_input(&self, key: InputMapKey) -> Option<&Entity<TextInput>> {
        self.map.get(&key)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity<TextInput>> {
        self.map.values()
    }
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
            .children(
                self.input_map
                    .iter()
                    .map(|input| div().w_full().p(px(2.)).child(input.clone())),
            )
    }
}

impl Focusable for Sidebar {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<InspectorEvent> for Sidebar {}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DebugProperties {
    pub position: LocalPoint,
    pub width: f32,
    pub height: f32,
}

impl Default for DebugProperties {
    fn default() -> Self {
        Self {
            position: LocalPoint::new(0.0, 0.0, 0.0),
            width: 0.0,
            height: 0.0,
        }
    }
}

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
