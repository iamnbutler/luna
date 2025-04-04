#![allow(unused, dead_code)]
use crate::interactivity::{ResizeHandle, ResizeOperation};
use gpui::{
    hsla, prelude::*, px, relative, App, BorderStyle, ContentMask, DispatchPhase, ElementId,
    Entity, Focusable, Hitbox, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Pixels, Style, TextStyle, TextStyleRefinement, TransformationMatrix, Window,
};
use gpui::{point, size, Bounds, Point, Size};
use std::collections::HashSet;

/// Defines z-ordering for rendering layers with reserved index ranges
///
/// Z-indices are allocated in blocks of 10,000 per layer:
/// - Canvas: 10000-19999
/// - CanvasOverlay: 20000-29999
/// - CanvasModal: 30000-39999
/// - UI: 40000-49999
/// - UIOverlay: 50000-59999
/// - UIModal: 60000-69999
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeferIndex {
    /// Canvas elements (base layer) - 10000-19999
    Canvas(usize),
    /// Canvas overlay elements (selection, resize handles) - 20000-29999
    CanvasOverlay(usize),
    /// Canvas modal elements (dialogs specific to canvas) - 30000-39999
    CanvasModal(usize),
    /// UI elements (toolbar, panels) - 40000-49999
    UI(usize),
    /// UI overlay elements (tooltips, dropdowns) - 50000-59999
    UIOverlay(usize),
    /// UI modal elements (dialogs, popups) - 60000-69999
    UIModal(usize),
}

impl DeferIndex {
    // Base values for each layer
    const CANVAS_BASE: usize = 10000;
    const CANVAS_OVERLAY_BASE: usize = 20000;
    const CANVAS_MODAL_BASE: usize = 30000;
    const UI_BASE: usize = 40000;
    const UI_OVERLAY_BASE: usize = 50000;
    const UI_MODAL_BASE: usize = 60000;

    // Maximum allowed index within a layer (10,000 values per layer)
    const MAX_LAYER_INDEX: usize = 9999;

    /// Returns the absolute priority value for use with GPUI's defer_draw
    pub fn priority(&self) -> usize {
        match *self {
            DeferIndex::Canvas(idx) => {
                assert!(idx <= Self::MAX_LAYER_INDEX, "Canvas index out of range");
                Self::CANVAS_BASE + idx
            }
            DeferIndex::CanvasOverlay(idx) => {
                assert!(
                    idx <= Self::MAX_LAYER_INDEX,
                    "CanvasOverlay index out of range"
                );
                Self::CANVAS_OVERLAY_BASE + idx
            }
            DeferIndex::CanvasModal(idx) => {
                assert!(
                    idx <= Self::MAX_LAYER_INDEX,
                    "CanvasModal index out of range"
                );
                Self::CANVAS_MODAL_BASE + idx
            }
            DeferIndex::UI(idx) => {
                assert!(idx <= Self::MAX_LAYER_INDEX, "UI index out of range");
                Self::UI_BASE + idx
            }
            DeferIndex::UIOverlay(idx) => {
                assert!(idx <= Self::MAX_LAYER_INDEX, "UIOverlay index out of range");
                Self::UI_OVERLAY_BASE + idx
            }
            DeferIndex::UIModal(idx) => {
                assert!(idx <= Self::MAX_LAYER_INDEX, "UIModal index out of range");
                Self::UI_MODAL_BASE + idx
            }
        }
    }

    /// Canvas layer constants
    pub const CANVAS_BG: Self = Self::Canvas(0);
    pub const CANVAS_NODES: Self = Self::Canvas(1000);

    /// Canvas overlay constants
    pub const SELECTION_OUTLINE: Self = Self::CanvasOverlay(0);
    pub const RESIZE_HANDLES: Self = Self::CanvasOverlay(100);
}

/// Tests for AABB intersection between two bounds
fn bounds_intersect(a: &Bounds<f32>, b: &Bounds<f32>) -> bool {
    // Check if one rectangle is to the left of the other
    if a.origin.x + a.size.width < b.origin.x || b.origin.x + b.size.width < a.origin.x {
        return false;
    }

    // Check if one rectangle is above the other
    if a.origin.y + a.size.height < b.origin.y || b.origin.y + b.size.height < a.origin.y {
        return false;
    }

    true
}

/// Detects if a point intersects with a resize handle on the node boundaries
fn point_in_resize_handle(point: Point<f32>, node_bounds: &Bounds<f32>) -> Option<ResizeHandle> {
    use ResizeHandle;

    // Define handle size and boundaries
    const HANDLE_SIZE: f32 = 7.0;
    const HALF_HANDLE: f32 = HANDLE_SIZE / 2.0;

    // Create bounds for each corner handle
    let handles = [
        // Top left
        (
            Bounds {
                origin: Point::new(
                    node_bounds.origin.x - HALF_HANDLE,
                    node_bounds.origin.y - HALF_HANDLE,
                ),
                size: Size::new(HANDLE_SIZE, HANDLE_SIZE),
            },
            ResizeHandle::TopLeft,
        ),
        // Top right
        (
            Bounds {
                origin: Point::new(
                    node_bounds.origin.x + node_bounds.size.width - HALF_HANDLE,
                    node_bounds.origin.y - HALF_HANDLE,
                ),
                size: Size::new(HANDLE_SIZE, HANDLE_SIZE),
            },
            ResizeHandle::TopRight,
        ),
        // Bottom left
        (
            Bounds {
                origin: Point::new(
                    node_bounds.origin.x - HALF_HANDLE,
                    node_bounds.origin.y + node_bounds.size.height - HALF_HANDLE,
                ),
                size: Size::new(HANDLE_SIZE, HANDLE_SIZE),
            },
            ResizeHandle::BottomLeft,
        ),
        // Bottom right
        (
            Bounds {
                origin: Point::new(
                    node_bounds.origin.x + node_bounds.size.width - HALF_HANDLE,
                    node_bounds.origin.y + node_bounds.size.height - HALF_HANDLE,
                ),
                size: Size::new(HANDLE_SIZE, HANDLE_SIZE),
            },
            ResizeHandle::BottomRight,
        ),
    ];

    // Check if point is inside any handle
    for (bounds, handle) in handles {
        if bounds.contains(&point) {
            return Some(handle);
        }
    }

    None
}

use crate::scene_graph::SceneGraph;
use crate::theme::Theme;
use crate::AppState;
use crate::{
    canvas::{register_canvas_action, ClearSelection, LunaCanvas},
    interactivity::{ActiveDrag, DragType},
    node::{NodeCommon, NodeId, NodeLayout, NodeType, RectangleNode},
    util::{round_to_pixel, rounded_point},
    GlobalState, ToolKind,
};

#[derive(Clone)]
pub struct CanvasStyle {
    pub background: Hsla,
    pub cursor_color: Hsla,
    pub scrollbar_thickness: Pixels,
    pub text: TextStyle,
}

impl CanvasStyle {
    pub fn new(cx: &App) -> Self {
        let theme = Theme::get_global(cx);

        Self {
            background: theme.tokens.background,
            cursor_color: theme.tokens.cursor,
            ..Default::default()
        }
    }
}

impl Default for CanvasStyle {
    fn default() -> Self {
        Self {
            background: hsla(0.0, 0.0, 0.0, 1.0),
            cursor_color: hsla(0.0, 0.0, 1.0, 1.0),
            scrollbar_thickness: px(6.0),
            text: TextStyle::default(),
        }
    }
}

pub struct CanvasLayout {
    hitbox: Hitbox,
}

/// CanvasElement uses  prefixes for identifying the role of methods within the canvas.
///
/// - handle_: handle user input events
/// - layout_: layout elements within the canvas
/// - paint_: paint elements within the canvas
/// - data_:  returns some derived data for other methods to use within the canvas
pub struct CanvasElement {
    canvas: Entity<LunaCanvas>,
    style: CanvasStyle,
}

impl CanvasElement {
    pub fn new(
        canvas: &Entity<LunaCanvas>,
        scene_graph: &Entity<SceneGraph>,
        cx: &mut App,
    ) -> Self {
        let style = CanvasStyle::new(cx);

        Self {
            canvas: canvas.clone(),
            style,
        }
    }

    pub fn register_actions(&self, window: &mut Window, cx: &mut App) {
        let canvas = &self.canvas;
        canvas.update(cx, |canvas, cx| {
            for action in canvas.actions.borrow().values() {
                (action)(window, cx)
            }
        });

        register_canvas_action(canvas, window, LunaCanvas::clear_selection);
    }

    // handle_mouse_down, etc
    // Helper function to find the top node at a given point using scene graph for efficiency
    fn find_top_node_at_point(
        canvas: &LunaCanvas,
        window_point: Point<f32>,
        cx: &Context<LunaCanvas>,
    ) -> Option<NodeId> {
        // Convert window coordinate to canvas coordinate
        let canvas_point = canvas.window_to_canvas_point(window_point);

        // Direct node testing with 1x1 selection point for hit detection
        let select_point_bounds = Bounds {
            origin: canvas_point,
            size: Size::new(1.0, 1.0),
        };

        // Test each node to see if it contains this point
        // Iterate in reverse order to match the painting order (last node is visually on top)
        for node in canvas.nodes.iter().rev() {
            let node_bounds = node.bounds();
            if node_bounds.contains(&canvas_point) {
                return Some(node.id());
            }
        }

        None
    }

    fn handle_left_mouse_down(
        canvas: &mut LunaCanvas,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<LunaCanvas>,
    ) {
        if window.default_prevented() {
            return;
        }

        let position = event.position;
        let canvas_point = point(position.x.0, position.y.0);

        match canvas.active_tool {
            ToolKind::Selection => {
                // First, check if we've clicked on a resize handle when only a single node is selected
                if canvas.selected_nodes.len() == 1 {
                    // Get the bounds of the selected node
                    let selected_node_id = *canvas.selected_nodes.iter().next().unwrap();
                    if let Some(node) = canvas.nodes.iter().find(|n| n.id() == selected_node_id) {
                        let node_layout = node.layout();

                        // Create node bounds to check for resize handle hits
                        let node_bounds = Bounds {
                            origin: Point::new(node_layout.x, node_layout.y),
                            size: Size::new(node_layout.width, node_layout.height),
                        };

                        // Convert canvas point to world coordinates for hit detection
                        let world_point = canvas.window_to_canvas_point(canvas_point);

                        // Check if the point is within any resize handle
                        if let Some(handle) = point_in_resize_handle(world_point, &node_bounds) {
                            // Create a resize operation with the original node dimensions
                            let resize_op = ResizeOperation::new(
                                handle,
                                node_layout.x,
                                node_layout.y,
                                node_layout.width,
                                node_layout.height,
                            );

                            // Start a resize drag operation
                            canvas.active_drag = Some(ActiveDrag::new_resize(position, resize_op));
                            canvas.mark_dirty(cx);
                            cx.stop_propagation();
                            return;
                        }
                    }
                }

                // If we didn't hit a resize handle, proceed with normal selection behavior
                // Attempt to find a node at the clicked point
                if let Some(node_id) = Self::find_top_node_at_point(canvas, canvas_point, cx) {
                    // Check if we clicked on a node that's already selected
                    let already_selected = canvas.is_node_selected(node_id);

                    // If shift is not pressed, clear current selection first (unless clicking on already selected)
                    let modifiers = event.modifiers;
                    if !modifiers.shift && !already_selected {
                        canvas.clear_selection(&ClearSelection, window, cx);
                    }

                    // If shift is pressed and node is already selected, deselect it
                    if modifiers.shift && already_selected {
                        canvas.deselect_node(node_id);
                    } else {
                        // Otherwise select the node
                        canvas.select_node(node_id);
                    }

                    // If we clicked on a selected node, we should start dragging it
                    if canvas.is_node_selected(node_id) {
                        // Save initial positions of all selected elements
                        canvas.save_selected_nodes_positions();

                        // Start a move elements drag operation
                        canvas.active_drag = Some(ActiveDrag::new_move_elements(position));
                    }

                    canvas.mark_dirty(cx);
                } else {
                    // Clicked on empty space - start a selection rectangle drag
                    // First clear selection if shift is not pressed
                    if !event.modifiers.shift {
                        canvas.clear_selection(&ClearSelection, window, cx);
                    }

                    canvas.active_drag = Some(ActiveDrag::new_selection(position));
                    canvas.mark_dirty(cx);
                }
            }
            ToolKind::Rectangle => {
                // Use the generate_id method directly since it already returns the correct type
                let new_node_id = canvas.generate_id();

                let active_drag = ActiveDrag::new_create_element(position);
                canvas.active_element_draw = Some((new_node_id, NodeType::Rectangle, active_drag));
                canvas.mark_dirty(cx);
            }
            _ => {}
        }

        cx.stop_propagation();
    }

    fn handle_left_mouse_up(
        canvas: &mut LunaCanvas,
        event: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<LunaCanvas>,
    ) {
        // check if selection is pending
        // if so, clear it and fire any selection events

        let position = event.position;
        let canvas_point = point(position.x.0, position.y.0);
        let state = GlobalState::get(cx);
        let app_state = canvas.app_state().clone().read(cx);
        let current_background_color = app_state.current_background_color.clone();
        let current_border_color = app_state.current_border_color.clone();

        // Check if we have an active element draw operation
        if let Some((node_id, node_type, active_drag)) = canvas.active_element_draw.take() {
            match (node_type, &canvas.active_tool) {
                (NodeType::Rectangle, ToolKind::Rectangle) => {
                    // Calculate rectangle dimensions
                    let start_pos = active_drag.start_position;
                    let end_pos = active_drag.current_position;

                    let min_x = start_pos.x.0.min(end_pos.x.0);
                    let min_y = start_pos.y.0.min(end_pos.y.0);
                    let width = (start_pos.x.0 - end_pos.x.0).abs();
                    let height = (start_pos.y.0 - end_pos.y.0).abs();

                    // Only create a rectangle if it has meaningful dimensions
                    if width >= 2.0 && height >= 2.0 {
                        // Convert window coordinates to canvas coordinates
                        let canvas_point = canvas.window_to_canvas_point(Point::new(min_x, min_y));
                        let rel_x = canvas_point.x;
                        let rel_y = canvas_point.y;

                        // Create a new rectangle node
                        let mut rect = RectangleNode::new(node_id);

                        // Set position and size
                        *rect.layout_mut() = NodeLayout::new(rel_x, rel_y, width, height);

                        // Set colors
                        rect.set_fill(Some(current_background_color));
                        rect.set_border(Some(current_border_color), 1.0);

                        // Add the node to the canvas
                        canvas.add_node(rect, cx);
                        canvas.mark_dirty(cx);
                    }
                }
                _ => {}
            }
        }

        // Handle ending drag operations
        if let Some(active_drag) = canvas.active_drag.take() {
            match active_drag.drag_type {
                DragType::MoveElements => {
                    // Finalize the move by clearing initial positions
                    canvas.element_initial_positions.clear();
                }
                DragType::Selection => {
                    // Selection handling is already done in the drag handler
                }
                DragType::CreateElement => {
                    // Element creation is handled above
                }
                DragType::Resize(_) => {
                    // Finalize the resize operation - nothing special needed here
                    // The resize has already been applied to the node during drag
                }
            }
        }

        cx.stop_propagation();
    }

    fn handle_mouse_drag(
        canvas: &mut LunaCanvas,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<LunaCanvas>,
    ) {
        let position = event.position;
        let canvas_point = point(position.x.0, position.y.0);

        // Handle active drag operations
        if let Some(active_drag) = canvas.active_drag.take() {
            // Update the drag with new position
            let new_drag = ActiveDrag {
                start_position: active_drag.start_position,
                current_position: position,
                drag_type: active_drag.drag_type.clone(),
            };
            canvas.active_drag = Some(new_drag.clone());

            match new_drag.drag_type {
                DragType::Selection => {
                    // Handle selection rectangle
                    if canvas.active_tool == ToolKind::Selection {
                        // Calculate the selection rectangle in canvas coordinates
                        let start_pos = active_drag.start_position;
                        let min_x = start_pos.x.0.min(position.x.0);
                        let min_y = start_pos.y.0.min(position.y.0);
                        let max_x = start_pos.x.0.max(position.x.0);
                        let max_y = start_pos.y.0.max(position.y.0);

                        // Convert to canvas coordinates
                        let min_point = canvas.window_to_canvas_point(Point::new(min_x, min_y));
                        let max_point = canvas.window_to_canvas_point(Point::new(max_x, max_y));

                        // Create selection bounds
                        let selection_bounds = Bounds {
                            origin: min_point,
                            size: Size::new(max_point.x - min_point.x, max_point.y - min_point.y),
                        };

                        // Pre-calculate all nodes that intersect with selection
                        let nodes_in_selection: HashSet<NodeId> = canvas
                            .nodes
                            .iter()
                            .filter(|node| bounds_intersect(&selection_bounds, &node.bounds()))
                            .map(|node| node.id())
                            .collect();

                        // Check if we want to add to existing selection (shift pressed)
                        // or replace it (shift not pressed)
                        if event.modifiers.shift {
                            // Add new nodes to selection
                            for node_id in nodes_in_selection {
                                canvas.select_node(node_id);
                            }
                        } else {
                            // Replace selection
                            if nodes_in_selection != canvas.selected_nodes {
                                canvas.clear_selection(&ClearSelection, window, cx);
                                for node_id in nodes_in_selection {
                                    canvas.select_node(node_id);
                                }
                            }
                        }
                    }
                }
                DragType::MoveElements => {
                    // Move selected elements based on drag delta
                    if !canvas.selected_nodes.is_empty() {
                        // Calculate the drag delta in canvas coordinates
                        let delta = new_drag.delta();

                        // Move all selected nodes with the drag delta
                        canvas.move_selected_nodes_with_drag(delta, cx);
                    }
                }
                DragType::CreateElement => {
                    // Nothing to do here - handled in the rectangle drawing code below
                }
                DragType::Resize(mut resize_op) => {
                    // Handle resize operation
                    if canvas.selected_nodes.len() == 1 {
                        // Get the zoom value before any mutable borrows
                        let zoom = canvas.zoom();

                        // Get the selected node
                        let selected_node_id = *canvas.selected_nodes.iter().next().unwrap();
                        if let Some(node) =
                            canvas.nodes.iter_mut().find(|n| n.id() == selected_node_id)
                        {
                            // Convert window delta to canvas delta
                            let delta = Point::new(
                                (position.x.0 - active_drag.start_position.x.0) / zoom,
                                (position.y.0 - active_drag.start_position.y.0) / zoom,
                            );

                            // Check modifiers: shift for aspect ratio, option (alt) for resize from center
                            let preserve_aspect_ratio = event.modifiers.shift;
                            let resize_from_center = event.modifiers.alt;

                            // Update resize config
                            resize_op.config.preserve_aspect_ratio = preserve_aspect_ratio;
                            resize_op.config.resize_from_center = resize_from_center;

                            // Calculate new dimensions based on resize handle and modifiers
                            let mut new_x = resize_op.original_x;
                            let mut new_y = resize_op.original_y;
                            let mut new_width = resize_op.original_width;
                            let mut new_height = resize_op.original_height;

                            // Calculate aspect ratio if needed
                            let aspect_ratio = if preserve_aspect_ratio {
                                resize_op.original_width / resize_op.original_height
                            } else {
                                0.0 // Not used when not preserving aspect ratio
                            };

                            // Adjust dimensions based on which handle is being dragged
                            match resize_op.handle {
                                ResizeHandle::TopLeft => {
                                    // Width/height change is negative of delta for top-left
                                    let width_delta = -delta.x;
                                    let height_delta = -delta.y;

                                    if preserve_aspect_ratio {
                                        // Use whichever delta would make the shape larger
                                        if width_delta.abs() / aspect_ratio > height_delta.abs() {
                                            let adj_height = width_delta / aspect_ratio;
                                            new_width = resize_op.original_width + width_delta;
                                            new_height = resize_op.original_height + adj_height;
                                            new_x = resize_op.original_x - width_delta;
                                            new_y = resize_op.original_y - adj_height;
                                        } else {
                                            let adj_width = height_delta * aspect_ratio;
                                            new_width = resize_op.original_width + adj_width;
                                            new_height = resize_op.original_height + height_delta;
                                            new_x = resize_op.original_x - adj_width;
                                            new_y = resize_op.original_y - height_delta;
                                        }
                                    } else {
                                        // Standard resize without aspect ratio constraint
                                        new_width = resize_op.original_width + width_delta;
                                        new_height = resize_op.original_height + height_delta;
                                        new_x = resize_op.original_x - width_delta;
                                        new_y = resize_op.original_y - height_delta;
                                    }
                                }
                                ResizeHandle::TopRight => {
                                    // Width change is positive, height change is negative
                                    let width_delta = delta.x;
                                    let height_delta = -delta.y;

                                    if preserve_aspect_ratio {
                                        if width_delta.abs() / aspect_ratio > height_delta.abs() {
                                            let adj_height = width_delta / aspect_ratio;
                                            new_width = resize_op.original_width + width_delta;
                                            new_height = resize_op.original_height + adj_height;
                                            new_y = resize_op.original_y - adj_height;
                                        } else {
                                            let adj_width = height_delta * aspect_ratio;
                                            new_width = resize_op.original_width + adj_width;
                                            new_height = resize_op.original_height + height_delta;
                                            new_y = resize_op.original_y - height_delta;
                                        }
                                    } else {
                                        new_width = resize_op.original_width + width_delta;
                                        new_height = resize_op.original_height + height_delta;
                                        new_y = resize_op.original_y - height_delta;
                                    }
                                }
                                ResizeHandle::BottomLeft => {
                                    // Width change is negative, height change is positive
                                    let width_delta = -delta.x;
                                    let height_delta = delta.y;

                                    if preserve_aspect_ratio {
                                        if width_delta.abs() / aspect_ratio > height_delta.abs() {
                                            let adj_height = width_delta / aspect_ratio;
                                            new_width = resize_op.original_width + width_delta;
                                            new_height = resize_op.original_height + adj_height;
                                            new_x = resize_op.original_x - width_delta;
                                        } else {
                                            let adj_width = height_delta * aspect_ratio;
                                            new_width = resize_op.original_width + adj_width;
                                            new_height = resize_op.original_height + height_delta;
                                            new_x = resize_op.original_x - adj_width;
                                        }
                                    } else {
                                        new_width = resize_op.original_width + width_delta;
                                        new_height = resize_op.original_height + height_delta;
                                        new_x = resize_op.original_x - width_delta;
                                    }
                                }
                                ResizeHandle::BottomRight => {
                                    let width_delta = delta.x;
                                    let height_delta = delta.y;

                                    if preserve_aspect_ratio {
                                        if width_delta.abs() / aspect_ratio > height_delta.abs() {
                                            let adj_height = width_delta / aspect_ratio;
                                            new_width = resize_op.original_width + width_delta;
                                            new_height = resize_op.original_height + adj_height;
                                        } else {
                                            let adj_width = height_delta * aspect_ratio;
                                            new_width = resize_op.original_width + adj_width;
                                            new_height = resize_op.original_height + height_delta;
                                        }
                                    } else {
                                        new_width = resize_op.original_width + width_delta;
                                        new_height = resize_op.original_height + height_delta;
                                    }
                                }
                            }

                            // If resize from center is enabled, adjust position to keep center fixed
                            if resize_from_center {
                                let orig_center_x =
                                    resize_op.original_x + resize_op.original_width / 2.0;
                                let orig_center_y =
                                    resize_op.original_y + resize_op.original_height / 2.0;
                                new_x = orig_center_x - new_width / 2.0;
                                new_y = orig_center_y - new_height / 2.0;
                            }

                            // Ensure minimum dimensions (prevent negative width/height)
                            if new_width > 1.0 && new_height > 1.0 {
                                // Update node dimensions
                                let layout = node.layout_mut();
                                layout.x = new_x;
                                layout.y = new_y;
                                layout.width = new_width;
                                layout.height = new_height;

                                // Update scene graph
                                if let Some(scene_node_id) = canvas
                                    .scene_graph()
                                    .update(cx, |sg, _cx| sg.get_scene_node_id(selected_node_id))
                                {
                                    canvas.scene_graph().update(cx, |sg, _cx| {
                                        sg.set_local_bounds(
                                            scene_node_id,
                                            Bounds {
                                                origin: Point::new(new_x, new_y),
                                                size: Size::new(new_width, new_height),
                                            },
                                        );
                                    });
                                }
                            }

                            // Update the resize operation in the drag
                            let updated_drag = ActiveDrag {
                                start_position: active_drag.start_position,
                                current_position: position,
                                drag_type: DragType::Resize(resize_op),
                            };
                            canvas.active_drag = Some(updated_drag);
                        }
                    }
                }
            }

            canvas.mark_dirty(cx);
        }

        // Handle rectangle drawing
        if let Some(active_draw) = canvas.active_element_draw.take() {
            match canvas.active_tool {
                ToolKind::Rectangle => {
                    let new_drag = ActiveDrag {
                        start_position: active_draw.2.start_position,
                        current_position: position,
                        drag_type: DragType::CreateElement,
                    };
                    canvas.active_element_draw = Some((active_draw.0, active_draw.1, new_drag));
                    canvas.mark_dirty(cx);
                }
                _ => {}
            }
        }
    }

    fn handle_mouse_move(
        canvas: &mut LunaCanvas,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<LunaCanvas>,
    ) {
        let position = event.position;
        let canvas_point = point(position.x.0, position.y.0);

        // Find node under cursor for hover effect
        let hovered = Self::find_top_node_at_point(canvas, canvas_point, cx);

        // Only update and redraw if hover state changed
        if canvas.hovered_node != hovered {
            canvas.hovered_node = hovered;
            canvas.mark_dirty(cx);
        }
    }

    fn paint_selection(
        &self,
        active_drag: &ActiveDrag,
        layout: &CanvasLayout,
        window: &mut Window,
        theme: &Theme,
    ) {
        // Only draw selection rectangle if this is actually a selection drag
        // Don't draw it when dragging elements
        match active_drag.drag_type {
            DragType::Selection => {
                // Continue with drawing the selection rectangle
            }
            _ => return, // Don't draw for other drag types
        }

        let min_x = round_to_pixel(
            active_drag
                .start_position
                .x
                .min(active_drag.current_position.x),
        );
        let min_y = round_to_pixel(
            active_drag
                .start_position
                .y
                .min(active_drag.current_position.y),
        );
        let width =
            round_to_pixel((active_drag.start_position.x - active_drag.current_position.x).abs());
        let height =
            round_to_pixel((active_drag.start_position.y - active_drag.current_position.y).abs());

        window.paint_layer(layout.hitbox.bounds, |window| {
            let position = rounded_point(min_x, min_y);

            let rect_bounds = Bounds {
                origin: position,
                size: Size::new(width, height),
            };

            window.paint_quad(gpui::fill(rect_bounds, theme.tokens.overlay2.opacity(0.25)));
            window.paint_quad(gpui::outline(
                rect_bounds,
                theme.tokens.active_border,
                BorderStyle::Solid,
            ));
            window.request_animation_frame();
        });
    }

    /// Paint a retangular element like a rectangle, square or frame
    /// as it is being created by clicking and dragging the tool
    fn paint_draw_rectangle(
        &self,
        new_node_id: NodeId,
        active_drag: &ActiveDrag,
        layout: &CanvasLayout,
        window: &mut Window,
        cx: &App,
    ) {
        // Get the raw cursor positions directly from the drag event
        // These are in absolute window coordinates where the mouse is positioned
        let start_pos = active_drag.start_position;
        let current_pos = active_drag.current_position;

        // Calculate rectangle bounds in window coordinates
        // Don't round yet to avoid accumulating rounding errors
        let min_x = start_pos.x.min(current_pos.x);
        let min_y = start_pos.y.min(current_pos.y);
        let width = (start_pos.x - current_pos.x).abs();
        let height = (start_pos.y - current_pos.y).abs();

        // Round once after all coordinate conversions for pixel-perfect rendering
        let position = rounded_point(min_x, min_y);

        let rect_bounds = Bounds {
            origin: position,
            size: Size::new(width, height),
        };

        // Read canvas and app_state separately to avoid multiple borrows
        let canvas_read = self.canvas.read(cx);
        let app_state_entity = canvas_read.app_state().clone();

        let app_state = app_state_entity.read(cx);

        window.paint_quad(gpui::fill(rect_bounds, app_state.current_background_color));
        window.paint_quad(gpui::outline(
            rect_bounds,
            app_state.current_border_color,
            BorderStyle::Solid,
        ));
        window.request_animation_frame();
    }

    /// Paint the background layer of the canvas.
    ///
    /// Everything on this layer has the same draw order.
    pub fn paint_canvas_background(
        &self,
        layout: &CanvasLayout,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.paint_layer(layout.hitbox.bounds, |window| {
            window.paint_quad(gpui::fill(layout.hitbox.bounds, self.style.background));
        });
    }

    /// Register mouse listeners like click, hover and drag events.
    ///
    /// Despite not being visually "painted", mouse listeners are registered
    /// using `window.on_{}_event`, which is only available in the paint phase.
    ///
    /// Thus the `paint` prefix.
    fn paint_mouse_listeners(&mut self, layout: &CanvasLayout, window: &mut Window, cx: &mut App) {
        window.on_mouse_event({
            let canvas = self.canvas.clone();
            move |event: &MouseDownEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble {
                    match event.button {
                        MouseButton::Left => canvas.update(cx, |canvas, cx| {
                            Self::handle_left_mouse_down(canvas, event, window, cx);
                        }),
                        MouseButton::Right => canvas.update(cx, |canvas, cx| {
                            // todo
                        }),
                        _ => {}
                    }
                }
            }
        });

        window.on_mouse_event({
            let canvas = self.canvas.clone();
            move |event: &MouseUpEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble {
                    match event.button {
                        MouseButton::Left => canvas.update(cx, |canvas, cx| {
                            Self::handle_left_mouse_up(canvas, event, window, cx)
                        }),
                        MouseButton::Right => canvas.update(cx, |canvas, cx| {
                            // todo
                        }),
                        _ => {}
                    }
                }
            }
        });

        window.on_mouse_event({
            let canvas = self.canvas.clone();
            move |event: &MouseMoveEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble {
                    canvas.update(cx, |canvas, cx| {
                        if event.pressed_button == Some(MouseButton::Left)
                            || event.pressed_button == Some(MouseButton::Middle)
                        {
                            Self::handle_mouse_drag(canvas, event, window, cx)
                        }

                        Self::handle_mouse_move(canvas, event, window, cx)
                    });
                }
            }
        });
    }

    fn paint_nodes(&self, layout: &CanvasLayout, window: &mut Window, cx: &mut App) {
        let canvas = self.canvas.clone();

        // Collect ALL data we need up front to avoid any borrow issues
        struct NodeRenderInfo {
            node_id: NodeId,
            bounds: gpui::Bounds<Pixels>,
            fill_color: Option<Hsla>,
            border_color: Option<Hsla>,
            border_width: f32,
            corner_radius: f32,
        }

        // Get all the data we need in one place
        let (nodes_to_render, theme, selected_node_ids, hovered_node) =
            canvas.update(cx, |canvas, cx| {
                let visible_nodes = canvas.visible_nodes(cx);
                let scene_graph = canvas.scene_graph().read(cx);
                let selected_nodes = canvas.selected_nodes.clone();
                let theme = canvas.theme.clone();
                let hovered_node = canvas.hovered_node.clone();

                // Collect all node rendering information into owned structures
                let mut nodes_to_render = Vec::new();

                for node in visible_nodes {
                    let node_id = node.id();

                    if let Some(scene_node_id) = scene_graph.get_scene_node_id(node_id) {
                        if let Some(world_bounds) = scene_graph.get_world_bounds(scene_node_id) {
                            nodes_to_render.push(NodeRenderInfo {
                                node_id,
                                bounds: gpui::Bounds {
                                    origin: gpui::Point::new(
                                        gpui::Pixels(world_bounds.origin.x),
                                        gpui::Pixels(world_bounds.origin.y),
                                    ),
                                    size: gpui::Size::new(
                                        gpui::Pixels(world_bounds.size.width),
                                        gpui::Pixels(world_bounds.size.height),
                                    ),
                                },
                                fill_color: node.fill(),
                                border_color: node.border_color(),
                                border_width: node.border_width(),
                                corner_radius: node.corner_radius(),
                            });
                        }
                    }
                }

                (nodes_to_render, theme, selected_nodes, hovered_node)
            });

        window.paint_layer(layout.hitbox.bounds, |window| {
            // FIRST PASS: Paint all nodes and hover effects
            // ==============================================
            for node_info in &nodes_to_render {
                // Paint the fill if it exists
                if let Some(fill_color) = node_info.fill_color {
                    window.paint_quad(gpui::PaintQuad {
                        bounds: node_info.bounds,
                        corner_radii: (node_info.corner_radius).into(),
                        background: fill_color.into(),
                        border_widths: (0.).into(),
                        border_color: gpui::transparent_black().into(),
                        border_style: BorderStyle::Solid,
                    });
                }

                // Paint the border if it exists
                if let Some(border_color) = node_info.border_color {
                    window.paint_quad(gpui::PaintQuad {
                        bounds: node_info.bounds,
                        corner_radii: (node_info.corner_radius).into(),
                        background: gpui::transparent_black().into(),
                        border_widths: (node_info.border_width).into(),
                        border_color: border_color.into(),
                        border_style: BorderStyle::Solid,
                    });
                }

                // Process hover effects (only for non-selected nodes)
                if hovered_node.as_ref() == Some(&node_info.node_id)
                    && !selected_node_ids.contains(&node_info.node_id)
                {
                    // Create a slightly larger bounds for hover indicator
                    let hover_bounds = gpui::Bounds {
                        origin: gpui::Point::new(
                            node_info.bounds.origin.x - gpui::Pixels(2.0),
                            node_info.bounds.origin.y - gpui::Pixels(2.0),
                        ),
                        size: gpui::Size::new(
                            node_info.bounds.size.width + gpui::Pixels(4.0),
                            node_info.bounds.size.height + gpui::Pixels(4.0),
                        ),
                    };

                    let hover_color = theme.tokens.active_border.opacity(0.6);
                    window.paint_quad(gpui::outline(hover_bounds, hover_color, BorderStyle::Solid));
                }
            }

            // SECOND PASS: Paint all selection outlines and resize handles
            // ===========================================================

            // First draw individual selection outlines
            for node_info in &nodes_to_render {
                if selected_node_ids.contains(&node_info.node_id) {
                    // Create a slightly larger bounds for selection indicator
                    let selection_bounds = gpui::Bounds {
                        origin: gpui::Point::new(
                            node_info.bounds.origin.x - gpui::Pixels(2.0),
                            node_info.bounds.origin.y - gpui::Pixels(2.0),
                        ),
                        size: gpui::Size::new(
                            node_info.bounds.size.width + gpui::Pixels(4.0),
                            node_info.bounds.size.height + gpui::Pixels(4.0),
                        ),
                    };

                    // Reduce outline opacity to 20% when multiple elements are selected
                    let selection_color = if selected_node_ids.len() > 1 {
                        theme.tokens.active_border.opacity(0.2)
                    } else {
                        theme.tokens.active_border
                    };

                    window.paint_quad(gpui::outline(
                        selection_bounds,
                        selection_color,
                        BorderStyle::Solid,
                    ));

                    // Only draw resize handles if this is the only selected node
                    if selected_node_ids.len() == 1 {
                        const HANDLE_SIZE: f32 = 7.0;
                        const HALF_HANDLE: f32 = HANDLE_SIZE / 2.0;

                        // Center handles on the selection outline
                        let corners = [
                            // Top-left
                            (
                                selection_bounds.origin.x - gpui::Pixels(HALF_HANDLE - 0.5),
                                selection_bounds.origin.y - gpui::Pixels(HALF_HANDLE - 0.5),
                            ),
                            // Top-right
                            (
                                selection_bounds.origin.x + selection_bounds.size.width
                                    - gpui::Pixels(HALF_HANDLE + 0.5),
                                selection_bounds.origin.y - gpui::Pixels(HALF_HANDLE - 0.5),
                            ),
                            // Bottom-left
                            (
                                selection_bounds.origin.x - gpui::Pixels(HALF_HANDLE - 0.5),
                                selection_bounds.origin.y + selection_bounds.size.height
                                    - gpui::Pixels(HALF_HANDLE + 0.5),
                            ),
                            // Bottom-right
                            (
                                selection_bounds.origin.x + selection_bounds.size.width
                                    - gpui::Pixels(HALF_HANDLE + 0.5),
                                selection_bounds.origin.y + selection_bounds.size.height
                                    - gpui::Pixels(HALF_HANDLE + 0.5),
                            ),
                        ];

                        for (x, y) in corners {
                            let handle_bounds = gpui::Bounds {
                                origin: gpui::Point::new(x, y),
                                size: gpui::Size::new(
                                    gpui::Pixels(HANDLE_SIZE),
                                    gpui::Pixels(HANDLE_SIZE),
                                ),
                            };

                            window.paint_quad(gpui::fill(
                                handle_bounds,
                                gpui::hsla(0.0, 0.0, 1.0, 1.0),
                            ));
                            window.paint_quad(gpui::outline(
                                handle_bounds,
                                selection_color,
                                BorderStyle::Solid,
                            ));
                        }
                    }
                }
            }

            // THIRD PASS: Draw the group selection outline (if multiple nodes are selected)
            // =========================================================================
            if selected_node_ids.len() > 1 {
                // Calculate the axis-aligned bounding box (AABB) that contains all selected elements
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;

                for node_info in &nodes_to_render {
                    if selected_node_ids.contains(&node_info.node_id) {
                        min_x = min_x.min(node_info.bounds.origin.x.0);
                        min_y = min_y.min(node_info.bounds.origin.y.0);
                        max_x =
                            max_x.max(node_info.bounds.origin.x.0 + node_info.bounds.size.width.0);
                        max_y =
                            max_y.max(node_info.bounds.origin.y.0 + node_info.bounds.size.height.0);
                    }
                }

                // Only draw if we found valid bounds
                if min_x != f32::MAX && min_y != f32::MAX && max_x != f32::MIN && max_y != f32::MIN
                {
                    // Create the group selection bounds with some padding
                    let group_selection_bounds = gpui::Bounds {
                        origin: gpui::Point::new(
                            gpui::Pixels(min_x - 5.0),
                            gpui::Pixels(min_y - 5.0),
                        ),
                        size: gpui::Size::new(
                            gpui::Pixels(max_x - min_x + 10.0),
                            gpui::Pixels(max_y - min_y + 10.0),
                        ),
                    };

                    window.paint_quad(gpui::outline(
                        group_selection_bounds,
                        theme.tokens.active_border,
                        BorderStyle::Solid,
                    ));
                }
            }

            window.request_animation_frame();
        });
    }
}

impl Element for CanvasElement {
    type RequestLayoutState = ();
    type PrepaintState = CanvasLayout;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> (gpui::LayoutId, ()) {
        // prepare the overall dimensions of the canvas before
        // we prepaint it
        self.canvas.update(cx, |canvas, cx| {
            let layout_id = {
                let mut style = Style::default();
                // TODO: impl actual size
                style.size.height = relative(1.).into();
                style.size.width = relative(1.).into();
                // style.size.height = px(500.).into();
                // style.size.width = px(700.).into();

                // TODO: use data_furthest_node_positions to calculate
                // how big the initial canvas should be
                window.request_layout(style, None, cx)
            };
            (layout_id, ())
        })
    }

    fn prepaint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> Self::PrepaintState {
        // set up canvas styles
        let text_style = TextStyleRefinement {
            font_size: Some(self.style.text.font_size),
            line_height: Some(self.style.text.line_height),
            ..Default::default()
        };
        // let focus_handle = self.focus_handle(cx);
        // window.set_focus_handle(&focus_handle, cx);

        window.with_text_style(Some(text_style), |window| {
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                // todo: we probably need somethink like zed::editor::EditorSnapshot here

                let style = self.style.clone();
                let hitbox = window.insert_hitbox(bounds, false);

                // Check for active drags in the canvas itself
                let has_active_drag = {
                    let canvas = self.canvas.read(cx);
                    canvas.active_drag.is_some()
                };

                if !has_active_drag {
                    // anything that shouldn't be painted when
                    // dragging goes in here
                }

                CanvasLayout { hitbox }
            })
        })
    }

    fn paint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        layout: &mut Self::PrepaintState,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) {
        let canvas = self.canvas.clone();
        // let key_context = self.canvas.update(cx, |canvas, cx| canvas.key_context());

        // window.set_key_context(key_context);

        // register_actions
        // register_key_listeners

        let text_style = TextStyleRefinement {
            font_size: Some(self.style.text.font_size),
            line_height: Some(self.style.text.line_height),
            ..Default::default()
        };

        window.with_text_style(Some(text_style), |window| {
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                // Clone the canvas to avoid multiple borrows of cx
                let canvas_clone = self.canvas.clone();
                self.paint_mouse_listeners(layout, window, cx);
                self.paint_canvas_background(layout, window, cx);
                self.paint_nodes(layout, window, cx);

                // Read canvas once to get all needed data
                let canvas_read = canvas_clone.read(cx);
                let active_drag = canvas_read.active_drag.clone();
                let active_element_draw = canvas_read.active_element_draw.clone();
                let active_tool = canvas_read.active_tool.clone();
                let theme = &canvas_read.theme;

                // Paint selection rectangle if dragging with selection tool
                if let Some(active_drag) = active_drag {
                    self.paint_selection(&active_drag, layout, window, theme);
                }

                // Paint rectangle preview if drawing with rectangle tool
                if let Some((node_id, node_type, drag)) = active_element_draw {
                    match active_tool {
                        ToolKind::Rectangle => {
                            self.paint_draw_rectangle(node_id, &drag, layout, window, cx);
                        }
                        _ => {}
                    }
                }
            });
        })
    }
}

impl IntoElement for CanvasElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}
