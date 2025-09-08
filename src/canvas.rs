#![allow(unused, dead_code)]

use crate::{
    interactivity::ActiveDrag,
    node::{frame::FrameNode, NodeCommon, NodeId, NodeLayout, NodeType},
    scene_graph::{SceneGraph, SceneNodeId},
    theme::Theme,
    AppState, Tool,
};
use gpui::{
    actions, canvas as gpui_canvas, div, hsla, point, prelude::*, px, size, Action, App, Bounds,
    Context, ContextEntry, DispatchPhase, Element, Entity, EntityInputHandler, FocusHandle,
    Focusable, InputHandler, InteractiveElement, IntoElement, KeyContext, ParentElement, Pixels,
    Point, Render, ScaledPixels, Size, Styled, TransformationMatrix, Window,
};
use std::{
    any::TypeId,
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    rc::Rc,
};

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
    canvas: &Entity<LunaCanvas>,
    window: &mut Window,
    listener: impl Fn(&mut LunaCanvas, &T, &mut Window, &mut Context<LunaCanvas>) + 'static,
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
pub struct LunaCanvas {
    app_state: Entity<AppState>,

    /// The scene graph for managing spatial relationships between nodes
    scene_graph: Entity<SceneGraph>,

    /// The canvas root node in scene graph
    canvas_node: SceneNodeId,

    /// Flat list of nodes (the data model)
    nodes: Vec<FrameNode>,

    /// Currently selected nodes
    selected_nodes: HashSet<NodeId>,

    /// Currently hovered node (for hover effects)
    hovered_node: Option<NodeId>,

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

    /// Whether the canvas needs to be re-rendered
    dirty: bool,

    focus_handle: FocusHandle,
    pub actions:
        Rc<RefCell<BTreeMap<CanvasActionId, Box<dyn Fn(&mut Window, &mut Context<Self>)>>>>,
    active_drag: Option<ActiveDrag>,

    /// Tracks an active drawing operation (e.g., rectangle being drawn)
    active_element_draw: Option<(NodeId, NodeType, ActiveDrag)>,

    /// The initial positions of selected elements before dragging
    /// Used to calculate relative positions when dragging multiple elements
    element_initial_positions: HashMap<NodeId, Point<f32>>,

    /// Tracks a potential parent frame when dragging elements
    /// Used to highlight frames that can become parents when dropping elements
    potential_parent_frame: Option<NodeId>,

    theme: Theme,
}

impl LunaCanvas {
    /// Create a new canvas
    pub fn new(
        app_state: &Entity<AppState>,
        scene_graph: &Entity<SceneGraph>,
        theme: &Theme,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let initial_viewport_px = window.viewport_size();
        let initial_viewport = size(initial_viewport_px.width.0, initial_viewport_px.height.0);

        // Create an initial viewport with reasonable size
        let viewport = Bounds {
            origin: Point::new(0.0, 0.0),
            size: initial_viewport,
        };

        let content_bounds = viewport.clone();

        // Create canvas root node in scene graph
        let canvas_node = scene_graph.update(cx, |sg, _cx| sg.create_node(None, None));

        let mut canvas = Self {
            app_state: app_state.clone(),
            scene_graph: scene_graph.clone(),
            canvas_node,
            nodes: Vec::new(),
            selected_nodes: HashSet::new(),
            viewport,
            scroll_position: Point::new(0.0, 0.0),
            zoom: 1.0,
            content_bounds,
            next_id: 1,
            dirty: true,
            focus_handle: cx.focus_handle(),
            actions: Rc::default(),
            active_drag: None,
            active_element_draw: None,
            element_initial_positions: HashMap::new(),
            potential_parent_frame: None,
            theme: theme.clone(),
            hovered_node: None,
        };

        // Load rectangles from CSS file
        let app_state_read = app_state.read(cx);
        let current_background_color = app_state_read.current_background_color;
        let current_border_color = app_state_read.current_border_color;

        // Try to load the CSS file from assets
        let mut node_to_select = None;

        if let Ok(css_content) = std::fs::read_to_string("assets/css/buttons.css") {
            // Use our CSS parser to create rectangle nodes
            let mut factory = crate::node::NodeFactory::default();
            let frames = crate::css_parser::parse_frames_from_css_file(&css_content, &mut factory);

            // Add all rectangles to the canvas
            for (index, mut rect) in frames.into_iter().enumerate() {
                // Add the node and capture the ID
                let node_id = canvas.add_node(rect, None, cx);

                // Select the second node (index 1) if it exists
                if index == 1 {
                    node_to_select = Some(node_id);
                }

                // Make sure our next_id is higher than any loaded ID to prevent collisions
                // NodeId stores an internal usize, so we access it with .0
                canvas.next_id = canvas.next_id.max(node_id.0 + 1);
            }
        } else {
            // Fallback to creating a single default rectangle if CSS loading fails
            let node_id = canvas.generate_id();
            let mut rect = FrameNode::with_rect(node_id, 100.0, 100.0, 200.0, 150.0);
            rect.set_fill(Some(current_background_color));
            rect.set_border(Some(current_border_color), 1.0);
            let node_id = canvas.add_node(rect, None, cx);

            // Make sure our next_id is higher than the ID we just used
            canvas.next_id = canvas.next_id.max(node_id.0 + 1);

            node_to_select = Some(node_id);
        }

        // Select a node if we have one
        if let Some(node_id) = node_to_select {
            canvas.select_node(node_id);
        }

        // Select the second element (blue rectangle)

        canvas
    }

    /// Generate a unique ID for a new node
    pub fn generate_id(&mut self) -> NodeId {
        let id = NodeId::new(self.next_id);
        self.next_id += 1;
        println!("Generated new node ID: {}", id); // Debug logging
        id
    }

    pub fn nodes(&self) -> &Vec<FrameNode> {
        &self.nodes
    }

    pub fn selected_nodes(&self) -> &HashSet<NodeId> {
        &self.selected_nodes
    }

    pub fn app_state(&self) -> &Entity<AppState> {
        &self.app_state
    }

    pub fn active_drag(&self) -> Option<ActiveDrag> {
        self.active_drag.clone()
    }

    pub fn set_active_drag(&mut self, active_drag: ActiveDrag) {
        self.active_drag = Some(active_drag);
    }

    pub fn clear_active_drag(&mut self) {
        self.active_drag = None;
    }

    pub fn active_element_draw(&self) -> Option<(NodeId, NodeType, ActiveDrag)> {
        self.active_element_draw.clone()
    }

    pub fn set_active_element_draw(&mut self, active_element_draw: (NodeId, NodeType, ActiveDrag)) {
        self.active_element_draw = Some(active_element_draw);
    }

    pub fn clear_active_element_draw(&mut self) {
        self.active_element_draw = None;
    }

    pub fn element_initial_positions(&self) -> &HashMap<NodeId, Point<f32>> {
        &self.element_initial_positions
    }
    pub fn element_initial_positions_mut(&mut self) -> &mut HashMap<NodeId, Point<f32>> {
        &mut self.element_initial_positions
    }

    pub fn potential_parent_frame(&self) -> Option<NodeId> {
        self.potential_parent_frame
    }

    pub fn set_potential_parent_frame(&mut self, frame_id: Option<NodeId>) {
        self.potential_parent_frame = frame_id;
    }

    pub fn hovered_node(&self) -> Option<NodeId> {
        self.hovered_node
    }

    pub fn set_hovered_node(&mut self, hovered_node: Option<NodeId>) {
        self.hovered_node = hovered_node;
    }

    pub fn get_node(&self, node_id: NodeId) -> Option<&FrameNode> {
        self.nodes.iter().find(|n| n.id() == node_id)
    }

    pub fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut FrameNode> {
        self.nodes.iter_mut().find(|n| n.id() == node_id)
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

    pub fn scene_graph(&self) -> &Entity<SceneGraph> {
        &self.scene_graph
    }

    /// Add a node to the canvas with an optional parent
    ///
    /// If parent_id is provided, the node will be added as a child of that parent in both
    /// the data model and scene graph. The node's coordinates will be transformed to be
    /// relative to the parent's coordinate system.
    pub fn add_node(
        &mut self,
        mut node: FrameNode,
        parent_id: Option<NodeId>,
        cx: &mut Context<Self>,
    ) -> NodeId {
        let node_id = node.id();

        // Get parent node's scene node ID if specified, otherwise use canvas node
        let parent_scene_node_id = match parent_id {
            Some(parent) => {
                // If we have a parent, adjust coordinates to be relative to parent
                if let Some(parent_node) = self.get_node(parent) {
                    // Get parent layout information first to avoid borrow issues
                    let parent_x = parent_node.layout().x;
                    let parent_y = parent_node.layout().y;

                    // Convert node's absolute coordinates to parent-relative coordinates
                    let node_layout = node.layout_mut();
                    node_layout.x -= parent_x;
                    node_layout.y -= parent_y;

                    // Add child to parent in data model
                    if let Some(parent_node_mut) = self.get_node_mut(parent) {
                        parent_node_mut.add_child(node_id);
                    }

                    // Get parent's scene node ID
                    self.scene_graph.update(cx, |sg, _| {
                        sg.get_scene_node_id(parent).unwrap_or(self.canvas_node)
                    })
                } else {
                    self.canvas_node
                }
            }
            None => self.canvas_node,
        };

        // Add node to flat list
        self.nodes.push(node);

        // Create scene node as child of parent scene node
        self.scene_graph.update(cx, |sg, _cx| {
            let scene_node = sg.create_node(Some(parent_scene_node_id), Some(node_id));

            // Set initial bounds from node layout
            let node = self.nodes.last().unwrap();
            let layout = node.layout();
            let bounds = Bounds {
                origin: Point::new(layout.x, layout.y),
                size: Size::new(layout.width, layout.height),
            };

            sg.set_local_bounds(scene_node, bounds);
        });

        self.dirty = true;
        node_id
    }

    /// Add a child node to a parent node
    ///
    /// This updates both the data model and scene graph to maintain the parent-child relationship.
    /// The child's coordinates will be transformed to be relative to the parent's coordinate system.
    pub fn add_child_to_parent(
        &mut self,
        parent_id: NodeId,
        child_id: NodeId,
        cx: &mut Context<Self>,
    ) -> bool {
        // 1. Verify both nodes exist
        if self.get_node(parent_id).is_none() || self.get_node(child_id).is_none() {
            return false;
        }

        // 2. Check for circular references (child can't be an ancestor of parent)
        if self.is_ancestor_of(child_id, parent_id) {
            return false;
        }

        // Store relevant coordinate values before mutating
        // Get absolute position of child (depends on current parent)
        let (child_absolute_x, child_absolute_y) = self.get_absolute_position(child_id, cx);

        // Get parent position
        let (parent_x, parent_y) = if let Some(parent) = self.get_node(parent_id) {
            let parent_layout = parent.layout();
            (parent_layout.x, parent_layout.y)
        } else {
            return false;
        };

        // 3. Update data model - add child to new parent
        let data_updated = if let Some(parent_node) = self.get_node_mut(parent_id) {
            parent_node.add_child(child_id)
        } else {
            false
        };

        // 4. Update child's position to be relative to new parent
        if let Some(child_node) = self.get_node_mut(child_id) {
            let child_layout = child_node.layout_mut();
            child_layout.x = child_absolute_x - parent_x;
            child_layout.y = child_absolute_y - parent_y;
        }

        // 5. Update scene graph - move child node to be under parent node
        let scene_updated = self.scene_graph.update(cx, |sg, _| {
            let parent_scene_id = sg.get_scene_node_id(parent_id);
            let child_scene_id = sg.get_scene_node_id(child_id);

            match (parent_scene_id, child_scene_id) {
                (Some(parent_scene), Some(child_scene)) => sg.add_child(parent_scene, child_scene),
                _ => false,
            }
        });

        if data_updated || scene_updated {
            self.dirty = true;
        }

        data_updated && scene_updated
    }

    /// Remove a child node from its parent
    ///
    /// The child will remain in the canvas but will be moved to the root level.
    /// Its coordinates will be converted from parent-relative to absolute.
    pub fn remove_child_from_parent(&mut self, child_id: NodeId, cx: &mut Context<Self>) -> bool {
        // Find the parent of this child
        let parent_id = self.find_parent(child_id);

        if let Some(parent_id) = parent_id {
            // Get absolute position before changing parent
            let (absolute_x, absolute_y) = self.get_absolute_position(child_id, cx);

            // Update data model - remove child from parent
            let data_updated = if let Some(parent_node) = self.get_node_mut(parent_id) {
                parent_node.remove_child(child_id)
            } else {
                false
            };

            // Update child's position to absolute coordinates
            if let Some(child_node) = self.get_node_mut(child_id) {
                let child_layout = child_node.layout_mut();
                child_layout.x = absolute_x;
                child_layout.y = absolute_y;
            }

            // Update scene graph - move child to canvas root
            let scene_updated = self.scene_graph.update(cx, |sg, _| {
                let child_scene_id = sg.get_scene_node_id(child_id);

                match child_scene_id {
                    Some(child_scene) => sg.add_child(self.canvas_node, child_scene),
                    _ => false,
                }
            });

            if data_updated || scene_updated {
                self.dirty = true;
            }

            data_updated && scene_updated
        } else {
            // Node wasn't a child of any parent
            false
        }
    }

    /// Find the parent node of a child node
    fn find_parent(&self, child_id: NodeId) -> Option<NodeId> {
        for node in &self.nodes {
            if node.children().contains(&child_id) {
                return Some(node.id());
            }
        }
        None
    }

    /// Check if a node is an ancestor of another node
    ///
    /// Returns true if ancestor_id is an ancestor (parent, grandparent, etc.) of descendant_id
    fn is_ancestor_of(&self, ancestor_id: NodeId, descendant_id: NodeId) -> bool {
        if ancestor_id == descendant_id {
            return true; // Same node
        }

        let mut current = Some(descendant_id);
        while let Some(node_id) = current {
            if node_id == ancestor_id {
                return true;
            }
            current = self.find_parent(node_id);
        }

        false
    }

    /// Get the absolute position of a node, accounting for all parent transformations
    fn get_absolute_position(&self, node_id: NodeId, _cx: &mut Context<Self>) -> (f32, f32) {
        let mut current_id = node_id;
        let mut x = 0.0;
        let mut y = 0.0;

        // First collect all nodes in the parent chain to avoid borrow checker issues
        let mut node_chain = Vec::new();

        // Start with the node itself
        if let Some(node) = self.get_node(current_id) {
            node_chain.push((current_id, node.layout().x, node.layout().y));
        } else {
            return (0.0, 0.0);
        }

        // Collect all parents
        let mut parent_id = self.find_parent(current_id);
        while let Some(p_id) = parent_id {
            if let Some(parent) = self.get_node(p_id) {
                node_chain.push((p_id, parent.layout().x, parent.layout().y));
            }

            current_id = p_id;
            parent_id = self.find_parent(current_id);
        }

        // Now compute the absolute position using the collected chain
        // Start with the node's own position
        if let Some((_, node_x, node_y)) = node_chain.first() {
            x = *node_x;
            y = *node_y;
        }

        // Add all parent positions (skip the first node which is the starting node)
        for (_, parent_x, parent_y) in node_chain.iter().skip(1) {
            x += parent_x;
            y += parent_y;
        }

        (x, y)
    }

    /// Remove a node from the canvas and update the scene graph
    ///
    /// This method removes the specified node and all its children recursively
    /// from both the data model and the scene graph.
    pub fn remove_node(
        &mut self,
        node_id: NodeId,
        cx: &mut Context<Self>,
    ) -> Option<crate::node::frame::FrameNode> {
        // Remove from selection
        self.selected_nodes.remove(&node_id);

        // Get a copy of this node's children first
        let children = if let Some(node) = self.get_node(node_id) {
            node.children().clone()
        } else {
            Vec::new()
        };

        // Recursively remove all children first
        for child_id in children {
            self.remove_node(child_id, cx);
        }

        // Remove from scene graph if it exists there
        let scene_node_id = self
            .scene_graph
            .update(cx, |sg, _cx| sg.get_scene_node_id(node_id));
        if let Some(scene_node_id) = scene_node_id {
            self.scene_graph.update(cx, |sg, _cx| {
                sg.remove_node(scene_node_id);
            });
        }

        // Find and remove the node from our vector
        let position = self.nodes.iter().position(|node| node.id() == node_id);
        let node = position.map(|idx| self.nodes.remove(idx));

        // Mark canvas as dirty
        self.dirty = true;

        node
    }

    /// Select a node
    pub fn select_node(&mut self, node_id: NodeId) {
        if self.nodes.iter().any(|node| node.id() == node_id) {
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
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.selected_nodes.clear();
        self.dirty = true;
    }

    /// Toggle selection state of a node
    pub fn toggle_node_selection(&mut self, node_id: NodeId) {
        if self.selected_nodes.contains(&node_id) {
            self.selected_nodes.remove(&node_id);
        } else if self.nodes.iter().any(|node| node.id() == node_id) {
            self.selected_nodes.insert(node_id);
        }
        self.dirty = true;
    }

    /// Check if a node is selected
    pub fn is_node_selected(&self, node_id: NodeId) -> bool {
        self.selected_nodes.contains(&node_id)
    }

    /// Select all root nodes in the canvas
    pub fn select_all_nodes(&mut self) {
        // Check if all nodes are already selected to avoid unnecessary work
        if self.selected_nodes.len() == self.nodes.len() && !self.nodes.is_empty() {
            return;
        }

        self.selected_nodes.clear();
        self.selected_nodes
            .extend(self.nodes.iter().map(|node| node.id()));
        self.dirty = true;
    }

    /// Update the layout for the entire canvas
    pub fn update_layout(&mut self) {
        if !self.dirty {
            return;
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
        for node in &self.nodes {
            let bounds = node.bounds();
            min_x = min_x.min(bounds.origin.x);
            min_y = min_y.min(bounds.origin.y);
            max_x = max_x.max(bounds.origin.x + bounds.size.width);
            max_y = max_y.max(bounds.origin.y + bounds.size.height);
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
    pub fn visible_nodes(&self, cx: &mut App) -> Vec<&FrameNode> {
        // Create viewport bounds in window coordinates
        let viewport = Bounds {
            origin: Point::new(0.0, 0.0),
            size: self.viewport.size,
        };

        // Convert to gpui::Bounds
        let gpui_viewport = gpui::Bounds {
            origin: point(
                gpui::Pixels(viewport.origin.x),
                gpui::Pixels(viewport.origin.y),
            ),
            size: size(
                gpui::Pixels(viewport.size.width),
                gpui::Pixels(viewport.size.height),
            ),
        };

        // Use scene graph to find visible nodes
        let visible_node_ids = self.scene_graph.update(cx, |sg, _cx| {
            let mut visible_ids = Vec::new();

            // Start from canvas node children
            if let Some(canvas_node) = sg.get_node(self.canvas_node) {
                for &child_id in canvas_node.children() {
                    self.collect_visible_nodes(child_id, gpui_viewport, sg, &mut visible_ids);
                }
            }

            visible_ids
        });

        // Return references to visible nodes
        self.nodes
            .iter()
            .filter(|node| visible_node_ids.contains(&node.id()))
            .collect()
    }

    /// Helper method to recursively collect visible nodes
    fn collect_visible_nodes(
        &self,
        node_id: SceneNodeId,
        viewport: gpui::Bounds<gpui::Pixels>,
        sg: &SceneGraph,
        result: &mut Vec<NodeId>,
    ) {
        // TODO: Implement proper visibility checking
        // For now, just add the node and its children to the result
        if let Some(node) = sg.get_node(node_id) {
            // If node has an associated data node, add it to results
            if let Some(data_id) = node.data_node_id() {
                result.push(data_id);
            }

            // Process all children
            for &child_id in node.children() {
                self.collect_visible_nodes(child_id, viewport, sg, result);
            }
        }
    }

    /// Helper function to check if two gpui::Bounds rectangles intersect
    fn bounds_intersect_gpui(
        a: &gpui::Bounds<gpui::Pixels>,
        b: &gpui::Bounds<gpui::Pixels>,
    ) -> bool {
        // Check if one rectangle is to the left of the other
        if a.origin.x.0 + a.size.width.0 < b.origin.x.0
            || b.origin.x.0 + b.size.width.0 < a.origin.x.0
        {
            return false;
        }

        // Check if one rectangle is above the other
        if a.origin.y.0 + a.size.height.0 < b.origin.y.0
            || b.origin.y.0 + b.size.height.0 < a.origin.y.0
        {
            return false;
        }

        true
    }

    /// Get all root nodes (all nodes since we removed hierarchy)
    pub fn get_root_nodes(&self) -> Vec<NodeId> {
        self.nodes.iter().map(|node| node.id()).collect()
    }

    /// Create a new node with the given type at a position
    pub fn create_node(
        &mut self,
        _node_type: NodeType,
        position: Point<f32>,
        cx: &mut Context<Self>,
    ) -> NodeId {
        let id = self.generate_id();

        // Create a rectangle node at the specified position
        let mut rect = FrameNode::new(id);
        *rect.layout_mut() = NodeLayout::new(position.x, position.y, 100.0, 100.0);

        self.add_node(rect, None, cx)
    }

    /// Move selected nodes by a delta
    pub fn move_selected_nodes(&mut self, delta: Point<f32>) {
        for node in &mut self.nodes {
            if self.selected_nodes.contains(&node.id()) {
                let layout = node.layout_mut();
                layout.x += delta.x;
                layout.y += delta.y;
            }
        }

        self.dirty = true;
    }

    /// Captures initial coordinates of all selected nodes in element_initial_positions
    ///
    /// This method should be called at the start of an element drag operation to establish
    /// a reference point for relative transformations. The stored positions are used by
    /// move_selected_nodes_with_drag to preserve element relationships during movement.
    pub fn save_selected_nodes_positions(&mut self) {
        self.element_initial_positions.clear();

        for node in &self.nodes {
            if self.selected_nodes.contains(&node.id()) {
                let layout = node.layout();
                self.element_initial_positions
                    .insert(node.id(), Point::new(layout.x, layout.y));
            }
        }
    }

    /// Transforms selected elements by applying the provided delta to their initial positions
    ///
    /// This method operates on the captured initial positions, ensuring that multiple elements
    /// maintain their relative spatial relationships during dragging. It also updates the
    /// scene graph to reflect the visual changes.
    ///
    /// # Arguments
    /// * `delta` - The transformation vector to apply to all selected elements
    /// * `cx` - Context used for scene graph updates
    pub fn move_selected_nodes_with_drag(&mut self, delta: Point<f32>, cx: &mut Context<Self>) {
        for node in &mut self.nodes {
            // Get the node ID first before any mutable borrows
            let node_id = node.id();

            if self.selected_nodes.contains(&node_id) {
                if let Some(initial_pos) = self.element_initial_positions.get(&node_id) {
                    // First, update the layout
                    let layout = node.layout_mut();
                    layout.x = initial_pos.x + delta.x;
                    layout.y = initial_pos.y + delta.y;

                    // Store values we need before releasing the mutable borrow
                    let new_x = layout.x;
                    let new_y = layout.y;
                    let width = layout.width;
                    let height = layout.height;

                    // Update the scene graph bounds
                    if let Some(scene_node_id) = self
                        .scene_graph
                        .update(cx, |sg, _cx| sg.get_scene_node_id(node_id))
                    {
                        self.scene_graph.update(cx, |sg, _cx| {
                            sg.set_local_bounds(
                                scene_node_id,
                                Bounds {
                                    origin: Point::new(new_x, new_y),
                                    size: Size::new(width, height),
                                },
                            );
                        });
                    }
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
    pub fn set_scroll_position(&mut self, position: Point<f32>, cx: &mut Context<Self>) {
        self.scroll_position = position;

        self.scene_graph.update(cx, |sg, _cx| {
            let transform = TransformationMatrix::unit()
                .scale(size(self.zoom, self.zoom))
                .translate(point(
                    Pixels(-self.scroll_position.x.floor()).scale(1.0),
                    Pixels(-self.scroll_position.y.floor()).scale(1.0),
                ));

            sg.set_local_transform(self.canvas_node, transform);
        });

        self.dirty = true;
    }

    /// Set zoom level
    pub fn set_zoom(&mut self, zoom: f32, cx: &mut Context<Self>) {
        self.zoom = zoom.max(0.1).min(10.0); // Limit zoom range

        // Update canvas root transform
        self.scene_graph.update(cx, |sg, _cx| {
            let transform = TransformationMatrix::unit()
                .scale(size(self.zoom, self.zoom))
                .translate(point(
                    Pixels(-self.scroll_position.x.floor()).scale(1.0),
                    Pixels(-self.scroll_position.y.floor()).scale(1.0),
                ));

            sg.set_local_transform(self.canvas_node, transform);
        });

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

    pub fn deselect_all_nodes(&mut self, cx: &mut Context<Self>) {
        self.selected_nodes.clear();
        self.mark_dirty(cx);
    }
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
