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
    pub fn process(&mut self, ecs: Entity<LunaEcs>, cx: &mut Context<Self>) {
        // Take the dirty entities to process
        let entities_to_process: Vec<_> = self.dirty_entities.drain(..).collect();

        for entity in entities_to_process {
            // Skip if entity no longer exists
            if !ecs.read(cx).entity_exists(entity) {
                continue;
            }

            ecs.update(cx, |ecs, cx| {
                let parent_chain = ecs.hierarchy().get_parent_chain(entity);

                // Update the world transform
                if let Some(world_transform) = ecs
                    .transforms_mut()
                    .compute_world_transform(entity, parent_chain)
                {
                    // Get and process any children to update their world transforms
                    if let Some(children) = ecs.hierarchy().get_children(entity) {
                        for child in children.clone() {
                            self.mark_dirty(child);
                        }
                    }
                }
            })
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
        cx: &mut Context<LunaEcs>,
    ) {
        self.mark_dirty(root);

        if let Some(children) = ecs.hierarchy().get_children(root) {
            for child in children.clone() {
                self.mark_branch_dirty(ecs, child, cx);
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
        let mut transform_system = cx.new(|cx| TransformSystem::new());

        ecs.update(cx, |ecs_mut, cx| {
            // Create a simple parent-child hierarchy
            let parent = ecs_mut.create_entity();
            let child = ecs_mut.create_entity();

            // Setup hierarchy
            ecs_mut.hierarchy_mut().set_parent(child, parent);

            // Add transforms
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

            transform_system.update(cx, |transform_system, cx| {
                // Mark parent as dirty
                transform_system.mark_dirty(parent);

                // Process updates
                transform_system.process(ecs.clone(), cx);
            });

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
