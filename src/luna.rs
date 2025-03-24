#![allow(dead_code, unused)]

use ecs::LunaEcs;
use gpui::{
    div, impl_actions, point, prelude::*, px, rgb, size, App, Application, Bounds, CursorStyle,
    Entity, FocusHandle, Focusable, Hsla, MouseMoveEvent, ParentElement, Pixels, Point,
    SharedString, Size, WeakEntity, Window, WindowOptions,
};
use schemars_derive::JsonSchema;
use serde::{Deserialize, Serialize};
use slotmap::KeyData;
use std::collections::HashMap;
use std::hash::Hash;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use uuid::Uuid;

pub mod components;
pub mod ecs;
pub mod prelude;
pub mod systems;

pub const SELECTED_COLOR: Hsla = Hsla {
    h: 205.0 / 360.0,
    s: 0.9,
    l: 0.48,
    a: 1.0,
};

slotmap::new_key_type! {
    /// A Luna Entity ID, a unique identifier for an entity across the entire app.
    pub struct LunaEntityId;
}

impl LunaEntityId {
    /// Converts this id to a [u64]
    pub fn as_u64(self) -> u64 {
        self.0.as_ffi()
    }
}

impl From<u64> for LunaEntityId {
    fn from(value: u64) -> Self {
        Self(KeyData::from_ffi(value))
    }
}

impl std::fmt::Display for LunaEntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_u64())
    }
}

#[derive(Debug)]
struct Luna {
    weak_self: WeakEntity<Self>,
    ecs: Entity<LunaEcs>,
    viewport_size: Size<Pixels>,
    focus_handle: FocusHandle,
}

impl Luna {
    pub fn new(window: &mut Window, viewport_size: Size<Pixels>, cx: &mut Context<Self>) -> Self {
        let weak_self = cx.entity().downgrade();
        let ecs = cx.new(|_| LunaEcs::new());
        let focus_handle = cx.focus_handle();

        let luna = Luna {
            weak_self,
            ecs,
            viewport_size,
            focus_handle,
        };

        luna
    }
}

impl Render for Luna {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("luna")
            .key_context("App")
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
        // This updates the viewport size when the window is resized
        // reintroduce this later.
        //
        // .child({
        //     let this = cx.entity().clone();
        //     gpui::canvas(
        //         move |bounds, window, cx| {
        //             this.update(cx, |this, cx| {
        //                 let bounds_changed = this.bounds != bounds;
        //                 this.bounds = bounds;
        //                 if bounds_changed {
        //                     this.scene_graph.update(cx, |scene_graph, cx| {
        //                         scene_graph.update_viewport(bounds.size, window, cx);
        //                     });
        //                 }
        //             })
        //         },
        //         |_, _, _, _| {},
        //     )
        //     .absolute()
        //     .size_full()
        // })
    }
}

impl Focusable for Luna {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |window, cx| {
            cx.new(|cx| Luna::new(window, window.viewport_size(), cx))
        })
        .unwrap();

        cx.activate(true)
    });
}
