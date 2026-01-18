use crate::canvas::{Canvas, DragState, Tool};
use glam::Vec2;
use gpui::{
    point, px, size, App, BorderStyle, Bounds, ContentMask, DispatchPhase, Element, ElementId,
    Entity, Hitbox, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, ScrollDelta, ScrollWheelEvent, Style, Window,
};
use node_2::ShapeKind;

/// A GPUI element that renders and handles interaction for a Canvas.
pub struct CanvasElement {
    canvas: Entity<Canvas>,
}

impl CanvasElement {
    pub fn new(canvas: Entity<Canvas>) -> Self {
        Self { canvas }
    }
}

impl IntoElement for CanvasElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

pub struct CanvasElementState {
    hitbox: Hitbox,
}

impl Element for CanvasElement {
    type RequestLayoutState = ();
    type PrepaintState = CanvasElementState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = gpui::relative(1.).into();
        style.size.height = gpui::relative(1.).into();
        let layout_id = window.request_layout(style, None, cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        let hitbox = window.insert_hitbox(bounds, gpui::HitboxBehavior::BlockMouse);
        CanvasElementState { hitbox }
    }

    fn paint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let canvas_entity = self.canvas.clone();

        // Read canvas state for rendering
        let (shapes, selection, hovered, viewport, theme, drag) =
            self.canvas.read(cx).clone_render_state();

        // Paint background
        window.paint_quad(gpui::fill(bounds, theme.canvas_background));

        // Set up content mask for clipping
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            // Paint shapes
            for shape in &shapes {
                let screen_bounds = viewport.canvas_to_screen_bounds(shape.position, shape.size);
                let screen_bounds = Bounds {
                    origin: point(
                        bounds.origin.x + px(screen_bounds.origin.x),
                        bounds.origin.y + px(screen_bounds.origin.y),
                    ),
                    size: size(px(screen_bounds.size.width), px(screen_bounds.size.height)),
                };

                // Skip if not visible
                if !bounds.intersects(&screen_bounds) {
                    continue;
                }

                let corner_radius = px(shape.corner_radius * viewport.zoom);

                // Paint fill
                if let Some(fill) = &shape.fill {
                    match shape.kind {
                        ShapeKind::Rectangle => {
                            window.paint_quad(
                                gpui::fill(screen_bounds, fill.color).corner_radii(corner_radius),
                            );
                        }
                        ShapeKind::Ellipse => {
                            // For ellipse, use corner radius = half the smaller dimension
                            let w: f32 = screen_bounds.size.width.into();
                            let h: f32 = screen_bounds.size.height.into();
                            let radius = px(w.min(h) / 2.0);
                            window.paint_quad(
                                gpui::fill(screen_bounds, fill.color).corner_radii(radius),
                            );
                        }
                    }
                }

                // Paint stroke
                if let Some(stroke) = &shape.stroke {
                    match shape.kind {
                        ShapeKind::Rectangle => {
                            window.paint_quad(gpui::outline(
                                screen_bounds,
                                stroke.color,
                                BorderStyle::Solid,
                            ).corner_radii(corner_radius));
                        }
                        ShapeKind::Ellipse => {
                            let w: f32 = screen_bounds.size.width.into();
                            let h: f32 = screen_bounds.size.height.into();
                            let radius = px(w.min(h) / 2.0);
                            window.paint_quad(gpui::outline(
                                screen_bounds,
                                stroke.color,
                                BorderStyle::Solid,
                            ).corner_radii(radius));
                        }
                    }
                }

                // Paint hover indicator
                if hovered == Some(shape.id) && !selection.contains(&shape.id) {
                    window.paint_quad(gpui::outline(
                        screen_bounds,
                        theme.hover,
                        BorderStyle::Solid,
                    ).corner_radii(corner_radius));
                }

                // Paint selection indicator
                if selection.contains(&shape.id) {
                    window.paint_quad(gpui::outline(
                        screen_bounds,
                        theme.selection,
                        BorderStyle::Solid,
                    ).corner_radii(corner_radius));

                    // Paint corner handles
                    paint_selection_handles(window, screen_bounds, theme.selection);
                }
            }

            // Paint multi-selection bounding box
            if selection.len() > 1 {
                if let Some((min, max)) =
                    selection_bounds_from_shapes(&shapes, &selection, &viewport)
                {
                    let screen_bounds = Bounds {
                        origin: point(bounds.origin.x + px(min.x), bounds.origin.y + px(min.y)),
                        size: size(px(max.x - min.x), px(max.y - min.y)),
                    };
                    window.paint_quad(gpui::outline(
                        screen_bounds,
                        theme.selection,
                        BorderStyle::Solid,
                    ));
                    paint_selection_handles(window, screen_bounds, theme.selection);
                }
            }

            // Paint drag selection rectangle if active
            if let Some(DragState::Selecting { start: _ }) = &drag {
                // Would need current mouse position - skip for now
            }
        });

        // Register mouse event handlers
        let hitbox = prepaint.hitbox.clone();

        // Mouse down
        window.on_mouse_event({
            let canvas = canvas_entity.clone();
            let bounds = bounds;
            let hitbox = hitbox.clone();
            move |event: &MouseDownEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
                    handle_mouse_down(&canvas, event, bounds, cx);
                }
            }
        });

        // Mouse move
        window.on_mouse_event({
            let canvas = canvas_entity.clone();
            let bounds = bounds;
            let hitbox = hitbox.clone();
            move |event: &MouseMoveEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
                    handle_mouse_move(&canvas, event, bounds, cx);
                }
            }
        });

        // Mouse up
        window.on_mouse_event({
            let canvas = canvas_entity.clone();
            let hitbox = hitbox.clone();
            move |event: &MouseUpEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
                    handle_mouse_up(&canvas, event, cx);
                }
            }
        });

        // Scroll wheel (zoom)
        window.on_mouse_event({
            let canvas = canvas_entity.clone();
            let bounds = bounds;
            let hitbox = hitbox.clone();
            move |event: &ScrollWheelEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
                    handle_scroll(&canvas, event, bounds, cx);
                }
            }
        });
    }
}

fn paint_selection_handles(window: &mut Window, bounds: Bounds<Pixels>, color: gpui::Hsla) {
    let handle_size = px(8.0);
    let half = handle_size / 2.0;

    let corners = [
        point(bounds.origin.x - half, bounds.origin.y - half), // top-left
        point(
            bounds.origin.x + bounds.size.width - half,
            bounds.origin.y - half,
        ), // top-right
        point(
            bounds.origin.x - half,
            bounds.origin.y + bounds.size.height - half,
        ), // bottom-left
        point(
            bounds.origin.x + bounds.size.width - half,
            bounds.origin.y + bounds.size.height - half,
        ), // bottom-right
    ];

    for corner in corners {
        let handle_bounds = Bounds {
            origin: corner,
            size: size(handle_size, handle_size),
        };
        window.paint_quad(gpui::fill(handle_bounds, gpui::white()));
        window.paint_quad(gpui::outline(handle_bounds, color, BorderStyle::Solid));
    }
}

fn selection_bounds_from_shapes(
    shapes: &[node_2::Shape],
    selection: &std::collections::HashSet<node_2::ShapeId>,
    viewport: &crate::Viewport,
) -> Option<(Vec2, Vec2)> {
    let selected: Vec<_> = shapes.iter().filter(|s| selection.contains(&s.id)).collect();

    if selected.is_empty() {
        return None;
    }

    let mut min = Vec2::new(f32::MAX, f32::MAX);
    let mut max = Vec2::new(f32::MIN, f32::MIN);

    for shape in selected {
        let screen_min = viewport.canvas_to_screen(shape.position);
        let screen_max = viewport.canvas_to_screen(shape.position + shape.size);
        min.x = min.x.min(screen_min.x);
        min.y = min.y.min(screen_min.y);
        max.x = max.x.max(screen_max.x);
        max.y = max.y.max(screen_max.y);
    }

    Some((min, max))
}

fn handle_mouse_down(
    canvas: &Entity<Canvas>,
    event: &MouseDownEvent,
    bounds: Bounds<Pixels>,
    cx: &mut App,
) {
    if event.button != MouseButton::Left {
        return;
    }

    let local_x: f32 = (event.position.x - bounds.origin.x).into();
    let local_y: f32 = (event.position.y - bounds.origin.y).into();
    let local_pos = point(local_x, local_y);

    canvas.update(cx, |canvas, cx| {
        let canvas_pos = canvas.viewport.screen_to_canvas(local_pos);

        match canvas.tool {
            Tool::Select => {
                if let Some(shape_id) = canvas.shape_at_point(canvas_pos) {
                    let add_to_selection = event.modifiers.shift;
                    if !canvas.selection.contains(&shape_id) {
                        canvas.select(shape_id, add_to_selection, cx);
                    }
                    canvas.start_move(cx);
                } else {
                    canvas.clear_selection(cx);
                }
            }
            Tool::Pan => {
                canvas.start_pan();
            }
            Tool::Rectangle => {
                canvas.start_draw(ShapeKind::Rectangle, canvas_pos, cx);
            }
            Tool::Ellipse => {
                canvas.start_draw(ShapeKind::Ellipse, canvas_pos, cx);
            }
        }
    });
}

fn handle_mouse_move(
    canvas: &Entity<Canvas>,
    event: &MouseMoveEvent,
    bounds: Bounds<Pixels>,
    cx: &mut App,
) {
    let local_x: f32 = (event.position.x - bounds.origin.x).into();
    let local_y: f32 = (event.position.y - bounds.origin.y).into();
    let local_pos = point(local_x, local_y);

    canvas.update(cx, |canvas, cx| {
        let canvas_pos = canvas.viewport.screen_to_canvas(local_pos);

        // Clone drag state to avoid borrow issues
        let drag = canvas.drag.clone();

        match drag {
            Some(DragState::MovingShapes { start_positions }) => {
                if let Some((_, first_start)) = start_positions.first() {
                    let screen_start = canvas.viewport.canvas_to_screen(*first_start);
                    let delta = Vec2::new(local_x - screen_start.x, local_y - screen_start.y)
                        / canvas.viewport.zoom;

                    // Update positions directly
                    for (id, start_pos) in &start_positions {
                        if let Some(shape) = canvas.shapes.iter_mut().find(|s| s.id == *id) {
                            shape.position = *start_pos + delta;
                        }
                    }
                    cx.notify();
                }
            }
            Some(DragState::DrawingShape { shape_id, start }) => {
                // Calculate size and position (handle negative drag)
                let min = Vec2::new(start.x.min(canvas_pos.x), start.y.min(canvas_pos.y));
                let max = Vec2::new(start.x.max(canvas_pos.x), start.y.max(canvas_pos.y));

                if let Some(shape) = canvas.shapes.iter_mut().find(|s| s.id == shape_id) {
                    shape.position = min;
                    shape.size = max - min;
                }
                cx.notify();
            }
            Some(DragState::Panning) => {
                // For panning we need previous position - store it or use a different approach
                // For now, skip delta-based panning in mouse move
            }
            Some(DragState::Selecting { .. }) => {
                // TODO: implement drag selection
            }
            None => {
                // Update hover state
                let new_hovered = canvas.shape_at_point(canvas_pos);
                if canvas.hovered != new_hovered {
                    canvas.hovered = new_hovered;
                    cx.notify();
                }
            }
        }
    });
}

fn handle_mouse_up(canvas: &Entity<Canvas>, event: &MouseUpEvent, cx: &mut App) {
    if event.button != MouseButton::Left {
        return;
    }

    canvas.update(cx, |canvas, cx| {
        match &canvas.drag {
            Some(DragState::MovingShapes { .. }) => {
                canvas.finish_move(cx);
            }
            Some(DragState::DrawingShape { .. }) => {
                canvas.finish_draw(cx);
            }
            Some(DragState::Panning) => {
                canvas.finish_pan();
            }
            Some(DragState::Selecting { .. }) => {
                canvas.drag = None;
            }
            None => {}
        }
    });
}

fn handle_scroll(
    canvas: &Entity<Canvas>,
    event: &ScrollWheelEvent,
    bounds: Bounds<Pixels>,
    cx: &mut App,
) {
    let local_x: f32 = (event.position.x - bounds.origin.x).into();
    let local_y: f32 = (event.position.y - bounds.origin.y).into();
    let local_pos = point(local_x, local_y);

    // Get scroll delta
    let delta_y: f32 = match event.delta {
        ScrollDelta::Pixels(p) => p.y.into(),
        ScrollDelta::Lines(l) => l.y * 20.0, // Approximate pixels per line
    };

    // Zoom with scroll wheel
    let factor = if delta_y > 0.0 { 1.1 } else { 0.9 };

    canvas.update(cx, |canvas, cx| {
        canvas.zoom_at(local_pos, factor, cx);
    });
}

// Helper trait for Canvas to clone state for rendering
trait CloneRenderState {
    fn clone_render_state(
        &self,
    ) -> (
        Vec<node_2::Shape>,
        std::collections::HashSet<node_2::ShapeId>,
        Option<node_2::ShapeId>,
        crate::Viewport,
        theme_2::Theme,
        Option<DragState>,
    );
}

impl CloneRenderState for Canvas {
    fn clone_render_state(
        &self,
    ) -> (
        Vec<node_2::Shape>,
        std::collections::HashSet<node_2::ShapeId>,
        Option<node_2::ShapeId>,
        crate::Viewport,
        theme_2::Theme,
        Option<DragState>,
    ) {
        (
            self.shapes.clone(),
            self.selection.clone(),
            self.hovered,
            self.viewport.clone(),
            self.theme.clone(),
            self.drag.clone(),
        )
    }
}
