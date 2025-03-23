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
        ecs: Entity<LunaEcs>,
        entity: LunaEntityId,
        cx: &mut Context<LunaEcs>,
    ) {
        if let Some(transform) = ecs.read(cx).transforms(cx).read(cx).get_transform(entity) {
            // Get the parent chain to compute world transform
            let parent_chain = ecs.read(cx).hierarchy(cx).read(cx).get_parent_chain(entity);

            // Compute world transform
            if let Some(world_transform) = ecs.update(cx, |ecs, cx| {
                ecs.transforms(cx).update(cx, |transforms, cx| {
                    transforms.compute_world_transform(entity, parent_chain)
                })
            }) {
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
        cx: &Context<LunaEcs>,
    ) -> Option<LunaEntityId> {
        let candidates = self.spatial_index.query_point(x, y);

        // Sort candidates by Z-order (children above parents)
        // First, group entities by their depth in the hierarchy
        let mut depth_map: Vec<(LunaEntityId, usize)> = candidates
            .into_iter()
            .map(|entity| {
                let depth = ecs.hierarchy(cx).read(cx).get_parent_chain(entity).len();
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
        cx: &Context<LunaEcs>,
    ) -> Vec<LunaEntityId> {
        let candidates = self.spatial_index.query_region(x, y, width, height);

        // Sort candidates by Z-order (children above parents)
        let mut depth_map: Vec<(LunaEntityId, usize)> = candidates
            .into_iter()
            .map(|entity| {
                let depth = ecs.hierarchy(cx).read(cx).get_parent_chain(entity).len();
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_hit_test_point() {
//         let mut ecs = LunaEcs::new();
//         let mut hit_test = HitTestSystem::new(100.0, 100.0);

//         // Create a parent-child hierarchy
//         let parent = ecs.create_entity();
//         let child = ecs.create_entity();

//         // Setup hierarchy
//         ecs.hierarchy_mut().set_parent(child, Some(parent));

//         // Add transforms
//         ecs.transforms_mut().set_transform(
//             parent,
//             LocalTransform {
//                 position: LocalPosition { x: 10.0, y: 10.0 },
//                 scale: Vector2D { x: 1.0, y: 1.0 },
//                 rotation: 0.0,
//             },
//         );

//         ecs.transforms_mut().set_transform(
//             child,
//             LocalTransform {
//                 position: LocalPosition { x: 5.0, y: 5.0 },
//                 scale: Vector2D { x: 1.0, y: 1.0 },
//                 rotation: 0.0,
//             },
//         );

//         // Update spatial index
//         hit_test.update_entity(&ecs, parent);
//         hit_test.update_entity(&ecs, child);

//         // Test hit testing - child should be on top
//         if let Some(hit) = hit_test.hit_test_point(&ecs, 15.0, 15.0) {
//             assert_eq!(hit, child);
//         }
//     }

//     #[test]
//     fn test_hit_test_region() {
//         let mut ecs = LunaEcs::new();
//         let mut hit_test = HitTestSystem::new(100.0, 100.0);

//         // Create some test entities
//         let e1 = ecs.create_entity();
//         let e2 = ecs.create_entity();

//         // Add transforms
//         ecs.transforms_mut().set_transform(
//             e1,
//             LocalTransform {
//                 position: LocalPosition { x: 10.0, y: 10.0 },
//                 scale: Vector2D { x: 1.0, y: 1.0 },
//                 rotation: 0.0,
//             },
//         );

//         ecs.transforms_mut().set_transform(
//             e2,
//             LocalTransform {
//                 position: LocalPosition { x: 20.0, y: 20.0 },
//                 scale: Vector2D { x: 1.0, y: 1.0 },
//                 rotation: 0.0,
//             },
//         );

//         // Update spatial index
//         hit_test.update_entity(&ecs, e1);
//         hit_test.update_entity(&ecs, e2);

//         // Test region query
//         let hits = hit_test.hit_test_region(&ecs, 0.0, 0.0, 30.0, 30.0);
//         assert_eq!(hits.len(), 2);
//     }
// }
