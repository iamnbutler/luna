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
    pub fn process(&mut self, ecs: &mut LunaEcs) {
        // Take the dirty entities to process
        let entities_to_process: Vec<_> = self.dirty_entities.drain(..).collect();

        for entity in entities_to_process {
            // Skip if entity no longer exists
            if !ecs.entity_exists(entity) {
                continue;
            }

            // Update world transforms for this entity
            ecs.update_world_transforms(entity);

            // Mark children as dirty to update their transforms
            if let Some(children) = ecs.get_children(entity).cloned() {
                for child in children {
                    self.mark_dirty(child);
                }
            }
        }
    }

    /// Marks an entity and all its descendants as dirty
    pub fn mark_branch_dirty(&mut self, ecs: &LunaEcs, root: LunaEntityId) {
        self.mark_dirty(root);

        if let Some(children) = ecs.get_children(root) {
            for &child in children {
                self.mark_branch_dirty(ecs, child);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_system() {
        let mut ecs = LunaEcs::new();
        let mut transform_system = TransformSystem::new();

        // Create a simple parent-child hierarchy
        let parent = ecs.create_entity();
        let child = ecs.create_entity();

        // Setup hierarchy
        ecs.set_parent(child, parent);

        // Add transforms
        ecs.set_transform(
            parent,
            LocalTransform {
                position: LocalPosition { x: 10.0, y: 10.0 },
                scale: Vector2D { x: 2.0, y: 2.0 },
                rotation: 0.0,
            },
        );

        ecs.set_transform(
            child,
            LocalTransform {
                position: LocalPosition { x: 5.0, y: 5.0 },
                scale: Vector2D { x: 1.5, y: 1.5 },
                rotation: 0.0,
            },
        );

        // Mark parent as dirty and process updates
        transform_system.mark_dirty(parent);
        transform_system.process(&mut ecs);

        // Get the cached world transform
        if let Some(world_transform) = ecs.world_transform_cache.get(&child) {
            assert_eq!(world_transform.position.x, 20.0); // 10 + (5 * 2)
            assert_eq!(world_transform.position.y, 20.0); // 10 + (5 * 2)
            assert_eq!(world_transform.scale.x, 3.0); // 2 * 1.5
            assert_eq!(world_transform.scale.y, 3.0); // 2 * 1.5
        } else {
            panic!("World transform not computed for child");
        }
    }
}
