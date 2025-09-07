use crate::{
    register_canvas_action,
    tools::{ActiveTool, GlobalTool, Tool},
    ClearSelection, LunaCanvas,
};
use gpui::{
    hsla, prelude::*, px, relative, App, BorderStyle, ContentMask, DispatchPhase, ElementId,
    Entity, Hitbox, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Style,
    TextStyle, TextStyleRefinement, Window,
};
use gpui::{point, Bounds, Point, Size};
use luna_core::interactivity::DragPosition;
use luna_core::{
    interactivity::{ActiveDrag, DragType, ResizeHandle, ResizeOperation},
    util::{round_to_pixel, rounded_point},
};
use node::{frame::FrameNode, NodeCommon, NodeId, NodeLayout, NodeType, Shadow};
use scene_graph::SceneGraph;
use smallvec::SmallVec;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use theme::{ActiveTheme, Theme};

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
    const HANDLE_SIZE: f32 = 11.0; // Increased from 7.0 to provide a larger hit area
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

    // Helper function to get the depth of a node in the hierarchy
    fn get_node_depth(canvas: &LunaCanvas, node_id: NodeId) -> usize {
        let mut depth = 0;
        let mut current_id = node_id;

        // Walk up the parent chain counting levels
        while let Some(parent_id) = canvas.find_parent(current_id) {
            depth += 1;
            current_id = parent_id;
        }

        depth
    }

    // Helper function to find all nodes at a given point
    fn find_all_nodes_at_point(canvas: &LunaCanvas, canvas_point: Point<f32>) -> Vec<NodeId> {
        let mut nodes_at_point = Vec::new();

        // Test each node to see if it contains this point
        for node in canvas.nodes().values() {
            // Get absolute position of the node (accumulating parent offsets)
            let mut absolute_x = node.layout().x;
            let mut absolute_y = node.layout().y;

            // Walk up the parent chain to get absolute position
            let mut current_parent = canvas.find_parent(node.id());
            while let Some(parent_id) = current_parent {
                if let Some(parent_node) = canvas.nodes().get(&parent_id) {
                    absolute_x += parent_node.layout().x;
                    absolute_y += parent_node.layout().y;
                    current_parent = canvas.find_parent(parent_id);
                } else {
                    break;
                }
            }

            // Create bounds using absolute position
            let absolute_bounds = Bounds {
                origin: Point::new(absolute_x, absolute_y),
                size: Size::new(node.layout().width, node.layout().height),
            };

            if absolute_bounds.contains(&canvas_point) {
                nodes_at_point.push(node.id());
            }
        }

        nodes_at_point
    }

    // Helper function to find the top node at a given point with selection rules
    fn find_top_node_at_point(
        canvas: &LunaCanvas,
        window_point: Point<f32>,
        deep_selection: bool,
        cx: &Context<LunaCanvas>,
    ) -> Option<NodeId> {
        // Convert window coordinate to canvas coordinate
        let canvas_point = canvas.window_to_canvas_point(window_point);

        // Find all nodes at this point
        let nodes_at_point = Self::find_all_nodes_at_point(canvas, canvas_point);

        if nodes_at_point.is_empty() {
            return None;
        }

        // If deep selection is enabled (Cmd/Super held), find the deepest node
        if deep_selection {
            return nodes_at_point
                .into_iter()
                .max_by_key(|&node_id| Self::get_node_depth(canvas, node_id));
        }

        // Default behavior: prefer immediate children over parents (one level only)
        // First, find if any of the nodes at point have a parent that's also at the point
        for &node_id in &nodes_at_point {
            if let Some(parent_id) = canvas.find_parent(node_id) {
                // Check if the parent is also at this point
                if nodes_at_point.contains(&parent_id) {
                    // We found a child whose parent is also under the cursor
                    // Return the child (this implements the one-level preference)
                    return Some(node_id);
                }
            }
        }

        // If no parent-child relationship exists at the point, return any node
        // (preferably the top-most one, but without z-ordering we just return the first)
        nodes_at_point.into_iter().next()
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

        let active_tool = cx.active_tool().clone();

        match *active_tool {
            Tool::Selection => {
                // First, check if we've clicked on a resize handle when only a single node is selected
                if canvas.selected_nodes().len() == 1 {
                    // Get the bounds of the selected node
                    let selected_node_id = *canvas.selected_nodes().iter().next().unwrap();
                    if let Some(node) = canvas.nodes().get(&selected_node_id) {
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
                            canvas.set_active_drag(ActiveDrag::new_resize(position, resize_op));
                            canvas.mark_dirty(cx);
                            cx.stop_propagation();
                            return;
                        }
                    }
                }

                // If we didn't hit a resize handle, proceed with normal selection behavior
                // Attempt to find a node at the clicked point
                // Check if Cmd/Super is held for deep selection
                let deep_selection = event.modifiers.platform;

                if let Some(node_id) =
                    Self::find_top_node_at_point(canvas, canvas_point, deep_selection, cx)
                {
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
                        canvas.set_active_drag(ActiveDrag::new_move(position));
                    }

                    canvas.mark_dirty(cx);
                } else {
                    // Clicked on empty space - start a selection rectangle drag
                    // First clear selection if shift is not pressed
                    if !event.modifiers.shift {
                        canvas.clear_selection(&ClearSelection, window, cx);
                    }

                    // Only start a selection drag if using the Selection tool
                    if *active_tool == Tool::Selection {
                        canvas.set_active_drag(ActiveDrag::new_selection(position));
                    }
                    canvas.mark_dirty(cx);
                }
            }
            Tool::Frame => {
                // Use the generate_id method directly since it already returns the correct type
                let new_node_id = canvas.generate_id();

                let active_drag = ActiveDrag::new_create(position);
                canvas.set_active_element_draw((new_node_id, NodeType::Frame, active_drag));
                canvas.mark_dirty(cx);
            }
            Tool::Rectangle => {
                // Create a new shape node ID
                let new_node_id = canvas.generate_id();

                let active_drag = ActiveDrag::new_create(position);
                canvas.set_active_element_draw((new_node_id, NodeType::Shape, active_drag));
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
        let app_state = canvas.app_state().clone().read(cx);
        let current_background_color = app_state.current_background_color.clone();
        let current_border_color = app_state.current_border_color.clone();
        let active_tool = *cx.active_tool().clone();

        // Check if we have an active element draw operation
        if let Some((node_id, node_type, active_drag)) = canvas.active_element_draw().take() {
            match (node_type, active_tool) {
                (NodeType::Frame, Tool::Frame) => {
                    // Calculate rectangle dimensions
                    let start_pos = active_drag.start_position;
                    let end_pos = active_drag.current_position;

                    let min_x = start_pos.x().min(end_pos.x());
                    let min_y = start_pos.y().min(end_pos.y());
                    let width = (start_pos.x() - end_pos.x()).abs();
                    let height = (start_pos.y() - end_pos.y()).abs();

                    // Only create a rectangle if it has meaningful dimensions
                    if width >= 2.0 && height >= 2.0 {
                        // Convert window coordinates to canvas coordinates
                        let canvas_point = canvas.window_to_canvas_point(Point::new(min_x, min_y));
                        let rel_x = canvas_point.x;
                        let rel_y = canvas_point.y;

                        // Create a new rectangle node
                        let mut rect = FrameNode::new(node_id);

                        // Set position and size
                        *rect.layout_mut() = NodeLayout::new(rel_x, rel_y, width, height);

                        // Set colors
                        rect.set_fill(Some(current_background_color));
                        rect.set_border(Some(current_border_color), 1.0);

                        // Add the node to the canvas
                        let new_node_id = canvas.add_node(rect, None, cx);

                        // Clear any existing selection
                        canvas.deselect_all_nodes(cx);

                        // Select only the newly created element
                        canvas.select_node(new_node_id);

                        cx.set_global(GlobalTool(Arc::new(Tool::Selection)));

                        canvas.mark_dirty(cx);
                    }
                }
                (NodeType::Shape, Tool::Rectangle) => {
                    // Calculate rectangle dimensions
                    let start_pos = active_drag.start_position;
                    let end_pos = active_drag.current_position;

                    let min_x = start_pos.x().min(end_pos.x());
                    let min_y = start_pos.y().min(end_pos.y());
                    let width = (start_pos.x() - end_pos.x()).abs();
                    let height = (start_pos.y() - end_pos.y()).abs();

                    // Only create a shape if it has meaningful dimensions
                    if width >= 2.0 && height >= 2.0 {
                        // Convert window coordinates to canvas coordinates
                        let canvas_point = canvas.window_to_canvas_point(Point::new(min_x, min_y));
                        let rel_x = canvas_point.x;
                        let rel_y = canvas_point.y;

                        // Create a new shape node (rectangle)
                        let mut shape =
                            node::shape::ShapeNode::rectangle(node_id, rel_x, rel_y, width, height);

                        // Set colors
                        shape.set_fill(Some(current_background_color));
                        shape.set_border(Some(current_border_color), 1.0);

                        // Add the node to the canvas
                        let new_node_id = canvas.add_node(shape, None, cx);

                        // Clear any existing selection
                        canvas.deselect_all_nodes(cx);

                        // Select only the newly created element
                        canvas.select_node(new_node_id);

                        cx.set_global(GlobalTool(Arc::new(Tool::Selection)));

                        canvas.mark_dirty(cx);
                    }
                }
                _ => {}
            }
        }

        // Handle ending drag operations
        if let Some(active_drag) = canvas.active_drag().take() {
            match active_drag.drag_type {
                DragType::MoveElements => {
                    // Parenting/unparenting is now handled during drag
                    // Just finalize the move by clearing initial positions
                    canvas.element_initial_positions_mut().clear();
                    canvas.mark_dirty(cx);
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

        // Reset the potential parent frame when drag ends
        canvas.set_potential_parent_frame(None);
        canvas.clear_active_drag();
        canvas.clear_active_element_draw();

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
        let active_tool = *cx.active_tool().clone();

        // Handle active drag operations
        if let Some(mut active_drag) = canvas.active_drag().take() {
            // Update the drag with new position
            active_drag.update_position(position);

            // For any non-Selection tool, only allow specific drag types
            if active_tool != Tool::Selection
                && matches!(active_drag.drag_type, DragType::Selection)
            {
                // If using a non-Selection tool but having a Selection drag, cancel it
                canvas.clear_active_drag();
                return;
            }

            match &active_drag.drag_type {
                DragType::Selection => {
                    // Handle selection rectangle
                    if active_tool == Tool::Selection {
                        // Calculate the selection rectangle in canvas coordinates
                        let start_pos = active_drag.start_position;
                        let min_x = start_pos.x().min(position.x.0);
                        let min_y = start_pos.y().min(position.y.0);
                        let max_x = start_pos.x().max(position.x.0);
                        let max_y = start_pos.y().max(position.y.0);

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
                            .nodes()
                            .values()
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
                            if nodes_in_selection != canvas.selected_nodes().clone() {
                                canvas.clear_selection(&ClearSelection, window, cx);
                                for node_id in nodes_in_selection {
                                    canvas.select_node(node_id);
                                }
                            }
                        }

                        // Put the active drag back after all modifications
                        canvas.set_active_drag(active_drag);
                    }
                }
                DragType::MoveElements => {
                    // Move selected elements based on drag delta
                    if !canvas.selected_nodes().is_empty() {
                        // Calculate the drag delta in canvas coordinates
                        let delta = active_drag.delta();

                        // Move all selected nodes with the drag delta first
                        // This now properly handles coordinate conversion for parented nodes
                        canvas.move_selected_nodes_with_drag(delta, cx);

                        // Get all the selected node IDs
                        let selected_ids: Vec<NodeId> =
                            canvas.selected_nodes().iter().cloned().collect();

                        // Only check for potential parent frames occasionally to avoid expensive iteration
                        // Check every 10 pixels of movement to reduce CPU load
                        const PARENT_CHECK_THRESHOLD: f32 = 10.0;

                        let should_check_parent =
                            if let Some(last_pos) = active_drag.last_parent_check_position {
                                let distance = ((position.x.0 - last_pos.x).powi(2)
                                    + (position.y.0 - last_pos.y).powi(2))
                                .sqrt();
                                distance > PARENT_CHECK_THRESHOLD
                            } else {
                                true // First check
                            };

                        if should_check_parent {
                            // Get current canvas point to check for potential parent frames
                            let canvas_point = canvas
                                .window_to_canvas_point(Point::new(position.x.0, position.y.0));

                            // First, check if any selected nodes need to be unparented
                            // (they're no longer inside their parent's bounds)
                            for &node_id in &selected_ids {
                                if let Some(current_parent_id) = canvas.find_parent(node_id) {
                                    // Get absolute position of the node (already updated by move_selected_nodes_with_drag)
                                    let (node_abs_x, node_abs_y) =
                                        canvas.get_absolute_position(node_id, cx);

                                    // Get the parent's absolute bounds
                                    let (parent_abs_x, parent_abs_y) =
                                        canvas.get_absolute_position(current_parent_id, cx);
                                    let parent_bounds = if let Some(parent_node) =
                                        canvas.get_node(current_parent_id)
                                    {
                                        let layout = parent_node.layout();
                                        Bounds {
                                            origin: Point::new(parent_abs_x, parent_abs_y),
                                            size: Size::new(layout.width, layout.height),
                                        }
                                    } else {
                                        continue;
                                    };

                                    // Get the node's size
                                    let node_size = if let Some(node) = canvas.get_node(node_id) {
                                        let layout = node.layout();
                                        Size::new(layout.width, layout.height)
                                    } else {
                                        continue;
                                    };

                                    // Check if the node's bounds are completely outside the parent
                                    let node_bounds = Bounds {
                                        origin: Point::new(node_abs_x, node_abs_y),
                                        size: node_size,
                                    };

                                    // If the node is completely outside its parent, unparent it
                                    let intersects = !(node_bounds.origin.x
                                        + node_bounds.size.width
                                        < parent_bounds.origin.x
                                        || node_bounds.origin.x
                                            > parent_bounds.origin.x + parent_bounds.size.width
                                        || node_bounds.origin.y + node_bounds.size.height
                                            < parent_bounds.origin.y
                                        || node_bounds.origin.y
                                            > parent_bounds.origin.y + parent_bounds.size.height);

                                    if !intersects {
                                        // Remove from parent and convert to absolute positioning
                                        canvas.remove_child_from_parent(node_id, cx);
                                    }
                                }
                            }

                            // Find potential parent frame at the current position
                            let potential_parent = canvas
                                .nodes()
                                .values()
                                .filter(|node| {
                                    // Only nodes that can have children can be parents
                                    node.can_have_children() && !selected_ids.contains(&node.id())
                                })
                                .find(|node| {
                                    // Get absolute bounds of this potential parent
                                    let (abs_x, abs_y) =
                                        canvas.get_absolute_position(node.id(), cx);
                                    let abs_bounds = Bounds {
                                        origin: Point::new(abs_x, abs_y),
                                        size: Size::new(node.layout().width, node.layout().height),
                                    };
                                    abs_bounds.contains(&canvas_point)
                                })
                                .map(|node| node.id());

                            // If we found a potential parent, apply parenting to nodes that aren't already its children
                            if let Some(parent_id) = potential_parent {
                                let parent_children = canvas.get_node_children(parent_id);

                                for &node_id in &selected_ids {
                                    // Only parent if not already a child of this parent
                                    if !parent_children.contains(&node_id)
                                        && canvas.find_parent(node_id) != Some(parent_id)
                                    {
                                        // Remove from current parent if it has one
                                        if canvas.find_parent(node_id).is_some() {
                                            canvas.remove_child_from_parent(node_id, cx);
                                        }

                                        // Add to new parent
                                        canvas.add_child_to_parent(parent_id, node_id, cx);
                                    }
                                }
                            }

                            // Update the potential parent frame for visual feedback
                            canvas.set_potential_parent_frame(potential_parent);

                            // Update the last check position in the drag state
                            active_drag.last_parent_check_position =
                                Some(Point::new(position.x.0, position.y.0));
                        }

                        // Put the active drag back after modifications
                        canvas.set_active_drag(active_drag);
                    }
                }
                DragType::CreateElement => {
                    // Nothing to do here - handled in the rectangle drawing code below
                    // Put the active drag back
                    canvas.set_active_drag(active_drag);
                }
                DragType::Resize(resize_op) => {
                    // Handle resize operation
                    if canvas.selected_nodes().len() == 1 {
                        // Get the zoom value before any mutable borrows
                        let zoom = canvas.zoom();

                        // Get the selected node
                        let selected_node_id = *canvas.selected_nodes().iter().next().unwrap();
                        if let Some(node) = canvas.get_node_mut(selected_node_id) {
                            // Convert window delta to canvas delta
                            let delta = Point::new(
                                (position.x.0 - active_drag.start_position.x()) / zoom,
                                (position.y.0 - active_drag.start_position.y()) / zoom,
                            );

                            // Check modifiers: shift for aspect ratio, option (alt) for resize from center
                            let preserve_aspect_ratio = event.modifiers.shift;
                            let resize_from_center = event.modifiers.alt;

                            // Create a new resize_op with updated configuration
                            let mut new_resize_op = resize_op.clone();
                            new_resize_op.config.preserve_aspect_ratio = preserve_aspect_ratio;
                            new_resize_op.config.resize_from_center = resize_from_center;

                            // Calculate new dimensions based on resize handle and modifiers
                            let mut new_x = new_resize_op.original_x();
                            let mut new_y = new_resize_op.original_y();
                            let mut new_width = new_resize_op.original_width();
                            let mut new_height = new_resize_op.original_height();

                            // Calculate aspect ratio if needed
                            let aspect_ratio = if preserve_aspect_ratio {
                                new_resize_op.original_width() / new_resize_op.original_height()
                            } else {
                                0.0 // Not used when not preserving aspect ratio
                            };

                            // Adjust dimensions based on which handle is being dragged
                            match new_resize_op.handle {
                                ResizeHandle::TopLeft => {
                                    // Width/height change is negative of delta for top-left
                                    let width_delta = -delta.x;
                                    let height_delta = -delta.y;

                                    if preserve_aspect_ratio {
                                        // Use whichever delta would make the shape larger
                                        if width_delta.abs() / aspect_ratio > height_delta.abs() {
                                            let adj_height = width_delta / aspect_ratio;
                                            new_width =
                                                new_resize_op.original_width() + width_delta;
                                            new_height =
                                                new_resize_op.original_height() + adj_height;
                                            new_x = new_resize_op.original_x() - width_delta;
                                            new_y = new_resize_op.original_y() - adj_height;
                                        } else {
                                            let adj_width = height_delta * aspect_ratio;
                                            new_width = new_resize_op.original_width() + adj_width;
                                            new_height =
                                                new_resize_op.original_height() + height_delta;
                                            new_x = new_resize_op.original_x() - adj_width;
                                            new_y = new_resize_op.original_y() - height_delta;
                                        }
                                    } else {
                                        // Standard resize without aspect ratio constraint
                                        new_width =
                                            new_resize_op.original_width() + 2.0 * width_delta;
                                        new_height =
                                            new_resize_op.original_height() + 2.0 * height_delta;
                                        new_x = new_resize_op.original_x() - width_delta;
                                        new_y = new_resize_op.original_y() - height_delta;
                                    }
                                }
                                ResizeHandle::TopRight => {
                                    // Width change is positive, height change is negative
                                    let width_delta = delta.x;
                                    let height_delta = -delta.y;

                                    if preserve_aspect_ratio {
                                        if width_delta.abs() / aspect_ratio > height_delta.abs() {
                                            let adj_height = width_delta / aspect_ratio;
                                            new_width =
                                                new_resize_op.original_width() + width_delta;
                                            new_height =
                                                new_resize_op.original_height() - adj_height;
                                            new_y = new_resize_op.original_y() + adj_height;
                                        } else {
                                            let adj_width = height_delta * aspect_ratio;
                                            new_width = new_resize_op.original_width() + adj_width;
                                            new_height =
                                                new_resize_op.original_height() + height_delta;
                                            new_y = new_resize_op.original_y() - height_delta;
                                        }
                                    } else {
                                        new_width = new_resize_op.original_width() + width_delta;
                                        new_height = new_resize_op.original_height() + height_delta;
                                        new_y = new_resize_op.original_y() - height_delta;
                                    }
                                }
                                ResizeHandle::BottomLeft => {
                                    // Width change is negative, height change is positive
                                    let width_delta = -delta.x;
                                    let height_delta = delta.y;

                                    if preserve_aspect_ratio {
                                        if width_delta.abs() / aspect_ratio > height_delta.abs() {
                                            let adj_height = width_delta / aspect_ratio;
                                            new_width =
                                                new_resize_op.original_width() - width_delta;
                                            new_height =
                                                new_resize_op.original_height() + adj_height;
                                            new_x = new_resize_op.original_x() + width_delta;
                                        } else {
                                            let adj_width = height_delta * aspect_ratio;
                                            new_width = new_resize_op.original_width() + adj_width;
                                            new_height =
                                                new_resize_op.original_height() + height_delta;
                                            new_x = new_resize_op.original_x() - adj_width;
                                        }
                                    } else {
                                        new_width = new_resize_op.original_width() + width_delta;
                                        new_height = new_resize_op.original_height() + height_delta;
                                        new_x = new_resize_op.original_x() - width_delta;
                                    }
                                }
                                ResizeHandle::BottomRight => {
                                    let width_delta = delta.x;
                                    let height_delta = delta.y;

                                    if preserve_aspect_ratio {
                                        if width_delta.abs() / aspect_ratio > height_delta.abs() {
                                            let adj_height = width_delta / aspect_ratio;
                                            new_width =
                                                new_resize_op.original_width() + width_delta;
                                            new_height =
                                                new_resize_op.original_height() + adj_height;
                                        } else {
                                            let adj_width = height_delta * aspect_ratio;
                                            new_width = new_resize_op.original_width() + adj_width;
                                            new_height =
                                                new_resize_op.original_height() + height_delta;
                                        }
                                    } else {
                                        new_width = new_resize_op.original_width() + width_delta;
                                        new_height = new_resize_op.original_height() + height_delta;
                                    }
                                }
                            }

                            // If resize from center is enabled, adjust position to keep center fixed
                            if resize_from_center {
                                let orig_center_x = new_resize_op.original_x()
                                    + new_resize_op.original_width() / 2.0;
                                let orig_center_y = new_resize_op.original_y()
                                    + new_resize_op.original_height() / 2.0;
                                new_x = orig_center_x - new_width / 2.0;
                                new_y = orig_center_y - new_height / 2.0;
                            }

                            // Calculate the correct position and dimensions for each handle type
                            match resize_op.handle {
                                ResizeHandle::TopLeft => {
                                    // Handle horizontal resizing (left edge)
                                    if new_width < 0.0 {
                                        // Crossed right edge - fixed point switches to left
                                        new_width = -new_width;
                                        // Left edge is now at original right edge + the overflow
                                        new_x = new_resize_op.original_x()
                                            + new_resize_op.original_width();
                                    } else {
                                        // Normal case - right edge stays fixed
                                        new_x = new_resize_op.original_x()
                                            + new_resize_op.original_width()
                                            - new_width;
                                    }

                                    // Handle vertical resizing (top edge)
                                    if new_height < 0.0 {
                                        // Crossed bottom edge - fixed point switches to top
                                        new_height = -new_height;
                                        // Top edge is now at original bottom edge + the overflow
                                        new_y = new_resize_op.original_y()
                                            + new_resize_op.original_height();
                                    } else {
                                        // Normal case - bottom edge stays fixed
                                        new_y = new_resize_op.original_y()
                                            + new_resize_op.original_height()
                                            - new_height;
                                    }
                                }
                                ResizeHandle::TopRight => {
                                    // Handle horizontal resizing (right edge)
                                    if new_width < 0.0 {
                                        // Crossed left edge - fixed point switches to right
                                        new_width = -new_width;
                                        // Keep the original x, width grows to the left
                                        new_x = new_resize_op.original_x() - new_width;
                                    } else {
                                        // Normal case - left edge stays fixed at original x
                                        new_x = new_resize_op.original_x();
                                    }

                                    // Handle vertical resizing (top edge)
                                    if new_height < 0.0 {
                                        // Crossed bottom edge - fixed point switches to top
                                        new_height = -new_height;
                                        // Top edge is now at original bottom edge + the overflow
                                        new_y = new_resize_op.original_y()
                                            + new_resize_op.original_height();
                                    } else {
                                        // Normal case - bottom edge stays fixed
                                        new_y = new_resize_op.original_y()
                                            + new_resize_op.original_height()
                                            - new_height;
                                    }
                                }
                                ResizeHandle::BottomLeft => {
                                    // Handle horizontal resizing (left edge)
                                    if new_width < 0.0 {
                                        // Crossed right edge - fixed point switches to left
                                        new_width = -new_width;
                                        // Left edge is now at original right edge + the overflow
                                        new_x = new_resize_op.original_x()
                                            + new_resize_op.original_width();
                                    } else {
                                        // Normal case - right edge stays fixed
                                        new_x = new_resize_op.original_x()
                                            + new_resize_op.original_width()
                                            - new_width;
                                    }

                                    // Handle vertical resizing (bottom edge)
                                    if new_height < 0.0 {
                                        // Crossed top edge - fixed point switches to bottom
                                        new_height = -new_height;
                                        // Keep original y, height grows upward
                                        new_y = new_resize_op.original_y() - new_height;
                                    } else {
                                        // Normal case - top edge stays fixed at original y
                                        new_y = new_resize_op.original_y();
                                    }
                                }
                                ResizeHandle::BottomRight => {
                                    // Handle horizontal resizing (right edge)
                                    if new_width < 0.0 {
                                        // Crossed left edge - fixed point switches to right
                                        new_width = -new_width;
                                        // Keep the original x, width grows to the left
                                        new_x = new_resize_op.original_x() - new_width;
                                    } else {
                                        // Normal case - left edge stays fixed at original x
                                        new_x = new_resize_op.original_x();
                                    }

                                    // Handle vertical resizing (bottom edge)
                                    if new_height < 0.0 {
                                        // Crossed top edge - fixed point switches to bottom
                                        new_height = -new_height;
                                        // Keep original y, height grows upward
                                        new_y = new_resize_op.original_y() - new_height;
                                    } else {
                                        // Normal case - top edge stays fixed at original y
                                        new_y = new_resize_op.original_y();
                                    }
                                }
                            }

                            // Ensure minimum dimensions (very small but positive)
                            if new_width > 0.1 && new_height > 0.1 {
                                // Update node dimensions
                                let layout = node.layout_mut();
                                layout.x = new_x;
                                layout.y = new_y;
                                layout.width = new_width;
                                layout.height = new_height;

                                // Update scene graph
                                if let Some(scene_node_id) =
                                    canvas.scene_graph().update(cx, |sg, _cx| {
                                        sg.get_scene_node_from_data_node(selected_node_id)
                                    })
                                {
                                    // Position is stored in FrameNode, no need to update scene graph bounds

                                    // Update child node layouts to reflect parent's resize
                                    canvas.update_child_layouts_after_parent_resize(
                                        selected_node_id,
                                        Size::new(0.0, 0.0), // old_size placeholder
                                        Size::new(0.0, 0.0), // new_size placeholder
                                        cx,
                                    );
                                }
                            }

                            // Update the resize operation in the drag
                            let mut updated_drag = active_drag.clone();
                            updated_drag.update_position(position);
                            updated_drag.drag_type = DragType::Resize(new_resize_op);
                            canvas.set_active_drag(updated_drag);
                        }
                    }
                }
            }
            // Mark dirty to trigger redraw after drag update
            canvas.mark_dirty(cx);
        }

        // Handle rectangle drawing
        if let Some(active_draw) = canvas.active_element_draw().take() {
            match *cx.active_tool().clone() {
                Tool::Frame | Tool::Rectangle => {
                    let new_drag = ActiveDrag {
                        start_position: active_draw.2.start_position,
                        current_position: DragPosition::from_point_pixels(position),
                        drag_type: DragType::CreateElement,
                        last_parent_check_position: None,
                    };
                    canvas.set_active_element_draw((active_draw.0, active_draw.1, new_drag));
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
        let hovered = Self::find_top_node_at_point(canvas, canvas_point, false, cx);

        // Only update and redraw if hover state changed
        if canvas.hovered_node() != hovered {
            canvas.set_hovered_node(hovered);
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

        let min_x = round_to_pixel(Pixels(
            active_drag
                .start_position
                .x()
                .min(active_drag.current_position.x()),
        ));
        let min_y = round_to_pixel(Pixels(
            active_drag
                .start_position
                .y()
                .min(active_drag.current_position.y()),
        ));
        let width = round_to_pixel(Pixels(
            (active_drag.start_position.x() - active_drag.current_position.x()).abs(),
        ));
        let height = round_to_pixel(Pixels(
            (active_drag.start_position.y() - active_drag.current_position.y()).abs(),
        ));

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
        let min_x = start_pos.x().min(current_pos.x());
        let min_y = start_pos.y().min(current_pos.y());
        let width = (start_pos.x() - current_pos.x()).abs();
        let height = (start_pos.y() - current_pos.y()).abs();

        // Round once after all coordinate conversions for pixel-perfect rendering
        let position = rounded_point(Pixels(min_x), Pixels(min_y));

        let rect_bounds = Bounds {
            origin: position,
            size: Size::new(Pixels(width), Pixels(height)),
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
    fn paint_scroll_wheel_listener(
        &mut self,
        layout: &CanvasLayout,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.on_mouse_event({
            let canvas = self.canvas.clone();
            let hitbox = layout.hitbox.clone();
            move |event: &gpui::ScrollWheelEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
                    canvas.update(cx, |canvas, cx| {
                        // Handle scrolling/panning of the canvas
                        let delta = match event.delta {
                            gpui::ScrollDelta::Pixels(pixels) => {
                                // Trackpad input - direct pixel movement
                                pixels
                            }
                            gpui::ScrollDelta::Lines(lines) => {
                                // Mouse wheel input - convert lines to pixels
                                // Scale lines by a factor to make it feel natural
                                // Convert lines to pixels - multiply by 30 for natural feel
                                gpui::Point::new(
                                    gpui::Pixels(lines.x * 30.0),
                                    gpui::Pixels(lines.y * 30.0),
                                )
                            }
                        };

                        // Invert delta for natural feeling panning
                        let inverted_delta =
                            gpui::Point::new(gpui::Pixels(-delta.x.0), gpui::Pixels(-delta.y.0));

                        // Get current canvas position through getter
                        let current_position = canvas.get_scroll_position();

                        // Calculate new position
                        let new_position = gpui::Point::new(
                            current_position.x + inverted_delta.x.0 / canvas.zoom(),
                            current_position.y + inverted_delta.y.0 / canvas.zoom(),
                        );

                        // Update canvas scroll position
                        canvas.set_scroll_position(new_position, cx);
                        cx.stop_propagation();
                    });
                }
            }
        });
    }

    fn paint_mouse_listeners(&mut self, layout: &CanvasLayout, window: &mut Window, cx: &mut App) {
        window.on_mouse_event({
            let canvas = self.canvas.clone();
            let hitbox = layout.hitbox.clone();
            move |event: &MouseDownEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
                    // Check if click is outside UI panels (inspector on right, sidebar on left)
                    let window_width = window.viewport_size().width.0;
                    let click_x = event.position.x.0;

                    const SIDEBAR_WIDTH: f32 = 220.0; // From Sidebar::INITIAL_WIDTH
                    const INSPECTOR_WIDTH: f32 = 200.0; // From inspector.rs

                    // Only process if click is not on sidebar (left) or inspector (right)
                    if click_x > SIDEBAR_WIDTH && click_x < (window_width - INSPECTOR_WIDTH) {
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
            }
        });

        window.on_mouse_event({
            let canvas = self.canvas.clone();
            let hitbox = layout.hitbox.clone();
            move |event: &MouseUpEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
                    // Check if release is outside UI panels
                    let window_width = window.viewport_size().width.0;
                    let click_x = event.position.x.0;

                    const SIDEBAR_WIDTH: f32 = 220.0;
                    const INSPECTOR_WIDTH: f32 = 200.0;

                    if click_x > SIDEBAR_WIDTH && click_x < (window_width - INSPECTOR_WIDTH) {
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
            }
        });

        window.on_mouse_event({
            let canvas = self.canvas.clone();
            let hitbox = layout.hitbox.clone();
            move |event: &MouseMoveEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
                    // Check if mouse move is outside UI panels
                    let window_width = window.viewport_size().width.0;
                    let mouse_x = event.position.x.0;

                    const SIDEBAR_WIDTH: f32 = 220.0;
                    const INSPECTOR_WIDTH: f32 = 200.0;

                    if mouse_x > SIDEBAR_WIDTH && mouse_x < (window_width - INSPECTOR_WIDTH) {
                        canvas.update(cx, |canvas, cx| {
                            // Throttle mouse move processing to ~60fps to avoid overwhelming the system
                            if event.pressed_button == Some(MouseButton::Left)
                                || event.pressed_button == Some(MouseButton::Middle)
                            {
                                // Only process drag events at throttled rate
                                if canvas.should_process_mouse_move() {
                                    Self::handle_mouse_drag(canvas, event, window, cx)
                                }
                            } else {
                                // Always process hover updates (they're less expensive)
                                Self::handle_mouse_move(canvas, event, window, cx)
                            }
                        });
                    }
                }
            }
        });
    }

    fn paint_nodes(&self, layout: &CanvasLayout, window: &mut Window, cx: &mut App) {
        let canvas = self.canvas.clone();
        let theme = cx.theme().clone();

        // Collect ALL data we need up front to avoid any borrow issues
        #[derive(Clone)]
        struct NodeRenderInfo {
            node_id: NodeId,
            bounds: gpui::Bounds<Pixels>,
            fill_color: Option<Hsla>,
            border_color: Option<Hsla>,
            border_width: f32,
            corner_radius: f32,
            shadows: SmallVec<[Shadow; 1]>,
            children: Vec<NodeId>,
        }

        // Helper function to organize nodes into a hierarchy
        fn organize_nodes_hierarchically(
            all_nodes: &[NodeRenderInfo],
        ) -> (Vec<NodeRenderInfo>, HashMap<NodeId, Vec<NodeRenderInfo>>) {
            let mut root_nodes = Vec::new();
            let mut children_map: HashMap<NodeId, Vec<NodeRenderInfo>> = HashMap::new();

            // First, create a mapping of parent NodeId to child nodes
            for node in all_nodes {
                let node_id = node.node_id;

                // For each child ID in the node's children list
                for &child_id in &node.children {
                    // Find the corresponding NodeRenderInfo for this child
                    if let Some(child_node) = all_nodes.iter().find(|n| n.node_id == child_id) {
                        children_map
                            .entry(node_id)
                            .or_default()
                            .push(child_node.clone());
                    }
                }
            }

            // Identify root nodes (not children of any other node)
            let all_children: HashSet<NodeId> = children_map
                .values()
                .flat_map(|nodes| nodes.iter().map(|n| n.node_id))
                .collect();

            for node in all_nodes {
                if !all_children.contains(&node.node_id) {
                    root_nodes.push(node.clone());
                }
            }

            (root_nodes, children_map)
        }

        // Get all the data we need in one place
        let (nodes_to_render, selected_node_ids, hovered_node, potential_parent_frame, active_drag) =
            canvas.update(cx, |canvas, cx| {
                let visible_nodes = canvas.visible_nodes(cx);
                let scene_graph = canvas.scene_graph().read(cx);
                let selected_nodes = canvas.selected_nodes().clone();
                let theme = cx.theme().clone();
                let hovered_node = canvas.hovered_node().clone();

                // Collect all node rendering information into owned structures
                let mut nodes_to_render = Vec::new();

                for node in visible_nodes {
                    let node_id = node.id();

                    if let Some(scene_node_id) = scene_graph.get_scene_node_from_data_node(node_id)
                    {
                        // Calculate world bounds on demand
                        let world_bounds = scene_graph
                            .get_world_bounds(scene_node_id, |id| canvas.nodes().get(&id).cloned());

                        // Convert to window coordinates using canvas transform
                        let canvas_pos = glam::Vec2::new(world_bounds.min.x, world_bounds.min.y);
                        let canvas_size = world_bounds.size();
                        let window_pos = canvas
                            .canvas_to_window_point(gpui::Point::new(canvas_pos.x, canvas_pos.y));

                        // Apply zoom to size
                        let window_size = gpui::Size::new(
                            gpui::Pixels(canvas_size.x * canvas.zoom()),
                            gpui::Pixels(canvas_size.y * canvas.zoom()),
                        );

                        nodes_to_render.push(NodeRenderInfo {
                            node_id,
                            bounds: gpui::Bounds {
                                origin: gpui::Point::new(
                                    gpui::Pixels(window_pos.x),
                                    gpui::Pixels(window_pos.y),
                                ),
                                size: window_size,
                            },
                            fill_color: node.fill(),
                            border_color: node.border_color(),
                            border_width: node.border_width(),
                            corner_radius: node.corner_radius(),
                            shadows: node.shadows(),
                            children: node
                                .as_frame()
                                .map(|frame| frame.children().clone())
                                .unwrap_or_else(Vec::new),
                        });
                    }
                }

                (
                    nodes_to_render,
                    selected_nodes,
                    hovered_node,
                    canvas.potential_parent_frame(),
                    canvas.active_drag(),
                )
            });

        window.paint_layer(layout.hitbox.bounds, |window| {
            // Organize nodes into a hierarchy
            let (root_nodes, children_map) = organize_nodes_hierarchically(&nodes_to_render);

            // Recursive function to paint a node and its children
            fn paint_node_recursively(
                node_info: &NodeRenderInfo,
                children_map: &HashMap<NodeId, Vec<NodeRenderInfo>>,
                selected_node_ids: &HashSet<NodeId>,
                hovered_node: &Option<NodeId>,
                potential_parent_frame: &Option<NodeId>,
                has_active_drag: bool,
                theme: &Theme,
                window: &mut gpui::Window,
            ) {
                // Use the already-calculated world bounds directly
                let transformed_bounds = node_info.bounds;

                // FIRST: Paint any shadows behind the node
                // Shadows need to be rendered before the node itself
                if !node_info.shadows.is_empty() {
                    // Convert our Shadow types to gpui::BoxShadow types
                    let box_shadows: Vec<gpui::BoxShadow> = node_info
                        .shadows
                        .iter()
                        .map(|shadow| gpui::BoxShadow {
                            offset: gpui::Point::new(
                                gpui::Pixels(shadow.offset.x()),
                                gpui::Pixels(shadow.offset.y()),
                            ),
                            blur_radius: gpui::Pixels(shadow.blur_radius),
                            spread_radius: gpui::Pixels(shadow.spread_radius),
                            color: shadow.color,
                        })
                        .collect();

                    // Use the dedicated shadow rendering function
                    window.paint_shadows(
                        transformed_bounds,
                        gpui::Corners::all(gpui::Pixels(node_info.corner_radius)),
                        &box_shadows,
                    );
                }

                // SECOND: Paint the node itself (background and frame)
                // Paint the fill if it exists
                if let Some(fill_color) = node_info.fill_color {
                    window.paint_quad(gpui::PaintQuad {
                        bounds: transformed_bounds,
                        corner_radii: (node_info.corner_radius).into(),
                        background: fill_color.into(),
                        border_widths: (0.).into(),
                        border_color: gpui::transparent_black().into(),
                        border_style: BorderStyle::Solid,
                    });
                }

                // SECOND: Paint all children (if any) with clipping and proper transformation
                // We paint children AFTER the parent's fill but BEFORE the parent's border
                // This ensures children appear on top of the parent's background
                if let Some(children) = children_map.get(&node_info.node_id) {
                    // Create a mask for children to clip them to the frame bounds
                    window.with_content_mask(
                        Some(ContentMask {
                            bounds: transformed_bounds,
                        }),
                        |window| {
                            for child in children {
                                paint_node_recursively(
                                    child,
                                    children_map,
                                    selected_node_ids,
                                    hovered_node,
                                    potential_parent_frame,
                                    has_active_drag,
                                    theme,
                                    window,
                                );
                            }
                        },
                    );
                }

                // THIRD: Paint the border if it exists (after children, so it's on top)
                if let Some(border_color) = node_info.border_color {
                    window.paint_quad(gpui::PaintQuad {
                        bounds: transformed_bounds,
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
                            transformed_bounds.origin.x - gpui::Pixels(2.0),
                            transformed_bounds.origin.y - gpui::Pixels(2.0),
                        ),
                        size: gpui::Size::new(
                            transformed_bounds.size.width + gpui::Pixels(4.0),
                            transformed_bounds.size.height + gpui::Pixels(4.0),
                        ),
                    };

                    let hover_color = theme.tokens.active_border.opacity(0.6);
                    window.paint_quad(gpui::outline(hover_bounds, hover_color, BorderStyle::Solid));
                }

                // Show yellow border for potential parent frames during drag operations
                if has_active_drag && potential_parent_frame.as_ref() == Some(&node_info.node_id) {
                    // Create a slightly larger bounds for the parent indicator
                    let parent_indicator_bounds = gpui::Bounds {
                        origin: gpui::Point::new(
                            transformed_bounds.origin.x - gpui::Pixels(3.0),
                            transformed_bounds.origin.y - gpui::Pixels(3.0),
                        ),
                        size: gpui::Size::new(
                            transformed_bounds.size.width + gpui::Pixels(6.0),
                            transformed_bounds.size.height + gpui::Pixels(6.0),
                        ),
                    };

                    let yellow_highlight = gpui::hsla(60.0 / 360.0, 1.0, 0.5, 0.8);
                    window.paint_quad(gpui::outline(
                        parent_indicator_bounds,
                        yellow_highlight,
                        BorderStyle::Solid,
                    ));

                    // Make the border thicker for more emphasis
                    let inner_border = gpui::Bounds {
                        origin: gpui::Point::new(
                            transformed_bounds.origin.x - gpui::Pixels(2.0),
                            transformed_bounds.origin.y - gpui::Pixels(2.0),
                        ),
                        size: gpui::Size::new(
                            transformed_bounds.size.width + gpui::Pixels(4.0),
                            transformed_bounds.size.height + gpui::Pixels(4.0),
                        ),
                    };
                    window.paint_quad(gpui::outline(
                        inner_border,
                        yellow_highlight,
                        BorderStyle::Solid,
                    ));
                }
            }

            // Check if we have an active drag operation
            let has_active_drag = active_drag.is_some()
                && matches!(
                    active_drag.as_ref().map(|d| &d.drag_type),
                    Some(DragType::MoveElements)
                );

            // FIRST PASS: Paint all root nodes and their children recursively
            // =================================================================
            for node_info in &root_nodes {
                paint_node_recursively(
                    node_info,
                    &children_map,
                    &selected_node_ids,
                    &hovered_node,
                    &potential_parent_frame,
                    has_active_drag,
                    &theme,
                    window,
                );
            }

            // First draw individual selection outlines
            for node_info in &nodes_to_render {
                if selected_node_ids.contains(&node_info.node_id) {
                    // Use the already calculated world bounds directly
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
                    canvas.active_drag().is_some()
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
        let active_tool = *cx.active_tool().clone();
        let theme = cx.theme().clone();
        let key_context = self.canvas.update(cx, |canvas, cx| canvas.key_context());

        window.set_key_context(key_context);

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
                self.paint_scroll_wheel_listener(layout, window, cx);
                self.paint_canvas_background(layout, window, cx);
                self.paint_nodes(layout, window, cx);

                // Read canvas once to get all needed data
                let canvas_read = canvas_clone.read(cx);
                let active_drag = canvas_read.active_drag().clone();
                let active_element_draw = canvas_read.active_element_draw().clone();

                // Paint selection rectangle if dragging with selection tool
                if let Some(active_drag) = active_drag {
                    self.paint_selection(&active_drag, layout, window, &theme.clone());
                }

                // Paint rectangle preview if drawing with rectangle tool
                if let Some((node_id, node_type, drag)) = active_element_draw {
                    match active_tool {
                        Tool::Frame | Tool::Rectangle => {
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
