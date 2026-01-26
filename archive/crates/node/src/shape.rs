//! # Shape Node Implementation
//!
//! Implements the ShapeNode type, a fundamental visual element for rendering geometric shapes.
//! Shapes are leaf nodes in the Luna canvas system that render various geometric primitives
//! with configurable styling properties.

use crate::{NodeCommon, NodeId, NodeLayout, NodeType, Shadow};
use gpui::Hsla;
use smallvec::{smallvec, SmallVec};

/// Defines the geometric form that a shape node renders
///
/// Each shape type determines how the node's bounds are interpreted
/// and how the fill, border, and other properties are applied during rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShapeType {
    /// A rectangular shape that fills the entire node bounds
    Rectangle,
    /// An elliptical shape inscribed within the node bounds
    /// When width equals height, renders as a circle
    Ellipse,
    /// A polygonal shape with a specified number of sides
    /// Inscribed within the node bounds and oriented with a vertex at the top
    Polygon { sides: u8 },
    /// A star shape with specified number of points
    /// The inner radius is calculated as a ratio of the outer radius
    Star { points: u8, inner_radius_ratio: f32 },
}

impl ShapeType {
    /// Create a regular polygon with the specified number of sides
    pub fn polygon(sides: u8) -> Self {
        ShapeType::Polygon {
            sides: sides.max(3), // Ensure at least 3 sides
        }
    }

    /// Create a star with the specified number of points
    pub fn star(points: u8) -> Self {
        ShapeType::Star {
            points: points.max(3), // Ensure at least 3 points
            inner_radius_ratio: 0.5,
        }
    }

    /// Create a triangle (3-sided polygon)
    pub fn triangle() -> Self {
        Self::polygon(3)
    }

    /// Create a pentagon (5-sided polygon)
    pub fn pentagon() -> Self {
        Self::polygon(5)
    }

    /// Create a hexagon (6-sided polygon)
    pub fn hexagon() -> Self {
        Self::polygon(6)
    }

    /// Check if this shape type supports corner radius
    pub fn supports_corner_radius(&self) -> bool {
        matches!(self, ShapeType::Rectangle)
    }
}

/// Concrete implementation of a shape visual element
///
/// ShapeNode represents a geometric primitive that can be rendered with configurable:
/// - Shape type (rectangle, ellipse, polygon, star)
/// - Position and dimensions via NodeLayout
/// - Fill color (optional)
/// - Border properties (color and width)
/// - Corner radius (for rectangles only)
/// - Shadows for depth effects
///
/// Unlike frame nodes, shapes are leaf nodes that cannot contain children.
/// They serve as the basic visual building blocks for creating graphics and UI elements.
#[derive(Debug, Clone)]
pub struct ShapeNode {
    pub id: NodeId,
    pub layout: NodeLayout,
    pub shape_type: ShapeType,
    pub fill: Option<Hsla>,
    pub border_color: Option<Hsla>,
    pub border_width: f32,
    pub corner_radius: f32,
    pub shadows: SmallVec<[Shadow; 1]>,
}

impl ShapeNode {
    /// Create a new shape node with default rectangle type
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            layout: NodeLayout::new(0.0, 0.0, 100.0, 100.0),
            shape_type: ShapeType::Rectangle,
            fill: Some(Hsla::white()),
            border_color: Some(Hsla::black()),
            border_width: 1.0,
            corner_radius: 0.0,
            shadows: smallvec![],
        }
    }

    /// Create a shape with specific type and dimensions
    pub fn with_type_and_rect(
        id: NodeId,
        shape_type: ShapeType,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Self {
        let mut node = Self::new(id);
        node.shape_type = shape_type;
        node.layout = NodeLayout::new(x, y, width, height);
        node
    }

    /// Create a rectangle shape with specific dimensions
    pub fn rectangle(id: NodeId, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::with_type_and_rect(id, ShapeType::Rectangle, x, y, width, height)
    }

    /// Create an ellipse shape with specific dimensions
    pub fn ellipse(id: NodeId, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::with_type_and_rect(id, ShapeType::Ellipse, x, y, width, height)
    }

    /// Create a circle shape (ellipse with equal width and height)
    pub fn circle(id: NodeId, x: f32, y: f32, radius: f32) -> Self {
        Self::ellipse(id, x, y, radius * 2.0, radius * 2.0)
    }

    /// Set the shape type
    pub fn set_shape_type(&mut self, shape_type: ShapeType) {
        self.shape_type = shape_type;

        // Clear corner radius if the new shape doesn't support it
        if !shape_type.supports_corner_radius() {
            self.corner_radius = 0.0;
        }
    }

    /// Get the shape type
    pub fn shape_type(&self) -> ShapeType {
        self.shape_type
    }
}

impl NodeCommon for ShapeNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Shape
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
        // Only set corner radius if the shape supports it
        if self.shape_type.supports_corner_radius() {
            self.corner_radius = radius;
        }
    }

    fn shadows(&self) -> SmallVec<[Shadow; 1]> {
        self.shadows.clone()
    }

    fn set_shadows(&mut self, shadows: SmallVec<[Shadow; 1]>) {
        self.shadows = shadows;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Point;

    #[test]
    fn test_shape_node_creation() {
        let id = NodeId::new(1);
        let shape = ShapeNode::new(id);

        assert_eq!(shape.id(), id);
        assert_eq!(shape.node_type(), NodeType::Shape);
        assert_eq!(shape.shape_type(), ShapeType::Rectangle);
        assert_eq!(shape.corner_radius(), 0.0);
    }

    #[test]
    fn test_shape_types() {
        let id = NodeId::new(1);

        // Test rectangle
        let rect = ShapeNode::rectangle(id, 10.0, 10.0, 100.0, 50.0);
        assert_eq!(rect.shape_type(), ShapeType::Rectangle);
        assert_eq!(rect.layout().width, 100.0);
        assert_eq!(rect.layout().height, 50.0);

        // Test ellipse
        let ellipse = ShapeNode::ellipse(NodeId::new(2), 0.0, 0.0, 80.0, 60.0);
        assert_eq!(ellipse.shape_type(), ShapeType::Ellipse);

        // Test circle
        let circle = ShapeNode::circle(NodeId::new(3), 50.0, 50.0, 25.0);
        assert_eq!(circle.shape_type(), ShapeType::Ellipse);
        assert_eq!(circle.layout().width, 50.0);
        assert_eq!(circle.layout().height, 50.0);
    }

    #[test]
    fn test_polygon_creation() {
        let triangle = ShapeType::triangle();
        assert_eq!(triangle, ShapeType::Polygon { sides: 3 });

        let hexagon = ShapeType::hexagon();
        assert_eq!(hexagon, ShapeType::Polygon { sides: 6 });

        // Test minimum sides enforcement
        let invalid = ShapeType::polygon(2);
        assert_eq!(invalid, ShapeType::Polygon { sides: 3 });
    }

    #[test]
    fn test_star_creation() {
        let star = ShapeType::star(5);
        assert_eq!(
            star,
            ShapeType::Star {
                points: 5,
                inner_radius_ratio: 0.5
            }
        );

        // Test minimum points enforcement
        let invalid = ShapeType::star(2);
        assert_eq!(
            invalid,
            ShapeType::Star {
                points: 3,
                inner_radius_ratio: 0.5
            }
        );
    }

    #[test]
    fn test_corner_radius_support() {
        let id = NodeId::new(1);
        let mut shape = ShapeNode::new(id);

        // Rectangle supports corner radius
        shape.set_shape_type(ShapeType::Rectangle);
        shape.set_corner_radius(10.0);
        assert_eq!(shape.corner_radius(), 10.0);

        // Ellipse doesn't support corner radius
        shape.set_shape_type(ShapeType::Ellipse);
        assert_eq!(shape.corner_radius(), 0.0);

        // Try to set corner radius on ellipse - should be ignored
        shape.set_corner_radius(5.0);
        assert_eq!(shape.corner_radius(), 0.0);
    }

    #[test]
    fn test_contains_point() {
        let id = NodeId::new(1);
        let shape = ShapeNode::rectangle(id, 10.0, 10.0, 100.0, 100.0);

        // Test points inside and outside
        let point_inside = Point::new(50.0, 50.0);
        let point_outside = Point::new(200.0, 200.0);

        assert!(shape.contains_point(&point_inside));
        assert!(!shape.contains_point(&point_outside));
    }

    #[test]
    fn test_shape_modification() {
        let id = NodeId::new(1);
        let mut shape = ShapeNode::new(id);

        // Change shape type
        shape.set_shape_type(ShapeType::Ellipse);
        assert_eq!(shape.shape_type(), ShapeType::Ellipse);

        shape.set_shape_type(ShapeType::Polygon { sides: 6 });
        assert_eq!(shape.shape_type(), ShapeType::Polygon { sides: 6 });
    }
}
