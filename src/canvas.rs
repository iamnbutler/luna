#![allow(unused, dead_code)]

use crate::{
    coordinates::{
        CanvasPoint, CanvasRect, CanvasSize, PointSource, UnresolvedCanvasPoint, WindowPoint,
        WindowRect, WindowSize,
    },
    interactivity::ActiveDrag,
    node::{AnyNode, CanvasNode, NodeId, NodeType, RectangleNode, ShapeNode},
    ToolKind,
};
use gpui::{
    actions, canvas as gpui_canvas, div, hsla, prelude::*, size, Action, App, Bounds, Context,
    ContextEntry, DispatchPhase, Element, Entity, EntityInputHandler, FocusHandle, Focusable,
    InputHandler, InteractiveElement, IntoElement, KeyContext, ParentElement, Point, Render, Size,
    Styled, Window, px,
};
use std::{
    any::TypeId,
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    rc::Rc,
};
use taffy::{prelude::*, Rect};

actions!(canvas, [ClearSelection]);

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Default)]
pub struct CanvasActionId(usize);

impl CanvasActionId {
    pub fn increment(&mut self) -> Self {
        let new_id = self.0;

        *self = Self(new_id + 1);

        Self(new_id)
    }
}

pub fn register_canvas_action<T: Action>(
    canvas: &Entity<Canvas>,
    window: &mut Window,
    listener: impl Fn(&mut Canvas, &T, &mut Window, &mut Context<Canvas>) + 'static,
) {
    let canvas = canvas.clone();
    window.on_action(TypeId::of::<T>(), move |action, phase, window, cx| {
        let action = action.downcast_ref().unwrap();
        if phase == DispatchPhase::Bubble {
            canvas.update(cx, |canvas, cx| {
                listener(canvas, action, window, cx);
            })
        }
    })
}

/// A Canvas manages a collection of nodes that can be rendered and manipulated
pub struct Canvas {
    /// Mapping of NodeId to actual nodes
    pub nodes: HashMap<NodeId, AnyNode>,

    /// Currently selected nodes
    pub selected_nodes: HashSet<NodeId>,

    /// The visible viewport of the canvas in canvas coordinates
    viewport: Bounds<f32>,

    /// The current scroll position (origin offset) of the canvas
    scroll_position: Point<f32>,

    /// Zoom level of the canvas (1.0 = 100%)
    zoom: f32,

    /// The full content bounds of all nodes
    content_bounds: Bounds<f32>,

    /// Next ID to assign to a new node
    next_id: usize,

    /// Layout engine instance
    taffy: TaffyTree,

    /// Mapping from our NodeId to taffy NodeId
    node_to_taffy: HashMap<NodeId, taffy::NodeId>,

    /// Whether the canvas needs to be re-rendered
    dirty: bool,

    focus_handle: FocusHandle,
    pub actions:
        Rc<RefCell<BTreeMap<CanvasActionId, Box<dyn Fn(&mut Window, &mut Context<Self>)>>>>,
    pub active_tool: ToolKind,
    pub active_drag: Option<ActiveDrag>,
    
    /// Tracks an active drawing operation (e.g., rectangle being drawn)
    pub active_element_draw: Option<(NodeId, NodeType, ActiveDrag)>,
}

impl Canvas {
    /// Create a new canvas
    pub fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        // Create the taffy layout engine
        let taffy = TaffyTree::new();

        let initial_viewport_px = window.viewport_size();
        let initial_viewport = size(initial_viewport_px.width.0, initial_viewport_px.height.0);

        // Create an initial viewport with reasonable size
        let viewport = Bounds {
            origin: Point::new(0.0, 0.0),
            size: initial_viewport,
        };

        let content_bounds = viewport.clone();

        let mut canvas = Self {
            nodes: HashMap::new(),
            selected_nodes: HashSet::new(),
            viewport,
            scroll_position: Point::new(0.0, 0.0),
            zoom: 1.0,
            content_bounds,
            next_id: 1,
            taffy,
            node_to_taffy: HashMap::new(),
            dirty: true,
            focus_handle: cx.focus_handle(),
            actions: Rc::default(),
            active_tool: ToolKind::default(),
            active_drag: None,
            active_element_draw: None,
        };

        canvas.create_test_rectangle();
        canvas
    }

    /// Generate a unique ID for a new node
    pub fn generate_id(&mut self) -> NodeId {
        let id = NodeId::new(self.next_id);
        self.next_id += 1;
        id
    }

    /// Resolve an UnresolvedCanvasPoint to a fully resolved CanvasPoint
    /// This is the only way to get a valid CanvasPoint
    pub fn resolve_canvas_point(&self, unresolved: UnresolvedCanvasPoint) -> CanvasPoint {
        match unresolved.source {
            PointSource::Window => {
                // For window points, apply zoom and scroll transformations
                let canvas_x = (unresolved.x / self.zoom) + self.scroll_position.x;
                let canvas_y = (unresolved.y / self.zoom) + self.scroll_position.y;
                CanvasPoint::new(canvas_x, canvas_y)
            }
            PointSource::PartiallyResolved => {
                // For partially resolved points, assume they are already in canvas space
                CanvasPoint::new(unresolved.x, unresolved.y)
            }
        }
    }
    
    /// Convert a fully resolved CanvasPoint to a WindowPoint
    pub fn canvas_to_window_point(&self, canvas_point: CanvasPoint) -> WindowPoint {
        let window_x = (canvas_point.x - self.scroll_position.x) * self.zoom;
        let window_y = (canvas_point.y - self.scroll_position.y) * self.zoom;
        WindowPoint::new(px(window_x), px(window_y))
    }
    
    /// Convert window coordinates to canvas coordinates (legacy method)
    /// This will be deprecated eventually, use resolve_canvas_point instead
    pub fn window_to_canvas_point(&self, window_point: Point<f32>) -> Point<f32> {
        let unresolved = UnresolvedCanvasPoint {
            x: window_point.x,
            y: window_point.y,
            source: PointSource::Window,
        };
        let canvas_point = self.resolve_canvas_point(unresolved);
        Point::new(canvas_point.x, canvas_point.y)
    }
    
    /// Convert a CanvasRect to a WindowRect
    pub fn canvas_rect_to_window_rect(&self, canvas_rect: CanvasRect) -> WindowRect {
        let top_left = self.canvas_to_window_point(canvas_rect.origin);
        
        // Apply zoom to get window dimensions
        let width = px(canvas_rect.size.width * self.zoom);
        let height = px(canvas_rect.size.height * self.zoom);
        
        WindowRect::new(top_left, WindowSize::new(width, height))
    }
    
    /// Resolve an UnresolvedCanvasPoint to a WindowPoint
    pub fn unresolved_to_window_point(&self, unresolved: UnresolvedCanvasPoint) -> WindowPoint {
        let canvas_point = self.resolve_canvas_point(unresolved);
        self.canvas_to_window_point(canvas_point)
    }

    pub fn active_tool(&self) -> &ToolKind {
        &self.active_tool
    }

    pub fn set_active_tool(&mut self, tool: ToolKind) {
        self.active_tool = tool;
    }

    /// Add a node to the canvas
    pub fn add_node<T: CanvasNode + 'static>(&mut self, node: T) -> NodeId {
        let node_id = node.id();

        // Create a taffy node for layout
        let taffy_style = node.common().style.clone();
        let taffy_node = self.taffy.new_leaf(taffy_style).unwrap();

        // Map our node ID to taffy node ID
        self.node_to_taffy.insert(node_id, taffy_node);

        // Create an AnyNode and add to nodes map
        let any_node = AnyNode::new(node);
        self.nodes.insert(node_id, any_node);

        // Mark canvas as dirty
        self.dirty = true;

        node_id
    }

    /// Remove a node from the canvas
    pub fn remove_node(&mut self, node_id: NodeId) -> Option<AnyNode> {
        // Remove from taffy
        if let Some(taffy_node) = self.node_to_taffy.remove(&node_id) {
            let _ = self.taffy.remove(taffy_node);
        }

        // Remove from selection
        self.selected_nodes.remove(&node_id);

        // Remove from nodes map
        let node = self.nodes.remove(&node_id);

        // Mark canvas as dirty
        self.dirty = true;

        node
    }

    /// Select a node
    pub fn select_node(&mut self, node_id: NodeId) {
        if self.nodes.contains_key(&node_id) {
            self.selected_nodes.insert(node_id);
            self.dirty = true;
        }
    }

    /// Deselect a node
    pub fn deselect_node(&mut self, node_id: NodeId) {
        self.selected_nodes.remove(&node_id);
        self.dirty = true;
    }

    /// Clear all selections
    pub fn clear_selection(
        &mut self,
        _: &ClearSelection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selected_nodes.clear();
        self.dirty = true;
    }

    /// Toggle selection state of a node
    pub fn toggle_node_selection(&mut self, node_id: NodeId) {
        if self.selected_nodes.contains(&node_id) {
            self.selected_nodes.remove(&node_id);
        } else if self.nodes.contains_key(&node_id) {
            self.selected_nodes.insert(node_id);
        }
        self.dirty = true;
    }

    /// Check if a node is selected
    pub fn is_node_selected(&self, node_id: NodeId) -> bool {
        self.selected_nodes.contains(&node_id)
    }

    /// Get nodes at a specific window point (for hit testing)
    pub fn nodes_at_window_point(&self, window_point: WindowPoint) -> Vec<NodeId> {
        // Convert window point to canvas coordinates
        let unresolved = window_point.to_unresolved_canvas();
        let canvas_point = self.resolve_canvas_point(unresolved);

        // Get all shape nodes that contain this point
        let mut hit_nodes = Vec::new();

        for (id, node) in &self.nodes {
            // Test if point is inside rectangle node
            if let Some(rect_node) = node.downcast_ref::<RectangleNode>() {
                if ShapeNode::contains_point(rect_node, &canvas_point) {
                    hit_nodes.push(*id);
                }
            }
        }

        // Sort nodes by z-order (higher IDs drawn on top)
        hit_nodes.sort_by(|a, b| b.0.cmp(&a.0));

        hit_nodes
    }
    
    /// Get nodes at a specific canvas point (for hit testing)
    pub fn nodes_at_canvas_point(&self, canvas_point: CanvasPoint) -> Vec<NodeId> {
        // Get all shape nodes that contain this point
        let mut hit_nodes = Vec::new();

        for (id, node) in &self.nodes {
            // Test if point is inside rectangle node
            if let Some(rect_node) = node.downcast_ref::<RectangleNode>() {
                if ShapeNode::contains_point(rect_node, &canvas_point) {
                    hit_nodes.push(*id);
                }
            }
        }

        // Sort nodes by z-order (higher IDs drawn on top)
        hit_nodes.sort_by(|a, b| b.0.cmp(&a.0));

        hit_nodes
    }

    /// Get the topmost node at a specific window point
    pub fn top_node_at_window_point(&self, window_point: WindowPoint) -> Option<NodeId> {
        self.nodes_at_window_point(window_point).first().copied()
    }
    
    /// Get the topmost node at a specific canvas point
    pub fn top_node_at_canvas_point(&self, canvas_point: CanvasPoint) -> Option<NodeId> {
        self.nodes_at_canvas_point(canvas_point).first().copied()
    }
    
    /// Legacy method for backward compatibility
    /// Will be removed once all code is updated
    pub fn top_node_at_point(&self, point: Point<f32>) -> Option<NodeId> {
        let window_point = WindowPoint::new(px(point.x), px(point.y));
        self.top_node_at_window_point(window_point)
    }

    /// Perform rectangular selection with a window rectangle
    pub fn select_nodes_in_window_rect(&mut self, window_rect: WindowRect) {
        // Convert window rectangle corners to canvas coordinates
        let top_left = WindowPoint::new(window_rect.origin.x, window_rect.origin.y);
        let bottom_right = WindowPoint::new(
            window_rect.origin.x + window_rect.size.width,
            window_rect.origin.y + window_rect.size.height
        );
        
        let unresolved_tl = top_left.to_unresolved_canvas();
        let unresolved_br = bottom_right.to_unresolved_canvas();
        
        let canvas_tl = self.resolve_canvas_point(unresolved_tl);
        let canvas_br = self.resolve_canvas_point(unresolved_br);

        // Create selection canvas rectangle
        let selection_rect = CanvasRect {
            origin: CanvasPoint::new(
                canvas_tl.x.min(canvas_br.x),
                canvas_tl.y.min(canvas_br.y)
            ),
            size: CanvasSize::new(
                (canvas_tl.x - canvas_br.x).abs(),
                (canvas_tl.y - canvas_br.y).abs()
            ),
        };

        // Find all nodes that intersect with the selection rect
        let mut selected_nodes = HashSet::new();

        for (id, node) in &self.nodes {
            if let Some(bounds) = node.common().bounds() {
                let node_rect = CanvasRect::from_gpui_bounds(bounds);
                if selection_rect.intersects(&node_rect) {
                    selected_nodes.insert(*id);
                }
            }
        }

        // Update selection
        self.selected_nodes = selected_nodes;
        self.dirty = true;
    }
    
    /// Legacy method for backward compatibility
    /// Will be removed once all code is updated
    pub fn select_nodes_in_rect(&mut self, rect: Rect<f32>) {
        let window_rect = WindowRect {
            origin: WindowPoint::new(px(rect.left), px(rect.top)),
            size: WindowSize::new(
                px(rect.right - rect.left),
                px(rect.bottom - rect.top)
            ),
        };
        self.select_nodes_in_window_rect(window_rect);
    }

    /// Update the layout for the entire canvas
    pub fn update_layout(&mut self) {
        if !self.dirty {
            return;
        }

        // Compute layout using taffy for each node
        for (node_id, node) in &mut self.nodes {
            if let Some(taffy_node) = self.node_to_taffy.get(node_id) {
                // Compute layout using taffy
                let available_space = taffy::prelude::Size {
                    width: taffy::prelude::AvailableSpace::MaxContent,
                    height: taffy::prelude::AvailableSpace::MaxContent,
                };
                let _ = self.taffy.compute_layout(*taffy_node, available_space);

                // Get the computed layout
                if let Ok(layout) = self.taffy.layout(*taffy_node) {
                    // Update our node layout
                    let node_common = node.common_mut();
                    node_common.layout = Some(Layout {
                        order: layout.order,
                        size: taffy::prelude::Size {
                            width: layout.size.width,
                            height: layout.size.height,
                        },
                        location: taffy::Point {
                            x: layout.location.x,
                            y: layout.location.y,
                        },
                        border: layout.border,
                        padding: layout.padding,
                        scrollbar_size: layout.scrollbar_size,
                        // TODO: Calculate content size
                        ..Default::default()
                    });
                }
            }
        }

        // Compute content bounds
        self.update_content_bounds();

        self.dirty = false;
    }

    /// Update the content bounds of the canvas
    fn update_content_bounds(&mut self) {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        // Find the bounds that contain all nodes
        for node in self.nodes.values() {
            if let Some(bounds) = node.common().bounds() {
                min_x = min_x.min(bounds.origin.x);
                min_y = min_y.min(bounds.origin.y);
                max_x = max_x.max(bounds.origin.x + bounds.size.width);
                max_y = max_y.max(bounds.origin.y + bounds.size.height);
            }
        }

        // Update content bounds if we have nodes
        if min_x != f32::MAX {
            self.content_bounds = Bounds {
                origin: Point::new(min_x, min_y),
                size: Size::new(max_x - min_x, max_y - min_y),
            };
        }
    }

    /// Get nodes that are visible in the current viewport
    pub fn visible_nodes(&self) -> Vec<NodeId> {
        // Convert viewport to canvas coordinates
        let viewport_tl = WindowPoint::new(
            px(self.viewport.origin.x),
            px(self.viewport.origin.y)
        );
        let viewport_br = WindowPoint::new(
            px(self.viewport.origin.x + self.viewport.size.width),
            px(self.viewport.origin.y + self.viewport.size.height)
        );
        
        // Resolve to canvas coordinates
        let unresolved_tl = viewport_tl.to_unresolved_canvas();
        let unresolved_br = viewport_br.to_unresolved_canvas();
        let canvas_tl = self.resolve_canvas_point(unresolved_tl);
        let canvas_br = self.resolve_canvas_point(unresolved_br);

        // Create viewport bounds in canvas coordinates
        let viewport_rect = CanvasRect {
            origin: CanvasPoint::new(
                canvas_tl.x.min(canvas_br.x),
                canvas_tl.y.min(canvas_br.y)
            ),
            size: CanvasSize::new(
                (canvas_br.x - canvas_tl.x).abs(),
                (canvas_br.y - canvas_tl.y).abs()
            ),
        };

        // Find all nodes intersecting with the viewport
        let mut visible = Vec::new();

        for (id, node) in &self.nodes {
            if let Some(bounds) = node.common().bounds() {
                let node_rect = CanvasRect::from_gpui_bounds(bounds);
                if viewport_rect.intersects(&node_rect) {
                    visible.push(*id);
                }
            }
        }

        visible
    }

    /// Get all root nodes (nodes with no parent)
    pub fn get_root_nodes(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.common().parent.is_none())
            .map(|(id, _)| *id)
            .collect()
    }

    /// Create a test rectangle for development purposes
    fn create_test_rectangle(&mut self) -> NodeId {
        let id = self.generate_id();
        let mut rect = RectangleNode::new(id);
        
        // Set position at canvas origin (center of canvas) and large size
        // This explicitly uses canvas coordinates
        let origin = CanvasPoint::new(0.0, 0.0);
        let size = CanvasSize::new(400.0, 300.0);
        
        rect.common_mut().set_position(origin.x, origin.y);
        rect.common_mut().set_size(size.width, size.height);
        
        // Set extremely bright colors
        rect.common_mut().set_fill(Some(hsla(0.0, 1.0, 0.5, 1.0))); // Bright red
        rect.common_mut().set_border(Some(hsla(0.33, 1.0, 0.5, 1.0)), 5.0); // Thick green border
        
        println!("Created test rectangle at canvas origin (0,0) with size 400x300");
        
        self.add_node(rect)
    }

    /// Create a new node with the given type at a canvas position
    pub fn create_node(&mut self, _node_type: NodeType, position: CanvasPoint) -> NodeId {
        let id = self.generate_id();

        // For simplicity, as requested, we'll just create rectangles for all types
        let mut rect = RectangleNode::new(id);
        rect.common_mut().set_position(position.x, position.y);
        self.add_node(rect)
    }
    
    /// Create a new node with the given type at a window position
    pub fn create_node_at_window_point(&mut self, node_type: NodeType, window_point: WindowPoint) -> NodeId {
        let unresolved = window_point.to_unresolved_canvas();
        let canvas_point = self.resolve_canvas_point(unresolved);
        self.create_node(node_type, canvas_point)
    }

    /// Move selected nodes by a canvas delta
    pub fn move_selected_nodes(&mut self, delta: CanvasPoint) {
        for node_id in &self.selected_nodes {
            if let Some(node) = self.nodes.get_mut(node_id) {
                let common = node.common_mut();
                if let Some(bounds) = common.bounds() {
                    // Convert gpui::Bounds to our CanvasRect
                    let canvas_rect = CanvasRect::from_gpui_bounds(bounds);
                    
                    // Apply delta
                    let new_origin = CanvasPoint::new(
                        canvas_rect.origin.x + delta.x,
                        canvas_rect.origin.y + delta.y
                    );
                    
                    // Update position
                    common.set_position(new_origin.x, new_origin.y);
                }
            }
        }

        self.dirty = true;
    }
    
    /// Legacy method to move selected nodes by a point delta
    /// Will be removed once all code is updated
    pub fn move_selected_nodes_legacy(&mut self, delta: Point<f32>) {
        self.move_selected_nodes(CanvasPoint::new(delta.x, delta.y));
    }

    /// Set viewport bounds (when window resizes)
    pub fn set_viewport(&mut self, viewport: Bounds<f32>) {
        self.viewport = viewport;
        self.dirty = true;
    }

    /// Set scroll position
    pub fn set_scroll_position(&mut self, position: Point<f32>) {
        self.scroll_position = position;
        self.dirty = true;
    }

    /// Set zoom level
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.max(0.1).min(10.0); // Limit zoom range
        self.dirty = true;
    }
    
    /// Get current zoom level
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Check if the canvas is dirty and needs redrawing
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark the canvas as dirty (needing redraw)
    pub fn mark_dirty(&mut self, cx: &mut Context<Self>) {
        self.dirty = true;
        cx.notify();
    }

    /// Get content bounds
    pub fn content_bounds(&self) -> Bounds<f32> {
        self.content_bounds
    }

    pub fn key_context(&self) -> KeyContext {
        let mut key_context = KeyContext::new_with_defaults();
        key_context.set("canvas", "Canvas");
        key_context
    }
}

impl gpui::Element for Canvas {
    type RequestLayoutState = gpui::LayoutId;
    type PrepaintState = Option<gpui::Hitbox>;

    fn id(&self) -> Option<gpui::ElementId> {
        Some(gpui::ElementId::Name("canvas".into()))
    }

    fn request_layout(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        // Create a style for the canvas container
        let mut style = gpui::Style::default();

        // Canvas should take full available space
        style.size.width = gpui::DefiniteLength::Fraction(1.0).into();
        style.size.height = gpui::DefiniteLength::Fraction(1.0).into();

        // Canvas is relative positioned
        style.position = taffy::style::Position::Relative;

        // Request the layout from the window
        let layout_id = window.request_layout(style, [], cx);

        (layout_id, layout_id)
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _layout_id: &mut Self::RequestLayoutState,
        window: &mut gpui::Window,
        _cx: &mut gpui::App,
    ) -> Self::PrepaintState {
        // Update canvas viewport based on the layout bounds
        self.set_viewport(gpui::Bounds {
            origin: gpui::Point::new(bounds.origin.x.0, bounds.origin.y.0),
            size: gpui::Size::new(bounds.size.width.0, bounds.size.height.0),
        });

        // Update layout for all nodes
        self.update_layout();

        // Create a hitbox for the entire canvas to capture events
        let hitbox = window.insert_hitbox(bounds, false);
        Some(hitbox)
    }

    fn paint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _layout_id: &mut Self::RequestLayoutState,
        _hitbox: &mut Self::PrepaintState,
        window: &mut gpui::Window,
        _cx: &mut gpui::App,
    ) {
        // Get visible nodes and render them
        let visible_nodes = self.visible_nodes();

        // Render each visible node
        for node_id in visible_nodes {
            if let Some(node) = self.nodes.get(&node_id) {
                if let Some(node_bounds) = node.common().bounds() {
                    // Convert the node bounds to our coordinate system
                    let canvas_rect = CanvasRect::from_gpui_bounds(node_bounds);
                    
                    // Convert canvas coordinates to window coordinates
                    let window_rect = self.canvas_rect_to_window_rect(canvas_rect);
                    
                    // Convert to gpui bounds for rendering
                    let pixel_bounds = window_rect.to_gpui_bounds();

                    // Draw fill and border for each node
                    if let Some(fill) = node.common().fill {
                        window.paint_quad(gpui::fill(pixel_bounds, fill));
                    }

                    if let Some(border_color) = node.common().border_color {
                        if node.common().border_width > 0.0 {
                            window.paint_quad(gpui::outline(pixel_bounds, border_color));
                        }
                    }

                    // Draw selection indicator if node is selected
                    if self.is_node_selected(node_id) {
                        // Create a slightly larger bounds for selection indicator
                        let selection_bounds = gpui::Bounds {
                            origin: gpui::Point::new(
                                pixel_bounds.origin.x - gpui::Pixels(2.0),
                                pixel_bounds.origin.y - gpui::Pixels(2.0),
                            ),
                            size: gpui::Size::new(
                                pixel_bounds.size.width + gpui::Pixels(4.0),
                                pixel_bounds.size.height + gpui::Pixels(4.0),
                            ),
                        };
                        window.paint_quad(gpui::outline(
                            selection_bounds,
                            gpui::hsla(210.0 / 360.0, 0.92, 0.65, 1.0),
                        ));
                    }
                }
            }
        }

        // Mark canvas as no longer dirty after rendering
        self.dirty = false;
    }
}

impl gpui::IntoElement for Canvas {
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let mut element = div()
            .absolute()
            .top_0()
            .left_0()
            .id("canvas")
            .track_focus(&self.focus_handle(cx))
            .size_full()
            .flex_1()
            .into_any();

        gpui_canvas(
            move |bounds, window, cx| {
                element.prepaint_as_root(bounds.origin, bounds.size.into(), window, cx);
                element
            },
            |_, mut element, window, cx| {
                element.paint(window, cx);
            },
        )
        .size_full()
        .flex_1()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinates::{CanvasPoint, CanvasRect, CanvasSize};

    #[test]
    fn test_canvas_rect_intersection() {
        // Overlapping rectangles
        let a = CanvasRect {
            origin: CanvasPoint::new(0.0, 0.0),
            size: CanvasSize::new(100.0, 100.0),
        };
        let b = CanvasRect {
            origin: CanvasPoint::new(50.0, 50.0),
            size: CanvasSize::new(100.0, 100.0),
        };
        assert!(a.intersects(&b));

        // Non-overlapping on x-axis
        let c = CanvasRect {
            origin: CanvasPoint::new(200.0, 0.0),
            size: CanvasSize::new(100.0, 100.0),
        };
        assert!(!a.intersects(&c));

        // Non-overlapping on y-axis
        let d = CanvasRect {
            origin: CanvasPoint::new(0.0, 200.0),
            size: CanvasSize::new(100.0, 100.0),
        };
        assert!(!a.intersects(&d));
    }
}
