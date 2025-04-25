use crate::canvas::CanvasId;
use crate::geometry::{LocalPoint, LocalToWorld, WorldPoint, space};
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
    /// The center point of the canvas, used for coordinate transformation
    canvas_center: Option<(gpui::Pixels, gpui::Pixels)>,
    /// The transformation from local to world coordinates
    local_to_world: Option<LocalToWorld>,
}

impl SceneGraph {
    /// Create a new scene graph for a canvas
    pub fn new(canvas_id: CanvasId) -> Self {
        Self {
            canvas_id,
            nodes: HashMap::new(),
            canvas_center: None,
            local_to_world: None,
        }
    }
    
    /// Set the canvas center point which defines the transform between local and world space
    pub fn set_canvas_center(&mut self, center_x: gpui::Pixels, center_y: gpui::Pixels) {
        self.canvas_center = Some((center_x, center_y));
        
        // Create a transform that converts from local coordinates to world (screen) coordinates
        // 1. Translate to center the origin (0,0) at the canvas center
        // 2. Invert Y-axis because in local space Y increases upward, in screen space Y increases downward
        let translation = glam::Mat4::from_translation(glam::Vec3::new(center_x.0, center_y.0, 0.0));
        let y_inversion = glam::Mat4::from_scale(glam::Vec3::new(1.0, -1.0, 1.0));
        
        self.local_to_world = Some(LocalToWorld::new(translation * y_inversion));
    }
    
    /// Get the local-to-world transform for this scene graph
    pub fn local_to_world_transform(&self) -> Option<&LocalToWorld> {
        self.local_to_world.as_ref()
    }
    
    /// Calculate the world bounds for a rectangular node at the given local position with dimensions
    pub fn calculate_world_bounds(
        &self,
        local_position: LocalPoint,
        width: f32,
        height: f32,
    ) -> Option<gpui::Bounds<gpui::Pixels>> {
        let transform = self.local_to_world_transform()?;
        
        // Convert the local center position to world space
        let world_center = local_position.to_world(transform);
        
        // Create the world bounds centered at the world position
        Some(gpui::Bounds {
            origin: gpui::Point::new(
                gpui::px(world_center.x() - width / 2.0),
                gpui::px(world_center.y() - height / 2.0),
            ),
            size: gpui::Size::new(gpui::px(width), gpui::px(height)),
        })
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
        // Calculate the center of the canvas
        let center_x = world_bounds.origin.x + world_bounds.size.width / 2.0;
        let center_y = world_bounds.origin.y + world_bounds.size.height / 2.0;
        
        // Create the local-to-world transform directly without modifying self
        let translation = glam::Mat4::from_translation(glam::Vec3::new(center_x.0, center_y.0, 0.0));
        let y_inversion = glam::Mat4::from_scale(glam::Vec3::new(1.0, -1.0, 1.0));
        let transform = LocalToWorld::new(translation * y_inversion);
        
        window.paint_layer(world_bounds, |window| {
            // Paint each node
            for node in self.nodes() {
                self.paint_node_with_transform(node, &transform, window);
            }

            // Request next frame for animations (if needed later)
            window.request_animation_frame();
        });
    }
    
    /// Paint a single node using the provided transform
    fn paint_node_with_transform(&self, node: &Node, transform: &LocalToWorld, window: &mut gpui::Window) {
        match node.shape {
            ShapeType::Rectangle { width, height } => {
                // Convert the local center position to world space
                let world_center = node.position.to_world(transform);
                
                // Create the world bounds centered at the world position
                let rect_bounds = gpui::Bounds {
                    origin: gpui::Point::new(
                        gpui::px(world_center.x() - width / 2.0),
                        gpui::px(world_center.y() - height / 2.0),
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
