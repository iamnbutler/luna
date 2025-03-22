#![allow(dead_code, unused)]

mod canvas;
mod element;
mod frame;
mod layer_list;
mod scene_graph;
mod titlebar;

use canvas::Canvas;
use element::ElementStyle;
use gpui::{prelude::FluentBuilder as _, *};

use layer_list::LayerList;
use scene_graph::{BoundingBox, QuadTree, SceneGraph};
use schemars_derive::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use titlebar::TITLEBAR_HEIGHT;
use uuid::Uuid;

use gpui::{div, impl_actions, px, Hsla, ParentElement, Pixels, Point, Size};

pub const EDGE_HITBOX_PADDING: f32 = 12.0;
pub const CORNER_HANDLE_SIZE: f32 = 7.0;

pub const THEME_SELECTED: Hsla = Hsla {
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

#[derive(Debug)]
struct Luna {
    weak_self: WeakEntity<Self>,
    titlebar: Entity<Titlebar>,
    // canvas: Entity<Canvas>,
    // element_list: Entity<LayerList>,
    scene_graph: Entity<SceneGraph>,
    viewport_size: Size<Pixels>,
    bounds: Bounds<Pixels>,
    focus_handle: FocusHandle,
}

impl Luna {
    pub fn new(window: &mut Window, viewport_size: Size<Pixels>, cx: &mut Context<Self>) -> Self {
        let titlebar = cx.new(|cx| Titlebar::new(window, cx));
        // let element_list = cx.new(|cx| LayerList::new(canvas.clone(), cx));

        let scene_graph = cx.new(|cx| SceneGraph::new("scene-graph", cx));

        let weak_self = cx.entity().downgrade();

        let bounds = Bounds::new(
            point(px(0.0), px(0.0)),
            size(viewport_size.width, viewport_size.height),
        );

        let focus_handle = cx.focus_handle();

        let luna = Luna {
            weak_self,
            titlebar,
            // canvas,
            // element_list,
            scene_graph,
            viewport_size,
            bounds,
            focus_handle,
        };

        cx.focus_self(window);

        luna
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        println!("Mouse moved: {:?}", event);
    }
}

impl Render for Luna {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bounds = self.bounds;

        div()
            .id("luna")
            .key_context("Canvas")
            .track_focus(&self.focus_handle(cx))
            .text_xs()
            .text_color(rgb(0xA9AFBC))
            .font_family("Berkeley Mono")
            .flex()
            .flex_col()
            .relative()
            .bg(rgb(0x3B414D))
            .size_full()
            .text_color(rgb(0xffffff))
            .child({
                let this = cx.entity().clone();
                canvas(
                    move |bounds, window, cx| {
                        this.update(cx, |this, cx| {
                            let bounds_changed = this.bounds != bounds;
                            this.bounds = bounds;
                            if bounds_changed {
                                this.scene_graph.update(cx, |scene_graph, cx| {
                                    scene_graph.update_viewport(bounds.size, window, cx);
                                });
                            }
                        })
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full()
            })
            .child(self.scene_graph.clone())
        // .child(
        //     div()
        //         .absolute()
        //         .top_8()
        //         .left_8()
        //         .text_sm()
        //         .text_color(gpui::red())
        //         // .child(format!("{}x{}", bounds.size.width.0, bounds.size.height.0)),
        //         .child(format!("{:?}, {:?}", self.focus_handle, window.focused(cx))),
        // )
        // .child(
        //     div()
        //         .absolute()
        //         .top_0()
        //         .left_0()
        //         .right_0()
        //         .bottom_0()
        //         .size_full()
        //         .overflow_hidden()

        // )
        // .child(self.element_list.clone())
        // .child(self.titlebar.clone())
    }
}

impl Focusable for Luna {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle.clone()
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
            // let canvas = Canvas::new(window, cx);
            // canvas.update(cx, |canvas, cx| {
            //     let element_1 = ElementStyle::new(cx).size(size(px(32.), px(128.)));
            //     let element_2 = ElementStyle::new(cx);
            //     let element_3 = ElementStyle::new(cx).size(size(px(64.), px(64.)));
            //     let element_4 = ElementStyle::new(cx).size(size(px(128.), px(128.)));

            //     canvas.add_element(element_1, point(px(0.), px(0.)), cx);
            //     canvas.add_element(element_2, point(px(300.), px(300.)), cx);
            //     canvas.add_element(element_3, point(px(600.), px(150.)), cx);
            //     canvas.add_element(element_4, point(px(240.), px(550.)), cx);
            // });

            cx.new(|cx| Luna::new(window, window.viewport_size(), cx))
        })
        .unwrap();

        cx.activate(true)
    });
}
