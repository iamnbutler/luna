use crate::prelude::*;

/// Represents a rectangular region in 2D space
#[derive(Debug, Clone, Copy)]
struct Region {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Region {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Region {
            x,
            y,
            width,
            height,
        }
    }

    fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    fn intersects_box(&self, bbox: &BoundingBox) -> bool {
        let min = bbox.min();
        let max = bbox.max();

        !(min.x > self.x + self.width
            || max.x < self.x
            || min.y > self.y + self.height
            || max.y < self.y)
    }
}

#[derive(Clone)]
/// A node in the QuadTree containing entities and their bounding boxes
struct QuadTreeNode {
    region: Region,
    entities: Vec<(LunaEntityId, BoundingBox)>,
    children: Option<Box<[QuadTreeNode; 4]>>,
    max_depth: u32,
    max_entities: usize,
}

impl QuadTreeNode {
    fn new(region: Region, max_depth: u32, max_entities: usize) -> Self {
        QuadTreeNode {
            region,
            entities: Vec::new(),
            children: None,
            max_depth,
            max_entities,
        }
    }

    fn insert(&mut self, entity: LunaEntityId, bbox: BoundingBox, depth: u32) {
        // If we don't intersect with this node's region, ignore
        if !self.region.intersects_box(&bbox) {
            return;
        }

        // If we have children, insert into them
        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                child.insert(entity, bbox, depth + 1);
            }
            return;
        }

        // Add to this node
        self.entities.push((entity, bbox));

        // Split if needed
        if self.entities.len() > self.max_entities && depth < self.max_depth {
            self.split();
        }
    }

    fn split(&mut self) {
        let half_width = self.region.width / 2.0;
        let half_height = self.region.height / 2.0;

        // Create four child nodes
        let children = Box::new([
            // Top left
            QuadTreeNode::new(
                Region::new(self.region.x, self.region.y, half_width, half_height),
                self.max_depth,
                self.max_entities,
            ),
            // Top right
            QuadTreeNode::new(
                Region::new(
                    self.region.x + half_width,
                    self.region.y,
                    half_width,
                    half_height,
                ),
                self.max_depth,
                self.max_entities,
            ),
            // Bottom left
            QuadTreeNode::new(
                Region::new(
                    self.region.x,
                    self.region.y + half_height,
                    half_width,
                    half_height,
                ),
                self.max_depth,
                self.max_entities,
            ),
            // Bottom right
            QuadTreeNode::new(
                Region::new(
                    self.region.x + half_width,
                    self.region.y + half_height,
                    half_width,
                    half_height,
                ),
                self.max_depth,
                self.max_entities,
            ),
        ]);

        // Move existing entities to children
        let entities = std::mem::take(&mut self.entities);
        self.children = Some(children);

        // Reinsert entities into children
        if let Some(children) = &mut self.children {
            for (entity, bbox) in entities {
                for child in children.iter_mut() {
                    child.insert(entity, bbox, 1);
                }
            }
        }
    }

    fn query_point(&self, x: f32, y: f32) -> Vec<LunaEntityId> {
        let mut result = Vec::new();

        // If point isn't in this node's region, return empty
        if !self.region.contains_point(x, y) {
            return result;
        }

        // Check children first if they exist
        if let Some(children) = &self.children {
            for child in children.iter() {
                result.extend(child.query_point(x, y));
            }
        } else {
            // Add entities whose bounding boxes contain the point
            for (entity, bbox) in &self.entities {
                let min = bbox.min();
                let max = bbox.max();
                if x >= min.x && x <= max.x && y >= min.y && y <= max.y {
                    result.push(*entity);
                }
            }
        }

        result
    }

    fn query_region(&self, region: &Region) -> Vec<LunaEntityId> {
        let mut result = Vec::new();

        // If regions don't intersect, return empty
        if !self.region.intersects_box(&BoundingBox::new(
            vec2(region.x, region.y),
            vec2(region.x + region.width, region.y + region.height),
        )) {
            return result;
        }

        // Check children first if they exist
        if let Some(children) = &self.children {
            for child in children.iter() {
                result.extend(child.query_region(region));
            }
        } else {
            // Add entities whose bounding boxes intersect with the region
            for (entity, bbox) in &self.entities {
                if region.intersects_box(bbox) {
                    result.push(*entity);
                }
            }
        }

        result
    }
}

#[derive(Clone)]
/// A QuadTree spatial index for efficient entity queries
pub struct QuadTree {
    root: QuadTreeNode,
}

impl QuadTree {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        QuadTree {
            root: QuadTreeNode::new(Region::new(x, y, width, height), 8, 16),
        }
    }

    pub fn insert(&mut self, entity: LunaEntityId, bbox: BoundingBox) {
        self.root.insert(entity, bbox, 0);
    }

    pub fn query_point(&self, x: f32, y: f32) -> Vec<LunaEntityId> {
        self.root.query_point(x, y)
    }

    pub fn query_region(&self, x: f32, y: f32, width: f32, height: f32) -> Vec<LunaEntityId> {
        self.root.query_region(&Region::new(x, y, width, height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadtree_point_query() {
        let mut tree = QuadTree::new(0.0, 0.0, 100.0, 100.0);

        // Insert some test entities
        let e1 = LunaEntityId::from(1);
        let e2 = LunaEntityId::from(2);

        tree.insert(e1, BoundingBox::new(vec2(10.0, 10.0), vec2(20.0, 20.0)));
        tree.insert(e2, BoundingBox::new(vec2(30.0, 30.0), vec2(40.0, 40.0)));

        // Test point queries
        let result = tree.query_point(15.0, 15.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], e1);

        let result = tree.query_point(35.0, 35.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], e2);

        let result = tree.query_point(0.0, 0.0);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_quadtree_region_query() {
        let mut tree = QuadTree::new(0.0, 0.0, 100.0, 100.0);

        // Insert some test entities
        let e1 = LunaEntityId::from(1);
        let e2 = LunaEntityId::from(2);
        let e3 = LunaEntityId::from(3);

        tree.insert(e1, BoundingBox::new(vec2(10.0, 10.0), vec2(20.0, 20.0)));
        tree.insert(e2, BoundingBox::new(vec2(30.0, 30.0), vec2(40.0, 40.0)));
        tree.insert(e3, BoundingBox::new(vec2(15.0, 15.0), vec2(35.0, 35.0)));

        // Test region query
        let result = tree.query_region(0.0, 0.0, 25.0, 25.0);
        assert!(result.contains(&e1));
        assert!(result.contains(&e3));
        assert_eq!(result.len(), 2);
    }
}
