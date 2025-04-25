use crate::canvas::CanvasId;
use crate::geometry::{space, LocalPoint, Transform, WorldPoint};
use gpui::{App, Window};
use std::collections::HashMap;

/// Unique identifier for a node in the scene graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    fn next() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        NodeId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Basic shape types supported in the scene graph
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShapeType {
    /// A rectangular shape
    Rectangle { width: f32, height: f32 },
}

/// A node in the scene graph
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique identifier for this node
    pub id: NodeId,
    /// The shape represented by this node
    pub shape: ShapeType,
    /// Position in local space
    pub position: LocalPoint,
    /// Fill color
    pub fill_color: gpui::Hsla,
    /// Border color
    pub border_color: gpui::Hsla,
}

impl Node {
    /// Create a new rectangular node
    pub fn rectangle(position: LocalPoint, width: f32, height: f32) -> Self {
        Self {
            id: NodeId::next(),
            shape: ShapeType::Rectangle { width, height },
            position,
            fill_color: gpui::hsla(0.6, 0.6, 0.6, 1.0),
            border_color: gpui::hsla(0.0, 0.0, 0.0, 1.0),
        }
    }

    /// Create a square node with equal width and height
    pub fn square(position: LocalPoint, size: f32) -> Self {
        Self::rectangle(position, size, size)
    }

    /// Set the fill color
    pub fn with_fill(mut self, color: gpui::Hsla) -> Self {
        self.fill_color = color;
        self
    }

    /// Set the border color
    pub fn with_border(mut self, color: gpui::Hsla) -> Self {
        self.border_color = color;
        self
    }
}

/// The scene graph that contains and manages nodes
pub struct SceneGraph {
    /// The canvas this scene graph belongs to
    canvas_id: CanvasId,
    /// All nodes in the scene
    nodes: HashMap<NodeId, Node>,
}

impl SceneGraph {
    /// Create a new scene graph for a canvas
    pub fn new(canvas_id: CanvasId) -> Self {
        Self {
            canvas_id,
            nodes: HashMap::new(),
        }
    }

    /// Add a node to the scene graph
    pub fn add_node(&mut self, node: Node) -> NodeId {
        let id = node.id;
        self.nodes.insert(id, node);
        id
    }

    /// Remove a node from the scene graph
    pub fn remove_node(&mut self, id: NodeId) -> Option<Node> {
        self.nodes.remove(&id)
    }

    /// Get a reference to a node by ID
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Get a mutable reference to a node by ID
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    /// Get all nodes in the scene graph
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    /// Set up a scene with four squares in the quadrants
    pub fn setup_demo_scene(&mut self) {
        // Clear any existing nodes
        self.nodes.clear();

        // Add four 100px squares, one in each quadrant
        // Top-left quadrant (negative x, positive y)
        self.add_node(
            Node::square(LocalPoint::new_2d(-150.0, 50.0), 100.0)
                .with_fill(gpui::hsla(0.0, 0.7, 0.7, 1.0)),
        ); // Red

        // Top-right quadrant (positive x, positive y)
        self.add_node(
            Node::square(LocalPoint::new_2d(50.0, 50.0), 100.0)
                .with_fill(gpui::hsla(0.3, 0.7, 0.7, 1.0)),
        ); // Green

        // Bottom-left quadrant (negative x, negative y)
        self.add_node(
            Node::square(LocalPoint::new_2d(-150.0, -150.0), 100.0)
                .with_fill(gpui::hsla(0.6, 0.7, 0.7, 1.0)),
        ); // Blue

        // Bottom-right quadrant (positive x, negative y)
        self.add_node(
            Node::square(LocalPoint::new_2d(50.0, -150.0), 100.0)
                .with_fill(gpui::hsla(0.8, 0.7, 0.7, 1.0)),
        ); // Purple
    }

    /// Paint all nodes in the scene
    pub fn paint_nodes(
        &self,
        window: &mut Window,
        cx: &mut App,
        world_bounds: gpui::Bounds<gpui::Pixels>,
    ) {
        window.paint_layer(world_bounds, |window| {
            // Calculate the center of the canvas as our origin
            let center_x = world_bounds.origin.x + world_bounds.size.width / 2.0;
            let center_y = world_bounds.origin.y + world_bounds.size.height / 2.0;

            // Paint each node
            for node in self.nodes() {
                self.paint_node(node, center_x, center_y, window);
            }

            // Request next frame for animations (if needed later)
            window.request_animation_frame();
        });
    }

    /// Paint a single node
    fn paint_node(
        &self,
        node: &Node,
        center_x: gpui::Pixels,
        center_y: gpui::Pixels,
        window: &mut gpui::Window,
    ) {
        // Convert local position to world space (centered on canvas)
        let world_x = center_x + gpui::px(node.position.x());
        let world_y = center_y - gpui::px(node.position.y()); // Invert y-axis for screen space

        match node.shape {
            ShapeType::Rectangle { width, height } => {
                // Calculate rectangle bounds
                let rect_bounds = gpui::Bounds {
                    origin: gpui::Point::new(
                        world_x - gpui::px(width / 2.0),
                        world_y - gpui::px(height / 2.0),
                    ),
                    size: gpui::Size::new(gpui::px(width), gpui::px(height)),
                };

                // Paint fill
                window.paint_quad(gpui::fill(rect_bounds, node.fill_color));

                // Paint border
                window.paint_quad(gpui::outline(
                    rect_bounds,
                    node.border_color,
                    gpui::BorderStyle::Solid,
                ));
            }
        }
    }
}
