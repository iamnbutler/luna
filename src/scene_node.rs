//! # Scene Graph Node
//!
//! This module defines the core structural element of the scene graph system.
//! SceneNode represents a single node in the spatial hierarchy and maintains
//! both hierarchical relationships and transformation state.
//!
//! ## Architecture
//!
//! Scene nodes form the backbone of Luna's spatial organization:
//! 
//! - They track parent-child relationships in the scene hierarchy
//! - They maintain both local and world transformation matrices
//! - They store both local and world bounds for efficient spatial operations
//! - They link to corresponding data nodes in the application's data model
//!
//! Scene nodes provide the spatial organization layer, while data nodes (NodeId)
//! contain the actual element properties.

use crate::node::NodeId;
use crate::scene_graph::SceneNodeId;
use gpui::{Bounds, TransformationMatrix};
use std::fmt::Debug;

/// Primary structural component of the scene graph hierarchy
///
/// SceneNode maintains both the structural relationships (parent/children hierarchy)
/// and the spatial state (transformations and bounds) of an element in the scene.
/// It acts as the spatial representation layer of visual elements, while maintaining
/// a link to the corresponding data node that contains element properties.
///
/// The dual transform and bounds storage (local and world) enables efficient:
/// - Hierarchy traversal and manipulation
/// - Spatial queries (hit testing, visibility culling)
/// - Coordinate space conversions
/// - Transform propagation through the hierarchy
#[derive(Debug)]
pub struct SceneNode {
    /// Parent node reference
    pub(crate) parent: Option<SceneNodeId>,

    /// Child nodes
    pub(crate) children: Vec<SceneNodeId>,

    /// Transform relative to parent
    pub(crate) local_transform: TransformationMatrix,

    /// Absolute transform in world space
    pub(crate) world_transform: TransformationMatrix,

    /// Bounds in local coordinate space
    pub(crate) local_bounds: Bounds<f32>,

    /// Bounds in world space after transformation
    pub(crate) world_bounds: Bounds<f32>,

    /// Associated data node ID
    pub(crate) data_node_id: Option<NodeId>,

    /// Visibility flag
    pub(crate) visible: bool,
}

impl SceneNode {
    /// Creates a new scene node with default transformation state
    ///
    /// This factory method initializes a scene node with:
    /// - Optional parent link for hierarchy placement
    /// - Optional data node link for model association
    /// - Identity transformations (no translation, rotation, or scale)
    /// - Zero-sized bounds
    /// - Visible by default
    ///
    /// After creation, the node must be inserted into the scene graph and
    /// have its transforms and bounds updated accordingly.
    pub fn new(parent: Option<SceneNodeId>, data_node_id: Option<NodeId>) -> Self {
        Self {
            parent,
            children: Vec::new(),
            local_transform: TransformationMatrix::unit(),
            world_transform: TransformationMatrix::unit(),
            local_bounds: Bounds::default(),
            world_bounds: Bounds::default(),
            data_node_id,
            visible: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_scene_node() {
        let node = SceneNode::new(None, None);
        assert!(node.parent.is_none());
        assert!(node.children.is_empty());
        assert!(node.visible);
    }
}
