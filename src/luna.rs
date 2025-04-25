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

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use canvas::{CanvasData, CanvasElement, CanvasId, CanvasPages};
use geometry::LocalPoint;
use gpui::{
    actions, div, hsla, point, prelude::*, px, rgba, App, AppContext, Application, ElementId,
    Entity, EventEmitter, FocusHandle, Focusable, Global, Hsla, KeyBinding, Keystroke, Menu,
    MenuItem, MouseButton, MouseUpEvent, Rgba, SharedString, TitlebarOptions, WeakEntity, Window,
    WindowOptions,
};
use input::{InputMap, InputMapKey, TextInput};
use sidebar::inspector::SidebarInspector;

mod canvas;
mod geometry;
mod input;
mod sidebar;
mod typography;

actions!(luna, [Quit]);

#[derive(Clone, Debug)]
pub struct Theme {
    fg: Hsla,
    bg: Hsla,
    accent: Hsla,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            fg: hsla(220.0 / 360.0, 9.0 / 100.0, 72.0 / 100.0, 1.0),
            bg: hsla(220.0 / 360.0, 14.0 / 100.0, 18.0 / 100.0, 1.0),
            accent: hsla(207.0 / 360.0, 82.0 / 100.0, 66.0 / 100.0, 1.0),
        }
    }
}

impl Theme {
    pub fn get_global(cx: &App) -> &Arc<Theme> {
        &cx.global::<GlobalTheme>().0
    }
}

#[derive(Clone, Debug)]
pub struct GlobalTheme(pub Arc<Theme>);

impl Deref for GlobalTheme {
    type Target = Arc<Theme>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for GlobalTheme {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Global for GlobalTheme {}

pub trait ActiveTheme {
    fn theme(&self) -> &Arc<Theme>;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Arc<Theme> {
        &self.global::<GlobalTheme>().0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewCanvas;

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCanvas(pub CanvasId);

struct Luna {
    canvas_pages: CanvasPages,
    focus_handle: FocusHandle,
    sidebar: Entity<SidebarInspector>,
}

impl Luna {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let view = cx.entity();
        let sidebar = cx.new(|cx| SidebarInspector::new(view.clone(), cx));
        let view = cx.entity();
        let mut canvas_pages = CanvasPages::new();
        canvas_pages.add_canvas(cx);

        let mut luna = Self {
            canvas_pages,
            focus_handle: cx.focus_handle(),
            sidebar,
        };

        luna
    }

    fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    fn active_canvas(&self) -> Option<&Entity<CanvasData>> {
        self.canvas_pages.active_canvas()
    }

    fn active_canvas_id(&self) -> Option<CanvasId> {
        self.canvas_pages.active_id()
    }

    fn new_canvas(&mut self, _: &NewCanvas, cx: &mut Context<Self>) {
        let new_id = self.canvas_pages.add_canvas(cx);

        self.canvas_pages.set_active_canvas(new_id);
        cx.notify();
    }

    fn switch_canvas(&mut self, action: &SwitchCanvas, cx: &mut Context<Self>) {
        if self.canvas_pages.set_active_canvas(action.0) {
            cx.notify();
        }
    }
}

impl Focusable for Luna {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Luna {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_canvas_id = self.active_canvas_id();

        div()
            .relative()
            .track_focus(&self.focus_handle())
            .items_center()
            .justify_center()
            .id("app")
            .key_context("Luna")
            .track_focus(&self.focus_handle())
            .text_xs()
            .font_family("Berkeley Mono")
            .flex()
            .flex_row()
            .relative()
            .size_full()
            .text_color(cx.theme().fg)
            .bg(cx.theme().bg)
            .when_some(active_canvas_id.clone(), |this, id| {
                this.child(CanvasElement::new(id, cx))
            })
            .when(active_canvas_id.is_none(), |this| {
                this.child(
                    div()
                        .flex()
                        .text_center()
                        .items_center()
                        .child("No active canvas"),
                )
            })
            .children([
                // Sidebar
                self.sidebar.clone(),
                // Canvas area
            ])
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

        cx.set_global(GlobalTheme(Arc::new(Theme::default())));

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

                view.new_canvas(&NewCanvas, cx);
            })
            .unwrap();
    });
}

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}
