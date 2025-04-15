//! # Frame Node Implementation
//!
//! Implements the FrameNode type, a fundamental container node that can hold child nodes.
//! Frames are the core building blocks of the Luna canvas system, serving as containers
//! for other visual elements with configurable styling properties.

use crate::node::{NodeCommon, NodeId, NodeLayout, NodeType, NodeTypography};
use gpui::Hsla;
use smallvec::{smallvec, SmallVec};

use super::Shadow;

/// Concrete implementation of a frame visual element
///
/// FrameNode represents a rectangular element that can contain children nodes with configurable:
/// - Position and dimensions via NodeLayout
/// - Fill color (optional)
/// - Border properties (color and width)
/// - Corner radius for rounded rectangles
/// - Children nodes that are displayed inside and clipped to the frame bounds
///
/// As the fundamental building block in the canvas system, frames
/// serve as the basis for many other visual elements and are optimized
/// for efficient rendering and manipulation. Frames can contain other nodes as children,
/// creating a hierarchy of elements.
#[derive(Debug, Clone)]
pub struct FrameNode {
    pub id: NodeId,
    pub layout: NodeLayout,
    pub fill: Option<Hsla>,
    pub border_color: Option<Hsla>,
    pub border_width: f32,
    pub corner_radius: f32,
    pub shadows: SmallVec<[Shadow; 1]>,
    pub children: Vec<NodeId>,
    pub font_family: Option<String>,
    pub font_size: Option<String>,
    pub font_weight: Option<String>,
    pub text_color: Option<Hsla>,
    pub text_align: Option<String>,
}

impl FrameNode {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            layout: NodeLayout::new(0.0, 0.0, 100.0, 100.0),
            fill: Some(Hsla::white()),
            border_color: Some(Hsla::black()),
            border_width: 1.0,
            corner_radius: 0.0,
            shadows: smallvec![],
            children: Vec::new(),
            font_family: None,
            font_size: None,
            font_weight: None,
            text_color: None,
            text_align: None,
        }
    }

    /// Create a frame with specific dimensions and position
    pub fn with_rect(id: NodeId, x: f32, y: f32, width: f32, height: f32) -> Self {
        let mut node = Self::new(id);
        node.layout = NodeLayout::new(x, y, width, height);
        node
    }

    /// Add a child node to this frame
    ///
    /// Returns true if the child was added (it wasn't already a child)
    pub fn add_child(&mut self, child_id: NodeId) -> bool {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
            true
        } else {
            false
        }
    }

    /// Remove a child node from this frame
    ///
    /// Returns true if the child was removed (it was present)
    pub fn remove_child(&mut self, child_id: NodeId) -> bool {
        let len_before = self.children.len();
        self.children.retain(|id| *id != child_id);
        len_before != self.children.len()
    }

    /// Check if this frame contains a specific child
    pub fn has_child(&self, child_id: NodeId) -> bool {
        self.children.contains(&child_id)
    }

    /// Get a reference to the children of this frame
    pub fn children(&self) -> &Vec<NodeId> {
        &self.children
    }
}

impl NodeCommon for FrameNode {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Frame
    }

    fn layout(&self) -> &NodeLayout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut NodeLayout {
        &mut self.layout
    }

    fn fill(&self) -> Option<Hsla> {
        self.fill
    }

    fn set_fill(&mut self, color: Option<Hsla>) {
        self.fill = color;
    }

    fn border_color(&self) -> Option<Hsla> {
        self.border_color
    }

    fn border_width(&self) -> f32 {
        self.border_width
    }

    fn set_border(&mut self, color: Option<Hsla>, width: f32) {
        self.border_color = color;
        self.border_width = width;
    }

    fn corner_radius(&self) -> f32 {
        self.corner_radius
    }

    fn set_corner_radius(&mut self, radius: f32) {
        self.corner_radius = radius;
    }

    fn shadows(&self) -> SmallVec<[Shadow; 1]> {
        self.shadows.clone()
    }

    fn set_shadows(&mut self, shadows: SmallVec<[Shadow; 1]>) {
        self.shadows = shadows
    }
}

impl NodeTypography for FrameNode {
    fn font_family(&self) -> Option<String> {
        self.font_family.clone()
    }
    
    fn set_font_family(&mut self, family: Option<String>) {
        self.font_family = family;
    }
    
    fn font_size(&self) -> Option<String> {
        self.font_size.clone()
    }
    
    fn set_font_size(&mut self, size: Option<String>) {
        self.font_size = size;
    }
    
    fn font_weight(&self) -> Option<String> {
        self.font_weight.clone()
    }
    
    fn set_font_weight(&mut self, weight: Option<String>) {
        self.font_weight = weight;
    }
    
    fn text_color(&self) -> Option<Hsla> {
        self.text_color
    }
    
    fn set_text_color(&mut self, color: Option<Hsla>) {
        self.text_color = color;
    }
    
    fn text_align(&self) -> Option<String> {
        self.text_align.clone()
    }
    
    fn set_text_align(&mut self, align: Option<String>) {
        self.text_align = align;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Point;

    #[test]
    fn test_frame_node() {
        let id = NodeId::new(2);
        let frame = FrameNode::new(id);

        assert_eq!(frame.node_type(), NodeType::Frame);
        assert_eq!(frame.id(), id);
        assert_eq!(frame.corner_radius(), 0.0);
        assert!(frame.children().is_empty());
    }

    #[test]
    fn test_contains_point() {
        let id = NodeId::new(1);
        let frame = FrameNode::with_rect(id, 10.0, 10.0, 100.0, 100.0);

        // Test points inside and outside using WorldPoint
        use crate::coordinates::WorldPoint;
        let point_inside = WorldPoint::new(50.0, 50.0);
        let point_outside = WorldPoint::new(200.0, 200.0);

        assert!(frame.contains_point(&point_inside));
        assert!(!frame.contains_point(&point_outside));
    }

    #[test]
    fn test_frame_children() {
        let parent_id = NodeId::new(1);
        let child_id = NodeId::new(2);

        let mut frame = FrameNode::new(parent_id);

        // Initially no children
        assert_eq!(frame.children().len(), 0);

        // Add a child
        frame.add_child(child_id);
        assert_eq!(frame.children().len(), 1);
        assert_eq!(frame.children()[0], child_id);

        // Remove the child
        frame.remove_child(child_id);
        assert_eq!(frame.children().len(), 0);
    }
}
