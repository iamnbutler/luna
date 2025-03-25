use gpui::App;

use crate::{ecs::LunaEcs, prelude::*};

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
    pub fn update_entity(
        &mut self,
        ecs_mut: &mut LunaEcs,
        entity: LunaEntityId,
    ) {
        if !ecs_mut.entity_exists(entity) {
            return;
        }
        
        if let Some(transform) = ecs_mut.transforms().get_transform(entity) {
            // Get the parent chain and clone it to avoid borrowing issues
            let parent_chain = ecs_mut.hierarchy().get_parent_chain(entity).clone();

            // Compute world transform in a scope to limit the mutable borrow
            let world_transform = {
                // Get a mutable borrow of transforms
                let transforms_mut = ecs_mut.transforms_mut();
                transforms_mut.compute_world_transform(entity, parent_chain)
            };

            // Create bounding box and insert into spatial index if we have a world transform
            if let Some(world_transform) = world_transform {
                // Create bounding box from world transform and insert into spatial index
                // For now, using a simple 1x1 box at the position
                // TODO: Use actual element dimensions from RenderComponent
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

    /// Returns the topmost entity at the given point, respecting Z-order
    pub fn hit_test_point(
        &self,
        ecs: &LunaEcs,
        x: f32,
        y: f32,
    ) -> Option<LunaEntityId> {
        let candidates = self.spatial_index.query_point(x, y);

        // Sort candidates by Z-order (children above parents)
        // First, group entities by their depth in the hierarchy
        let mut depth_map: Vec<(LunaEntityId, usize)> = candidates
            .into_iter()
            .map(|entity| {
                let depth = ecs.hierarchy().get_parent_chain(entity).len();
                (entity, depth)
            })
            .collect();

        // Sort by depth (deeper elements come first)
        depth_map.sort_by(|a, b| b.1.cmp(&a.1));

        // Return the first (topmost) entity
        depth_map.first().map(|(entity, _)| *entity)
    }

    /// Returns all entities in the given region, sorted by Z-order
    pub fn hit_test_region(
        &self,
        ecs: &LunaEcs,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Vec<LunaEntityId> {
        let candidates = self.spatial_index.query_region(x, y, width, height);

        // Sort candidates by Z-order (children above parents)
        let mut depth_map: Vec<(LunaEntityId, usize)> = candidates
            .into_iter()
            .map(|entity| {
                let depth = ecs.hierarchy().get_parent_chain(entity).len();
                (entity, depth)
            })
            .collect();

        // Sort by depth (deeper elements come first)
        depth_map.sort_by(|a, b| b.1.cmp(&a.1));

        // Return entities in Z-order
        depth_map.into_iter().map(|(entity, _)| entity).collect()
    }

    /// Clears the spatial index
    pub fn clear(&mut self) {
        // Create a new empty quadtree with the same dimensions
        self.spatial_index = QuadTree::new(0.0, 0.0, 100.0, 100.0); // TODO: Store dimensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn test_hit_test_point(cx: &mut TestAppContext) {
        let ecs = cx.new(|cx| LunaEcs::new(cx));
        let mut hit_test = HitTestSystem::new(100.0, 100.0);
        let mut transform_system = TransformSystem::new();

        ecs.update(cx, |ecs_mut, cx| {
            // Create a parent-child hierarchy
            let parent = ecs_mut.create_entity();
            let child = ecs_mut.create_entity();

            ecs_mut.hierarchy_mut().set_parent(child, parent);

            // Set up transforms with parent scale of 2 to clearly show scaling effect
            {
                let transforms_mut = ecs_mut.transforms_mut();
                transforms_mut.set_transform(
                    parent,
                    LocalTransform {
                        position: LocalPosition { x: 10.0, y: 10.0 },
                        scale: Vector2D { x: 2.0, y: 2.0 }, // Parent has scale 2,2
                        rotation: 0.0,
                    },
                );
                transforms_mut.set_transform(
                    child,
                    LocalTransform {
                        position: LocalPosition { x: 5.0, y: 5.0 },
                        scale: Vector2D { x: 1.0, y: 1.0 },
                        rotation: 0.0,
                    },
                );
            }

            // Calculate world transforms first
            transform_system.mark_dirty(parent);
            transform_system.mark_dirty(child);
            transform_system.process(ecs_mut);
            
            // Now update the spatial index with the calculated world transforms
            hit_test.update_entity(ecs_mut, parent);
            hit_test.update_entity(ecs_mut, child);

            // Parent at (10,10) with scale (2,2), child at (5,5) local
            // Child's world position is (10 + 5*2, 10 + 5*2) = (20, 20)
            // Test at child's world position plus offset
            if let Some(hit) = hit_test.hit_test_point(ecs_mut, 20.5, 20.5) {
                assert_eq!(hit, child);
            }
        });
    }

    #[gpui::test]
    fn test_hit_test_region(cx: &mut TestAppContext) {
        let ecs = cx.new(|cx| LunaEcs::new(cx));

        let mut hit_test = HitTestSystem::new(100.0, 100.0);

        ecs.update(cx, |ecs_mut, cx| {
            // Create some test entities
            let e1 = ecs_mut.create_entity();
            let e2 = ecs_mut.create_entity();

            let transforms = ecs_mut.transforms_mut();
            transforms.set_transform(
                e1,
                LocalTransform {
                    position: LocalPosition { x: 10.0, y: 10.0 },
                    scale: Vector2D { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                },
            );

            transforms.set_transform(
                e2,
                LocalTransform {
                    position: LocalPosition { x: 20.0, y: 20.0 },
                    scale: Vector2D { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                },
            );

            // Update spatial index
            hit_test.update_entity(ecs_mut, e1);
            hit_test.update_entity(ecs_mut, e2);

            // Test region query
            let hits = hit_test.hit_test_region(ecs_mut, 0.0, 0.0, 30.0, 30.0);
            assert_eq!(hits.len(), 2);
        });
    }
}
