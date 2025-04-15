// TODO: Can use use slot_map or similar to get stable ids?

use std::collections::{HashMap, HashSet};

use crate::node::NodeId;
use anyhow::Error;
use gpui::{App, Bounds, Pixels, Point, Size, TransformationMatrix, Window};
use slotmap::{KeyData, SlotMap};

#[derive(Debug, PartialEq)]
/// Controls the flow of scene graph operations to maintain data consistency
enum GraphPhase {
    /// During the Modification Phase, all external changes (user edits, animation updates,
    /// collaborative patches) are applied to the scene graph. This phase handles creating/deleting
    /// nodes, modifying properties (local transforms, bounds, visibility), and updating component
    /// overrides. Changes during this phase mark affected nodes as "dirty" so derived state isn't
    /// computed immediately.
    Mod,

    /// The Update Phase processes all pending changes from the Modification Phase. It performs
    /// layout computations (including any auto-layout if needed), propagates local transforms
    /// into correct world transforms, and recalculates world bounds. This phase flushes all
    /// dirty flags and must run to completion so the scene is in a consistent state.
    Update,

    /// The Query Phase allows safe, read-only access to the fully updated scene graph. During
    /// this phase, you can perform hit testing, spatial queries (e.g., selecting nodes within
    /// a region), and read out any computed properties. No changes are allowed here—you should
    /// only query the already computed, stable state.
    Query,

    /// The Render Preparation Phase organizes nodes into a draw order suitable for the painter's
    /// algorithm. It culls off-screen/invisible nodes, sorts nodes by z-order or stacking context,
    /// and produces an ordered batch of render commands or a draw list. This list is then handed
    /// off to the rendering engine (gpui) for drawing.
    Prep,
}

slotmap::new_key_type! {
    /// Used to link a node to its properties.
    pub struct NodePropertiesId;
}

impl From<u64> for NodePropertiesId {
    fn from(value: u64) -> Self {
        Self(KeyData::from_ffi(value))
    }
}

impl NodePropertiesId {
    pub fn as_u64(self) -> u64 {
        self.0.as_ffi()
    }
}

slotmap::new_key_type! {
    /// Unique identifier for components.
    pub struct ComponentId;
}

impl From<u64> for ComponentId {
    fn from(value: u64) -> Self {
        Self(KeyData::from_ffi(value))
    }
}

impl ComponentId {
    pub fn as_u64(self) -> u64 {
        self.0.as_ffi()
    }
}

slotmap::new_key_type! {
    /// Indentifies a component instance's overiden properties.
    pub struct OverrideIndex;
}

impl From<u64> for OverrideIndex {
    fn from(value: u64) -> Self {
        Self(KeyData::from_ffi(value))
    }
}

impl OverrideIndex {
    pub fn as_u64(self) -> u64 {
        self.0.as_ffi()
    }
}

slotmap::new_key_type! {
    /// A unique identifier for node elements.
    pub struct NodeElementId;
}

impl From<u64> for NodeElementId {
    fn from(value: u64) -> Self {
        Self(KeyData::from_ffi(value))
    }
}

impl NodeElementId {
    pub fn as_u64(self) -> u64 {
        self.0.as_ffi()
    }
}

slotmap::new_key_type! {
    pub struct InstanceId;
}

impl From<u64> for InstanceId {
    fn from(value: u64) -> Self {
        Self(KeyData::from_ffi(value))
    }
}

impl InstanceId {
    pub fn as_u64(self) -> u64 {
        self.0.as_ffi()
    }
}

#[derive(Debug)]
pub enum GraphNodeId {
    Component(ComponentId),
    Instance(InstanceId),
    Node(NodeElementId),
}

impl Clone for GraphNodeId {
    fn clone(&self) -> Self {
        match self {
            Self::Component(id) => Self::Component(*id),
            Self::Instance(id) => Self::Instance(*id),
            Self::Node(id) => Self::Node(*id),
        }
    }
}

impl PartialEq for GraphNodeId {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Component(a), Self::Component(b)) => a == b,
            (Self::Node(a), Self::Node(b)) => a == b,
            (Self::Instance(a), Self::Instance(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for GraphNodeId {}

impl std::hash::Hash for GraphNodeId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Component(id) => {
                std::mem::discriminant(self).hash(state);
                id.hash(state);
            }
            Self::Instance(id) => {
                std::mem::discriminant(self).hash(state);
                id.hash(state);
            }
            Self::Node(id) => {
                std::mem::discriminant(self).hash(state);
                id.hash(state);
            }
        }
    }
}

// TODO: Should transforms & bounds be storeed on the node in
// both local and world values?
pub struct GraphNode {
    parent: Option<GraphNodeId>,
    children: Vec<NodeElementId>,
    local_transform: TransformationMatrix,
    world_transform: TransformationMatrix,
    local_bounds: Bounds<f32>,
    world_bounds: Bounds<f32>,
    data_node_id: Option<NodeId>,
    visible: bool,
    dirty: bool,
}

impl GraphNode {
    pub fn new() -> Self {
        Self {
            parent: None,
            children: Vec::new(),
            local_transform: TransformationMatrix::default(),
            world_transform: TransformationMatrix::default(),
            local_bounds: Bounds::default(),
            world_bounds: Bounds::default(),
            data_node_id: None,
            visible: true,
            dirty: false,
        }
    }
}

pub struct ComponentDefinition {
    name: String,
    base_node: NodeElementId,
}

pub struct ComponentInstance {
    component_id: ComponentId,
}

// Placeholder for properties overrides
pub struct PropertyOverrides {}

pub struct SceneGraph2 {
    /// What phase the scene graph is in.
    ///
    /// Each phase has it's own context which allows for different
    /// operations and interactions.
    ///
    /// This allows the scene graph to ensure that operations are
    /// performed at the correct time, and that data is not read
    /// while operations are being performed.
    phase: GraphPhase,
    root: NodeElementId,
    nodes: SlotMap<NodeElementId, GraphNode>,
    components: SlotMap<ComponentId, ComponentDefinition>,
    instances: SlotMap<InstanceId, ComponentInstance>,
    overrides: SlotMap<OverrideIndex, PropertyOverrides>,
    node_properties: HashMap<NodePropertiesId, GraphNodeId>,
    dirty_nodes: HashSet<GraphNodeId>,

    /// Maps from data node IDs to scene node IDs, allowing bidirectional lookups
    node_mapping: HashMap<NodeId, NodeElementId>,
}

impl SceneGraph2 {
    pub fn new() -> Self {
        let mut nodes = SlotMap::with_key();
        let root_node = GraphNode::new();
        let root = nodes.insert(root_node);
        Self {
            phase: GraphPhase::Mod,
            root,
            nodes,
            components: SlotMap::with_key(),
            instances: SlotMap::with_key(),
            overrides: SlotMap::with_key(),
            node_properties: HashMap::new(),
            dirty_nodes: HashSet::new(),
            node_mapping: HashMap::new(),
        }
    }

    // Phase transition API:
    pub fn mod_phase(&mut self) -> ModContext {
        self.phase = GraphPhase::Mod;
        ModContext { scene: self }
    }

    // Internal methods used by context objects
    fn create_node(
        &mut self,
        parent: Option<GraphNodeId>,
        data_node_id: Option<NodeId>,
    ) -> NodeElementId {
        // Determine parent ID, default to root if none provided
        let parent_id = match parent {
            Some(GraphNodeId::Node(id)) => id,
            _ => self.root,
        };

        let mut new_node = GraphNode::new();
        new_node.parent = Some(GraphNodeId::Node(parent_id));
        new_node.data_node_id = data_node_id;

        // Insert the node and get its ID
        let node_id = self.nodes.insert(new_node);

        // Add as child to parent
        if let Some(parent) = self.nodes.get_mut(parent_id) {
            parent.children.push(node_id);
        }

        // Map data node to scene node if provided
        if let Some(data_id) = data_node_id {
            self.node_mapping.insert(data_id, node_id);
        }

        // Mark the node as dirty since it's new
        self.dirty_nodes.insert(GraphNodeId::Node(node_id));

        node_id
    }

    fn set_local_transform(&mut self, node_id: GraphNodeId, transform: TransformationMatrix) {
        match node_id {
            GraphNodeId::Node(id) => {
                if let Some(node) = self.nodes.get_mut(id) {
                    node.local_transform = transform;
                    node.dirty = true;
                    self.dirty_nodes.insert(node_id);
                }
            }
            // Handle component and instance cases similarly
            _ => {}
        }
    }

    fn set_local_bounds(&mut self, node_id: GraphNodeId, bounds: Bounds<f32>) {
        match node_id {
            GraphNodeId::Node(id) => {
                if let Some(node) = self.nodes.get_mut(id) {
                    node.local_bounds = bounds;
                    node.dirty = true;
                    self.dirty_nodes.insert(node_id);
                }
            }
            // Handle component and instance cases similarly
            _ => {}
        }
    }

    fn flush_dirty(&mut self) {
        // Process all dirty nodes to update their world transforms and bounds
        // First identify all dirty nodes and their ancestors
        let mut nodes_to_update: HashSet<NodeElementId> = HashSet::new();

        // Collect all dirty nodes
        let dirty_nodes: Vec<_> = self.dirty_nodes.iter().cloned().collect();
        for node_id in dirty_nodes {
            match node_id {
                GraphNodeId::Node(id) => {
                    nodes_to_update.insert(id);

                    // Also add all descendants to the update list
                    self.collect_descendants(id, &mut nodes_to_update);
                }
                _ => {}
            }
        }

        // Update world transforms for all identified nodes, starting from root
        // to ensure parent transforms are updated before children
        self.update_world_transform(self.root);

        // Update world bounds for all identified nodes
        for node_id in &nodes_to_update {
            self.update_world_bounds(*node_id);
        }

        // Clear dirty flags
        for node_id in nodes_to_update {
            if let Some(node) = self.nodes.get_mut(node_id) {
                node.dirty = false;
            }
        }

        self.dirty_nodes.clear();
    }

    /// Recursively collects all descendants of a node
    fn collect_descendants(&self, node_id: NodeElementId, nodes: &mut HashSet<NodeElementId>) {
        if let Some(node) = self.nodes.get(node_id) {
            for &child_id in &node.children {
                nodes.insert(child_id);
                self.collect_descendants(child_id, nodes);
            }
        }
    }

    /// Updates the world transform for a node and all its children recursively
    fn update_world_transform(&mut self, node_id: NodeElementId) {
        // First gather parent transform
        let parent_transform = {
            let node = match self.nodes.get(node_id) {
                Some(n) => n,
                None => return,
            };

            // Get parent's world transform
            match node.parent {
                Some(GraphNodeId::Node(parent_id)) => self
                    .nodes
                    .get(parent_id)
                    .map(|parent| parent.world_transform)
                    .unwrap_or_else(TransformationMatrix::default),
                _ => TransformationMatrix::default(),
            }
        };

        // Get node's local transform and children
        let (local_transform, children) = {
            let node = match self.nodes.get(node_id) {
                Some(n) => n,
                None => return,
            };
            (node.local_transform, node.children.clone())
        };

        // Calculate world transform
        let world_transform = parent_transform.compose(local_transform);

        // Update node's world transform
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.world_transform = world_transform;
        }

        // Update all children recursively
        for child_id in children {
            self.update_world_transform(child_id);
        }
    }

    /// Updates the world bounds for a node
    fn update_world_bounds(&mut self, node_id: NodeElementId) {
        // First collect the data we need
        let (transform, local_bounds) = match self.nodes.get(node_id) {
            Some(node) => (node.world_transform, node.local_bounds),
            None => return,
        };

        // Transform the four corners of the bounds to create an axis-aligned bounding box
        let origin_x = local_bounds.origin.x;
        let origin_y = local_bounds.origin.y;
        let width = local_bounds.size.width;
        let height = local_bounds.size.height;

        // Create points for the four corners
        let top_left = gpui::Point::new(gpui::Pixels(origin_x), gpui::Pixels(origin_y));
        let top_right = gpui::Point::new(gpui::Pixels(origin_x + width), gpui::Pixels(origin_y));
        let bottom_left = gpui::Point::new(gpui::Pixels(origin_x), gpui::Pixels(origin_y + height));
        let bottom_right = gpui::Point::new(
            gpui::Pixels(origin_x + width),
            gpui::Pixels(origin_y + height),
        );

        // Apply the transformation
        let top_left_transformed = transform.apply(top_left);
        let top_right_transformed = transform.apply(top_right);
        let bottom_left_transformed = transform.apply(bottom_left);
        let bottom_right_transformed = transform.apply(bottom_right);

        // Calculate the extremes to create an axis-aligned bounding box
        let min_x = top_left_transformed
            .x
            .0
            .min(top_right_transformed.x.0)
            .min(bottom_left_transformed.x.0)
            .min(bottom_right_transformed.x.0);

        let min_y = top_left_transformed
            .y
            .0
            .min(top_right_transformed.y.0)
            .min(bottom_left_transformed.y.0)
            .min(bottom_right_transformed.y.0);

        let max_x = top_left_transformed
            .x
            .0
            .max(top_right_transformed.x.0)
            .max(bottom_left_transformed.x.0)
            .max(bottom_right_transformed.x.0);

        let max_y = top_left_transformed
            .y
            .0
            .max(top_right_transformed.y.0)
            .max(bottom_left_transformed.y.0)
            .max(bottom_right_transformed.y.0);

        // Update world bounds
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.world_bounds = Bounds {
                origin: Point::new(min_x, min_y),
                size: Size::new(max_x - min_x, max_y - min_y),
            };
        }
    }

    fn get_world_transform(&self, node_id: GraphNodeId) -> Option<TransformationMatrix> {
        match node_id {
            GraphNodeId::Node(id) => self.nodes.get(id).map(|node| node.world_transform),
            _ => None,
        }
    }

    fn get_world_bounds(&self, node_id: GraphNodeId) -> Option<Bounds<f32>> {
        match node_id {
            GraphNodeId::Node(id) => self.nodes.get(id).map(|node| node.world_bounds),
            _ => None,
        }
    }

    fn get_local_transform(&self, node_id: GraphNodeId) -> Option<TransformationMatrix> {
        match node_id {
            GraphNodeId::Node(id) => self.nodes.get(id).map(|node| node.local_transform),
            _ => None,
        }
    }

    fn get_local_bounds(&self, node_id: GraphNodeId) -> Option<Bounds<f32>> {
        match node_id {
            GraphNodeId::Node(id) => self.nodes.get(id).map(|node| node.local_bounds),
            _ => None,
        }
    }

    /// Sets visibility of a node
    fn set_node_visibility(&mut self, node_id: GraphNodeId, visible: bool) {
        match node_id {
            GraphNodeId::Node(id) => {
                if let Some(node) = self.nodes.get_mut(id) {
                    node.visible = visible;
                    node.dirty = true;
                    self.dirty_nodes.insert(node_id);
                }
            }
            _ => {}
        }
    }

    /// Removes a node and all its children from the scene graph
    fn remove_node(&mut self, node_id: GraphNodeId) -> Option<NodeId> {
        match node_id {
            GraphNodeId::Node(id) => {
                // Can't remove the root node
                if id == self.root {
                    return None;
                }

                // Remove from parent's children list
                if let Some(parent_id) = self.nodes.get(id).and_then(|node| match node.parent {
                    Some(GraphNodeId::Node(parent_id)) => Some(parent_id),
                    _ => None,
                }) {
                    if let Some(parent) = self.nodes.get_mut(parent_id) {
                        parent.children.retain(|&child| child != id);
                    }
                }

                // Get the data node ID before removing
                let data_node_id = self.nodes.get(id).and_then(|node| node.data_node_id);

                // Remove mapping
                if let Some(data_id) = data_node_id {
                    self.node_mapping.remove(&data_id);
                }

                // Remove all children recursively
                let children = self
                    .nodes
                    .get(id)
                    .map_or(Vec::new(), |node| node.children.clone());
                for child_id in children {
                    self.remove_node(GraphNodeId::Node(child_id));
                }

                // Remove the node itself
                self.nodes.remove(id);

                data_node_id
            }
            _ => None,
        }
    }

    /// Gets the data node ID associated with a scene node
    fn get_data_node_id(&self, scene_node_id: GraphNodeId) -> Option<NodeId> {
        match scene_node_id {
            GraphNodeId::Node(id) => self.nodes.get(id).and_then(|node| node.data_node_id),
            _ => None,
        }
    }

    /// Gets the scene node ID associated with a data node
    fn get_scene_node_id(&self, data_node_id: NodeId) -> Option<GraphNodeId> {
        self.node_mapping
            .get(&data_node_id)
            .map(|&id| GraphNodeId::Node(id))
    }

    /// Adds an existing node as a child of another node
    fn add_child(&mut self, parent_id: GraphNodeId, child_id: GraphNodeId) -> bool {
        match (parent_id, child_id) {
            (GraphNodeId::Node(parent), GraphNodeId::Node(child)) => {
                // Check that both nodes exist
                if !self.nodes.contains_key(parent) || !self.nodes.contains_key(child) {
                    return false;
                }

                // Check if this would create a cycle
                if self.is_ancestor(GraphNodeId::Node(child), GraphNodeId::Node(parent)) {
                    return false;
                }

                // Remove from current parent's children list
                if let Some(old_parent_id) =
                    self.nodes.get(child).and_then(|node| match node.parent {
                        Some(GraphNodeId::Node(id)) => Some(id),
                        _ => None,
                    })
                {
                    if let Some(old_parent) = self.nodes.get_mut(old_parent_id) {
                        old_parent.children.retain(|&id| id != child);
                    }
                }

                // Update parent reference
                if let Some(child_node) = self.nodes.get_mut(child) {
                    child_node.parent = Some(GraphNodeId::Node(parent));
                    child_node.dirty = true;
                }

                // Add to new parent's children list
                if let Some(parent_node) = self.nodes.get_mut(parent) {
                    parent_node.children.push(child);
                }

                // Mark as dirty
                self.dirty_nodes.insert(GraphNodeId::Node(child));

                true
            }
            _ => false,
        }
    }

    /// Determines if a node is an ancestor of another node
    fn is_ancestor(&self, node_id: GraphNodeId, potential_descendant_id: GraphNodeId) -> bool {
        match (node_id, potential_descendant_id) {
            (GraphNodeId::Node(node), GraphNodeId::Node(descendant)) => {
                let mut current = Some(descendant);

                while let Some(id) = current {
                    if id == node {
                        return true;
                    }

                    current = self.nodes.get(id).and_then(|node| match node.parent {
                        Some(GraphNodeId::Node(parent_id)) => Some(parent_id),
                        _ => None,
                    });
                }

                false
            }
            _ => false,
        }
    }

    /// Performs hit testing to find nodes at a given point
    fn hit_test(&self, point: LocalPoint) -> Vec<GraphNodeId> {
        let mut hits = Vec::new();
        self.hit_test_recursive(self.root, &point, &mut hits);
        hits
    }

    /// Recursively tests nodes for hits
    fn hit_test_recursive(
        &self,
        node_id: NodeElementId,
        point: &LocalPoint,
        hits: &mut Vec<GraphNodeId>,
    ) {
        let node = match self.nodes.get(node_id) {
            Some(n) => n,
            None => return,
        };

        // Skip invisible nodes
        if !node.visible {
            return;
        }

        // Check if point is within bounds
        let world_bounds = node.world_bounds;
        let origin_x = world_bounds.origin.x;
        let origin_y = world_bounds.origin.y;
        let width = world_bounds.size.width;
        let height = world_bounds.size.height;

        if point.x >= origin_x
            && point.x <= origin_x + width
            && point.y >= origin_y
            && point.y <= origin_y + height
        {
            // Add this node to hits
            hits.push(GraphNodeId::Node(node_id));
        }

        // Test children (front to back)
        // Note: In a real implementation, you'd want to sort by z-index or draw order
        for &child_id in node.children.iter().rev() {
            self.hit_test_recursive(child_id, point, hits);
        }
    }
}

pub struct ModContext<'a> {
    scene: &'a mut SceneGraph2,
}

pub struct UpdateContext<'a> {
    scene: &'a mut SceneGraph2,
}

pub struct QueryContext<'a> {
    scene: &'a mut SceneGraph2,
}

pub struct PrepContext<'a> {
    scene: &'a mut SceneGraph2,
}

impl<'a> ModContext<'a> {
    /// Creates a new node with the specified parent
    pub fn create_node(&mut self, parent: Option<GraphNodeId>) -> NodeElementId {
        let parent = parent.unwrap_or_else(|| GraphNodeId::Node(self.scene.root));
        // Use existing create_node logic
        let node_id = self.scene.create_node(Some(parent.clone()), None);
        // Remains in Mod phase.
        node_id
    }

    /// Creates a new node associated with a data node
    pub fn create_node_with_data(
        &mut self,
        parent: Option<GraphNodeId>,
        data_node_id: NodeId,
    ) -> NodeElementId {
        let parent = parent.unwrap_or_else(|| GraphNodeId::Node(self.scene.root));
        // Use existing create_node logic with data node ID
        let node_id = self
            .scene
            .create_node(Some(parent.clone()), Some(data_node_id));
        // Remains in Mod phase.
        node_id
    }

    /// Removes a node and all its children from the scene graph
    pub fn remove_node(&mut self, node_id: GraphNodeId) -> Option<NodeId> {
        self.scene.remove_node(node_id)
    }

    /// Add an existing node as a child of another node
    pub fn add_child(&mut self, parent_id: GraphNodeId, child_id: GraphNodeId) -> bool {
        self.scene.add_child(parent_id, child_id)
    }

    pub fn set_local_transform(&mut self, node_id: GraphNodeId, transform: TransformationMatrix) {
        // Allowed only in Mod phase; mark dirty.
        self.scene.set_local_transform(node_id, transform)
    }

    pub fn set_local_bounds(&mut self, node_id: GraphNodeId, bounds: Bounds<f32>) {
        self.scene.set_local_bounds(node_id, bounds)
    }

    pub fn set_node_visibility(&mut self, node_id: GraphNodeId, visible: bool) {
        self.scene.set_node_visibility(node_id, visible)
    }

    pub fn get_data_node_id(&self, scene_node_id: GraphNodeId) -> Option<NodeId> {
        self.scene.get_data_node_id(scene_node_id)
    }

    pub fn get_scene_node_id(&self, data_node_id: NodeId) -> Option<GraphNodeId> {
        self.scene.get_scene_node_id(data_node_id)
    }

    // Transition to Update phase.
    pub fn commit(self) -> UpdateContext<'a> {
        // Optionally, you might check that you're still in Mod phase.
        self.scene.phase = GraphPhase::Update;
        UpdateContext { scene: self.scene }
    }
}

impl<'a> UpdateContext<'a> {
    pub fn flush_updates(&mut self) {
        // Call the flush_dirty() method to update all dirty nodes
        self.scene.flush_dirty();
    }

    // Once done, move to Query phase.
    pub fn commit(self) -> QueryContext<'a> {
        self.scene.phase = GraphPhase::Query;
        QueryContext { scene: self.scene }
    }
}

impl<'a> QueryContext<'a> {
    pub fn get_world_transform(&self, node_id: GraphNodeId) -> Option<TransformationMatrix> {
        self.scene.get_world_transform(node_id)
    }

    pub fn get_world_bounds(&self, node_id: GraphNodeId) -> Option<Bounds<f32>> {
        self.scene.get_world_bounds(node_id)
    }

    pub fn get_local_transform(&self, node_id: GraphNodeId) -> Option<TransformationMatrix> {
        self.scene.get_local_transform(node_id)
    }

    pub fn get_local_bounds(&self, node_id: GraphNodeId) -> Option<Bounds<f32>> {
        self.scene.get_local_bounds(node_id)
    }

    pub fn get_data_node_id(&self, scene_node_id: GraphNodeId) -> Option<NodeId> {
        self.scene.get_data_node_id(scene_node_id)
    }

    pub fn get_scene_node_id(&self, data_node_id: NodeId) -> Option<GraphNodeId> {
        self.scene.get_scene_node_id(data_node_id)
    }

    pub fn hit_test(&self, point: LocalPoint) -> Vec<GraphNodeId> {
        // Real hit testing implementation
        self.scene.hit_test(point)
    }

    // Transition to Prep phase.
    pub fn commit(self) -> PrepContext<'a> {
        self.scene.phase = GraphPhase::Prep;
        PrepContext { scene: self.scene }
    }
}

impl<'a> PrepContext<'a> {
    pub fn get_draw_list(&self) -> Vec<RenderItem> {
        // Traverse the graph to build a painter’s ordered list, with culling and batching.
        vec![] // placeholder
    }

    // Transition back to Mod phase for next frame.
    pub fn finish(self) -> ModContext<'a> {
        self.scene.phase = GraphPhase::Mod;
        ModContext { scene: self.scene }
    }
}

// Placeholder type:
pub struct RenderItem;

// Using LocalPoint to avoid conflict with gpui::Point
pub struct LocalPoint {
    pub x: f32,
    pub y: f32,
}

impl LocalPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

//--|CANVAS|--//

pub enum Direction {
    North,
    East,
    South,
    West,
}

pub fn new_canvas(window: &mut Window, cx: &mut App) {}

pub struct Canvas {
    visible_bounds: Bounds<f32>,
    bounds: Bounds<f32>,
    // scale factor = zoom
    scale_factor: f32,
}

impl Canvas {
    pub fn update_visible_bounds(&mut self) {}
    pub fn expand_bounds(&mut self, direction: Direction, amount: f32) {}
}
