use std::{
    collections::HashMap,
    fmt::{self, Display},
    num::NonZeroU64,
};

use gpui::{Bounds, TransformationMatrix};
use slotmap::KeyData;

use crate::node::NodeId;

/// Defines a unique identifier for nodes within the scene graph.
slotmap::new_key_type! {
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
    nodes: HashMap<SceneNodeId, SceneNode>,

    /// Maps from data node IDs to scene node IDs, allowing lookups in both directions
    node_mapping: HashMap<NodeId, SceneNodeId>,

    /// The next unique ID
    next_id: usize,
}

/// SceneNode represents a single node in the scene graph hierarchy.
///
/// Each SceneNode maintains its position in the hierarchy (parent/children),
/// transformation information (both local and world), and bounds for rendering
/// and hit testing. A scene node may be associated with a data node from the
/// flat data model, or it may be a pure structural node (like the canvas root).
pub struct SceneNode {
    /// Unique identifier for this scene node
    id: SceneNodeId,

    /// Reference to the parent node, if any
    /// Root nodes have no parent (None)
    parent: Option<SceneNodeId>,

    /// References to all child nodes of this node
    children: Vec<SceneNodeId>,

    /// The transformation matrix relative to the parent node
    /// This defines how this node is positioned, scaled, and rotated relative to its parent
    local_transform: TransformationMatrix,

    /// The absolute transformation matrix in world space
    /// This is the combination of all parent transformations with the local transform
    world_transform: TransformationMatrix,

    /// The bounding box of this node in its local coordinate space before transformation
    /// This is typically derived from the node's layout properties
    local_bounds: Bounds<f32>,

    /// The bounding box of this node in world space after all transformations
    /// Used for visibility culling and hit testing
    world_bounds: Bounds<f32>,

    /// Reference to the associated data node in the flat data model, if any
    /// Structural nodes like the canvas root may not have a data node
    data_node_id: Option<NodeId>,

    /// Whether this node should be considered for rendering and hit testing
    /// Useful for temporarily hiding nodes without removing them
    visible: bool,
}
