use gpui::{Bounds, TransformationMatrix};
use std::fmt::Debug;

use crate::node::NodeId;
use crate::scene_graph::SceneNodeId;

/// A node in the scene graph hierarchy
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
    /// Creates a new scene node
    pub fn new(
        parent: Option<SceneNodeId>,
        data_node_id: Option<NodeId>,
    ) -> Self {
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