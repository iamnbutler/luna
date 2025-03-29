#![allow(unused_variables)]

use gpui::{px, solid_background, Hsla, Point, Size, Bounds};

/// A unique identifier for a canvas node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl NodeId {
    pub fn new(id: usize) -> Self {
        NodeId(id)
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node-{}", self.0)
    }
}

/// Types of nodes that can be rendered on the canvas
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    /// A rectangle shape
    Rectangle,
}

/// Layout information for a node
#[derive(Debug, Clone)]
pub struct NodeLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl NodeLayout {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn bounds(&self) -> Bounds<f32> {
        Bounds {
            origin: Point::new(self.x, self.y),
            size: Size::new(self.width, self.height),
        }
    }
}

/// Common properties shared by all canvas nodes
pub trait NodeCommon: std::fmt::Debug {
    /// Get the node's ID
    fn id(&self) -> NodeId;
    
    /// Get the node type
    fn node_type(&self) -> NodeType;
    
    /// Get the layout for this node
    fn layout(&self) -> &NodeLayout;
    
    /// Get mutable access to the layout
    fn layout_mut(&mut self) -> &mut NodeLayout;
    
    /// Get the fill color
    fn fill(&self) -> Option<Hsla>;
    
    /// Set the fill color
    fn set_fill(&mut self, color: Option<Hsla>);
    
    /// Get the border color
    fn border_color(&self) -> Option<Hsla>;
    
    /// Get the border width
    fn border_width(&self) -> f32;
    
    /// Set the border properties
    fn set_border(&mut self, color: Option<Hsla>, width: f32);
    
    /// Get the corner radius
    fn corner_radius(&self) -> f32;
    
    /// Set the corner radius
    fn set_corner_radius(&mut self, radius: f32);
    
    /// Check if a point is inside this node
    fn contains_point(&self, point: &Point<f32>) -> bool {
        let bounds = self.layout().bounds();
        bounds.contains(point)
    }
    
    /// Get the bounds of this node
    fn bounds(&self) -> Bounds<f32> {
        self.layout().bounds()
    }
}

/// A rectangle node that can be rendered on the canvas
#[derive(Debug)]
pub struct RectangleNode {
    pub id: NodeId,
    pub layout: NodeLayout,
    pub fill: Option<Hsla>,
    pub border_color: Option<Hsla>,
    pub border_width: f32,
    pub corner_radius: f32,
}

impl RectangleNode {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            layout: NodeLayout::new(0.0, 0.0, 100.0, 100.0),
            fill: Some(Hsla::white()),
            border_color: Some(Hsla::black()),
            border_width: 1.0,
            corner_radius: 0.0,
        }
    }

    /// Create a rectangle with specific dimensions and position
    pub fn with_rect(id: NodeId, x: f32, y: f32, width: f32, height: f32) -> Self {
        let mut node = Self::new(id);
        node.layout = NodeLayout::new(x, y, width, height);
        node
    }
}

impl NodeCommon for RectangleNode {
    fn id(&self) -> NodeId {
        self.id
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::Rectangle
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
}

/// A factory for creating and managing nodes
#[derive(Debug)]
pub struct NodeFactory {
    next_id: usize,
}

impl Default for NodeFactory {
    fn default() -> Self {
        Self { next_id: 1 }
    }
}

impl NodeFactory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate a new unique node ID
    pub fn next_id(&mut self) -> NodeId {
        let id = NodeId::new(self.next_id);
        self.next_id += 1;
        id
    }

    /// Create a new rectangle node
    pub fn create_rectangle(&mut self) -> RectangleNode {
        RectangleNode::new(self.next_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangle_node() {
        let id = NodeId::new(2);
        let rect = RectangleNode::new(id);

        assert_eq!(rect.node_type(), NodeType::Rectangle);
        assert_eq!(rect.id(), id);
        assert_eq!(rect.corner_radius(), 0.0);
    }

    #[test]
    fn test_contains_point() {
        let id = NodeId::new(1);
        let rect = RectangleNode::with_rect(id, 10.0, 10.0, 100.0, 100.0);

        // Test points inside and outside
        let point_inside = Point::new(50.0, 50.0);
        let point_outside = Point::new(200.0, 200.0);

        assert!(rect.contains_point(&point_inside));
        assert!(!rect.contains_point(&point_outside));
    }
}