#![allow(unused)]

use crate::{
    interactivity::ActiveDrag,
    node::{AnyNode, CanvasNode, NodeId, NodeType, RectangleNode, ShapeNode},
    ToolKind,
};
use gpui::{
    actions, canvas as gpui_canvas, div, hsla, prelude::*, size, Action, App, Bounds, Context,
    ContextEntry, DispatchPhase, Element, Entity, EntityInputHandler, FocusHandle, Focusable,
    InputHandler, InteractiveElement, IntoElement, KeyContext, ParentElement, Point, Render, Size,
    Styled, Window,
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
    /// Vector of nodes in insertion order
    pub nodes: Vec<(NodeId, AnyNode)>,

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
            nodes: Vec::new(),
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

        canvas
    }

    /// Generate a unique ID for a new node
    pub fn generate_id(&mut self) -> NodeId {
        let id = NodeId::new(self.next_id);
        self.next_id += 1;
        id
    }

    /// Convert a window-relative point to canvas-relative point
    pub fn window_to_canvas_point(&self, window_point: Point<f32>) -> Point<f32> {
        let canvas_x = (window_point.x / self.zoom) + self.scroll_position.x;
        let canvas_y = (window_point.y / self.zoom) + self.scroll_position.y;
        Point::new(canvas_x, canvas_y)
    }

    /// Convert a canvas-relative point to window-relative point
    pub fn canvas_to_window_point(&self, canvas_point: Point<f32>) -> Point<f32> {
        let window_x = (canvas_point.x - self.scroll_position.x) * self.zoom;
        let window_y = (canvas_point.y - self.scroll_position.y) * self.zoom;
        Point::new(window_x, window_y)
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

        // Create an AnyNode and add to nodes vec
        let any_node = AnyNode::new(node);
        self.nodes.push((node_id, any_node));

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

        // Find and remove the node from our vector
        let position = self.nodes.iter().position(|(id, _)| *id == node_id);
        let node = position.map(|idx| {
            let (_, node) = self.nodes.remove(idx);
            node
        });

        // Mark canvas as dirty
        self.dirty = true;

        node
    }

    /// Select a node
    pub fn select_node(&mut self, node_id: NodeId) {
        if self.nodes.iter().any(|(id, _)| *id == node_id) {
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
        } else if self.nodes.iter().any(|(id, _)| *id == node_id) {
            self.selected_nodes.insert(node_id);
        }
        self.dirty = true;
    }

    /// Check if a node is selected
    pub fn is_node_selected(&self, node_id: NodeId) -> bool {
        self.selected_nodes.contains(&node_id)
    }

    /// Get nodes at a specific point (for hit testing)
    pub fn nodes_at_point(&self, point: Point<f32>) -> Vec<NodeId> {
        // Convert window point to canvas coordinates
        let canvas_point = self.window_to_canvas_point(point);

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

    /// Get the topmost node at a specific point
    pub fn top_node_at_point(&self, point: Point<f32>) -> Option<NodeId> {
        self.nodes_at_point(point).first().copied()
    }

    /// Perform rectangular selection
    pub fn select_nodes_in_rect(&mut self, rect: Rect<f32>) {
        // Convert rectangle to canvas coordinates
        let start = self.window_to_canvas_point(Point::new(rect.left, rect.top));
        let end = self.window_to_canvas_point(Point::new(rect.right, rect.bottom));

        // Create selection bounds
        let selection_rect = Bounds {
            origin: Point::new(start.x.min(end.x), start.y.min(end.y)),
            size: Size::new((start.x - end.x).abs(), (start.y - end.y).abs()),
        };

        // Find all nodes that intersect with the selection rect
        let mut selected_nodes = HashSet::new();

        for (id, node) in &self.nodes {
            if let Some(bounds) = node.common().bounds() {
                if bounds_intersect(&selection_rect, &bounds) {
                    selected_nodes.insert(*id);
                }
            }
        }

        // Update selection
        self.selected_nodes = selected_nodes;
        self.dirty = true;
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
        for (_, node) in &self.nodes {
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
        let viewport_min = self.window_to_canvas_point(self.viewport.origin);
        let viewport_max = self.window_to_canvas_point(Point::new(
            self.viewport.origin.x + self.viewport.size.width,
            self.viewport.origin.y + self.viewport.size.height,
        ));

        // Create viewport bounds
        let viewport_bounds = Bounds {
            origin: Point::new(
                viewport_min.x.min(viewport_max.x),
                viewport_min.y.min(viewport_max.y),
            ),
            size: Size::new(
                (viewport_max.x - viewport_min.x).abs(),
                (viewport_max.y - viewport_min.y).abs(),
            ),
        };

        // Find all nodes intersecting with the viewport
        let mut visible = Vec::new();

        for (id, node) in &self.nodes {
            if let Some(bounds) = node.common().bounds() {
                if bounds_intersect(&viewport_bounds, &bounds) {
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

    /// Create a new node with the given type at a position
    pub fn create_node(&mut self, _node_type: NodeType, position: Point<f32>) -> NodeId {
        let id = self.generate_id();

        // For simplicity, as requested, we'll just create rectangles for all types
        let mut rect = RectangleNode::new(id);
        rect.common_mut().set_position(position.x, position.y);
        self.add_node(rect)
    }

    /// Move selected nodes by a delta
    pub fn move_selected_nodes(&mut self, delta: Point<f32>) {
        for node_id in &self.selected_nodes {
            if let Some(position) = self.nodes.iter_mut().position(|(id, _)| *id == *node_id) {
                let (_, node) = &mut self.nodes[position];
                let common = node.common_mut();
                if let Some(bounds) = common.bounds() {
                    let new_x = bounds.origin.x + delta.x;
                    let new_y = bounds.origin.y + delta.y;
                    common.set_position(new_x, new_y);
                }
            }
        }

        self.dirty = true;
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
            if let Some((_, node)) = self.nodes.iter().find(|(id, _)| *id == node_id) {
                if let Some(bounds) = node.common().bounds() {
                    // Apply zoom and scroll transformations
                    let adjusted_bounds = gpui::Bounds {
                        origin: gpui::Point::new(
                            (bounds.origin.x - self.scroll_position.x) * self.zoom,
                            (bounds.origin.y - self.scroll_position.y) * self.zoom,
                        ),
                        size: gpui::Size::new(
                            bounds.size.width * self.zoom,
                            bounds.size.height * self.zoom,
                        ),
                    };

                    // Convert to pixel bounds for rendering
                    let pixel_bounds = gpui::Bounds {
                        origin: gpui::Point::new(
                            gpui::Pixels(adjusted_bounds.origin.x),
                            gpui::Pixels(adjusted_bounds.origin.y),
                        ),
                        size: gpui::Size::new(
                            gpui::Pixels(adjusted_bounds.size.width),
                            gpui::Pixels(adjusted_bounds.size.height),
                        ),
                    };

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

/// Helper function to check if two bounds rectangles intersect
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_intersection() {
        // Overlapping bounds
        let a = Bounds {
            origin: Point::new(0.0, 0.0),
            size: Size::new(100.0, 100.0),
        };
        let b = Bounds {
            origin: Point::new(50.0, 50.0),
            size: Size::new(100.0, 100.0),
        };
        assert!(bounds_intersect(&a, &b));

        // Non-overlapping on x-axis
        let c = Bounds {
            origin: Point::new(200.0, 0.0),
            size: Size::new(100.0, 100.0),
        };
        assert!(!bounds_intersect(&a, &c));

        // Non-overlapping on y-axis
        let d = Bounds {
            origin: Point::new(0.0, 200.0),
            size: Size::new(100.0, 100.0),
        };
        assert!(!bounds_intersect(&a, &d));
    }
}
