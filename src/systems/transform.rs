use crate::{ecs::LunaEcs, prelude::*};

/// System that handles computing and updating world transforms
pub struct TransformSystem {
    /// Entities that need their world transforms updated
    dirty_entities: Vec<LunaEntityId>,
}

impl TransformSystem {
    pub fn new() -> Self {
        TransformSystem {
            dirty_entities: Vec::new(),
        }
    }

    /// Marks an entity as needing its world transform updated
    pub fn mark_dirty(&mut self, entity: LunaEntityId) {
        if !self.dirty_entities.contains(&entity) {
            self.dirty_entities.push(entity);
        }
    }

    /// Updates world transforms for all dirty entities
    pub fn process(&mut self, ecs_mut: &mut LunaEcs) {
        // Take the dirty entities to process
        let entities_to_process: Vec<_> = self.dirty_entities.drain(..).collect();

        for entity in entities_to_process {
            // Skip if entity no longer exists
            if !ecs_mut.entity_exists(entity) {
                continue;
            }

            // Get parent chain and clone it
            let parent_chain = ecs_mut.hierarchy().get_parent_chain(entity).clone();

            // Use a scope to limit the mutable borrow
            let has_transform = {
                // Get a mutable borrow of transforms
                let transforms_mut = ecs_mut.transforms_mut();
                // Compute world transform and return if it was successful
                transforms_mut.compute_world_transform(entity, parent_chain).is_some()
            };

            // If we successfully computed a world transform
            if has_transform {
                // Now get the children (after the mutable borrow is released)
                if let Some(children) = ecs_mut.hierarchy().get_children(entity) {
                    for child in children.clone() {
                        self.mark_dirty(child);
                    }
                }
            }
        }
    }

    /// Marks an entity and all its descendants as dirty
    ///
    /// Call this inside a `ecs.update` closure to get a mutable reference
    /// to [`LunaEcs`] and it's context
    pub fn mark_branch_dirty(
        &mut self,
        ecs: &mut LunaEcs,
        root: LunaEntityId,
    ) {
        self.mark_dirty(root);

        if let Some(children) = ecs.hierarchy().get_children(root) {
            for child in children.clone() {
                self.mark_branch_dirty(ecs, child);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn test_transform_system(cx: &mut TestAppContext) {
        let ecs = cx.new(|cx| LunaEcs::new(cx));
        let mut transform_system = TransformSystem::new();

        ecs.update(cx, |ecs_mut, cx| {
            // Create a simple parent-child hierarchy
            let parent = ecs_mut.create_entity();
            let child = ecs_mut.create_entity();

            // Setup hierarchy
            ecs_mut.hierarchy_mut().set_parent(child, parent);

            // Add transforms in a scope to limit the borrow
            {
                let transforms_mut = ecs_mut.transforms_mut();

                transforms_mut.set_transform(
                    parent,
                    LocalTransform {
                        position: LocalPosition { x: 10.0, y: 10.0 },
                        scale: Vector2D { x: 2.0, y: 2.0 },
                        rotation: 0.0,
                    },
                );

                transforms_mut.set_transform(
                    child,
                    LocalTransform {
                        position: LocalPosition { x: 5.0, y: 5.0 },
                        scale: Vector2D { x: 1.5, y: 1.5 },
                        rotation: 0.0,
                    },
                );
            }

            transform_system.mark_dirty(parent);

            // Use the simplified process method
            transform_system.process(ecs_mut);

            // Get transforms component to verify results 
            let transforms_mut = ecs_mut.transforms_mut();

            // Verify world transforms were updated correctly
            if let Some(world_transform) =
                transforms_mut.compute_world_transform(child, vec![parent])
            {
                assert_eq!(world_transform.position.x, 20.0); // 10 + (5 * 2)
                assert_eq!(world_transform.position.y, 20.0); // 10 + (5 * 2)
                assert_eq!(world_transform.scale.x, 3.0); // 2 * 1.5
                assert_eq!(world_transform.scale.y, 3.0); // 2 * 1.5
            }
        });
    }
}
