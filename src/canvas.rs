use crate::node::{CanvasNode, FrameNode, NodeId, NodeType, PathNode, RectangleNode, ShapeNode};
use gpui::{Bounds, Point, Size};
use std::collections::{HashMap, HashSet};
use taffy::{prelude::*, Rect};

/// A Canvas manages a collection of nodes that can be rendered and manipulated
pub struct Canvas {
    /// Mapping of NodeId to actual nodes
    pub nodes: HashMap<NodeId, Box<dyn CanvasNode>>,

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
}

impl Canvas {
    /// Create a new canvas
    pub fn new() -> Self {
        // Create the taffy layout engine
        let mut taffy = TaffyTree::new();

        // Create an initial viewport with reasonable size
        let viewport = Bounds {
            origin: Point::new(0.0, 0.0),
            size: Size::new(800.0, 600.0),
        };

        let content_bounds = viewport.clone();

        Self {
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
        }
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

    /// Add a node to the canvas
    pub fn add_node(&mut self, node: Box<dyn CanvasNode>) -> NodeId {
        let node_id = node.id();

        // Create a taffy node for layout
        let taffy_style = node.common().style.clone();
        let taffy_node = self.taffy.new_leaf(taffy_style).unwrap();

        // Map our node ID to taffy node ID
        self.node_to_taffy.insert(node_id, taffy_node);

        // Add to nodes map
        self.nodes.insert(node_id, node);

        // Mark canvas as dirty
        self.dirty = true;

        node_id
    }

    /// Remove a node from the canvas
    pub fn remove_node(&mut self, node_id: NodeId) -> Option<Box<dyn CanvasNode>> {
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
    pub fn clear_selection(&mut self) {
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

    /// Get nodes at a specific point (for hit testing)
    pub fn nodes_at_point(&self, point: Point<f32>) -> Vec<NodeId> {
        // Convert window point to canvas coordinates
        let canvas_point = self.window_to_canvas_point(point);

        // Get all shape nodes that contain this point
        let mut hit_nodes = Vec::new();

        for (id, node) in &self.nodes {
            if let Some(shape_node) = node.as_any().downcast_ref::<dyn ShapeNode>() {
                if shape_node.contains_point(&canvas_point) {
                    hit_nodes.push(*id);
                }
            } else if let Some(path_node) = node.as_any().downcast_ref::<dyn PathNode>() {
                if path_node.point_near_path(&canvas_point, 5.0) {
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
        let start = self.window_to_canvas_point(Point::new(rect.origin.x, rect.origin.y));
        let end = self.window_to_canvas_point(Point::new(
            rect.origin.x + rect.size.width,
            rect.origin.y + rect.size.height,
        ));

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
                let _ = self.taffy.compute_layout(*taffy_node, Size::MAX_CONTENT);

                // Get the computed layout
                if let Ok(layout) = self.taffy.layout(*taffy_node) {
                    // Update our node layout
                    let mut node_common = node.common_mut();
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

    /// Create a new node with the given type at a position
    pub fn create_node(&mut self, node_type: NodeType, position: Point<f32>) -> NodeId {
        let id = self.generate_id();

        let node: Box<dyn CanvasNode> = match node_type {
            NodeType::Rectangle => {
                let mut node = RectangleNode::new(id);
                node.common_mut().set_position(position.x, position.y);
                Box::new(node)
            }
            NodeType::Circle => {
                let mut node = CircleNode::new(id);
                node.common_mut().set_position(position.x, position.y);
                Box::new(node)
            }
            NodeType::Frame => {
                let mut node = FrameNode::new(id);
                node.common_mut().set_position(position.x, position.y);
                Box::new(node)
            }
            NodeType::Line => {
                // Default line from position to position + 100,100
                let end = Point::new(position.x + 100.0, position.y + 100.0);
                Box::new(LineNode::new(id, position, end))
            }
            _ => {
                // Default to rectangle for other types
                let mut node = RectangleNode::new(id);
                node.common_mut().set_position(position.x, position.y);
                Box::new(node)
            }
        };

        self.add_node(node)
    }

    /// Move selected nodes by a delta
    pub fn move_selected_nodes(&mut self, delta: Vec2) {
        for node_id in &self.selected_nodes {
            if let Some(node) = self.nodes.get_mut(node_id) {
                let mut common = node.common_mut();
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

    /// Check if the canvas is dirty and needs redrawing
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark the canvas as dirty (needing redraw)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Get content bounds
    pub fn content_bounds(&self) -> Bounds<f32> {
        self.content_bounds
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
    use crate::element::{CircleNode, RectangleNode};

    #[test]
    fn test_canvas_creation() {
        let canvas = Canvas::new();
        assert_eq!(canvas.nodes.len(), 0);
        assert_eq!(canvas.selected_nodes.len(), 0);
        assert_eq!(canvas.zoom, 1.0);
    }

    #[test]
    fn test_point_conversion() {
        let mut canvas = Canvas::new();
        canvas.set_scroll_position(Point::new(100.0, 100.0));
        canvas.set_zoom(2.0);

        // Window to canvas
        let window_point = Point::new(50.0, 60.0);
        let canvas_point = canvas.window_to_canvas_point(window_point);
        assert_eq!(canvas_point.x, 125.0); // (50 / 2) + 100
        assert_eq!(canvas_point.y, 130.0); // (60 / 2) + 100

        // Canvas to window
        let canvas_point = Point::new(200.0, 200.0);
        let window_point = canvas.canvas_to_window_point(canvas_point);
        assert_eq!(window_point.x, 200.0); // (200 - 100) * 2
        assert_eq!(window_point.y, 200.0); // (200 - 100) * 2
    }

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

    #[test]
    fn test_node_operations() {
        let mut canvas = Canvas::new();

        // Add a rectangle
        let id = canvas.generate_id();
        let rect = RectangleNode::new(id);
        let rect_id = canvas.add_node(Box::new(rect));

        assert_eq!(canvas.nodes.len(), 1);
        assert!(canvas.nodes.contains_key(&rect_id));

        // Select the node
        canvas.select_node(rect_id);
        assert!(canvas.is_node_selected(rect_id));

        // Deselect the node
        canvas.deselect_node(rect_id);
        assert!(!canvas.is_node_selected(rect_id));

        // Toggle selection
        canvas.toggle_node_selection(rect_id);
        assert!(canvas.is_node_selected(rect_id));

        // Remove the node
        let removed = canvas.remove_node(rect_id);
        assert!(removed.is_some());
        assert_eq!(canvas.nodes.len(), 0);
        assert!(!canvas.is_node_selected(rect_id));
    }
}
