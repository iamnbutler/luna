use crate::{ecs::LunaEcs, prelude::*, systems::spatial::QuadTree};

/// System that handles hit testing and spatial queries for the canvas
pub struct HitTestSystem {
    spatial_index: QuadTree,
}

impl HitTestSystem {
    pub fn new(width: f32, height: f32) -> Self {
        HitTestSystem {
            spatial_index: QuadTree::new(0.0, 0.0, width, height),
        }
    }

    /// Updates the spatial index for an entity
    pub fn update_entity(&mut self, ecs: &mut LunaEcs, entity: LunaEntityId) {
        if let Some(transform) = ecs.get_transform(entity) {
            // Update world transform to ensure it's current
            ecs.update_world_transforms(entity);
            
            // Get the cached world transform
            if let Some(world_transform) = ecs.world_transform_cache.get(&entity) {
                // If there are render properties, use them for bbox size
                if let Some(render_props) = ecs.get_render_properties(entity) {
                    let bbox = BoundingBox::new(
                        vec2(world_transform.position.x, world_transform.position.y),
                        vec2(
                            world_transform.position.x + render_props.width * world_transform.scale.x,
                            world_transform.position.y + render_props.height * world_transform.scale.y,
                        ),
                    );
                    self.spatial_index.insert(entity, bbox);
                } else {
                    // Create a simple 1x1 bounding box if no render properties
                    let bbox = BoundingBox::new(
                        vec2(world_transform.position.x, world_transform.position.y),
                        vec2(
                            world_transform.position.x + 1.0,
                            world_transform.position.y + 1.0,
                        ),
                    );
                    self.spatial_index.insert(entity, bbox);
                }
            }
        }
    }

    /// Returns the topmost entity at the given point, respecting Z-order
    pub fn hit_test_point(&self, ecs: &LunaEcs, x: f32, y: f32) -> Option<LunaEntityId> {
        let candidates = self.spatial_index.query_point(x, y);

        // Sort candidates by Z-order (children above parents)
        // First, group entities by their depth in the hierarchy
        let mut depth_map: Vec<(LunaEntityId, usize)> = Vec::new();
        
        for entity in candidates {
            let depth = ecs.get_parent_chain(entity).len();
            depth_map.push((entity, depth));
        }

        // Sort by depth (deeper elements come first)
        depth_map.sort_by(|a, b| b.1.cmp(&a.1));

        // Return the first (topmost) entity
        depth_map.first().map(|(entity, _)| *entity)
    }

    /// Returns all entities in the given region, sorted by Z-order
    pub fn hit_test_region(&self, ecs: &LunaEcs, x: f32, y: f32, width: f32, height: f32) -> Vec<LunaEntityId> {
        let candidates = self.spatial_index.query_region(x, y, width, height);

        // Sort candidates by Z-order (children above parents)
        let mut depth_map: Vec<(LunaEntityId, usize)> = Vec::new();
        
        for entity in candidates {
            let depth = ecs.get_parent_chain(entity).len();
            depth_map.push((entity, depth));
        }

        // Sort by depth (deeper elements come first)
        depth_map.sort_by(|a, b| b.1.cmp(&a.1));

        // Return entities in Z-order
        depth_map.into_iter().map(|(entity, _)| entity).collect()
    }

    /// Clears the spatial index
    pub fn clear(&mut self) {
        // Create a new quadtree with the same dimensions
        self.spatial_index = QuadTree::new(0.0, 0.0, 100.0, 100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_test_point() {
        let mut ecs = LunaEcs::new();
        let mut hit_test = HitTestSystem::new(100.0, 100.0);

        // Create a parent-child hierarchy
        let parent = ecs.create_entity();
        let child = ecs.create_entity();

        ecs.set_parent(child, parent);

        ecs.set_transform(
            parent,
            LocalTransform {
                position: LocalPosition { x: 10.0, y: 10.0 },
                scale: Vector2D { x: 1.0, y: 1.0 },
                rotation: 0.0,
            },
        );
        
        ecs.set_transform(
            child,
            LocalTransform {
                position: LocalPosition { x: 5.0, y: 5.0 },
                scale: Vector2D { x: 1.0, y: 1.0 },
                rotation: 0.0,
            },
        );

        // Add render properties for a proper bounding box
        ecs.set_render_properties(
            parent,
            RenderProperties {
                width: 20.0,
                height: 20.0,
                corner_radius: 0.0,
                fill_color: [1.0, 1.0, 1.0, 1.0],
                stroke_color: [0.0, 0.0, 0.0, 1.0],
                stroke_width: 1.0,
            },
        );

        ecs.set_render_properties(
            child,
            RenderProperties {
                width: 10.0,
                height: 10.0,
                corner_radius: 0.0,
                fill_color: [1.0, 1.0, 1.0, 1.0],
                stroke_color: [0.0, 0.0, 0.0, 1.0],
                stroke_width: 1.0,
            },
        );

        // Update spatial index
        hit_test.update_entity(&mut ecs, parent);
        hit_test.update_entity(&mut ecs, child);

        // Test hit testing - child should be on top
        if let Some(hit) = hit_test.hit_test_point(&ecs, 15.0, 15.0) {
            assert_eq!(hit, child);
        } else {
            panic!("Hit test failed to find any entity");
        }
    }

    #[test]
    fn test_hit_test_region() {
        let mut ecs = LunaEcs::new();
        let mut hit_test = HitTestSystem::new(100.0, 100.0);

        // Create some test entities
        let e1 = ecs.create_entity();
        let e2 = ecs.create_entity();

        ecs.set_transform(
            e1,
            LocalTransform {
                position: LocalPosition { x: 10.0, y: 10.0 },
                scale: Vector2D { x: 1.0, y: 1.0 },
                rotation: 0.0,
            },
        );

        ecs.set_transform(
            e2,
            LocalTransform {
                position: LocalPosition { x: 20.0, y: 20.0 },
                scale: Vector2D { x: 1.0, y: 1.0 },
                rotation: 0.0,
            },
        );

        // Add render properties
        ecs.set_render_properties(e1, RenderProperties {
            width: 10.0,
            height: 10.0,
            corner_radius: 0.0,
            fill_color: [1.0, 1.0, 1.0, 1.0],
            stroke_color: [0.0, 0.0, 0.0, 1.0],
            stroke_width: 1.0,
        });
        
        ecs.set_render_properties(e2, RenderProperties {
            width: 10.0,
            height: 10.0,
            corner_radius: 0.0,
            fill_color: [1.0, 1.0, 1.0, 1.0],
            stroke_color: [0.0, 0.0, 0.0, 1.0],
            stroke_width: 1.0,
        });

        // Update spatial index
        hit_test.update_entity(&mut ecs, e1);
        hit_test.update_entity(&mut ecs, e2);

        // Test region query
        let hits = hit_test.hit_test_region(&ecs, 0.0, 0.0, 30.0, 30.0);
        assert_eq!(hits.len(), 2);
    }
}
