use gpui::{Hsla, Point, Size};
use std::fmt::{Debug, Display};
use taffy::prelude::*;

/// A unique identifier for a canvas node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl NodeId {
    pub fn new(id: usize) -> Self {
        NodeId(id)
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node-{}", self.0)
    }
}

/// Types of nodes that can be rendered on the canvas
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    /// A container for grouping other nodes
    Frame,
    /// A rectangle shape
    Rectangle,
    /// A circle/ellipse shape
    Circle,
    /// A line connecting two points
    Line,
    /// A text element
    Text,
    /// An image element
    Image,
    /// A vector path
    Path,
}

/// Common properties shared by all canvas nodes
#[derive(Debug, Clone)]
pub struct NodeCommon {
    /// Unique identifier for this node
    pub id: NodeId,

    /// Human-readable name for this node
    pub name: String,

    /// Type of node
    pub node_type: NodeType,

    /// Parent node (if any)
    pub parent: Option<NodeId>,

    /// Children of this node (if any)
    pub children: Vec<NodeId>,

    /// The layout style for this node (ties into taffy)
    pub style: Style,

    /// The computed layout (filled in during layout calculation)
    pub layout: Option<Layout>,

    /// Fill color for the node
    pub fill: Option<Hsla>,

    /// Border color for the node
    pub border_color: Option<Hsla>,

    /// Border width for the node
    pub border_width: f32,

    /// Corner radius for rounded elements
    pub corner_radius: f32,

    /// Whether this node is visible
    pub visible: bool,

    /// Transform properties (rotation in degrees)
    pub rotation: f32,

    /// Node opacity (0.0 to 1.0)
    pub opacity: f32,
}

impl NodeCommon {
    /// Create a new NodeCommon with default values
    pub fn new(id: NodeId, node_type: NodeType) -> Self {
        let name = match node_type {
            NodeType::Frame => format!("Frame {}", id.0),
            NodeType::Rectangle => format!("Rectangle {}", id.0),
            NodeType::Circle => format!("Circle {}", id.0),
            NodeType::Line => format!("Line {}", id.0),
            NodeType::Text => format!("Text {}", id.0),
            NodeType::Image => format!("Image {}", id.0),
            NodeType::Path => format!("Path {}", id.0),
        };

        NodeCommon {
            id,
            name,
            node_type,
            parent: None,
            children: Vec::new(),
            style: Style {
                display: taffy::style::Display::Flex,
                position: Position::Relative,
                size: taffy::prelude::Size {
                    width: Dimension::Length(100.0),
                    height: Dimension::Length(100.0),
                },
                ..Style::default()
            },
            layout: None,
            fill: Some(Hsla::white()),
            border_color: Some(Hsla::black()),
            border_width: 1.0,
            corner_radius: 0.0,
            visible: true,
            rotation: 0.0,
            opacity: 1.0,
        }
    }

    /// Get the computed bounds of this node
    pub fn bounds(&self) -> Option<gpui::Bounds<f32>> {
        self.layout.map(|layout| gpui::Bounds {
            origin: Point::new(layout.location.x, layout.location.y),
            size: Size::new(layout.size.width, layout.size.height),
        })
    }

    /// Set the position of this node
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.style.inset.left = LengthPercentageAuto::Length(x);
        self.style.inset.top = LengthPercentageAuto::Length(y);
    }

    /// Set the size of this node
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.style.size.width = Dimension::Length(width);
        self.style.size.height = Dimension::Length(height);
    }

    /// Set the fill color
    pub fn set_fill(&mut self, color: Option<Hsla>) {
        self.fill = color;
    }

    /// Set the border properties
    pub fn set_border(&mut self, color: Option<Hsla>, width: f32) {
        self.border_color = color;
        self.border_width = width;

        // Update taffy style
        self.style.border.left = LengthPercentage::Length(width);
        self.style.border.right = LengthPercentage::Length(width);
        self.style.border.top = LengthPercentage::Length(width);
        self.style.border.bottom = LengthPercentage::Length(width);
    }

    /// Set corner radius for rounded elements
    pub fn set_corner_radius(&mut self, radius: f32) {
        self.corner_radius = radius;
    }

    /// Add a child to this node
    pub fn add_child(&mut self, child_id: NodeId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    /// Remove a child from this node
    pub fn remove_child(&mut self, child_id: NodeId) {
        self.children.retain(|id| *id != child_id);
    }
}

/// Base trait for all canvas nodes defining common functionality
pub trait CanvasNode: Debug {
    /// Get the common properties of this node
    fn common(&self) -> &NodeCommon;

    /// Get mutable access to common properties
    fn common_mut(&mut self) -> &mut NodeCommon;

    /// Get the node's ID
    fn id(&self) -> NodeId {
        self.common().id
    }

    /// Get the node type
    fn node_type(&self) -> NodeType {
        self.common().node_type.clone()
    }

    /// Determine if this node should be rendered
    fn should_render(&self) -> bool {
        self.common().visible
    }

    /// Clone the node
    fn clone_node(&self) -> Box<dyn CanvasNode>;
}

/// Extension trait for shape-like elements that have bounded areas
pub trait ShapeNode: CanvasNode {
    /// Check if a point is inside this node
    fn contains_point(&self, point: &Point<f32>) -> bool {
        if let Some(bounds) = self.common().bounds() {
            bounds.contains(point)
        } else {
            false
        }
    }

    /// Get the bounding box of this shape
    fn bounding_box(&self) -> Option<gpui::Bounds<f32>> {
        self.common().bounds()
    }
}

/// Extension trait for path-like elements that follow a trajectory
pub trait PathNode: CanvasNode {
    /// Check if a point is close to this path
    fn point_near_path(&self, point: &Point<f32>, tolerance: f32) -> bool;

    /// Get the path points that define this path
    fn path_points(&self) -> Vec<Point<f32>>;

    /// Get the bounding box that contains the entire path
    fn path_bounds(&self) -> Option<gpui::Bounds<f32>>;
}

/// A rectangle node that can be rendered on the canvas
#[derive(Debug, Clone)]
pub struct RectangleNode {
    pub common: NodeCommon,
}

impl RectangleNode {
    pub fn new(id: NodeId) -> Self {
        Self {
            common: NodeCommon::new(id, NodeType::Rectangle),
        }
    }

    /// Create a rectangle with specific dimensions and position
    pub fn with_rect(id: NodeId, x: f32, y: f32, width: f32, height: f32) -> Self {
        let mut node = Self::new(id);
        node.common.set_position(x, y);
        node.common.set_size(width, height);
        node
    }
}

impl CanvasNode for RectangleNode {
    fn common(&self) -> &NodeCommon {
        &self.common
    }

    fn common_mut(&mut self) -> &mut NodeCommon {
        &mut self.common
    }

    fn clone_node(&self) -> Box<dyn CanvasNode> {
        Box::new(self.clone())
    }
}

impl ShapeNode for RectangleNode {}

/// A circle/ellipse node that can be rendered on the canvas
#[derive(Debug, Clone)]
pub struct CircleNode {
    pub common: NodeCommon,
}

impl CircleNode {
    pub fn new(id: NodeId) -> Self {
        let mut node = Self {
            common: NodeCommon::new(id, NodeType::Circle),
        };
        node.common.corner_radius = f32::MAX; // Make it fully rounded
        node
    }

    /// Create a circle with specific center point and radius
    pub fn with_circle(id: NodeId, center_x: f32, center_y: f32, radius: f32) -> Self {
        let mut node = Self::new(id);
        node.common
            .set_position(center_x - radius, center_y - radius);
        node.common.set_size(radius * 2.0, radius * 2.0);
        node
    }

    /// Create an ellipse with specific center point and radii
    pub fn with_ellipse(
        id: NodeId,
        center_x: f32,
        center_y: f32,
        radius_x: f32,
        radius_y: f32,
    ) -> Self {
        let mut node = Self::new(id);
        node.common
            .set_position(center_x - radius_x, center_y - radius_y);
        node.common.set_size(radius_x * 2.0, radius_y * 2.0);
        node
    }
}

impl CanvasNode for CircleNode {
    fn common(&self) -> &NodeCommon {
        &self.common
    }

    fn common_mut(&mut self) -> &mut NodeCommon {
        &mut self.common
    }

    fn clone_node(&self) -> Box<dyn CanvasNode> {
        Box::new(self.clone())
    }
}

impl ShapeNode for CircleNode {
    fn contains_point(&self, point: &Point<f32>) -> bool {
        if let Some(bounds) = self.common().bounds() {
            // For a circle, we need to check if the point is within the radius
            let center_x = bounds.origin.x + bounds.size.width / 2.0;
            let center_y = bounds.origin.y + bounds.size.height / 2.0;
            let radius_x = bounds.size.width / 2.0;
            let radius_y = bounds.size.height / 2.0;

            let dx = (point.x - center_x) / radius_x;
            let dy = (point.y - center_y) / radius_y;

            return dx * dx + dy * dy <= 1.0;
        }
        false
    }
}

/// A frame node that acts as a container for other nodes
#[derive(Debug, Clone)]
pub struct FrameNode {
    pub common: NodeCommon,
}

impl FrameNode {
    pub fn new(id: NodeId) -> Self {
        Self {
            common: NodeCommon::new(id, NodeType::Frame),
        }
    }

    /// Create a frame with specific dimensions and position
    pub fn with_rect(id: NodeId, x: f32, y: f32, width: f32, height: f32) -> Self {
        let mut node = Self::new(id);
        node.common.set_position(x, y);
        node.common.set_size(width, height);
        node
    }
}

impl CanvasNode for FrameNode {
    fn common(&self) -> &NodeCommon {
        &self.common
    }

    fn common_mut(&mut self) -> &mut NodeCommon {
        &mut self.common
    }

    fn clone_node(&self) -> Box<dyn CanvasNode> {
        Box::new(self.clone())
    }
}

impl ShapeNode for FrameNode {}

/// A line node that connects two points
#[derive(Debug, Clone)]
pub struct LineNode {
    pub common: NodeCommon,
    pub start: Point<f32>,
    pub end: Point<f32>,
}

impl LineNode {
    pub fn new(id: NodeId, start: Point<f32>, end: Point<f32>) -> Self {
        let mut common = NodeCommon::new(id, NodeType::Line);
        common.fill = None; // Lines don't have fill

        // Calculate the position and size for layout
        let min_x = start.x.min(end.x);
        let min_y = start.y.min(end.y);
        let width = (start.x - end.x).abs();
        let height = (start.y - end.y).abs();

        common.set_position(min_x, min_y);
        common.set_size(width.max(1.0), height.max(1.0)); // Ensure non-zero size

        Self { common, start, end }
    }
}

impl CanvasNode for LineNode {
    fn common(&self) -> &NodeCommon {
        &self.common
    }

    fn common_mut(&mut self) -> &mut NodeCommon {
        &mut self.common
    }

    fn clone_node(&self) -> Box<dyn CanvasNode> {
        Box::new(self.clone())
    }
}

impl PathNode for LineNode {
    fn point_near_path(&self, point: &Point<f32>, tolerance: f32) -> bool {
        // For lines, check if point is close to the line segment
        let dx = self.end.x - self.start.x;
        let dy = self.end.y - self.start.y;
        let length_squared = dx * dx + dy * dy;

        if length_squared == 0.0 {
            // Start and end are the same point
            let distance_squared =
                (point.x - self.start.x).powi(2) + (point.y - self.start.y).powi(2);
            return distance_squared <= tolerance.powi(2);
        }

        // Calculate projection of point onto line
        let t = ((point.x - self.start.x) * dx + (point.y - self.start.y) * dy) / length_squared;

        if t < 0.0 {
            // Point is beyond the start point
            let distance_squared =
                (point.x - self.start.x).powi(2) + (point.y - self.start.y).powi(2);
            return distance_squared <= tolerance.powi(2);
        } else if t > 1.0 {
            // Point is beyond the end point
            let distance_squared = (point.x - self.end.x).powi(2) + (point.y - self.end.y).powi(2);
            return distance_squared <= tolerance.powi(2);
        } else {
            // Point projects onto the line segment
            let projection_x = self.start.x + t * dx;
            let projection_y = self.start.y + t * dy;
            let distance_squared =
                (point.x - projection_x).powi(2) + (point.y - projection_y).powi(2);
            return distance_squared <= tolerance.powi(2);
        }
    }

    fn path_points(&self) -> Vec<Point<f32>> {
        vec![self.start, self.end]
    }

    fn path_bounds(&self) -> Option<gpui::Bounds<f32>> {
        self.common().bounds()
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

    /// Create a new circle node
    pub fn create_circle(&mut self) -> CircleNode {
        CircleNode::new(self.next_id())
    }

    /// Create a new frame node
    pub fn create_frame(&mut self) -> FrameNode {
        FrameNode::new(self.next_id())
    }

    /// Create a new line node
    pub fn create_line(&mut self, start: Point<f32>, end: Point<f32>) -> LineNode {
        LineNode::new(self.next_id(), start, end)
    }
}

/// A registry that manages all nodes in the canvas
#[derive(Debug, Default)]
pub struct NodeRegistry {
    nodes: Vec<Box<dyn CanvasNode>>,
    factory: NodeFactory,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            factory: NodeFactory::new(),
        }
    }

    /// Get the node factory
    pub fn factory(&mut self) -> &mut NodeFactory {
        &mut self.factory
    }

    /// Add a node to the registry
    pub fn add_node(&mut self, node: Box<dyn CanvasNode>) {
        self.nodes.push(node);
    }

    /// Get a reference to a node by ID
    pub fn get_node(&self, id: NodeId) -> Option<&dyn CanvasNode> {
        self.nodes
            .iter()
            .find(|node| node.id() == id)
            .map(|node| node.as_ref())
    }

    /// Get a mutable reference to a node by ID
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Box<dyn CanvasNode>> {
        self.nodes.iter_mut().find(|node| node.id() == id)
    }

    /// Remove a node from the registry
    pub fn remove_node(&mut self, id: NodeId) -> Option<Box<dyn CanvasNode>> {
        if let Some(index) = self.nodes.iter().position(|node| node.id() == id) {
            Some(self.nodes.remove(index))
        } else {
            None
        }
    }

    /// Get all nodes in the registry
    pub fn get_all_nodes(&self) -> &[Box<dyn CanvasNode>] {
        &self.nodes
    }

    /// Create a node hierarchy - sets up parent-child relationships
    pub fn create_hierarchy(&mut self, parent_id: NodeId, child_id: NodeId) {
        // Update the parent node to add the child
        if let Some(parent) = self.get_node_mut(parent_id) {
            parent.common_mut().add_child(child_id);
        }

        // Update the child node to set its parent
        if let Some(child) = self.get_node_mut(child_id) {
            child.common_mut().parent = Some(parent_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Point;

    #[test]
    fn test_node_common_creation() {
        let id = NodeId::new(1);
        let node = NodeCommon::new(id, NodeType::Rectangle);

        assert_eq!(node.id, id);
        assert_eq!(node.name, "Rectangle 1");
        assert_eq!(node.children.len(), 0);
        assert_eq!(node.parent, None);
        assert!(node.visible);
    }

    #[test]
    fn test_rectangle_node() {
        let id = NodeId::new(2);
        let rect = RectangleNode::new(id);

        assert_eq!(rect.node_type(), NodeType::Rectangle);
        assert_eq!(rect.common().id, id);
        assert_eq!(rect.common().corner_radius, 0.0);
    }

    #[test]
    fn test_circle_node() {
        let id = NodeId::new(3);
        let circle = CircleNode::new(id);

        assert_eq!(circle.node_type(), NodeType::Circle);
        assert_eq!(circle.common().id, id);
        assert_eq!(circle.common().corner_radius, f32::MAX);
    }

    #[test]
    fn test_frame_node() {
        let id = NodeId::new(4);
        let frame = FrameNode::new(id);

        assert_eq!(frame.node_type(), NodeType::Frame);
        assert_eq!(frame.common().id, id);
        assert_eq!(frame.common().name, "Frame 4");
    }

    #[test]
    fn test_line_node() {
        let id = NodeId::new(5);
        let start = Point::new(10.0, 10.0);
        let end = Point::new(100.0, 100.0);
        let line = LineNode::new(id, start, end);

        assert_eq!(line.node_type(), NodeType::Line);
        assert_eq!(line.common().id, id);
        assert_eq!(line.start, start);
        assert_eq!(line.end, end);

        // Test point containment with a point on the line
        let point_on_line = Point::new(50.0, 50.0);
        assert!(line.point_near_path(&point_on_line, 2.0));

        // Test point containment with a point not on the line
        let point_not_on_line = Point::new(50.0, 60.0);
        assert!(!line.point_near_path(&point_not_on_line, 2.0));
    }

    #[test]
    fn test_factory() {
        let mut factory = NodeFactory::new();

        let rect = factory.create_rectangle();
        let circle = factory.create_circle();
        let frame = factory.create_frame();
        let line = factory.create_line(Point::new(0.0, 0.0), Point::new(10.0, 10.0));

        // Check that IDs are sequential and unique
        assert_eq!(rect.id().0, 1);
        assert_eq!(circle.id().0, 2);
        assert_eq!(frame.id().0, 3);
        assert_eq!(line.id().0, 4);

        // Check correct node types
        assert_eq!(rect.node_type(), NodeType::Rectangle);
        assert_eq!(circle.node_type(), NodeType::Circle);
        assert_eq!(frame.node_type(), NodeType::Frame);
        assert_eq!(line.node_type(), NodeType::Line);
    }

    #[test]
    fn test_shape_node_trait() {
        let id = NodeId::new(1);
        let rect = RectangleNode::new(id);

        // Test ShapeNode trait methods
        let point_inside = Point::new(50.0, 50.0);

        // Without layout information, contains_point should return false
        assert!(!<RectangleNode as ShapeNode>::contains_point(
            &rect,
            &point_inside
        ));

        // Similarly for bounding_box
        assert!(rect.bounding_box().is_none());
    }

    #[test]
    fn test_path_node_trait() {
        let id = NodeId::new(1);
        let start = Point::new(0.0, 0.0);
        let end = Point::new(100.0, 100.0);
        let line = LineNode::new(id, start, end);

        // Test PathNode trait methods
        let path_points = line.path_points();
        assert_eq!(path_points.len(), 2);
        assert_eq!(path_points[0], start);
        assert_eq!(path_points[1], end);

        // Test point near path
        let point_on_path = Point::new(50.0, 50.0);
        assert!(line.point_near_path(&point_on_path, 2.0));
    }
}
