//! # Scene Graph System
//!
//! The scene graph is a core architectural component that provides spatial organization
//! for visual elements in Luna. It implements a hierarchical tree structure that efficiently
//! manages transformations, visibility, and spatial operations.
//!
//! ## Key Concepts
//!
//! - **Scene Nodes**: Hierarchical nodes that form the structure of the graph
//! - **Transformations**: Each node has local and world transformations that propagate through the hierarchy
//! - **Bounds Computation**: Automatic calculation of axis-aligned bounding boxes in world space
//! - **Data Mapping**: Bi-directional mapping between scene nodes and data model nodes
//!
//! The scene graph is separated from the actual data model, functioning purely as a spatial
//! organization layer. This separation of concerns allows the data model to focus on properties
//! and relationships while the scene graph handles coordinate systems and transformations.

#![allow(unused, dead_code)]

pub mod scene_node;

use gpui::{Bounds, Point, Size};
use luna_core::bounds::Bounds as LunaBounds;
use node::{AnyNode, NodeCommon, NodeId};
use slotmap::{KeyData, SlotMap};
use std::{
    collections::HashMap,
    fmt::{self, Display},
    num::NonZeroU64,
};

slotmap::new_key_type! {
/// Defines a unique identifier for nodes within the scene graph.
    pub struct SceneNodeId;
}

impl From<u64> for SceneNodeId {
    fn from(value: u64) -> Self {
        Self(KeyData::from_ffi(value))
    }
}

impl SceneNodeId {
    /// Converts this scene node id to a [NonZeroU64]
    pub fn as_non_zero_u64(self) -> NonZeroU64 {
        NonZeroU64::new(self.0.as_ffi()).unwrap()
    }

    /// Converts this scene node id to a [u64]
    pub fn as_u64(self) -> u64 {
        self.0.as_ffi()
    }
}

impl Display for SceneNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_u64())
    }
}

/// SceneGraph manages the spatial relationships between nodes in the canvas.
///
/// While the data model maintains a flat list of nodes with their properties,
/// the scene graph adds hierarchical structure for efficient:
/// - Transformation propagation (zoom, scroll, etc.)
/// - Visibility culling (only rendering what's visible)
/// - Hit testing (determining which node is at a given point)
///
/// The scene graph serves as the spatial organization layer on top of the
/// data model, without affecting how the data is stored and managed.
pub struct SceneGraph {
    /// The root node of the scene graph, typically represents the canvas itself
    root: SceneNodeId,

    /// Storage for all scene nodes, indexed by their IDs
    nodes: SlotMap<SceneNodeId, SceneNode>,

    /// Maps from data node IDs to scene node IDs, allowing lookups in both directions
    node_mapping: HashMap<NodeId, SceneNodeId>,
}

impl SceneGraph {
    /// Creates a new, empty scene graph with a root node
    pub fn new() -> Self {
        let mut nodes = SlotMap::with_key();
        let root_node = SceneNode {
            parent: None,
            children: Vec::new(),
            data_node_id: None,
            visible: true,
        };

        let root = nodes.insert(root_node);

        Self {
            root,
            nodes,
            node_mapping: HashMap::new(),
        }
    }

    /// Returns the ID of the root node
    pub fn root(&self) -> SceneNodeId {
        self.root
    }

    /// Creates a new scene node as a child of the specified parent
    pub fn create_node(
        &mut self,
        parent_id: Option<SceneNodeId>,
        data_node_id: Option<NodeId>,
    ) -> SceneNodeId {
        let parent_id = parent_id.unwrap_or(self.root);

        // Create the new node
        let node = SceneNode {
            parent: Some(parent_id),
            children: Vec::new(),
            data_node_id,
            visible: true,
        };

        // Insert the node and get its ID
        let node_id = self.nodes.insert(node);

        // Add as child to parent
        if let Some(parent) = self.nodes.get_mut(parent_id) {
            parent.children.push(node_id);
        }

        // Map data node to scene node if provided
        if let Some(data_id) = data_node_id {
            self.node_mapping.insert(data_id, node_id);
        }

        node_id
    }

    /// Adds an existing node as a child of another node
    pub fn add_child(&mut self, parent_id: SceneNodeId, child_id: SceneNodeId) -> bool {
        // Check that both nodes exist
        if !self.nodes.contains_key(parent_id) || !self.nodes.contains_key(child_id) {
            return false;
        }

        // Check if this would create a cycle
        if self.is_ancestor(child_id, parent_id) {
            return false;
        }

        // Remove from current parent's children list
        if let Some(old_parent_id) = self.nodes.get(child_id).and_then(|node| node.parent) {
            if let Some(old_parent) = self.nodes.get_mut(old_parent_id) {
                old_parent.children.retain(|&id| id != child_id);
            }
        }

        // Update parent reference
        if let Some(child) = self.nodes.get_mut(child_id) {
            child.parent = Some(parent_id);
        }

        // Add to new parent's children list
        if let Some(parent) = self.nodes.get_mut(parent_id) {
            parent.children.push(child_id);
        }

        true
    }

    /// Removes a node and all its children from the scene graph
    pub fn remove_node(&mut self, node_id: SceneNodeId) -> Option<NodeId> {
        // Can't remove the root node
        if node_id == self.root {
            return None;
        }

        // Remove from parent's children list
        if let Some(parent_id) = self.nodes.get(node_id).and_then(|node| node.parent) {
            if let Some(parent) = self.nodes.get_mut(parent_id) {
                parent.children.retain(|&id| id != node_id);
            }
        }

        // Get the node and its data ID before removing
        let data_node_id = self.nodes.get(node_id).and_then(|node| node.data_node_id);

        // Remove mapping
        if let Some(data_id) = data_node_id {
            self.node_mapping.remove(&data_id);
        }

        // Remove all children recursively
        if let Some(node) = self.nodes.get(node_id) {
            let children = node.children.clone();
            for child_id in children {
                self.remove_node(child_id);
            }
        }

        // Remove the node itself
        self.nodes.remove(node_id);

        data_node_id
    }

    /// Gets the data node ID associated with a scene node
    pub fn get_data_node_id(&self, scene_node_id: SceneNodeId) -> Option<NodeId> {
        self.nodes
            .get(scene_node_id)
            .and_then(|node| node.data_node_id)
    }

    /// Gets the scene node ID associated with a data node
    pub fn get_scene_node_from_data_node(&self, data_node_id: NodeId) -> Option<SceneNodeId> {
        self.node_mapping.get(&data_node_id).copied()
    }

    /// Alias for get_scene_node_from_data_node for compatibility
    pub fn get_scene_node_for_data_node(&self, data_node_id: NodeId) -> Option<SceneNodeId> {
        self.get_scene_node_from_data_node(data_node_id)
    }

    /// Alias for get_data_node_id for compatibility
    pub fn get_data_node_for_scene_node(&self, scene_node_id: SceneNodeId) -> Option<NodeId> {
        self.get_data_node_id(scene_node_id)
    }

    /// Gets the children of a scene node
    pub fn get_children(&self, node_id: SceneNodeId) -> Vec<SceneNodeId> {
        self.nodes
            .get(node_id)
            .map(|node| node.children.clone())
            .unwrap_or_default()
    }

    /// Clears all nodes from the scene graph except the root
    pub fn clear(&mut self) {
        // Save the root node
        let root_node = SceneNode {
            parent: None,
            children: Vec::new(),
            data_node_id: None,
            visible: true,
        };

        // Clear everything
        self.nodes.clear();
        self.node_mapping.clear();

        // Re-insert the root node
        self.root = self.nodes.insert(root_node);
    }

    /// Calculate the world position of a node by traversing up the parent hierarchy
    /// and summing local positions from FrameNodes
    pub fn get_world_position(
        &self,
        node_id: SceneNodeId,
        get_node: impl Fn(NodeId) -> Option<AnyNode>,
    ) -> glam::Vec2 {
        let node = match self.nodes.get(node_id) {
            Some(n) => n,
            None => return glam::Vec2::ZERO,
        };

        // Get the local position from the frame node
        let local_pos = node
            .data_node_id
            .and_then(|id| get_node(id))
            .map(|node| {
                let layout = node.layout();
                glam::Vec2::new(layout.x, layout.y)
            })
            .unwrap_or(glam::Vec2::ZERO);

        // If there's a parent, add its world position
        if let Some(parent_id) = node.parent {
            self.get_world_position(parent_id, get_node) + local_pos
        } else {
            local_pos
        }
    }

    /// Calculate the world bounds of a node
    pub fn get_world_bounds(
        &self,
        node_id: SceneNodeId,
        get_node: impl Fn(NodeId) -> Option<AnyNode>,
    ) -> LunaBounds {
        let pos = self.get_world_position(node_id, &get_node);

        // Get size from frame node
        let size = self
            .nodes
            .get(node_id)
            .and_then(|node| node.data_node_id)
            .and_then(|id| get_node(id))
            .map(|node| {
                let layout = node.layout();
                glam::Vec2::new(layout.width, layout.height)
            })
            .unwrap_or(glam::Vec2::ZERO);

        LunaBounds::from_origin_size(pos, size)
    }

    /// Get a reference to a node by its ID
    pub fn get_node(&self, node_id: SceneNodeId) -> Option<&SceneNode> {
        self.nodes.get(node_id)
    }

    /// Sets the visibility of a node
    pub fn set_visible(&mut self, node_id: SceneNodeId, visible: bool) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.visible = visible;
        }
    }

    /// Determines if a node is an ancestor of another node in the hierarchy
    ///
    /// This method traverses the parent chain of the descendant node upward
    /// to determine if the specified node exists in its ancestry. This check
    /// is critical for preventing cycles during hierarchy modifications, which
    /// would create infinite loops during transformation propagation.
    ///
    /// The algorithm uses iterative parent traversal rather than recursion
    /// to handle arbitrary depth hierarchies efficiently.
    fn is_ancestor(&self, node_id: SceneNodeId, descendant_id: SceneNodeId) -> bool {
        let mut current = Some(descendant_id);
        while let Some(id) = current {
            if id == node_id {
                return true;
            }
            current = self.nodes.get(id).and_then(|node| node.parent);
        }
        false
    }
}

/// SceneNode represents a single node in the scene graph hierarchy.
///
/// Each SceneNode maintains its position in the hierarchy (parent/children),
/// transformation information (both local and world), and bounds for rendering
/// and hit testing. A scene node may be associated with a data node from the
/// flat data model, or it may be a pure structural node (like the canvas root).
#[derive(Debug)]
pub struct SceneNode {
    /// Reference to the parent node, if any
    /// Root nodes have no parent (None)
    parent: Option<SceneNodeId>,

    /// References to all child nodes of this node
    children: Vec<SceneNodeId>,

    /// Reference to the associated data node in the flat data model, if any
    /// Structural nodes like the canvas root may not have a data node
    data_node_id: Option<NodeId>,

    /// Whether this node should be considered for rendering and hit testing
    /// Useful for temporarily hiding nodes without removing them
    visible: bool,
}

impl SceneNode {
    /// Returns a reference to the node's children
    pub fn children(&self) -> &Vec<SceneNodeId> {
        &self.children
    }

    /// Returns the data node ID associated with this scene node
    pub fn data_node_id(&self) -> Option<NodeId> {
        self.data_node_id
    }

    /// Returns whether the node is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Point, Size};

    #[test]
    fn test_scene_graph_creation() {
        let graph = SceneGraph::new();

        // The graph should have a root node
        assert!(graph.get_node(graph.root()).is_some());

        // The root should have no parent
        assert!(graph.get_node(graph.root()).unwrap().parent.is_none());

        // The root should have no children initially
        assert!(graph.get_node(graph.root()).unwrap().children.is_empty());
    }

    #[test]
    fn test_create_node() {
        let mut graph = SceneGraph::new();
        let root = graph.root();

        // Create a node with no explicit parent (should use root)
        let node1 = graph.create_node(None, None);

        // Create a node with explicit parent
        let node2 = graph.create_node(Some(node1), None);

        // Check parent-child relationships
        assert_eq!(graph.get_node(node1).unwrap().parent, Some(root));
        assert_eq!(graph.get_node(node2).unwrap().parent, Some(node1));

        assert!(graph.get_node(root).unwrap().children.contains(&node1));
        assert!(graph.get_node(node1).unwrap().children.contains(&node2));
    }

    #[test]
    fn test_add_child() {
        let mut graph = SceneGraph::new();
        let root = graph.root();

        // Create two nodes
        let node1 = graph.create_node(None, None);
        let node2 = graph.create_node(None, None);

        // Both should be children of root initially
        assert!(graph.get_node(root).unwrap().children.contains(&node1));
        assert!(graph.get_node(root).unwrap().children.contains(&node2));

        // Make node2 a child of node1
        assert!(graph.add_child(node1, node2));

        // Check the new relationships
        assert!(!graph.get_node(root).unwrap().children.contains(&node2));
        assert!(graph.get_node(node1).unwrap().children.contains(&node2));
        assert_eq!(graph.get_node(node2).unwrap().parent, Some(node1));
    }

    #[test]
    fn test_remove_node() {
        let mut graph = SceneGraph::new();
        let root = graph.root();

        // Create a hierarchy: root -> node1 -> node2
        let node1 = graph.create_node(None, None);
        let node2 = graph.create_node(Some(node1), None);

        // Verify initial relationships
        assert!(graph.get_node(root).unwrap().children.contains(&node1));
        assert!(graph.get_node(node1).unwrap().children.contains(&node2));

        // Remove node1 (should also remove node2)
        graph.remove_node(node1);

        // Verify nodes are gone
        assert!(!graph.get_node(root).unwrap().children.contains(&node1));
        assert!(graph.get_node(node1).is_none());
        assert!(graph.get_node(node2).is_none());
    }

    #[test]
    fn test_cannot_create_cycle() {
        let mut graph = SceneGraph::new();

        // Create a hierarchy: root -> node1 -> node2 -> node3
        let node1 = graph.create_node(None, None);
        let node2 = graph.create_node(Some(node1), None);
        let node3 = graph.create_node(Some(node2), None);

        // Try to make node1 a child of node3 (would create a cycle)
        assert!(!graph.add_child(node3, node1));

        // Relationships should remain unchanged
        assert_eq!(graph.get_node(node1).unwrap().parent, Some(graph.root()));
        assert_eq!(graph.get_node(node2).unwrap().parent, Some(node1));
        assert_eq!(graph.get_node(node3).unwrap().parent, Some(node2));
    }

    #[test]
    fn test_node_mapping() {
        let mut graph = SceneGraph::new();

        // Create a data node ID
        let data_id = NodeId::new(123);

        // Create a scene node linked to the data node
        let scene_id = graph.create_node(None, Some(data_id));

        // Verify mapping works both ways
        assert_eq!(graph.get_data_node_id(scene_id), Some(data_id));
        assert_eq!(graph.get_scene_node_from_data_node(data_id), Some(scene_id));

        // Removing the scene node should remove the mapping
        graph.remove_node(scene_id);
        assert_eq!(graph.get_scene_node_from_data_node(data_id), None);
    }

    #[test]
    fn test_node_visibility() {
        let mut graph = SceneGraph::new();

        // Create a node
        let node = graph.create_node(None, None);

        // Node should be visible by default
        assert!(graph.get_node(node).unwrap().visible);

        // Change visibility to false
        graph.set_visible(node, false);
        assert!(!graph.get_node(node).unwrap().visible);

        // Change visibility back to true
        graph.set_visible(node, true);
        assert!(graph.get_node(node).unwrap().visible);
    }
}
