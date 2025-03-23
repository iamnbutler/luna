#![allow(dead_code, unused)]

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

/// Creates a new 2D vector.
pub fn vec2(x: f32, y: f32) -> Vector2D {
    Vector2D { x, y }
}

/// Represents a vector in 2D space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

impl From<(f32, f32)> for Vector2D {
    fn from(tuple: (f32, f32)) -> Self {
        Vector2D {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl From<Vector2D> for (f32, f32) {
    fn from(vec: Vector2D) -> Self {
        (vec.x, vec.y)
    }
}

impl From<[f32; 2]> for Vector2D {
    fn from(array: [f32; 2]) -> Self {
        Vector2D {
            x: array[0],
            y: array[1],
        }
    }
}

impl From<Vector2D> for [f32; 2] {
    fn from(vec: Vector2D) -> Self {
        [vec.x, vec.y]
    }
}

impl Default for Vector2D {
    fn default() -> Self {
        Vector2D { x: 0.0, y: 0.0 }
    }
}

/// Represents a position in world space coordinates.
///
/// World position is absolute within the entire canvas, independent of hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorldPosition {
    pub x: f32,
    pub y: f32,
}

impl Default for WorldPosition {
    fn default() -> Self {
        WorldPosition { x: 0.0, y: 0.0 }
    }
}

impl From<(f32, f32)> for WorldPosition {
    fn from(tuple: (f32, f32)) -> Self {
        WorldPosition {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl From<[f32; 2]> for WorldPosition {
    fn from(array: [f32; 2]) -> Self {
        WorldPosition {
            x: array[0],
            y: array[1],
        }
    }
}

impl From<Vector2D> for WorldPosition {
    fn from(vec: Vector2D) -> Self {
        WorldPosition { x: vec.x, y: vec.y }
    }
}

/// Represents a position in local space coordinates.
///
/// Local position is relative to the parent element in the hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LocalPosition {
    pub x: f32,
    pub y: f32,
}

impl Default for LocalPosition {
    fn default() -> Self {
        LocalPosition { x: 0.0, y: 0.0 }
    }
}

impl From<(f32, f32)> for LocalPosition {
    fn from(tuple: (f32, f32)) -> Self {
        LocalPosition {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl From<[f32; 2]> for LocalPosition {
    fn from(array: [f32; 2]) -> Self {
        LocalPosition {
            x: array[0],
            y: array[1],
        }
    }
}

impl From<Vector2D> for LocalPosition {
    fn from(vec: Vector2D) -> Self {
        LocalPosition { x: vec.x, y: vec.y }
    }
}

/// Represents a local transform, containing position, scale, and rotation relative to the parent.
pub struct LocalTransform {
    pub position: LocalPosition,
    pub scale: Vector2D,
    pub rotation: f32,
}

/// Represents a world transform, containing absolute position, scale, and rotation.
pub struct WorldTransform {
    pub position: WorldPosition,
    pub scale: Vector2D,
    pub rotation: f32,
}

/// An unrotated, rectangular bounding box (AABB) whose edges are parallel to the coordinate axes.
///
/// Used for efficient collision detection and spatial partitioning.
pub struct BoundingBox {
    min: Vector2D,
    max: Vector2D,
}

impl BoundingBox {
    pub fn new(min: Vector2D, max: Vector2D) -> Self {
        BoundingBox { min, max }
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn half_width(&self) -> f32 {
        self.width() / 2.0
    }

    pub fn half_height(&self) -> f32 {
        self.height() / 2.0
    }

    pub fn center(&self) -> Vector2D {
        vec2(
            self.min.x + self.width() / 2.0,
            self.min.y + self.height() / 2.0,
        )
    }
}

#[derive(Debug)]
struct Luna {
    weak_self: WeakEntity<Self>,
    viewport_size: Size<Pixels>,
    focus_handle: FocusHandle,
}

impl Luna {
    pub fn new(window: &mut Window, viewport_size: Size<Pixels>, cx: &mut Context<Self>) -> Self {
        let weak_self = cx.entity().downgrade();
        let focus_handle = cx.focus_handle();

        let luna = Luna {
            weak_self,
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
