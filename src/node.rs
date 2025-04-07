//! # Node System
//!
//! This module defines the core data model for visual elements in Luna. The node system
//! implements a type-safe approach to representing and manipulating canvas elements.
//!
//! ## Key Components
//!
//! - **NodeId**: Unique identifier for canvas nodes
//! - **NodeType**: Enumeration of supported element types (Frame, etc.)
//! - **NodeLayout**: Position and dimension properties shared by all nodes
//! - **NodeCommon**: Trait defining shared behavior across node types
//!
//! The node system focuses on managing the data model aspect of elements, while
//! the scene graph handles spatial relationships and transformations. This separation
//! allows for efficient data management independent of visual representation.

#![allow(unused, dead_code)]
use gpui::{Bounds, Hsla, Point, Size};

pub mod frame;

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
    /// A frame that can contain other nodes
    Frame,
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
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn bounds(&self) -> Bounds<f32> {
        Bounds {
            origin: Point::new(self.x, self.y),
            size: Size::new(self.width, self.height),
        }
    }
}

/// Core trait defining the common interface for all canvas elements
///
/// This trait establishes a unified API for interacting with different node types,
/// enforcing consistent behavior for essential operations like:
/// - Identity and type determination
/// - Layout manipulation and bounds calculation
/// - Visual styling (fill, border, corner radius)
/// - Spatial queries (point containment)
///
/// By implementing this trait, node types gain consistent behavior while allowing
/// type-specific customization. This enables polymorphic operations across
/// heterogeneous collections of nodes.
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

/// Factory for generating nodes with guaranteed unique identifiers
///
/// NodeFactory centralizes node creation and ID allocation, ensuring that:
/// - Each node receives a unique NodeId
/// - Node creation follows consistent initialization patterns
/// - Factory methods encapsulate creation logic for different node types
///
/// This pattern allows the application to maintain referential integrity
/// across the node system without exposing ID generation details.
#[derive(Debug)]
pub struct NodeFactory {
    /// Internal counter for generating sequential node IDs
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

    /// Create a new frame node
    pub fn create_frame(&mut self) -> frame::FrameNode {
        frame::FrameNode::new(self.next_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::frame::FrameNode;

    #[test]
    fn test_node_id() {
        let id = NodeId::new(42);
        assert_eq!(id.0, 42);
        assert_eq!(format!("{}", id), "Node-42");
    }

    #[test]
    fn test_node_layout() {
        let layout = NodeLayout::new(10.0, 20.0, 100.0, 200.0);
        assert_eq!(layout.x, 10.0);
        assert_eq!(layout.y, 20.0);
        assert_eq!(layout.width, 100.0);
        assert_eq!(layout.height, 200.0);

        let bounds = layout.bounds();
        assert_eq!(bounds.origin.x, 10.0);
        assert_eq!(bounds.origin.y, 20.0);
        assert_eq!(bounds.size.width, 100.0);
        assert_eq!(bounds.size.height, 200.0);
    }

    #[test]
    fn test_node_factory() {
        let mut factory = NodeFactory::new();
        
        let id1 = factory.next_id();
        let id2 = factory.next_id();
        
        assert_eq!(id1.0, 1);
        assert_eq!(id2.0, 2);
        
        let frame = factory.create_frame();
        assert_eq!(frame.id().0, 3);
    }
}