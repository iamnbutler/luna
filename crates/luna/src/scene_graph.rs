use gpui::{
    div, hsla, px, quad, App, AppContext, Bounds, Context, Element, ElementId, Entity,
    GlobalElementId, Hitbox, IntoElement, LayoutId, Pixels, Point, Position, Render, Size, Style,
    Window,
};
#[derive(Debug, Clone, Copy)]
pub struct Vector2D {
    x: f32,
    y: f32,
}

impl Vector2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn add(&self, other: &Vector2D) -> Vector2D {
        Vector2D::new(self.x + other.x, self.y + other.y)
    }

    pub fn scale(&self, sx: f32, sy: f32) -> Vector2D {
        Vector2D::new(self.x * sx, self.y * sy)
    }

    pub fn x(&self) -> f32 {
        self.x
    }
    pub fn y(&self) -> f32 {
        self.y
    }
}

use std::ops::Sub;

impl Sub<f32> for Vector2D {
    type Output = Vector2D;

    fn sub(self, rhs: f32) -> Self::Output {
        Vector2D::new(self.x - rhs, self.y - rhs)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WorldPosition(Vector2D);

impl WorldPosition {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vector2D::new(x, y))
    }

    pub fn x(&self) -> f32 {
        self.0.x
    }
    pub fn y(&self) -> f32 {
        self.0.y
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LocalPosition(Vector2D);

impl LocalPosition {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vector2D::new(x, y))
    }

    pub fn x(&self) -> f32 {
        self.0.x
    }
    pub fn y(&self) -> f32 {
        self.0.y
    }
}

#[derive(Debug, Clone)]
struct LocalTransform {
    position: LocalPosition,
    scale_x: f32,
    scale_y: f32,
    rotation: f32,
}

impl LocalTransform {
    fn to_world_transform(&self, parent: &LocalTransform) -> LocalTransform {
        LocalTransform {
            position: LocalPosition::new(
                parent.position.x() + self.position.x(),
                parent.position.y() + self.position.y(),
            ),
            scale_x: parent.scale_x * self.scale_x,
            scale_y: parent.scale_y * self.scale_y,
            rotation: parent.rotation + self.rotation,
        }
    }
}

#[derive(Debug, Clone)]
struct ElementStyle {
    width: f32,
    height: f32,
    corner_radius: f32,
}

impl ElementStyle {
    fn calculate_local_bounds(&self, transform: &LocalTransform) -> BoundingBox {
        BoundingBox {
            center_x: transform.position.x(),
            center_y: transform.position.y(),
            half_width: self.width * transform.scale_x * 0.5,
            half_height: self.height * transform.scale_y * 0.5,
        }
    }

    fn calculate_world_bounds(
        &self,
        local_transform: &LocalTransform,
        parent_transform: &LocalTransform,
    ) -> BoundingBox {
        let world_transform = local_transform.to_world_transform(parent_transform);
        self.calculate_local_bounds(&world_transform)
    }
}

#[derive(Debug, Clone)]
pub struct SceneNode {
    id: usize,
    transform: LocalTransform,
    element: Option<ElementStyle>,
    children: Vec<SceneNode>,
    clip_content: bool,
}

impl SceneNode {
    fn calculate_combined_bounds_with_parent(
        &self,
        parent_transform: &LocalTransform,
    ) -> Option<BoundingBox> {
        let world_transform = self.transform.to_world_transform(parent_transform);

        let mut bounds = Vec::new();

        if let Some(ref element) = self.element {
            bounds.push(element.calculate_local_bounds(&world_transform));
        }

        for child in &self.children {
            if let Some(child_bounds) =
                child.calculate_combined_bounds_with_parent(&world_transform)
            {
                if self.clip_content {
                    let parent_bounds = if let Some(ref element) = self.element {
                        element.calculate_local_bounds(&world_transform)
                    } else {
                        child_bounds
                    };

                    let min_x = parent_bounds.min_x();
                    let max_x = parent_bounds.max_x();
                    let min_y = parent_bounds.min_y();
                    let max_y = parent_bounds.max_y();

                    let child_min_x = child_bounds.min_x();
                    let child_max_x = child_bounds.max_x();
                    let child_min_y = child_bounds.min_y();
                    let child_max_y = child_bounds.max_y();

                    let intersect_min_x = child_min_x.max(min_x);
                    let intersect_max_x = child_max_x.min(max_x);
                    let intersect_min_y = child_min_y.max(min_y);
                    let intersect_max_y = child_max_y.min(max_y);

                    if intersect_max_x > intersect_min_x && intersect_max_y > intersect_min_y {
                        let clipped_bounds = BoundingBox {
                            center_x: (intersect_min_x + intersect_max_x) / 2.0,
                            center_y: (intersect_min_y + intersect_max_y) / 2.0,
                            half_width: (intersect_max_x - intersect_min_x) / 2.0,
                            half_height: (intersect_max_y - intersect_min_y) / 2.0,
                        };
                        bounds.push(clipped_bounds);
                    }
                    bounds.push(parent_bounds);
                } else {
                    bounds.push(child_bounds);
                }
            }
        }

        if bounds.is_empty() {
            return None;
        }

        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for bound in bounds {
            min_x = min_x.min(bound.min_x());
            max_x = max_x.max(bound.max_x());
            min_y = min_y.min(bound.min_y());
            max_y = max_y.max(bound.max_y());
        }

        Some(BoundingBox {
            center_x: (min_x + max_x) / 2.0,
            center_y: (min_y + max_y) / 2.0,
            half_width: (max_x - min_x) / 2.0,
            half_height: (max_y - min_y) / 2.0,
        })
    }

    fn calculate_combined_bounds(&self) -> Option<BoundingBox> {
        self.calculate_combined_bounds_with_parent(&LocalTransform {
            position: LocalPosition::new(0.0, 0.0),
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
        })
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.transform.scale_x = scale;
        self.transform.scale_y = scale;
        self
    }
}

/// A bounding box representation using center point and half-dimensions.
/// This representation is optimized for quad tree operations and intersection tests.
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    center_x: f32,
    center_y: f32,
    half_width: f32,
    half_height: f32,
}

impl BoundingBox {
    pub fn new(center_x: f32, center_y: f32, half_width: f32, half_height: f32) -> Self {
        Self {
            center_x,
            center_y,
            half_width,
            half_height,
        }
    }

    /// Creates a bounding box from minimum and maximum coordinates
    pub fn from_min_max(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            center_x: (min_x + max_x) / 2.0,
            center_y: (min_y + max_y) / 2.0,
            half_width: (max_x - min_x) / 2.0,
            half_height: (max_y - min_y) / 2.0,
        }
    }

    pub fn width(&self) -> f32 {
        self.half_width * 2.0
    }

    pub fn height(&self) -> f32 {
        self.half_height * 2.0
    }

    pub fn min_x(&self) -> f32 {
        self.center_x - self.half_width
    }

    pub fn max_x(&self) -> f32 {
        self.center_x + self.half_width
    }

    pub fn min_y(&self) -> f32 {
        self.center_y - self.half_height
    }

    pub fn max_y(&self) -> f32 {
        self.center_y + self.half_height
    }

    /// Returns true if this bounding box contains the given point
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.min_x() && x <= self.max_x() && y >= self.min_y() && y <= self.max_y()
    }

    /// Returns true if this bounding box intersects with another
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        !(other.min_x() > self.max_x()
            || other.max_x() < self.min_x()
            || other.min_y() > self.max_y()
            || other.max_y() < self.min_y())
    }
}

fn new_element_id() -> ElementId {
    ElementId::Uuid(uuid::Uuid::new_v4())
}

#[derive(Debug, Clone)]
pub struct QuadTree {
    id: ElementId,
    boundary: BoundingBox,
    capacity: usize,
    points: Vec<(f32, f32, usize)>,
    divided: bool,
    northeast: Option<Box<QuadTree>>,
    northwest: Option<Box<QuadTree>>,
    southeast: Option<Box<QuadTree>>,
    southwest: Option<Box<QuadTree>>,
}

impl QuadTree {
    fn collect_all_points(&self) -> Vec<(f32, f32, usize)> {
        let mut results = Vec::new();
        self.collect_all_points_into(&mut results);
        results
    }

    fn collect_all_points_into(&self, results: &mut Vec<(f32, f32, usize)>) {
        results.extend_from_slice(&self.points);

        if self.divided {
            if let Some(ref quad) = self.northeast {
                quad.collect_all_points_into(results);
            }
            if let Some(ref quad) = self.northwest {
                quad.collect_all_points_into(results);
            }
            if let Some(ref quad) = self.southeast {
                quad.collect_all_points_into(results);
            }
            if let Some(ref quad) = self.southwest {
                quad.collect_all_points_into(results);
            }
        }
    }

    fn clear_points(&mut self) {
        self.points.clear();

        if self.divided {
            if let Some(ref mut quad) = self.northeast {
                quad.clear_points();
            }
            if let Some(ref mut quad) = self.northwest {
                quad.clear_points();
            }
            if let Some(ref mut quad) = self.southeast {
                quad.clear_points();
            }
            if let Some(ref mut quad) = self.southwest {
                quad.clear_points();
            }
        }
    }

    fn update_bounds(&mut self, new_boundary: BoundingBox) {
        // Only rebuild if boundary changed significantly
        if (self.boundary.center_x - new_boundary.center_x).abs() > 1.0
            || (self.boundary.center_y - new_boundary.center_y).abs() > 1.0
            || (self.boundary.half_width - new_boundary.half_width).abs() > 1.0
            || (self.boundary.half_height - new_boundary.half_height).abs() > 1.0
        {
            // Save existing points
            let saved_points = self.collect_all_points();

            // Reset tree with new boundary
            self.boundary = new_boundary;
            self.points.clear();
            self.divided = false;
            self.northeast = None;
            self.northwest = None;
            self.southeast = None;
            self.southwest = None;

            // Reinsert points
            for (x, y, id) in saved_points {
                self.insert_point(x, y, id);
            }
        }
    }

    fn insert_with_bounds(&mut self, bounds: &BoundingBox, id: usize) -> bool {
        if !self.intersects(bounds) {
            return false;
        }

        if self.points.len() < self.capacity {
            // Store the center point of the bounds
            self.points.push((bounds.center_x, bounds.center_y, id));
            return true;
        }

        if !self.divided {
            self.subdivide();
        }

        // Try to insert into child nodes
        if let Some(ref mut quad) = self.northeast {
            if quad.insert_with_bounds(bounds, id) {
                return true;
            }
        }
        if let Some(ref mut quad) = self.northwest {
            if quad.insert_with_bounds(bounds, id) {
                return true;
            }
        }
        if let Some(ref mut quad) = self.southeast {
            if quad.insert_with_bounds(bounds, id) {
                return true;
            }
        }
        if let Some(ref mut quad) = self.southwest {
            if quad.insert_with_bounds(bounds, id) {
                return true;
            }
        }
        false
    }

    pub fn new(id: impl Into<ElementId>, boundary: BoundingBox, capacity: usize) -> Self {
        Self {
            id: id.into(),
            boundary,
            capacity,
            points: Vec::new(),
            divided: false,
            northeast: None,
            northwest: None,
            southeast: None,
            southwest: None,
        }
    }

    fn subdivide(&mut self) {
        let x = self.boundary.center_x;
        let y = self.boundary.center_y;
        let hw = self.boundary.half_width / 2.0;
        let hh = self.boundary.half_height / 2.0;
        self.northeast = Some(Box::new(QuadTree::new(
            new_element_id(),
            BoundingBox {
                center_x: x + hw,
                center_y: y - hh,
                half_width: hw,
                half_height: hh,
            },
            self.capacity,
        )));
        self.northwest = Some(Box::new(QuadTree::new(
            new_element_id(),
            BoundingBox {
                center_x: x - hw,
                center_y: y - hh,
                half_width: hw,
                half_height: hh,
            },
            self.capacity,
        )));
        self.southeast = Some(Box::new(QuadTree::new(
            new_element_id(),
            BoundingBox {
                center_x: x + hw,
                center_y: y + hh,
                half_width: hw,
                half_height: hh,
            },
            self.capacity,
        )));
        self.southwest = Some(Box::new(QuadTree::new(
            new_element_id(),
            BoundingBox {
                center_x: x - hw,
                center_y: y + hh,
                half_width: hw,
                half_height: hh,
            },
            self.capacity,
        )));
        self.divided = true;
    }

    fn insert(&mut self, x: f32, y: f32, id: usize) -> bool {
        if !self.contains(x, y) {
            return false;
        }
        self.insert_point(x, y, id)
    }

    fn insert_point(&mut self, x: f32, y: f32, id: usize) -> bool {
        if self.points.len() < self.capacity {
            self.points.push((x, y, id));
            return true;
        }
        if !self.divided {
            self.subdivide();
        }
        if let Some(ref mut quad) = self.northeast {
            if quad.insert(x, y, id) {
                return true;
            }
        }
        if let Some(ref mut quad) = self.northwest {
            if quad.insert(x, y, id) {
                return true;
            }
        }
        if let Some(ref mut quad) = self.southeast {
            if quad.insert(x, y, id) {
                return true;
            }
        }
        if let Some(ref mut quad) = self.southwest {
            if quad.insert(x, y, id) {
                return true;
            }
        }
        false
    }

    fn contains(&self, x: f32, y: f32) -> bool {
        self.boundary.contains_point(x, y)
    }

    fn intersects(&self, other: &BoundingBox) -> bool {
        self.boundary.intersects(other)
    }

    fn query_range(&self, query_range: &BoundingBox) -> Vec<(f32, f32, usize)> {
        let mut results = Vec::new();
        self.query_range_into(query_range, &mut results);
        results
    }

    fn query_range_into(&self, query_range: &BoundingBox, results: &mut Vec<(f32, f32, usize)>) {
        if !self.intersects(query_range) {
            return;
        }

        for &(x, y, id) in &self.points {
            if query_range.contains_point(x, y) {
                results.push((x, y, id));
            }
        }

        if self.divided {
            if let Some(ref quad) = self.northeast {
                quad.query_range_into(query_range, results);
            }
            if let Some(ref quad) = self.northwest {
                quad.query_range_into(query_range, results);
            }
            if let Some(ref quad) = self.southeast {
                quad.query_range_into(query_range, results);
            }
            if let Some(ref quad) = self.southwest {
                quad.query_range_into(query_range, results);
            }
        }
    }
}

pub struct SceneGraph {
    id: ElementId,
    tree: QuadTree,
    nodes: Vec<SceneNode>,
    viewport_size: Option<Size<Pixels>>,
}

impl SceneGraph {
    pub fn new(id: impl Into<ElementId>, cx: &mut Context<Self>) -> Self {
        let boundary = BoundingBox::new(0.0, 0.0, 1000.0, 1000.0);
        let tree = QuadTree::new("scene-graph-quad-tree", boundary, 4);

        let mut graph = SceneGraph {
            id: id.into(),
            tree,
            nodes: Vec::new(),
            viewport_size: None,
        };

        for _ in 0..1000 {
            let x = -200.0 + (rand::random::<f32>() * 2400.0);
            let y = -200.0 + (rand::random::<f32>() * 2400.0);
            let size = 1.0 + (rand::random::<f32>() * 32.0);
            if rand::random::<bool>() {
                graph.add_circle(x, y, size);
            } else {
                graph.add_rectangle(x, y, size * 2.0, size * 2.0);
            }
        }

        eprintln!("SceneGraph initialized with {} nodes", graph.nodes.len());

        graph
    }

    pub fn add_node(&mut self, node: SceneNode) -> usize {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }

    pub fn add_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) -> usize {
        let node = SceneNode {
            id: self.nodes.len(),
            transform: LocalTransform {
                position: LocalPosition::new(x, y),
                scale_x: 1.0,
                scale_y: 1.0,
                rotation: 0.0,
            },
            element: Some(ElementStyle {
                width,
                height,
                corner_radius: 0.0,
            }),
            children: Vec::new(),
            clip_content: false,
        };

        self.add_node(node)
    }

    pub fn add_circle(&mut self, x: f32, y: f32, radius: f32) -> usize {
        let node = SceneNode {
            id: self.nodes.len(),
            transform: LocalTransform {
                position: LocalPosition::new(x, y),
                scale_x: 1.0,
                scale_y: 1.0,
                rotation: 0.0,
            },
            element: Some(ElementStyle {
                width: radius * 2.0,
                height: radius * 2.0,
                corner_radius: radius,
            }),
            children: Vec::new(),
            clip_content: false,
        };

        self.add_node(node)
    }

    pub fn update_viewport(
        &mut self,
        viewport: Size<Pixels>,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let bounds = BoundingBox::new(
            viewport.width.0 as f32 / 2.0,
            viewport.height.0 as f32 / 2.0,
            viewport.width.0 as f32 / 2.0,
            viewport.height.0 as f32 / 2.0,
        );

        self.viewport_size = Some(viewport);
        self.tree.update_bounds(bounds);
        cx.notify();
    }
}

#[derive(Default)]
pub struct SceneGraphLayoutState {
    node_layouts: Vec<LayoutId>,
}

impl Element for SceneGraph {
    type RequestLayoutState = SceneGraphLayoutState;
    type PrepaintState = Option<Hitbox>;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut state = SceneGraphLayoutState::default();

        eprintln!("SceneGraph request_layout with {} nodes", self.nodes.len());

        // Request layout for each node
        for node in &mut self.nodes {
            // eprintln!("Processing node: {:?}", node.id);

            if let Some(ref element) = node.element {
                let style = Style {
                    position: Position::Absolute,
                    size: Size {
                        width: px(element.width).into(),
                        height: px(element.height).into(),
                    },
                    ..Default::default()
                };

                let layout_id = window.request_layout(style, vec![], cx);
                state.node_layouts.push(layout_id);
            }
        }

        // Create layout for the scene graph container
        let layout_id = window.request_layout(
            Style {
                position: Position::Absolute,
                ..Default::default()
            },
            state.node_layouts.iter().copied(),
            cx,
        );

        eprintln!(
            "SceneGraph layout created with {} child layouts",
            state.node_layouts.len()
        );

        (layout_id, state)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        self.viewport_size = Some(bounds.size);

        let new_bounds = BoundingBox::new(
            bounds.origin.x.0 as f32 + bounds.size.width.0 as f32 / 2.0,
            bounds.origin.y.0 as f32 + bounds.size.height.0 as f32 / 2.0,
            bounds.size.width.0 as f32 / 2.0,
            bounds.size.height.0 as f32 / 2.0,
        );

        self.tree.update_bounds(new_bounds);

        for (i, (node, layout_id)) in self
            .nodes
            .iter()
            .zip(&request_layout.node_layouts)
            .enumerate()
        {
            let node_bounds = window.layout_bounds(*layout_id);
            if let Some(element) = &node.element {
                let bounds = BoundingBox::new(
                    node_bounds.origin.x.0 as f32,
                    node_bounds.origin.y.0 as f32,
                    element.width * node.transform.scale_x / 2.0,
                    element.height * node.transform.scale_y / 2.0,
                );
                self.tree.insert_with_bounds(&bounds, i);
            }
        }

        Some(window.insert_hitbox(bounds, false))
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let query_bounds = BoundingBox::new(
            bounds.origin.x.0 as f32 + bounds.size.width.0 as f32 / 2.0,
            bounds.origin.y.0 as f32 + bounds.size.height.0 as f32 / 2.0,
            bounds.size.width.0 as f32 / 2.0,
            bounds.size.height.0 as f32 / 2.0,
        );

        let visible_nodes = self.tree.query_range(&query_bounds);
        eprintln!(
            "SceneGraph paint: found {} visible nodes in bounds {:?}",
            visible_nodes.len(),
            bounds
        );

        // Paint a background to visualize the scene graph bounds
        window.paint_quad(quad(
            bounds,
            0.0,
            hsla(0.7, 0.3, 0.9, 0.1), // Light purple background
            px(1.0),
            hsla(0.7, 0.5, 0.5, 0.5), // Purple border
        ));

        for (_, _, node_id) in visible_nodes {
            if let Some(node) = self.nodes.get(node_id) {
                // eprintln!(
                //     "Painting node {}: position ({}, {})",
                //     node.id, node.transform.position.0, node.transform.position.1
                // );

                if let Some(element) = &node.element {
                    // Create more visually distinct nodes for debugging
                    let node_color = match node.id % 12 {
                        0 => hsla(0.0, 0.7, 0.5, 0.3),  // Red
                        1 => hsla(0.3, 0.7, 0.5, 0.3),  // Green
                        2 => hsla(0.6, 0.7, 0.5, 0.3),  // Blue
                        3 => hsla(0.1, 0.7, 0.5, 0.3),  // Orange
                        4 => hsla(0.8, 0.7, 0.5, 0.3),  // Purple
                        5 => hsla(0.5, 0.7, 0.5, 0.3),  // Teal
                        6 => hsla(0.15, 0.7, 0.5, 0.3), // Yellow
                        7 => hsla(0.7, 0.7, 0.5, 0.3),  // Pink
                        8 => hsla(0.4, 0.7, 0.5, 0.3),  // Lime
                        9 => hsla(0.9, 0.7, 0.5, 0.3),  // Magenta
                        10 => hsla(0.2, 0.7, 0.5, 0.3), // Olive
                        _ => hsla(0.55, 0.7, 0.5, 0.3), // Sky Blue
                    };

                    let node_bounds = Bounds {
                        origin: Point {
                            x: px(node.transform.position.x()
                                - element.width * node.transform.scale_x / 2.0),
                            y: px(node.transform.position.y()
                                - element.height * node.transform.scale_y / 2.0),
                        },
                        size: Size {
                            width: px(element.width * node.transform.scale_x),
                            height: px(element.height * node.transform.scale_y),
                        },
                    };

                    window.paint_quad(quad(
                        node_bounds,
                        element.corner_radius,
                        node_color,
                        px(2.0),
                        hsla(0.0, 0.0, 0.0, 1.0),
                    ));
                }
            }
        }
    }
}

impl IntoElement for SceneGraph {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Render for SceneGraph {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.clone()
    }
}

impl Clone for SceneGraph {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            tree: self.tree.clone(),
            nodes: self.nodes.clone(),
            viewport_size: self.viewport_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn generate_random(min: f32, max: f32) -> f32 {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let random = (seed as f32 / u32::MAX as f32) * (max - min) + min;
        random
    }

    fn count_points(qt: &QuadTree) -> usize {
        let mut count = qt.points.len();
        if qt.divided {
            if let Some(ref ne) = qt.northeast {
                count += count_points(ne);
            }
            if let Some(ref nw) = qt.northwest {
                count += count_points(nw);
            }
            if let Some(ref se) = qt.southeast {
                count += count_points(se);
            }
            if let Some(ref sw) = qt.southwest {
                count += count_points(sw);
            }
        }
        count
    }

    fn check_node(qt: &QuadTree) {
        for (x, y, _id) in &qt.points {
            assert!(qt.boundary.contains_point(*x, *y));
        }
        if qt.divided {
            if let Some(ref ne) = qt.northeast {
                check_node(ne);
            }
            if let Some(ref nw) = qt.northwest {
                check_node(nw);
            }
            if let Some(ref se) = qt.southeast {
                check_node(se);
            }
            if let Some(ref sw) = qt.southwest {
                check_node(sw);
            }
        }
    }

    #[test]
    fn test_element_bounds() {
        let rect = ElementStyle {
            width: 100.0,
            height: 50.0,
            corner_radius: 0.0,
        };
        let rect_transform = LocalTransform {
            position: LocalPosition::new(10.0, 20.0),
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
        };
        let rect_bounds = rect.calculate_local_bounds(&rect_transform);
        assert_eq!(rect_bounds.center_x, 10.0);
        assert_eq!(rect_bounds.center_y, 20.0);
        assert_eq!(rect_bounds.half_width, 50.0);
        assert_eq!(rect_bounds.half_height, 25.0);

        let circle = ElementStyle {
            width: 60.0,
            height: 60.0,
            corner_radius: 30.0,
        };
        let circle_transform = LocalTransform {
            position: LocalPosition::new(-10.0, -20.0),
            scale_x: 2.0,
            scale_y: 2.0,
            rotation: 0.0,
        };
        let circle_bounds = circle.calculate_local_bounds(&circle_transform);
        assert_eq!(circle_bounds.center_x, -10.0);
        assert_eq!(circle_bounds.center_y, -20.0);
        assert_eq!(circle_bounds.half_width, 60.0);
        assert_eq!(circle_bounds.half_height, 60.0);
    }

    #[test]
    fn test_insert_point_within_boundary() {
        for i in 0..10 {
            let mut qt = QuadTree::new(
                "test",
                BoundingBox {
                    center_x: 0.0,
                    center_y: 0.0,
                    half_width: 10.0,
                    half_height: 10.0,
                },
                4,
            );

            let x = generate_random(-10.0, 10.0);
            let y = generate_random(-10.0, 10.0);

            let inserted = qt.insert(x, y, i);
            assert!(
                inserted,
                "Iteration {}: Failed to insert point ({}, {}) within boundary",
                i, x, y
            );
            assert_eq!(qt.points.len(), 1);
        }
    }

    #[test]
    fn test_insert_outside_point_boundary() {
        for i in 0..10 {
            let mut qt = QuadTree::new(
                "test",
                BoundingBox {
                    center_x: 0.0,
                    center_y: 0.0,
                    half_width: 10.0,
                    half_height: 10.0,
                },
                4,
            );

            let x = if i % 2 == 0 {
                generate_random(15.0, 25.0)
            } else {
                generate_random(-25.0, -15.0)
            };

            let y = if (i / 2) % 2 == 0 {
                generate_random(15.0, 25.0)
            } else {
                generate_random(-25.0, -15.0)
            };

            let inserted = qt.insert(x, y, i);
            assert!(
                !inserted,
                "Iteration {}: Point ({}, {}) outside boundary was incorrectly inserted",
                i, x, y
            );
            assert_eq!(qt.points.len(), 0);
        }
    }

    #[test]
    fn test_division() {
        let mut qt = QuadTree::new(
            "test",
            BoundingBox {
                center_x: 0.0,
                center_y: 0.0,
                half_width: 10.0,
                half_height: 10.0,
            },
            1,
        );
        assert!(!qt.divided);
        let inserted1 = qt.insert(1.0, 1.0, 1);
        assert!(inserted1);
        assert!(!qt.divided);
        let inserted2 = qt.insert(-1.0, -1.0, 2);
        assert!(inserted2);
        assert!(qt.divided);
        assert!(qt.northeast.is_some());
        assert!(qt.northwest.is_some());
        assert!(qt.southeast.is_some());
        assert!(qt.southwest.is_some());
    }

    #[test]
    fn test_complex_division() {
        let mut qt = QuadTree::new(
            "test",
            BoundingBox {
                center_x: 0.0,
                center_y: 0.0,
                half_width: 16.0,
                half_height: 16.0,
            },
            4,
        );

        let points = vec![
            (10.0, 10.0, 1),
            (12.0, 12.0, 2),
            (-10.0, 10.0, 3),
            (-12.0, 12.0, 4),
            (-10.0, -10.0, 5),
            (-12.0, -12.0, 6),
            (10.0, -10.0, 7),
            (12.0, -12.0, 8),
            (0.0, 0.0, 9),
            (5.0, 5.0, 10),
            (-5.0, 5.0, 11),
        ];

        for (x, y, id) in points.iter() {
            qt.insert(*x, *y, *id);
        }

        assert_eq!(count_points(&qt), points.len());
        check_node(&qt);
    }

    #[test]
    fn test_insert_element_with_bounds() {
        let mut qt = QuadTree::new(
            "test",
            BoundingBox {
                center_x: 0.0,
                center_y: 0.0,
                half_width: 100.0,
                half_height: 100.0,
            },
            4,
        );

        let rect = ElementStyle {
            width: 20.0,
            height: 10.0,
            corner_radius: 0.0,
        };
        let rect_transform = LocalTransform {
            position: LocalPosition::new(30.0, 40.0),
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
        };
        let rect_bounds = rect.calculate_local_bounds(&rect_transform);

        assert!(qt.insert_with_bounds(&rect_bounds, 1));

        let circle = ElementStyle {
            width: 30.0,
            height: 30.0,
            corner_radius: 15.0,
        };
        let circle_transform = LocalTransform {
            position: LocalPosition(Vector2D { x: 20.0, y: 30.0 }),
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
        };
        let circle_bounds = circle.calculate_local_bounds(&circle_transform);

        assert!(qt.insert_with_bounds(&circle_bounds, 2));

        let rect_query = qt.query_range(&rect_bounds);
        assert!(rect_query.iter().any(|&(_, _, id)| id == 1));

        let circle_query = qt.query_range(&circle_bounds);
        assert!(circle_query.iter().any(|&(_, _, id)| id == 2));
    }

    #[test]
    fn test_scene_node_with_quadtree() {
        let mut qt = QuadTree::new(
            "test",
            BoundingBox {
                center_x: 0.0,
                center_y: 0.0,
                half_width: 100.0,
                half_height: 100.0,
            },
            4,
        );

        let rect_node = SceneNode {
            id: 1,
            transform: LocalTransform {
                position: LocalPosition::new(25.0, 25.0),
                scale_x: 1.0,
                scale_y: 1.0,
                rotation: 0.0,
            },
            element: Some(ElementStyle {
                width: 30.0,
                height: 20.0,
                corner_radius: 0.0,
            }),
            children: vec![],
            clip_content: true,
        };

        let circle_node = SceneNode {
            id: 2,
            transform: LocalTransform {
                position: LocalPosition::new(-25.0, -25.0),
                scale_x: 2.0,
                scale_y: 2.0,
                rotation: 0.0,
            },
            element: Some(ElementStyle {
                width: 20.0,
                height: 20.0,
                corner_radius: 10.0,
            }),
            children: vec![],
            clip_content: true,
        };

        if let Some(ref element) = rect_node.element {
            let bounds = element.calculate_local_bounds(&rect_node.transform);
            assert!(qt.insert_with_bounds(&bounds, rect_node.id));
        }

        if let Some(ref element) = circle_node.element {
            let bounds = element.calculate_local_bounds(&circle_node.transform);
            assert!(qt.insert_with_bounds(&bounds, circle_node.id));
        }

        let query_rect = BoundingBox {
            center_x: 25.0,
            center_y: 25.0,
            half_width: 20.0,
            half_height: 20.0,
        };
        let rect_results = qt.query_range(&query_rect);
        assert!(rect_results.iter().any(|&(_, _, id)| id == rect_node.id));

        let query_circle = BoundingBox {
            center_x: -25.0,
            center_y: -25.0,
            half_width: 25.0,
            half_height: 25.0,
        };
        let circle_results = qt.query_range(&query_circle);
        assert!(circle_results
            .iter()
            .any(|&(_, _, id)| id == circle_node.id));
    }

    #[test]
    fn test_clipped_scene_node_with_child() {
        let parent_node = SceneNode {
            id: 1,
            transform: LocalTransform {
                position: LocalPosition::new(0.0, 0.0),
                scale_x: 1.0,
                scale_y: 1.0,
                rotation: 0.0,
            },
            element: Some(ElementStyle {
                width: 100.0,
                height: 50.0,
                corner_radius: 0.0,
            }),
            clip_content: true,
            children: vec![SceneNode {
                id: 2,
                transform: LocalTransform {
                    position: LocalPosition::new(60.0, 0.0),
                    scale_x: 1.0,
                    scale_y: 1.0,
                    rotation: 0.0,
                },
                element: Some(ElementStyle {
                    width: 40.0,
                    height: 40.0,
                    corner_radius: 20.0,
                }),
                clip_content: true,
                children: vec![],
            }],
        };

        let combined_bounds = parent_node.calculate_combined_bounds().unwrap();

        // The combined bounds should be clipped to the parent rectangle
        assert!(
            combined_bounds.half_width <= 50.0,
            "Width should be clipped to parent bounds"
        );
        assert!(
            combined_bounds.half_height <= 25.0,
            "Height should be clipped to parent bounds"
        );
    }

    #[test]
    fn test_unclipped_scene_node_with_child() {
        let parent_node = SceneNode {
            id: 1,
            transform: LocalTransform {
                position: LocalPosition::new(0.0, 0.0),
                scale_x: 1.0,
                scale_y: 1.0,
                rotation: 0.0,
            },
            element: Some(ElementStyle {
                width: 100.0,
                height: 50.0,
                corner_radius: 0.0,
            }),
            clip_content: false,
            children: vec![SceneNode {
                id: 2,
                transform: LocalTransform {
                    position: LocalPosition::new(60.0, 0.0),
                    scale_x: 1.0,
                    scale_y: 1.0,
                    rotation: 0.0,
                },
                element: Some(ElementStyle {
                    width: 40.0,
                    height: 40.0,
                    corner_radius: 20.0,
                }),
                clip_content: false,
                children: vec![],
            }],
        };

        let combined_bounds = parent_node.calculate_combined_bounds().unwrap();

        // The unclipped bounds should be large enough to contain both the parent rectangle and the full circle
        assert!(
            combined_bounds.half_width > 50.0,
            "Width should include full circle bounds"
        );
        assert!(
            combined_bounds.center_x > 0.0,
            "Center should shift right to accommodate circle"
        );
    }

    #[test]
    fn test_query_range() {
        let mut qt = QuadTree::new(
            "test",
            BoundingBox {
                center_x: 0.0,
                center_y: 0.0,
                half_width: 16.0,
                half_height: 16.0,
            },
            4,
        );
        let points = vec![
            (5.0, 5.0, 1),
            (-5.0, -5.0, 2),
            (10.0, 10.0, 3),
            (12.0, 12.0, 4),
            (0.0, 0.0, 5),
        ];
        for (x, y, id) in points {
            qt.insert(x, y, id);
        }
        let query_range = BoundingBox {
            center_x: 5.0,
            center_y: 5.0,
            half_width: 3.0,
            half_height: 3.0,
        };
        let found_points = qt.query_range(&query_range);
        assert_eq!(found_points.len(), 1);
        assert_eq!(found_points[0].2, 1);
    }
}
