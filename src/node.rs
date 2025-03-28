#![allow(unused, dead_code)]

use gpui::{hsla, px, solid_background, Hsla, Point, Size};
use std::{
    any::Any,
    fmt::{Debug, Display},
};
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

/// Layout information passed to the canvas
/// to render a root node–One with no parent
/// that is painted directly on the canvas
///
/// A root node's position uses canvas coordinates,
/// meaning it is relative to the center of the canvas (0,0).
#[derive(Debug, Clone)]
pub struct RootNodeLayout {
    id: NodeId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    background_color: Hsla,
    border_color: Option<Hsla>,
    border_width: f32,
    border_radius: f32,
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

// This is the main trait object that allows downcasting
trait CanvasNodeObject: 'static {
    // These are our escape hatches for downcasting, similar to ElementObject in gpui
    fn inner_node(&mut self) -> &mut dyn Any;
    fn inner_node_ref(&self) -> &dyn Any;

    // Forward the common operations that don't require knowing the concrete type
    fn common(&self) -> &NodeCommon;
    fn common_mut(&mut self) -> &mut NodeCommon;
    fn id(&self) -> NodeId;
    fn node_type(&self) -> NodeType;
    fn should_render(&self) -> bool;
}

/// Base trait for all canvas nodes defining common functionality
pub trait CanvasNode: Debug + 'static + Sized {
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
}

// Adapter to convert any CanvasNode to a CanvasNodeObject
struct NodeAdapter<T: CanvasNode> {
    node: T,
}

impl<T: CanvasNode> NodeAdapter<T> {
    fn new(node: T) -> Self {
        Self { node }
    }
}

impl<T: CanvasNode> CanvasNodeObject for NodeAdapter<T> {
    fn inner_node(&mut self) -> &mut dyn Any {
        &mut self.node
    }

    fn inner_node_ref(&self) -> &dyn Any {
        &self.node
    }

    fn common(&self) -> &NodeCommon {
        self.node.common()
    }

    fn common_mut(&mut self) -> &mut NodeCommon {
        self.node.common_mut()
    }

    fn id(&self) -> NodeId {
        self.node.id()
    }

    fn node_type(&self) -> NodeType {
        self.node.node_type()
    }

    fn should_render(&self) -> bool {
        self.node.should_render()
    }
}

/// A wrapper around a boxed canvas node that allows for dynamic dispatch and downcasting
pub struct AnyNode(Box<dyn CanvasNodeObject>);

impl AnyNode {
    /// Create a new AnyNode from a CanvasNode
    pub fn new<T: CanvasNode>(node: T) -> Self {
        Self(Box::new(NodeAdapter::new(node)))
    }

    /// Attempt to downcast to a specific node type (immutable)
    pub fn downcast_ref<T: CanvasNode>(&self) -> Option<&T> {
        self.0.inner_node_ref().downcast_ref::<T>()
    }

    /// Attempt to downcast to a specific node type (mutable)
    pub fn downcast_mut<T: CanvasNode>(&mut self) -> Option<&mut T> {
        self.0.inner_node().downcast_mut::<T>()
    }

    /// Get a reference to the common properties
    pub fn common(&self) -> &NodeCommon {
        self.0.common()
    }

    /// Get a mutable reference to the common properties
    pub fn common_mut(&mut self) -> &mut NodeCommon {
        self.0.common_mut()
    }

    /// Get the node's ID
    pub fn id(&self) -> NodeId {
        self.0.id()
    }

    /// Get the node type
    pub fn node_type(&self) -> NodeType {
        self.0.node_type()
    }

    /// Determine if this node should be rendered
    pub fn should_render(&self) -> bool {
        self.0.should_render()
    }
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
#[derive(Debug)]
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
}

impl ShapeNode for RectangleNode {}

// Implementing Element trait for RectangleNode
impl gpui::Element for RectangleNode {
    // For RectangleNode, we just need to store the layout ID
    type RequestLayoutState = gpui::LayoutId;

    // For prepaint, we store the hitbox ID for interaction
    type PrepaintState = Option<gpui::Hitbox>;

    fn id(&self) -> Option<gpui::ElementId> {
        // Use the node's ID as the element ID - use &str instead of String
        let id_str = format!("node-{}", self.common.id.0);
        Some(gpui::ElementId::Name(id_str.into()))
    }

    fn request_layout(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        // Create a GPUI style from our node's properties
        let mut style = gpui::Style::default();

        // Set position type (relative/absolute)
        style.position = self.common.style.position;

        // Convert taffy dimensions to GPUI dimensions for width/height
        if let Dimension::Length(width) = self.common.style.size.width {
            style.size.width = px(width).into();
        }

        if let Dimension::Length(height) = self.common.style.size.height {
            style.size.height = px(height).into();
        }

        // Set fill color from our node properties
        if let Some(fill) = self.common.fill {
            style.background = Some(solid_background(fill).into());
        }

        // Set border properties
        if self.common.border_width > 0.0 {
            style.border_widths = gpui::Edges {
                top: gpui::Pixels(self.common.border_width).into(),
                right: gpui::Pixels(self.common.border_width).into(),
                bottom: gpui::Pixels(self.common.border_width).into(),
                left: gpui::Pixels(self.common.border_width).into(),
            };

            if let Some(border_color) = self.common.border_color {
                style.border_color = Some(border_color);
            }
        }

        // Set corner radius
        if self.common.corner_radius > 0.0 {
            style.corner_radii = gpui::Corners::all(px(self.common.corner_radius).into());
        }

        // Set opacity
        style.opacity = Some(self.common.opacity);

        // Request layout from window
        let layout_id = window.request_layout(style, [], cx);

        (layout_id, layout_id)
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _layout_id: &mut Self::RequestLayoutState,
        window: &mut gpui::Window,
        _cx: &mut gpui::App,
    ) -> Self::PrepaintState {
        // Update our node's layout with the computed bounds from GPUI
        self.common.layout = Some(taffy::prelude::Layout {
            order: 0,
            size: taffy::prelude::Size {
                width: bounds.size.width.0,
                height: bounds.size.height.0,
            },
            location: taffy::Point {
                x: bounds.origin.x.0,
                y: bounds.origin.y.0,
            },
            border: taffy::prelude::Rect {
                left: self.common.border_width,
                right: self.common.border_width,
                top: self.common.border_width,
                bottom: self.common.border_width,
            },
            padding: taffy::prelude::Rect::zero(),
            scrollbar_size: taffy::prelude::Size::zero(),
            ..Default::default()
        });

        // Create a hitbox for this node so it can receive mouse events
        if self.common.visible {
            let hitbox = window.insert_hitbox(bounds, false);
            Some(hitbox)
        } else {
            None
        }
    }

    fn paint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _layout_id: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        window: &mut gpui::Window,
        _cx: &mut gpui::App,
    ) {
        if !self.common.visible {
            return;
        }

        // just skip opacity for now
        // window.with_opacity(self.common.opacity, |window| {
        // Paint background/fill
        if let Some(fill) = self.common.fill {
            // Draw the fill with correct parameters
            window.paint_quad(gpui::fill(bounds, fill));
        }

        // Paint border
        if self.common.border_width > 0.0 && self.common.border_color.is_some() {
            let border_color = self.common.border_color.unwrap();
            // Draw the outline with correct parameters
            window.paint_quad(gpui::outline(bounds, border_color));
        }

        // TODO: Implement selection logic
        let is_selected = false;

        // Draw selection indicator if node is selected
        if let Some(hitbox) = hitbox {
            if is_selected {
                // Create a slightly larger bounds manually instead of using inflate
                let selection_bounds = gpui::Bounds {
                    origin: gpui::Point::new(
                        bounds.origin.x - gpui::Pixels(2.0),
                        bounds.origin.y - gpui::Pixels(2.0),
                    ),
                    size: gpui::Size::new(
                        bounds.size.width + gpui::Pixels(4.0),
                        bounds.size.height + gpui::Pixels(4.0),
                    ),
                };
                window.paint_quad(gpui::outline(selection_bounds, gpui::Hsla::blue()));
            }
        };
        // });
    }
}

impl gpui::IntoElement for RectangleNode {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
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
    fn test_any_node() {
        let id = NodeId::new(1);
        let rect = RectangleNode::new(id);
        let mut any_node = AnyNode::new(rect);

        assert_eq!(any_node.id(), id);
        assert_eq!(any_node.node_type(), NodeType::Rectangle);

        // Test downcasting
        let rect_mut = any_node.downcast_mut::<RectangleNode>();
        assert!(rect_mut.is_some());
        if let Some(rect) = rect_mut {
            rect.common_mut().set_corner_radius(5.0);
        }

        assert_eq!(any_node.common().corner_radius, 5.0);
    }
}
