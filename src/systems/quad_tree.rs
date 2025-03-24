use crate::prelude::*;
use std::collections::HashSet;

/// A node in the QuadTree spatial index
struct QuadTreeNode {
    /// The bounds of this quad
    bounds: BoundingBox,
    /// Entities stored at this level (if they don't fit entirely in a child)
    entities: HashSet<LunaEntityId>,
    /// Child nodes (subdivisions of this quad)
    children: Option<Box<[QuadTreeNode; 4]>>,
}

/// Maximum number of entities before subdividing a node
const MAX_ENTITIES_PER_NODE: usize = 8;
/// Maximum depth of the quadtree
const MAX_DEPTH: u32 = 8;

impl QuadTreeNode {
    fn new(bounds: BoundingBox) -> Self {
        QuadTreeNode {
            bounds,
            entities: HashSet::new(),
            children: None,
        }
    }

    fn insert(&mut self, entity: LunaEntityId, entity_bounds: BoundingBox, depth: u32) {
        // If we're at max depth or the entity doesn't fit in any child, store it here
        if depth >= MAX_DEPTH || !self.bounds.contains(&entity_bounds) {
            self.entities.insert(entity);
            return;
        }

        // Create children if we don't have them and we're not at max depth
        if self.children.is_none() && self.entities.len() >= MAX_ENTITIES_PER_NODE {
            self.subdivide();
        }

        // If we have children, try to insert into them
        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                if child.bounds.contains(&entity_bounds) {
                    child.insert(entity, entity_bounds, depth + 1);
                    return;
                }
            }
        }

        // If we couldn't insert into a child, store it here
        self.entities.insert(entity);
    }

    fn subdivide(&mut self) {
        let center = self.bounds.center();
        let half_width = self.bounds.half_width();
        let half_height = self.bounds.half_height();

        // Create four child nodes
        let children = Box::new([
            // Northwest
            QuadTreeNode::new(BoundingBox::new(
                vec2(self.bounds.min().x, center.y),
                vec2(center.x, self.bounds.max().y),
            )),
            // Northeast
            QuadTreeNode::new(BoundingBox::new(
                vec2(center.x, center.y),
                vec2(self.bounds.max().x, self.bounds.max().y),
            )),
            // Southwest
            QuadTreeNode::new(BoundingBox::new(
                vec2(self.bounds.min().x, self.bounds.min().y),
                vec2(center.x, center.y),
            )),
            // Southeast
            QuadTreeNode::new(BoundingBox::new(
                vec2(center.x, self.bounds.min().y),
                vec2(self.bounds.max().x, center.y),
            )),
        ]);

        self.children = Some(children);
    }

    fn query_point(&self, point: Vector2D) -> HashSet<LunaEntityId> {
        let mut results = self.entities.clone();

        // If we have children and the point is within our bounds
        if let Some(children) = &self.children {
            for child in children.iter() {
                if child.bounds.contains_point(&point) {
                    results.extend(child.query_point(point));
                }
            }
        }

        results
    }

    fn query_region(&self, region: BoundingBox) -> HashSet<LunaEntityId> {
        let mut results = self.entities.clone();

        // If we have children and the regions overlap
        if let Some(children) = &self.children {
            for child in children.iter() {
                if child.bounds.intersects(&region) {
                    results.extend(child.query_region(region));
                }
            }
        }

        results
    }
}

/// A QuadTree spatial index for efficient entity queries
pub struct QuadTree {
    root: QuadTreeNode,
    /// Map of entity IDs to their bounds for updating
    entity_bounds: HashMap<LunaEntityId, BoundingBox>,
}

impl QuadTree {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        QuadTree {
            root: QuadTreeNode::new(BoundingBox::new(
                vec2(x, y),
                vec2(x + width, y + height),
            )),
            entity_bounds: HashMap::new(),
        }
    }

    /// Inserts or updates an entity in the quadtree
    pub fn insert(&mut self, entity: LunaEntityId, bounds: BoundingBox) {
        self.entity_bounds.insert(entity, bounds);
        self.root.insert(entity, bounds, 0);
    }

    /// Returns all entities that contain the given point
    pub fn query_point(&self, x: f32, y: f32) -> HashSet<LunaEntityId> {
        self.root.query_point(vec2(x, y))
    }

    /// Returns all entities that intersect with the given region
    pub fn query_region(&self, x: f32, y: f32, width: f32, height: f32) -> HashSet<LunaEntityId> {
        let region = BoundingBox::new(vec2(x, y), vec2(x + width, y + height));
        self.root.query_region(region)
    }

    /// Removes all entities from the quadtree
    pub fn clear(&mut self) {
        self.root = QuadTreeNode::new(self.root.bounds);
        self.entity_bounds.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_query() {
        let mut qt = QuadTree::new(0.0, 0.0, 100.0, 100.0);
        
        let e1 = LunaEntityId::from(1);
        let e2 = LunaEntityId::from(2);
        
        // Insert two overlapping entities
        qt.insert(e1, BoundingBox::new(vec2(10.0, 10.0), vec2(30.0, 30.0)));
        qt.insert(e2, BoundingBox::new(vec2(20.0, 20.0), vec2(40.0, 40.0)));
        
        // Point in both entities
        let hits = qt.query_point(25.0, 25.0);
        assert_eq!(hits.len(), 2);
        assert!(hits.contains(&e1));
        assert!(hits.contains(&e2));
        
        // Point in only first entity
        let hits = qt.query_point(15.0, 15.0);
        assert_eq!(hits.len(), 1);
        assert!(hits.contains(&e1));
    }

    #[test]
    fn test_region_query() {
        let mut qt = QuadTree::new(0.0, 0.0, 100.0, 100.0);
        
        let e1 = LunaEntityId::from(1);
        let e2 = LunaEntityId::from(2);
        let e3 = LunaEntityId::from(3);
        
        // Insert three entities
        qt.insert(e1, BoundingBox::new(vec2(10.0, 10.0), vec2(20.0, 20.0)));
        qt.insert(e2, BoundingBox::new(vec2(30.0, 30.0), vec2(40.0, 40.0)));
        qt.insert(e3, BoundingBox::new(vec2(50.0, 50.0), vec2(60.0, 60.0)));
        
        // Query region that overlaps first two entities
        let hits = qt.query_region(0.0, 0.0, 45.0, 45.0);
        assert_eq!(hits.len(), 2);
        assert!(hits.contains(&e1));
        assert!(hits.contains(&e2));
        
        // Query region that overlaps only last entity
        let hits = qt.query_region(45.0, 45.0, 20.0, 20.0);
        assert_eq!(hits.len(), 1);
        assert!(hits.contains(&e3));
    }
}