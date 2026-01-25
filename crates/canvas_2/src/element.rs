use crate::canvas::{Canvas, DragState, ResizeHandle, Tool};
use glam::Vec2;
use gpui::{
    point, px, size, transparent_black, App, BorderStyle, Bounds, ContentMask, DispatchPhase,
    Element, ElementId, Entity, Hitbox, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, PaintQuad, Pixels, ScrollDelta, ScrollWheelEvent, Style, Window,
};
use node_2::{
    compute_layout, CanvasPoint, CanvasSize, LayoutInput, ScreenPoint, Shape, ShapeId, ShapeKind,
};
use std::collections::HashSet;

/// Size of resize handles in pixels.
const HANDLE_SIZE: f32 = 8.0;

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
        let (mut shapes, selection, hovered, viewport, theme, drag) =
            self.canvas.read(cx).clone_render_state();

        // Compute and apply autolayout for all frames with layout enabled
        apply_all_layouts(&mut shapes);

        // Paint background
        window.paint_quad(gpui::fill(bounds, theme.canvas_background));

        // Set up content mask for clipping to canvas bounds
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            // Paint only root shapes (shapes with no parent)
            // Children are painted recursively by their parent frames
            for shape in shapes.iter().filter(|s| s.parent.is_none()) {
                paint_shape_recursive(
                    shape,
                    &shapes,
                    &selection,
                    hovered,
                    &viewport,
                    &theme,
                    bounds,
                    window,
                );
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
    let handle_size = px(HANDLE_SIZE);
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

/// Check if a screen point (relative to canvas element origin) hits a resize handle.
/// Returns the handle if hit.
fn hit_test_resize_handle(
    screen_point: Vec2,
    selection_bounds: Bounds<Pixels>,
) -> Option<ResizeHandle> {
    let hit_radius = HANDLE_SIZE; // Slightly larger hit area

    let origin_x: f32 = selection_bounds.origin.x.into();
    let origin_y: f32 = selection_bounds.origin.y.into();
    let width: f32 = selection_bounds.size.width.into();
    let height: f32 = selection_bounds.size.height.into();

    let corners = [
        (ResizeHandle::TopLeft, Vec2::new(origin_x, origin_y)),
        (ResizeHandle::TopRight, Vec2::new(origin_x + width, origin_y)),
        (ResizeHandle::BottomLeft, Vec2::new(origin_x, origin_y + height)),
        (ResizeHandle::BottomRight, Vec2::new(origin_x + width, origin_y + height)),
    ];

    for (handle, center) in corners {
        let dx = screen_point.x - center.x;
        let dy = screen_point.y - center.y;
        if dx.abs() <= hit_radius && dy.abs() <= hit_radius {
            return Some(handle);
        }
    }

    None
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
        let canvas_max = CanvasPoint(shape.position.0 + shape.size.0);
        let screen_min = viewport.canvas_to_screen(shape.position);
        let screen_max = viewport.canvas_to_screen(canvas_max);
        min.x = min.x.min(screen_min.x());
        min.y = min.y.min(screen_min.y());
        max.x = max.x.max(screen_max.x());
        max.y = max.y.max(screen_max.y());
    }

    Some((min, max))
}

fn handle_mouse_down(
    canvas: &Entity<Canvas>,
    event: &MouseDownEvent,
    bounds: Bounds<Pixels>,
    cx: &mut App,
) {
    let local_x: f32 = (event.position.x - bounds.origin.x).into();
    let local_y: f32 = (event.position.y - bounds.origin.y).into();
    let screen_pos = ScreenPoint::new(local_x, local_y);
    let local_vec = Vec2::new(local_x, local_y);

    // Middle mouse button always pans
    if event.button == MouseButton::Middle {
        canvas.update(cx, |canvas, _cx| {
            canvas.start_pan(screen_pos);
        });
        return;
    }

    if event.button != MouseButton::Left {
        return;
    }

    canvas.update(cx, |canvas, cx| {
        let canvas_pos = canvas.viewport.screen_to_canvas(screen_pos);

        match canvas.tool {
            Tool::Select => {
                // First check if clicking on a resize handle (only if something is selected)
                if !canvas.selection.is_empty() {
                    if let Some((min, max)) = canvas.selection_bounds() {
                        let screen_min = canvas.viewport.canvas_to_screen(min);
                        let screen_max = canvas.viewport.canvas_to_screen(max);
                        let selection_screen_bounds = Bounds {
                            origin: point(px(screen_min.x()), px(screen_min.y())),
                            size: size(px(screen_max.x() - screen_min.x()), px(screen_max.y() - screen_min.y())),
                        };

                        if let Some(handle) = hit_test_resize_handle(local_vec, selection_screen_bounds) {
                            canvas.start_resize(handle, canvas_pos, cx);
                            return;
                        }
                    }
                }

                // Then check if clicking on a shape
                if let Some(shape_id) = canvas.shape_at_point(canvas_pos) {
                    let add_to_selection = event.modifiers.shift;
                    if !canvas.selection.contains(&shape_id) {
                        canvas.select(shape_id, add_to_selection, cx);
                    }
                    canvas.start_move(canvas_pos, cx);
                } else {
                    canvas.clear_selection(cx);
                }
            }
            Tool::Pan => {
                canvas.start_pan(screen_pos);
            }
            Tool::Rectangle => {
                canvas.start_draw(ShapeKind::Rectangle, canvas_pos, cx);
            }
            Tool::Ellipse => {
                canvas.start_draw(ShapeKind::Ellipse, canvas_pos, cx);
            }
            Tool::Frame => {
                canvas.start_draw(ShapeKind::Frame, canvas_pos, cx);
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
    let screen_pos = ScreenPoint::new(local_x, local_y);

    canvas.update(cx, |canvas, cx| {
        let canvas_pos = canvas.viewport.screen_to_canvas(screen_pos);

        // Clone drag state to avoid borrow issues
        let drag = canvas.drag.clone();

        match drag {
            Some(DragState::MovingShapes { .. }) => {
                canvas.update_move(canvas_pos, cx);
            }
            Some(DragState::ResizingShapes { .. }) => {
                canvas.update_resize(canvas_pos, cx);
            }
            Some(DragState::DrawingShape { shape_id, start }) => {
                // Calculate size and position (handle negative drag)
                let min = Vec2::new(start.x().min(canvas_pos.x()), start.y().min(canvas_pos.y()));
                let max = Vec2::new(start.x().max(canvas_pos.x()), start.y().max(canvas_pos.y()));

                if let Some(shape) = canvas.shapes.iter_mut().find(|s| s.id == shape_id) {
                    shape.position = CanvasPoint(min);
                    shape.size = CanvasSize(max - min);
                }
                cx.notify();
            }
            Some(DragState::Panning { .. }) => {
                canvas.update_pan(screen_pos, cx);
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
    // Handle middle button release for panning
    if event.button == MouseButton::Middle {
        canvas.update(cx, |canvas, _cx| {
            canvas.finish_pan();
        });
        return;
    }

    if event.button != MouseButton::Left {
        return;
    }

    canvas.update(cx, |canvas, cx| {
        match &canvas.drag {
            Some(DragState::MovingShapes { .. }) => {
                canvas.finish_move(cx);
            }
            Some(DragState::ResizingShapes { .. }) => {
                canvas.finish_resize(cx);
            }
            Some(DragState::DrawingShape { .. }) => {
                canvas.finish_draw(cx);
            }
            Some(DragState::Panning { .. }) => {
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
    let (delta_x, delta_y): (f32, f32) = match event.delta {
        ScrollDelta::Pixels(p) => (p.x.into(), p.y.into()),
        ScrollDelta::Lines(l) => (l.x * 20.0, l.y * 20.0), // Approximate pixels per line
    };

    canvas.update(cx, |canvas, cx| {
        // Platform modifier (Cmd on macOS, Ctrl on others) + scroll = zoom
        let zoom_modifier = event.modifiers.platform;
        if zoom_modifier {
            let factor = if delta_y > 0.0 { 1.1 } else { 0.9 };
            canvas.zoom_at(local_pos, factor, cx);
        } else {
            // Regular scroll = pan
            let pan_delta = Vec2::new(delta_x, delta_y);
            canvas.viewport.pan(pan_delta);
            cx.notify();
        }
    });
}

/// Paint a shape and its children recursively.
///
/// For frames with `clip_children` enabled, children are rendered within
/// a content mask that clips to the frame's bounds.
fn paint_shape_recursive(
    shape: &Shape,
    all_shapes: &[Shape],
    selection: &HashSet<ShapeId>,
    hovered: Option<ShapeId>,
    viewport: &crate::Viewport,
    theme: &theme_2::Theme,
    canvas_bounds: Bounds<Pixels>,
    window: &mut Window,
) {
    // Calculate world position for this shape
    let world_pos = shape.world_position(all_shapes);

    // Convert to screen coordinates
    let screen_rect = viewport.canvas_to_screen_bounds(world_pos, shape.size);
    let screen_bounds = Bounds {
        origin: point(
            canvas_bounds.origin.x + px(screen_rect.origin.x),
            canvas_bounds.origin.y + px(screen_rect.origin.y),
        ),
        size: size(px(screen_rect.size.width), px(screen_rect.size.height)),
    };

    // Skip if not visible
    if !canvas_bounds.intersects(&screen_bounds) {
        return;
    }

    // Clamp corner radius to half the smaller dimension
    let max_radius = shape.size.width().min(shape.size.height()) / 2.0;
    let corner_radius = px(shape.corner_radius.min(max_radius) * viewport.zoom);

    // Paint fill
    if let Some(fill) = &shape.fill {
        match shape.kind {
            ShapeKind::Rectangle | ShapeKind::Frame => {
                window.paint_quad(
                    gpui::fill(screen_bounds, fill.color).corner_radii(corner_radius),
                );
            }
            ShapeKind::Ellipse => {
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
        let stroke_width = px(stroke.width * viewport.zoom);
        match shape.kind {
            ShapeKind::Rectangle | ShapeKind::Frame => {
                window.paint_quad(PaintQuad {
                    bounds: screen_bounds,
                    corner_radii: corner_radius.into(),
                    background: transparent_black().into(),
                    border_widths: stroke_width.into(),
                    border_color: stroke.color.into(),
                    border_style: BorderStyle::Solid,
                });
            }
            ShapeKind::Ellipse => {
                let w: f32 = screen_bounds.size.width.into();
                let h: f32 = screen_bounds.size.height.into();
                let radius = px(w.min(h) / 2.0);
                window.paint_quad(PaintQuad {
                    bounds: screen_bounds,
                    corner_radii: radius.into(),
                    background: transparent_black().into(),
                    border_widths: stroke_width.into(),
                    border_color: stroke.color.into(),
                    border_style: BorderStyle::Solid,
                });
            }
        }
    }

    // Paint hover indicator
    if hovered == Some(shape.id) && !selection.contains(&shape.id) {
        window.paint_quad(
            gpui::outline(screen_bounds, theme.hover, BorderStyle::Solid)
                .corner_radii(corner_radius),
        );
    }

    // Paint selection indicator
    if selection.contains(&shape.id) {
        window.paint_quad(
            gpui::outline(screen_bounds, theme.selection, BorderStyle::Solid)
                .corner_radii(corner_radius),
        );
        paint_selection_handles(window, screen_bounds, theme.selection);
    }

    // Paint children (for frames)
    if !shape.children.is_empty() {
        // Optionally clip children to frame bounds
        let clip_mask = if shape.clip_children {
            Some(ContentMask { bounds: screen_bounds })
        } else {
            None
        };

        window.with_content_mask(clip_mask, |window| {
            for child_id in &shape.children {
                if let Some(child) = all_shapes.iter().find(|s| s.id == *child_id) {
                    paint_shape_recursive(
                        child,
                        all_shapes,
                        selection,
                        hovered,
                        viewport,
                        theme,
                        canvas_bounds,
                        window,
                    );
                }
            }
        });
    }
}

/// Compute and apply autolayout for all frames with layout enabled.
///
/// This modifies child positions and sizes according to their parent frame's layout settings.
/// Handles nested layouts by processing from outermost to innermost frames.
fn apply_all_layouts(shapes: &mut [Shape]) {
    // Find all frames with layout enabled, sorted by depth (root frames first)
    let mut frame_ids: Vec<(ShapeId, usize)> = shapes
        .iter()
        .filter(|s| s.has_layout())
        .map(|s| (s.id, compute_depth(s.id, shapes)))
        .collect();

    // Sort by depth (outermost first)
    frame_ids.sort_by_key(|(_, depth)| *depth);

    // Process each frame
    for (frame_id, _) in frame_ids {
        apply_layout_for_frame(frame_id, shapes);
    }
}

/// Compute the nesting depth of a shape (0 = root, 1 = child of root, etc.)
fn compute_depth(shape_id: ShapeId, shapes: &[Shape]) -> usize {
    let mut depth = 0;
    let mut current_id = shape_id;

    while let Some(shape) = shapes.iter().find(|s| s.id == current_id) {
        if let Some(parent_id) = shape.parent {
            depth += 1;
            current_id = parent_id;
        } else {
            break;
        }
    }

    depth
}

/// Apply layout for a single frame to its children.
fn apply_layout_for_frame(frame_id: ShapeId, shapes: &mut [Shape]) {
    // Gather frame info and children IDs in order
    let (frame_size, layout, children_ids) = {
        let Some(frame) = shapes.iter().find(|s| s.id == frame_id) else {
            return;
        };
        let Some(layout) = frame.layout.clone() else {
            return;
        };
        (frame.size, layout, frame.children.clone())
    };

    // Gather children info in the order specified by frame.children
    let child_inputs: Vec<LayoutInput> = children_ids
        .iter()
        .filter_map(|child_id| {
            shapes.iter().find(|s| s.id == *child_id).map(|child| LayoutInput {
                id: child.id,
                size: child.size,
                width_mode: child.child_layout.width_mode,
                height_mode: child.child_layout.height_mode,
            })
        })
        .collect();

    if child_inputs.is_empty() {
        return;
    }

    // Compute layout
    let outputs = compute_layout(frame_size, &layout, &child_inputs);

    // Apply results to children
    for output in outputs {
        if let Some(child) = shapes.iter_mut().find(|s| s.id == output.id) {
            child.position = output.position;
            child.size = output.size;
        }
    }
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
