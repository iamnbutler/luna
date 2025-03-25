#![allow(dead_code, unused)]

use components::RenderProperties;
use ecs::LunaEcs;
use gpui::{
    div, impl_actions, point, prelude::*, px, rgb, size, App, Application, Bounds, CursorStyle,
    Element, ElementId, Entity, FocusHandle, Focusable, GlobalElementId, Hitbox, Hsla, IntoElement,
    LayoutId, MouseMoveEvent, ParentElement, Pixels, Point, Position, SharedString, Size, Style,
    WeakEntity, Window, WindowOptions,
};
use schemars_derive::JsonSchema;
use serde::{Deserialize, Serialize};
use slotmap::KeyData;
use std::collections::HashMap;
use std::hash::Hash;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use uuid::Uuid;

use crate::components::{
    hierarchy::HierarchyComponent,
    transform::{vec2, BoundingBox, LocalPosition, LocalTransform, Vector2D, WorldTransform},
};
use crate::prelude::{LayoutProperties, Margins, SizeConstraints};
use crate::systems::{hit_test::HitTestSystem, spatial::QuadTree};
use gpui::quad;

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
    canvas: Entity<Canvas>,
    focus_handle: FocusHandle,
}

impl Luna {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let weak_self = cx.entity().downgrade();
        let ecs = cx.new(|cx| LunaEcs::new(cx));
        let focus_handle = cx.focus_handle();
        let canvas = cx.new(|cx| Canvas::new("canvas", ecs.clone(), cx));

        let new_el = canvas.update(cx, |canvas, cx| canvas.add_element(cx));

        let root_entity = ecs.update(cx, |ecs, _cx| {
            ecs.render_mut()
                .set_properties(new_el, RenderProperties::default())
        });

        Luna {
            weak_self,
            ecs,
            canvas,
            focus_handle,
        }
    }
}

impl Render for Luna {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("luna")
            .key_context("App")
            .track_focus(&self.focus_handle(cx))
            .text_xs()
            .text_color(rgb(0x000000))
            .font_family("Berkeley Mono")
            .flex()
            .flex_col()
            .relative()
            .bg(rgb(0x00FF00))
            .size_full()
            .text_color(rgb(0xffffff))
            .child(self.canvas.clone())
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

#[derive(Clone)]
/// A Canvas that manages and renders UI elements using the ECS system
pub struct Canvas {
    /// Unique ID for this canvas
    id: ElementId,
    /// Reference to the ECS manager
    ecs: Entity<LunaEcs>,
    /// The root entity for this canvas
    root_entity: LunaEntityId,
    /// Current viewport size (visible area)
    viewport_size: Option<Size<Pixels>>,
    /// Total canvas size (can be larger than viewport)
    canvas_size: Size<Pixels>,
    /// Current viewport offset within the canvas
    viewport_offset: Point<Pixels>,
    /// Focus handle for input events
    focus_handle: FocusHandle,
    /// Hit test system for spatial queries
    hit_test: HitTestSystem,
}

impl Canvas {
    pub fn new(id: impl Into<ElementId>, ecs: Entity<LunaEcs>, cx: &mut Context<Self>) -> Self {
        let root_entity = ecs.update(cx, |ecs, _cx| ecs.create_entity());

        // Default canvas size of 3000x3000 pixels
        let canvas_size = Size {
            width: px(3000.0),
            height: px(3000.0),
        };

        // Use a reasonable default viewport size (will be updated in prepaint)
        let default_viewport = Size {
            width: px(800.0),
            height: px(600.0),
        };

        // Initialize viewport offset to center the view on (1500,1500)
        let viewport_offset = Point {
            x: px(1500.0 - default_viewport.width.0 / 2.0),
            y: px(1500.0 - default_viewport.height.0 / 2.0),
        };

        Canvas {
            id: id.into(),
            ecs,
            root_entity,
            viewport_size: Some(default_viewport),
            canvas_size,
            viewport_offset,
            focus_handle: cx.focus_handle(),
            hit_test: HitTestSystem::new(canvas_size.width.0 as f32, canvas_size.height.0 as f32),
        }
    }

    /// Updates the viewport size and adjusts systems accordingly
    pub fn update_viewport(&mut self, size: Size<Pixels>, cx: &mut Context<Self>) {
        self.viewport_size = Some(size);
        self.hit_test = HitTestSystem::new(size.width.0 as f32, size.height.0 as f32);
        cx.notify();
    }

    /// Adds a child element to the canvas
    pub fn add_element(&mut self, cx: &mut Context<Self>) -> LunaEntityId {
        self.ecs.update(cx, |ecs, _cx| {
            let entity = ecs.create_entity();

            // Add to hierarchy under root
            ecs.hierarchy_mut().add_child(self.root_entity, entity);

            // Position new elements in the center of the canvas (not viewport)
            let position = LocalPosition {
                x: self.canvas_size.width.0 as f32 / 2.0 - 50.0, // center of canvas - half width
                y: self.canvas_size.height.0 as f32 / 2.0 - 50.0, // center of canvas - half height
            };

            ecs.transforms_mut().set_transform(
                entity,
                LocalTransform {
                    position,
                    scale: vec2(100.0, 100.0),
                    rotation: 0.0,
                },
            );

            // Add default layout properties
            ecs.layout_mut().set_layout(
                entity,
                LayoutProperties {
                    width: None,
                    height: None,
                    constraints: SizeConstraints::default(),
                    margins: Margins::default(),
                },
            );

            entity
        })
    }
}

#[derive(Default)]
pub struct CanvasLayoutState {
    pub layout_id: Option<LayoutId>,
}

impl Element for Canvas {
    type RequestLayoutState = CanvasLayoutState;
    type PrepaintState = Option<Hitbox>;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        // Request layout for canvas container
        let layout_id = window.request_layout(
            Style {
                position: Position::Absolute,
                ..Default::default()
            },
            vec![],
            cx,
        );

        (
            layout_id,
            CanvasLayoutState {
                layout_id: Some(layout_id),
            },
        )
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        // Update viewport size if changed
        if self.viewport_size.map_or(true, |size| size != bounds.size) {
            let size = bounds.size;
            self.viewport_size = Some(size);
            // Keep using canvas_size for hit testing, only update viewport
        }

        // First collect all entity IDs
        let entities = self.ecs.update(cx, |ecs, _cx| {
            ecs.entities().keys().copied().collect::<Vec<_>>()
        });

        // Process each entity individually to avoid borrow conflicts
        for entity in entities {
            self.ecs.update(cx, |ecs, _cx| {
                self.hit_test.update_entity(ecs, entity);
            });
        }

        Some(window.insert_hitbox(bounds, false))
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        // if bounds.size.width == px(0.0) || bounds.size.height == px(0.0) {
        //     println!("Bounds width or height is zero");
        // }

        let canvas_bounds = Bounds {
            origin: Point {
                x: px(0.0),
                y: px(0.0),
            },
            size: self.canvas_size,
        };

        // Paint background
        window.paint_quad(quad(
            canvas_bounds,
            0.0,
            rgb(0x3B414D),
            px(1.0),
            rgb(0x1E1E1E),
        ));

        // Calculate scrollbar visibility and dimensions
        if let Some(viewport_size) = self.viewport_size {
            // Only show scrollbars if viewport is smaller than canvas
            let horizontal_ratio = viewport_size.width.0 / self.canvas_size.width.0;
            let vertical_ratio = viewport_size.height.0 / self.canvas_size.height.0;

            // Scrollbar dimensions
            let scrollbar_thickness = px(8.0);
            let scrollbar_color = rgb(0xFFFFFF);

            // Draw horizontal scrollbar if needed
            if horizontal_ratio < 1.0 {
                let scrollbar_width = viewport_size.width * horizontal_ratio;
                // Position horizontal scrollbar based on viewport offset
                let scroll_x_ratio =
                    self.viewport_offset.x.0 / (self.canvas_size.width.0 - viewport_size.width.0);
                let horizontal_bounds = Bounds {
                    origin: Point {
                        x: bounds.origin.x + (bounds.size.width - scrollbar_width) * scroll_x_ratio,
                        y: bounds.origin.y + bounds.size.height - scrollbar_thickness,
                    },
                    size: Size {
                        width: scrollbar_width,
                        height: scrollbar_thickness,
                    },
                };
                window.paint_quad(quad(
                    horizontal_bounds,
                    0.0,
                    scrollbar_color,
                    px(0.0),
                    scrollbar_color,
                ));
            }

            // Draw vertical scrollbar if needed
            if vertical_ratio < 1.0 {
                let scrollbar_height = viewport_size.height * vertical_ratio;
                // Position vertical scrollbar based on viewport offset
                let scroll_y_ratio =
                    self.viewport_offset.y.0 / (self.canvas_size.height.0 - viewport_size.height.0);
                let vertical_bounds = Bounds {
                    origin: Point {
                        x: bounds.origin.x + bounds.size.width - scrollbar_thickness,
                        y: bounds.origin.y
                            + (bounds.size.height - scrollbar_height) * scroll_y_ratio,
                    },
                    size: Size {
                        width: scrollbar_thickness,
                        height: scrollbar_height,
                    },
                };
                window.paint_quad(quad(
                    vertical_bounds,
                    0.0,
                    scrollbar_color,
                    px(0.0),
                    scrollbar_color,
                ));
            }
        }

        // Paint all entities
        self.ecs.update(cx, |ecs, _cx| {
            // Get entities in render order (bottom-up)
            let mut entities: Vec<_> = ecs.entities().keys().copied().collect();
            entities.sort_by_key(|entity| ecs.hierarchy().get_parent_chain(*entity).len());

            for entity in entities {
                if let Some(transform) = ecs.transforms().get_transform(entity) {
                    let parent_chain = ecs.hierarchy().get_parent_chain(entity);

                    // Compute world transform
                    if let Some(world_transform) = ecs
                        .transforms_mut()
                        .compute_world_transform(entity, parent_chain)
                    {
                        // Create bounds for entity, translated by viewport offset
                        let entity_bounds = Bounds {
                            origin: Point {
                                x: px(world_transform.position.x) - self.viewport_offset.x,
                                y: px(world_transform.position.y) - self.viewport_offset.y,
                            },
                            size: Size {
                                width: px(world_transform.scale.x),
                                height: px(world_transform.scale.y),
                            },
                        };

                        // Paint entity with default style if no render component exists
                        window.paint_quad(quad(
                            entity_bounds,
                            0.0,
                            rgb(0x5A6887),
                            px(1.0),
                            rgb(0x3A4867),
                        ));
                    }
                }
            }
        });
    }
}

impl IntoElement for Canvas {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Focusable for Canvas {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Canvas {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.clone()
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |window, cx| {
            cx.new(|cx| Luna::new(window, cx))
        })
        .unwrap();

        cx.activate(true)
    });
}
