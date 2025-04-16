use crate::{
    canvas::{register_canvas_action, ClearSelection, LunaCanvas},
    coordinates::{CanvasBounds, CanvasSize, PositionStore, WindowPoint, WorldPoint},
    interactivity::{ActiveDrag, DragType, ResizeHandle, ResizeOperation},
    node::{frame::FrameNode, NodeCommon, NodeId, NodeLayout, NodeType, Shadow},
    scene_graph::SceneGraph,
    theme::{ActiveTheme, Theme},
    tools::{ActiveTool, GlobalTool},
    util::{round_to_pixel, rounded_point},
    Tool,
};
use gpui::{
    hsla, prelude::*, px, relative, App, BorderStyle, ContentMask, DispatchPhase, ElementId,
    Entity, Hitbox, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Style,
    TextStyle, TextStyleRefinement, TransformationMatrix, Window,
};
use gpui::{point, Bounds, Point, Size};
use smallvec::SmallVec;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

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
fn point_in_resize_handle(point: WorldPoint, node_bounds: &CanvasBounds) -> Option<ResizeHandle> {
    use ResizeHandle;

    const HANDLE_SIZE: f32 = 11.0;
    const HALF_HANDLE: f32 = HANDLE_SIZE / 2.0;

    let bounds = |origin: WorldPoint| -> CanvasBounds {
        CanvasBounds {
            origin,
            size: CanvasSize::new(HANDLE_SIZE, HANDLE_SIZE),
        }
    };

    let handles = [
        // Top left
        (
            bounds(WorldPoint::new(
                node_bounds.origin.x - HALF_HANDLE,
                node_bounds.origin.y - HALF_HANDLE,
            )),
            ResizeHandle::TopLeft,
        ),
        // Top right
        (
            bounds(WorldPoint::new(
                node_bounds.origin.x + node_bounds.size.width - HALF_HANDLE,
                node_bounds.origin.y - HALF_HANDLE,
            )),
            ResizeHandle::TopRight,
        ),
        // Bottom left
        (
            bounds(WorldPoint::new(
                node_bounds.origin.x - HALF_HANDLE,
                node_bounds.origin.y + node_bounds.size.height - HALF_HANDLE,
            )),
            ResizeHandle::BottomLeft,
        ),
        // Bottom right
        (
            bounds(WorldPoint::new(
                node_bounds.origin.x + node_bounds.size.width - HALF_HANDLE,
                node_bounds.origin.y + node_bounds.size.height - HALF_HANDLE,
            )),
            ResizeHandle::BottomRight,
        ),
    ];

    for (bounds, handle) in handles {
        if bounds.contains(point) {
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

    fn find_top_node_at_point(
        canvas: &LunaCanvas,
        window_point: Point<f32>,
        cx: &Context<LunaCanvas>,
    ) -> Option<NodeId> {
        let position_data_arc = cx.position_data();
        let position_data = position_data_arc.read().unwrap();
        let scene_graph = canvas.scene_graph().read(cx);
        let zoom = canvas.zoom();

        let hit_point = WindowPoint::from_point(window_point);

        let world_point = position_data.window_to_world(hit_point, zoom);

        for node in canvas.nodes().iter().rev() {
            if let Some(scene_node_id) = scene_graph.get_scene_node_id(node.id()) {
                if let Some(canvas_bounds) = scene_graph.get_world_canvas_bounds(scene_node_id) {
                    if canvas_bounds.contains(world_point) {
                        return Some(node.id());
                    }
                }
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

        let active_tool = cx.active_tool().clone();

        match *active_tool {
            Tool::Selection => {
                if canvas.selected_nodes().len() == 1 {
                    let selected_node_id = *canvas.selected_nodes().iter().next().unwrap();
                    if let Some(node) = canvas.nodes().iter().find(|n| n.id() == selected_node_id) {
                        let node_layout = node.layout();

                        let node_bounds = CanvasBounds {
                            origin: WorldPoint::new(node_layout.x, node_layout.y),
                            size: CanvasSize::new(node_layout.width, node_layout.height),
                        };

                        let position_data_arc = cx.position_data();
                        let position_data = position_data_arc.read().unwrap();
                        let zoom = canvas.zoom();

                        let window_point = WindowPoint::from_point(canvas_point);
                        let world_point = position_data.window_to_world(window_point, zoom);

                        // are we resizing?
                        if let Some(handle) = point_in_resize_handle(world_point, &node_bounds) {
                            let resize_op = ResizeOperation::new(
                                handle,
                                node_layout.x,
                                node_layout.y,
                                node_layout.width,
                                node_layout.height,
                            );

                            canvas.set_active_drag(ActiveDrag::new_resize(position, resize_op));
                            canvas.mark_dirty(cx);
                            cx.stop_propagation();
                            return;
                        }
                    }
                }

                // selecting
                if let Some(node_id) = Self::find_top_node_at_point(canvas, canvas_point, cx) {
                    let already_selected = canvas.is_node_selected(node_id);

                    let modifiers = event.modifiers;
                    if !modifiers.shift && !already_selected {
                        canvas.clear_selection(&ClearSelection, window, cx);
                    }

                    if modifiers.shift && already_selected {
                        canvas.deselect_node(node_id);
                    } else {
                        canvas.select_node(node_id);
                    }

                    if canvas.is_node_selected(node_id) {
                        canvas.save_selected_nodes_positions();
                        canvas.set_active_drag(ActiveDrag::new_move_elements(position));
                    }

                    canvas.mark_dirty(cx);
                } else {
                    if !event.modifiers.shift {
                        canvas.clear_selection(&ClearSelection, window, cx);
                    }

                    // selection drag
                    if *active_tool == Tool::Selection {
                        canvas.set_active_drag(ActiveDrag::new_selection(position));
                    }
                    canvas.mark_dirty(cx);
                }
            }
            Tool::Frame => {
                let new_node_id = canvas.generate_id();

                let active_drag = ActiveDrag::new_create_element(position);
                canvas.set_active_element_draw((new_node_id, NodeType::Frame, active_drag));
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
                    let start_pos = active_drag.start_position;
                    let end_pos = active_drag.current_position;

                    let min_x = start_pos.x.min(end_pos.x);
                    let min_y = start_pos.y.min(end_pos.y);
                    let width = (start_pos.x - end_pos.x).abs();
                    let height = (start_pos.y - end_pos.y).abs();

                    // Only create a rectangle if it has meaningful dimensions
                    if width >= 2.0 && height >= 2.0 {
                        let position_data_arc = cx.position_data();
                        let position_data = position_data_arc.read().unwrap();
                        let zoom = canvas.zoom();

                        let window_point = WindowPoint::from_point(Point::new(min_x, min_y));
                        let world_point = position_data.window_to_world(window_point, zoom);

                        let mut rect = FrameNode::new(node_id);

                        *rect.layout_mut() =
                            NodeLayout::new(world_point.x, world_point.y, width, height);

                        rect.set_fill(Some(current_background_color));
                        rect.set_border(Some(current_border_color), 1.0);

                        let new_node_id = canvas.add_node(rect, None, cx);
                        canvas.deselect_all_nodes(cx);
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
                    let position_data_arc = cx.position_data();
                    let position_data = position_data_arc.read().unwrap();
                    let zoom = canvas.zoom();

                    let window_point =
                        WindowPoint::from_point(Point::new(position.x.0, position.y.0));
                    let drop_point = position_data.window_to_world(window_point, zoom).to_point();

                    let selected_ids: Vec<NodeId> =
                        canvas.selected_nodes().iter().cloned().collect();

                    struct ParentFrameInfo {
                        id: NodeId,
                        children: Vec<NodeId>,
                        x: f32,
                        y: f32,
                    }

                    let parent_info = canvas
                        .nodes()
                        .iter()
                        .rev() // Reverse to get top-to-bottom z-order
                        .filter(|node| !selected_ids.contains(&node.id()))
                        .find(|node| {
                            let world_point = WorldPoint::from_point(drop_point);
                            node.contains_point(&world_point)
                        })
                        .map(|parent_frame| ParentFrameInfo {
                            id: parent_frame.id(),
                            children: parent_frame.children().clone(),
                            x: parent_frame.layout().x,
                            y: parent_frame.layout().y,
                        });

                    if let Some(parent_info) = parent_info {
                        for &node_id in &selected_ids {
                            if !parent_info.children.contains(&node_id) {
                                let child_absolute_pos =
                                    if let Some(child_node) = canvas.get_node(node_id) {
                                        let child_layout = child_node.layout();
                                        canvas.get_absolute_position(node_id, cx)
                                    } else {
                                        continue;
                                    };

                                let parent_absolute_pos =
                                    canvas.get_absolute_position(parent_info.id, cx);

                                // Calculate child's position relative to parent
                                // This is the key part for correct parent-relative positioning
                                let relative_x = child_absolute_pos.x - parent_absolute_pos.x;
                                let relative_y = child_absolute_pos.y - parent_absolute_pos.y;

                                // Now update parent to add child
                                if let Some(parent_node) = canvas.get_node_mut(parent_info.id) {
                                    parent_node.add_child(node_id);
                                }

                                // Then set the child's position relative to parent
                                if let Some(child_node) = canvas.get_node_mut(node_id) {
                                    let child_layout = child_node.layout_mut();

                                    // Use the calculated relative coordinates
                                    child_layout.x = relative_x;
                                    child_layout.y = relative_y;
                                }

                                // Update the scene graph to reflect the new parent-child relationship
                                canvas.scene_graph().update(cx, |sg, _cx| {
                                    // Get scene node IDs for parent and child
                                    if let (Some(parent_scene_id), Some(child_scene_id)) = (
                                        sg.get_scene_node_id(parent_info.id),
                                        sg.get_scene_node_id(node_id),
                                    ) {
                                        // Update child bounds to be parent-relative
                                        if let Some(child_node) = canvas.get_node(node_id) {
                                            let layout = child_node.layout();
                                            let bounds = Bounds {
                                                origin: Point::new(layout.x, layout.y),
                                                size: Size::new(layout.width, layout.height),
                                            };
                                            sg.set_local_bounds(child_scene_id, bounds);
                                        }

                                        // Make child a child of parent in scene graph
                                        sg.add_child(parent_scene_id, child_scene_id);
                                    }
                                });
                            }
                        }
                        canvas.mark_dirty(cx);
                    }

                    // Finalize the move by clearing initial positions
                    canvas.element_initial_positions_mut().clear();
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
        if let Some(active_drag) = canvas.active_drag().take() {
            // Update the drag with new position
            let new_drag = ActiveDrag {
                start_position: active_drag.start_position,
                current_position: WindowPoint::new(position.x.0, position.y.0),
                drag_type: active_drag.drag_type.clone(),
            };
            canvas.set_active_drag(new_drag.clone());

            // For any non-Selection tool, only allow specific drag types
            if active_tool != Tool::Selection && matches!(new_drag.drag_type, DragType::Selection) {
                // If using a non-Selection tool but having a Selection drag, cancel it
                canvas.clear_active_drag();
                return;
            }

            match new_drag.drag_type {
                DragType::Selection => {
                    // Handle selection rectangle
                    if active_tool == Tool::Selection {
                        // Calculate the selection rectangle in canvas coordinates
                        let start_pos = active_drag.start_position;
                        let window_position = WindowPoint::new(position.x.0, position.y.0);
                        let min_x = start_pos.x.min(window_position.x);
                        let min_y = start_pos.y.min(window_position.y);
                        let max_x = start_pos.x.max(window_position.x);
                        let max_y = start_pos.y.max(window_position.y);

                        // Convert to canvas coordinates
                        use crate::coordinates::{PositionStore, WindowPoint, WorldPoint};

                        // Get position data and zoom from canvas
                        let position_data_arc = cx.position_data();
                        let position_data = position_data_arc.read().unwrap();
                        let zoom = canvas.zoom();

                        // Convert window points to world points
                        let min_window_point = WindowPoint::from_point(Point::new(min_x, min_y));
                        let max_window_point = WindowPoint::from_point(Point::new(max_x, max_y));
                        let min_point = position_data
                            .window_to_world(min_window_point, zoom)
                            .to_point();
                        let max_point = position_data
                            .window_to_world(max_window_point, zoom)
                            .to_point();

                        // Create selection bounds
                        let selection_bounds = Bounds {
                            origin: min_point,
                            size: Size::new(max_point.x - min_point.x, max_point.y - min_point.y),
                        };

                        // Pre-calculate all nodes that intersect with selection
                        let nodes_in_selection: HashSet<NodeId> = canvas
                            .nodes()
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
                            if nodes_in_selection != canvas.selected_nodes().clone() {
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
                    if !canvas.selected_nodes().is_empty() {
                        // Calculate the drag delta in canvas coordinates
                        let delta = new_drag.delta();

                        // Get current canvas point to check for potential parent frames
                        use crate::coordinates::{PositionStore, WindowPoint, WorldPoint};

                        // Get position data and zoom from canvas
                        let position_data_arc = cx.position_data();
                        let position_data = position_data_arc.read().unwrap();
                        let zoom = canvas.zoom();

                        // Convert window point to world point
                        let window_point =
                            WindowPoint::from_point(Point::new(position.x.0, position.y.0));
                        let canvas_point =
                            position_data.window_to_world(window_point, zoom).to_point();

                        // Get all the selected node IDs
                        let selected_ids: Vec<NodeId> =
                            canvas.selected_nodes().iter().cloned().collect();

                        // Find potential parent frame at the current position
                        let potential_parent = canvas
                            .nodes()
                            .iter()
                            .rev() // Reverse to get top-to-bottom z-order
                            .filter(|node| !selected_ids.contains(&node.id()))
                            .find(|node| {
                                // Convert to WorldPoint for contains_point check
                                let world_point =
                                    crate::coordinates::WorldPoint::from_point(canvas_point);
                                node.contains_point(&world_point)
                            })
                            .map(|node| node.id());

                        // Update the potential parent frame
                        canvas.set_potential_parent_frame(potential_parent);

                        // Move all selected nodes with the drag delta
                        canvas.move_selected_nodes_with_drag(delta, cx);
                    }
                }
                DragType::CreateElement => {
                    // Nothing to do here - handled in the rectangle drawing code below
                }
                DragType::Resize(mut resize_op) => {
                    if canvas.selected_nodes().len() == 1 {
                        let zoom = canvas.zoom();

                        let selected_node_id = *canvas.selected_nodes().iter().next().unwrap();
                        if let Some(node) = canvas.get_node_mut(selected_node_id) {
                            let delta = Point::new(
                                (position.x.0 - active_drag.start_position.x) / zoom,
                                (position.y.0 - active_drag.start_position.y) / zoom,
                            );

                            let preserve_aspect_ratio = event.modifiers.shift;
                            let resize_from_center = event.modifiers.alt;

                            resize_op.config.preserve_aspect_ratio = preserve_aspect_ratio;
                            resize_op.config.resize_from_center = resize_from_center;

                            let mut new_x = resize_op.original_position.x;
                            let mut new_y = resize_op.original_position.y;
                            let mut new_width = resize_op.original_size.width;
                            let mut new_height = resize_op.original_size.height;

                            let aspect_ratio = if preserve_aspect_ratio {
                                resize_op.original_size.width / resize_op.original_size.height
                            } else {
                                0.0
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
                                            new_width = resize_op.original_size.width + width_delta;
                                            new_height =
                                                resize_op.original_size.height + adj_height;
                                            new_x = resize_op.original_position.x - width_delta;
                                            new_y = resize_op.original_position.y - adj_height;
                                        } else {
                                            let adj_width = height_delta * aspect_ratio;
                                            new_width = resize_op.original_size.width + adj_width;
                                            new_height =
                                                resize_op.original_size.height + height_delta;
                                            new_x = resize_op.original_position.x - adj_width;
                                            new_y = resize_op.original_position.y - height_delta;
                                        }
                                    } else {
                                        // Standard resize without aspect ratio constraint
                                        new_width = resize_op.original_size.width + width_delta;
                                        new_height = resize_op.original_size.height + height_delta;
                                        new_x = resize_op.original_position.x - width_delta;
                                        new_y = resize_op.original_position.y - height_delta;
                                    }
                                }
                                ResizeHandle::TopRight => {
                                    // Width change is positive, height change is negative
                                    let width_delta = delta.x;
                                    let height_delta = -delta.y;

                                    if preserve_aspect_ratio {
                                        if width_delta.abs() / aspect_ratio > height_delta.abs() {
                                            let adj_height = width_delta / aspect_ratio;
                                            new_width = resize_op.original_size.width + width_delta;
                                            new_height =
                                                resize_op.original_size.height + adj_height;
                                            new_y = resize_op.original_position.y - adj_height;
                                        } else {
                                            let adj_width = height_delta * aspect_ratio;
                                            new_width = resize_op.original_size.width + adj_width;
                                            new_height =
                                                resize_op.original_size.height + height_delta;
                                            new_y = resize_op.original_position.y - height_delta;
                                        }
                                    } else {
                                        new_width = resize_op.original_size.width + width_delta;
                                        new_height = resize_op.original_size.height + height_delta;
                                        new_y = resize_op.original_position.y - height_delta;
                                    }
                                }
                                ResizeHandle::BottomLeft => {
                                    // Width change is negative, height change is positive
                                    let width_delta = -delta.x;
                                    let height_delta = delta.y;

                                    if preserve_aspect_ratio {
                                        if width_delta.abs() / aspect_ratio > height_delta.abs() {
                                            let adj_height = width_delta / aspect_ratio;
                                            new_width = resize_op.original_size.width + width_delta;
                                            new_height =
                                                resize_op.original_size.height + adj_height;
                                            new_x = resize_op.original_position.x - width_delta;
                                        } else {
                                            let adj_width = height_delta * aspect_ratio;
                                            new_width = resize_op.original_size.width + adj_width;
                                            new_height =
                                                resize_op.original_size.height + height_delta;
                                            new_x = resize_op.original_position.x - adj_width;
                                        }
                                    } else {
                                        new_width = resize_op.original_size.width + width_delta;
                                        new_height = resize_op.original_size.height + height_delta;
                                        new_x = resize_op.original_position.x - width_delta;
                                    }
                                }
                                ResizeHandle::BottomRight => {
                                    let width_delta = delta.x;
                                    let height_delta = delta.y;

                                    if preserve_aspect_ratio {
                                        if width_delta.abs() / aspect_ratio > height_delta.abs() {
                                            let adj_height = width_delta / aspect_ratio;
                                            new_width = resize_op.original_size.width + width_delta;
                                            new_height =
                                                resize_op.original_size.height + adj_height;
                                        } else {
                                            let adj_width = height_delta * aspect_ratio;
                                            new_width = resize_op.original_size.width + adj_width;
                                            new_height =
                                                resize_op.original_size.height + height_delta;
                                        }
                                    } else {
                                        new_width = resize_op.original_size.width + width_delta;
                                        new_height = resize_op.original_size.height + height_delta;
                                    }
                                }
                            }

                            // If resize from center is enabled, adjust position to keep center fixed
                            if resize_from_center {
                                let orig_center_x = resize_op.original_position.x
                                    + resize_op.original_size.width / 2.0;
                                let orig_center_y = resize_op.original_position.y
                                    + resize_op.original_size.height / 2.0;
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
                                        new_x = resize_op.original_position.x
                                            + resize_op.original_size.width;
                                    } else {
                                        // Normal case - right edge stays fixed
                                        new_x = resize_op.original_position.x
                                            + resize_op.original_size.width
                                            - new_width;
                                    }

                                    // Handle vertical resizing (top edge)
                                    if new_height < 0.0 {
                                        // Crossed bottom edge - fixed point switches to top
                                        new_height = -new_height;
                                        // Top edge is now at original bottom edge + the overflow
                                        new_y = resize_op.original_position.y
                                            + resize_op.original_size.height;
                                    } else {
                                        // Normal case - bottom edge stays fixed
                                        new_y = resize_op.original_position.y
                                            + resize_op.original_size.height
                                            - new_height;
                                    }
                                }
                                ResizeHandle::TopRight => {
                                    // Handle horizontal resizing (right edge)
                                    if new_width < 0.0 {
                                        // Crossed left edge - fixed point switches to right
                                        new_width = -new_width;
                                        // Keep the original x, width grows to the left
                                        new_x = resize_op.original_position.x - new_width;
                                    } else {
                                        // Normal case - left edge stays fixed at original x
                                        new_x = resize_op.original_position.x;
                                    }

                                    // Handle vertical resizing (top edge)
                                    if new_height < 0.0 {
                                        // Crossed bottom edge - fixed point switches to top
                                        new_height = -new_height;
                                        // Top edge is now at original bottom edge + the overflow
                                        new_y = resize_op.original_position.y
                                            + resize_op.original_size.height;
                                    } else {
                                        // Normal case - bottom edge stays fixed
                                        new_y = resize_op.original_position.y
                                            + resize_op.original_size.height
                                            - new_height;
                                    }
                                }
                                ResizeHandle::BottomLeft => {
                                    // Handle horizontal resizing (left edge)
                                    if new_width < 0.0 {
                                        // Crossed right edge - fixed point switches to left
                                        new_width = -new_width;
                                        // Left edge is now at original right edge + the overflow
                                        new_x = resize_op.original_position.x
                                            + resize_op.original_size.width;
                                    } else {
                                        // Normal case - right edge stays fixed
                                        new_x = resize_op.original_position.x
                                            + resize_op.original_size.width
                                            - new_width;
                                    }

                                    // Handle vertical resizing (bottom edge)
                                    if new_height < 0.0 {
                                        // Crossed top edge - fixed point switches to bottom
                                        new_height = -new_height;
                                        // Keep original y, height grows upward
                                        new_y = resize_op.original_position.y - new_height;
                                    } else {
                                        // Normal case - top edge stays fixed at original y
                                        new_y = resize_op.original_position.y;
                                    }
                                }
                                ResizeHandle::BottomRight => {
                                    // Handle horizontal resizing (right edge)
                                    if new_width < 0.0 {
                                        // Crossed left edge - fixed point switches to right
                                        new_width = -new_width;
                                        // Keep the original x, width grows to the left
                                        new_x = resize_op.original_position.x - new_width;
                                    } else {
                                        // Normal case - left edge stays fixed at original x
                                        new_x = resize_op.original_position.x;
                                    }

                                    // Handle vertical resizing (bottom edge)
                                    if new_height < 0.0 {
                                        // Crossed top edge - fixed point switches to bottom
                                        new_height = -new_height;
                                        // Keep original y, height grows upward
                                        new_y = resize_op.original_position.y - new_height;
                                    } else {
                                        // Normal case - top edge stays fixed at original y
                                        new_y = resize_op.original_position.y;
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

                                    // Update child node layouts to reflect parent's resize
                                    canvas.update_child_layouts_after_parent_resize(
                                        selected_node_id,
                                        cx,
                                    );
                                }
                            }

                            // Update the resize operation in the drag
                            let updated_drag = ActiveDrag {
                                start_position: active_drag.start_position,
                                current_position: WindowPoint::new(position.x.0, position.y.0),
                                drag_type: DragType::Resize(resize_op),
                            };
                            canvas.set_active_drag(updated_drag);
                        }
                    }
                }
            }

            canvas.mark_dirty(cx);
        }

        // Handle rectangle drawing
        if let Some(active_draw) = canvas.active_element_draw().take() {
            match *cx.active_tool().clone() {
                Tool::Frame => {
                    let new_drag = ActiveDrag {
                        start_position: active_draw.2.start_position,
                        current_position: WindowPoint::new(position.x.0, position.y.0),
                        drag_type: DragType::CreateElement,
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
        let hovered = Self::find_top_node_at_point(canvas, canvas_point, cx);

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
                .x
                .min(active_drag.current_position.x),
        ));
        let min_y = round_to_pixel(Pixels(
            active_drag
                .start_position
                .y
                .min(active_drag.current_position.y),
        ));
        let width = round_to_pixel(Pixels(
            (active_drag.start_position.x - active_drag.current_position.x).abs(),
        ));
        let height = round_to_pixel(Pixels(
            (active_drag.start_position.y - active_drag.current_position.y).abs(),
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
        let min_x = start_pos.x.min(current_pos.x);
        let min_y = start_pos.y.min(current_pos.y);
        let width = (start_pos.x - current_pos.x).abs();
        let height = (start_pos.y - current_pos.y).abs();

        // Get the scale factor from the window
        let scale_factor = window.scale_factor();

        // Convert to WorldPoint and apply snapping for pixel-perfect rendering
        // We use WorldPoint here because it's the most appropriate for this operation
        let world_point = WorldPoint::new(min_x, min_y);
        let snapped_point = world_point.snapped(true, scale_factor); // Use whole-pixel snapping

        let position = Point::new(Pixels(snapped_point.x), Pixels(snapped_point.y));

        // Also snap the size to prevent jitter
        let world_size = WorldPoint::new(width, height);
        let snapped_size = world_size.snapped(true, scale_factor);

        let rect_bounds = Bounds {
            origin: position,
            size: Size::new(Pixels(snapped_size.x), Pixels(snapped_size.y)),
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

    /// Paint a grid on the canvas.
    ///
    /// This draws a grid with major lines every 50px, minor lines every 10px,
    /// and a stronger line at the origin (0,0).
    fn paint_grid(&self, layout: &CanvasLayout, window: &mut Window, cx: &mut App) {
        window.paint_layer(layout.hitbox.bounds, |window| {
            // Get canvas state to determine current scroll position and zoom
            let canvas = self.canvas.read(cx);
            let scroll_position = canvas.get_scroll_position();
            let zoom = canvas.zoom();

            // Determine visible area in canvas coordinates
            use crate::coordinates::{PositionStore, WindowPoint, WorldPoint};

            // Get position data and zoom from canvas
            let position_data_arc = cx.position_data();
            let position_data = position_data_arc.read().unwrap();
            // No need to re-get the zoom since we already have it above

            let bounds = layout.hitbox.bounds;

            // Convert window points to world points
            let top_left_window =
                WindowPoint::from_point(Point::new(bounds.origin.x.0, bounds.origin.y.0));
            let bottom_right_window = WindowPoint::from_point(Point::new(
                bounds.origin.x.0 + bounds.size.width.0,
                bounds.origin.y.0 + bounds.size.height.0,
            ));

            let top_left = position_data
                .window_to_world(top_left_window, zoom)
                .to_point();
            let bottom_right = position_data
                .window_to_world(bottom_right_window, zoom)
                .to_point();

            // Calculate grid boundaries
            // Add some margin to ensure we cover the entire visible area
            let start_x = (top_left.x / 10.0).floor() * 10.0 - 10.0;
            let end_x = (bottom_right.x / 10.0).ceil() * 10.0 + 10.0;
            let start_y = (top_left.y / 10.0).floor() * 10.0 - 10.0;
            let end_y = (bottom_right.y / 10.0).ceil() * 10.0 + 10.0;

            // Get foreground color for grid lines with different alpha values
            let theme = Theme::get_global(cx);
            let minor_color = theme.tokens.text.opacity(0.05);
            let major_color = theme.tokens.text.opacity(0.1);
            let origin_color = theme.tokens.text.opacity(0.2);

            // Draw vertical grid lines
            for x in (start_x as i32..=end_x as i32).step_by(10) {
                let x_pos = x as f32;
                // Convert canvas coordinate to window coordinate
                let window_x = (x_pos - scroll_position.x) * zoom + bounds.origin.x.0;

                // Get line color based on position
                let color = if x == 0 {
                    // Origin line
                    origin_color
                } else if x % 50 == 0 {
                    // Major line
                    major_color
                } else {
                    // Minor line
                    minor_color
                };

                // Draw the vertical line
                let line_bounds = Bounds {
                    origin: Point::new(gpui::Pixels(window_x), bounds.origin.y),
                    size: Size::new(gpui::Pixels(1.0), bounds.size.height),
                };
                window.paint_quad(gpui::fill(line_bounds, color));
            }

            // Draw horizontal grid lines
            for y in (start_y as i32..=end_y as i32).step_by(10) {
                let y_pos = y as f32;
                // Convert canvas coordinate to window coordinate
                let window_y = (y_pos - scroll_position.y) * zoom + bounds.origin.y.0;

                // Get line color based on position
                let color = if y == 0 {
                    // Origin line
                    origin_color
                } else if y % 50 == 0 {
                    // Major line
                    major_color
                } else {
                    // Minor line
                    minor_color
                };

                // Draw the horizontal line
                let line_bounds = Bounds {
                    origin: Point::new(bounds.origin.x, gpui::Pixels(window_y)),
                    size: Size::new(bounds.size.width, gpui::Pixels(1.0)),
                };
                window.paint_quad(gpui::fill(line_bounds, color));
            }

            window.request_animation_frame();
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
                                shadows: node.shadows(),
                                children: node.children().clone(),
                            });
                        }
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
                parent_transform: Option<TransformationMatrix>,
                theme: &Theme,
                window: &mut gpui::Window,
            ) {
                // Get coordinates in parent space
                let (frame_x, frame_y) = (node_info.bounds.origin.x.0, node_info.bounds.origin.y.0);
                let (frame_width, frame_height) = (
                    node_info.bounds.size.width.0,
                    node_info.bounds.size.height.0,
                );

                // Apply parent's transform if available, or use node's bounds directly
                let transformed_bounds = if let Some(transform) = parent_transform {
                    // Convert to gpui Points and apply the transformation
                    let top_left = transform.apply(gpui::Point::new(
                        gpui::Pixels(frame_x),
                        gpui::Pixels(frame_y),
                    ));

                    let bottom_right = transform.apply(gpui::Point::new(
                        gpui::Pixels(frame_x + frame_width),
                        gpui::Pixels(frame_y + frame_height),
                    ));

                    // Create bounds from transformed points
                    gpui::Bounds {
                        origin: top_left,
                        size: gpui::Size::new(
                            gpui::Pixels(bottom_right.x.0 - top_left.x.0),
                            gpui::Pixels(bottom_right.y.0 - top_left.y.0),
                        ),
                    }
                } else {
                    // No parent transform, use bounds directly
                    node_info.bounds
                };

                // Create a transformation matrix for children
                // This creates a new coordinate system relative to this frame
                let child_transform = TransformationMatrix::unit()
                    .compose(parent_transform.unwrap_or_else(TransformationMatrix::unit))
                    .translate(point(
                        gpui::Pixels(frame_x).scale(1.0),
                        gpui::Pixels(frame_y).scale(1.0),
                    ));

                // FIRST: Paint any shadows behind the node
                // Shadows need to be rendered before the node itself
                if !node_info.shadows.is_empty() {
                    // Convert our Shadow types to gpui::BoxShadow types
                    let box_shadows: Vec<gpui::BoxShadow> = node_info
                        .shadows
                        .iter()
                        .map(|shadow| gpui::BoxShadow {
                            offset: gpui::Point::new(
                                gpui::Pixels(shadow.offset.x),
                                gpui::Pixels(shadow.offset.y),
                            ),
                            blur_radius: gpui::Pixels(shadow.blur_radius),
                            spread_radius: gpui::Pixels(shadow.spread_radius),
                            color: shadow.color,
                        })
                        .collect();

                    // Apply snapping for render-time only to prevent jittery shadows
                    // Convert bounds coordinates to WorldPoint, snap them, and convert back
                    let scale_factor = window.scale_factor();
                    let origin_world = WorldPoint::new(
                        transformed_bounds.origin.x.0,
                        transformed_bounds.origin.y.0,
                    );
                    let size_world = WorldPoint::new(
                        transformed_bounds.size.width.0,
                        transformed_bounds.size.height.0,
                    );

                    let snapped_origin = origin_world.snapped(true, scale_factor); // Use whole-pixel snapping for shadows
                    let snapped_size = size_world.snapped(true, scale_factor);

                    let snapped_bounds = gpui::Bounds {
                        origin: gpui::Point::new(
                            gpui::Pixels(snapped_origin.x),
                            gpui::Pixels(snapped_origin.y),
                        ),
                        size: gpui::Size::new(
                            gpui::Pixels(snapped_size.x),
                            gpui::Pixels(snapped_size.y),
                        ),
                    };

                    // Use the dedicated shadow rendering function with snapped bounds
                    window.paint_shadows(
                        snapped_bounds,
                        gpui::Corners::all(gpui::Pixels(node_info.corner_radius)),
                        &box_shadows,
                    );
                }

                // SECOND: Paint the node itself (background and frame)
                // Paint the fill if it exists
                if let Some(fill_color) = node_info.fill_color {
                    // Apply snapping for render-time only to prevent jittery fill
                    // Convert bounds coordinates to WorldPoint, snap them, and convert back
                    let scale_factor = window.scale_factor();
                    let origin_world = WorldPoint::new(
                        transformed_bounds.origin.x.0,
                        transformed_bounds.origin.y.0,
                    );
                    let size_world = WorldPoint::new(
                        transformed_bounds.size.width.0,
                        transformed_bounds.size.height.0,
                    );

                    let snapped_origin = origin_world.snapped(true, scale_factor); // Use whole-pixel snapping
                    let snapped_size = size_world.snapped(true, scale_factor);

                    let snapped_bounds = gpui::Bounds {
                        origin: gpui::Point::new(
                            gpui::Pixels(snapped_origin.x),
                            gpui::Pixels(snapped_origin.y),
                        ),
                        size: gpui::Size::new(
                            gpui::Pixels(snapped_size.x),
                            gpui::Pixels(snapped_size.y),
                        ),
                    };

                    window.paint_quad(gpui::PaintQuad {
                        bounds: snapped_bounds,
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
                                    Some(child_transform),
                                    theme,
                                    window,
                                );
                            }
                        },
                    );
                }

                // THIRD: Paint the border if it exists (after children, so it's on top)
                if let Some(border_color) = node_info.border_color {
                    // Apply snapping for render-time only to prevent jittery borders
                    // Convert bounds coordinates to WorldPoint, snap them, and convert back
                    let scale_factor = window.scale_factor();
                    let origin_world = WorldPoint::new(
                        transformed_bounds.origin.x.0,
                        transformed_bounds.origin.y.0,
                    );
                    let size_world = WorldPoint::new(
                        transformed_bounds.size.width.0,
                        transformed_bounds.size.height.0,
                    );

                    let snapped_origin = origin_world.snapped(true, scale_factor); // Use whole-pixel snapping
                    let snapped_size = size_world.snapped(true, scale_factor);

                    let snapped_bounds = gpui::Bounds {
                        origin: gpui::Point::new(
                            gpui::Pixels(snapped_origin.x),
                            gpui::Pixels(snapped_origin.y),
                        ),
                        size: gpui::Size::new(
                            gpui::Pixels(snapped_size.x),
                            gpui::Pixels(snapped_size.y),
                        ),
                    };

                    window.paint_quad(gpui::PaintQuad {
                        bounds: snapped_bounds,
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
                    None, // No parent transform for root nodes
                    &theme,
                    window,
                );
            }

            // SECOND PASS: Paint all selection outlines and resize handles
            // ===========================================================

            // Build a map of node ID to parent transform
            let mut node_transforms: HashMap<NodeId, TransformationMatrix> = HashMap::new();

            // Helper function to compute the absolute transform for a node
            fn compute_node_transform(
                node_id: NodeId,
                node_map: &HashMap<NodeId, Vec<NodeRenderInfo>>,
                transforms: &mut HashMap<NodeId, TransformationMatrix>,
                all_nodes: &[NodeRenderInfo],
            ) -> TransformationMatrix {
                // If we've already computed this node's transform, return it
                if let Some(transform) = transforms.get(&node_id) {
                    return *transform;
                }

                // Find this node's information
                if let Some(node_info) = all_nodes.iter().find(|n| n.node_id == node_id) {
                    // Find this node's parent (if any)
                    let mut parent_id = None;
                    for (pid, children) in node_map {
                        if children.iter().any(|c| c.node_id == node_id) {
                            parent_id = Some(*pid);
                            break;
                        }
                    }

                    // Get the parent's transform if it exists
                    let parent_transform = if let Some(pid) = parent_id {
                        compute_node_transform(pid, node_map, transforms, all_nodes)
                    } else {
                        TransformationMatrix::unit()
                    };

                    // Apply this node's local transform to the parent's transform
                    let (x, y) = (node_info.bounds.origin.x.0, node_info.bounds.origin.y.0);
                    let transform =
                        parent_transform.compose(TransformationMatrix::unit().translate(point(
                            gpui::Pixels(x).scale(1.0),
                            gpui::Pixels(y).scale(1.0),
                        )));

                    // Cache and return the combined transform
                    transforms.insert(node_id, transform);
                    transform
                } else {
                    // If node not found, return identity transform
                    TransformationMatrix::unit()
                }
            }

            // Compute transforms for all selected nodes
            for node_id in selected_node_ids.iter() {
                compute_node_transform(
                    *node_id,
                    &children_map,
                    &mut node_transforms,
                    &nodes_to_render,
                );
            }

            // First draw individual selection outlines
            for node_info in &nodes_to_render {
                if selected_node_ids.contains(&node_info.node_id) {
                    // Get the absolute transform for this node
                    let transform = node_transforms
                        .get(&node_info.node_id)
                        .cloned()
                        .unwrap_or_else(TransformationMatrix::unit);

                    // Get the bounds in local space (no parent translation)
                    let (width, height) = (
                        node_info.bounds.size.width.0,
                        node_info.bounds.size.height.0,
                    );

                    // Transform the corners to get absolute position
                    let top_left =
                        transform.apply(gpui::Point::new(gpui::Pixels(0.0), gpui::Pixels(0.0)));

                    let bottom_right = transform
                        .apply(gpui::Point::new(gpui::Pixels(width), gpui::Pixels(height)));

                    // Create selection bounds from transformed corners
                    let selection_bounds = gpui::Bounds {
                        origin: gpui::Point::new(
                            top_left.x - gpui::Pixels(2.0),
                            top_left.y - gpui::Pixels(2.0),
                        ),
                        size: gpui::Size::new(
                            gpui::Pixels(bottom_right.x.0 - top_left.x.0) + gpui::Pixels(4.0),
                            gpui::Pixels(bottom_right.y.0 - top_left.y.0) + gpui::Pixels(4.0),
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
                self.paint_grid(layout, window, cx); // Paint grid after background but before nodes
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
                        Tool::Frame => {
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
