use crate::{ecs::LunaEcs, prelude::*};

/// System that handles layout updates and constraints
pub struct LayoutSystem {
    /// Entities that need their layout recomputed
    dirty_entities: Vec<LunaEntityId>,
}

impl LayoutSystem {
    pub fn new() -> Self {
        LayoutSystem {
            dirty_entities: Vec::new(),
        }
    }

    /// Marks an entity as needing layout update
    pub fn mark_dirty(&mut self, entity: LunaEntityId) {
        if !self.dirty_entities.contains(&entity) {
            self.dirty_entities.push(entity);
        }
    }

    /// Updates layouts for all dirty entities
    pub fn process(&mut self, ecs: Entity<LunaEcs>, cx: &mut Context<LunaEcs>) {
        // Take the dirty entities to process
        let entities_to_process: Vec<_> = self.dirty_entities.drain(..).collect();

        for entity in entities_to_process {
            // Skip if entity no longer exists
            if !ecs.read(cx).entity_exists(entity) {
                continue;
            }

            // Get the layout properties
            if let Some(layout) = ecs.read(cx).layout(cx).read(cx).get_layout(entity) {
                // Apply size constraints
                let mut new_transform = if let Some(transform) =
                    ecs.read(cx).transforms(cx).read(cx).get_transform(entity)
                {
                    transform.clone()
                } else {
                    LocalTransform {
                        position: LocalPosition::default(),
                        scale: Vector2D::default(),
                        rotation: 0.0,
                    }
                };

                // Apply width constraint if specified
                if let Some(width) = layout.width {
                    new_transform.scale.x = width;

                    // Apply min/max constraints
                    if let Some(min_width) = layout.constraints.min_width {
                        new_transform.scale.x = new_transform.scale.x.max(min_width);
                    }
                    if let Some(max_width) = layout.constraints.max_width {
                        new_transform.scale.x = new_transform.scale.x.min(max_width);
                    }
                }

                // Apply height constraint if specified
                if let Some(height) = layout.height {
                    new_transform.scale.y = height;

                    // Apply min/max constraints
                    if let Some(min_height) = layout.constraints.min_height {
                        new_transform.scale.y = new_transform.scale.y.max(min_height);
                    }
                    if let Some(max_height) = layout.constraints.max_height {
                        new_transform.scale.y = new_transform.scale.y.min(max_height);
                    }
                }

                // Apply margins to position
                new_transform.position.x += layout.margins.left;
                new_transform.position.y += layout.margins.top;

                ecs.update(cx, |ecs, cx| {
                    // Update the transform
                    ecs.transforms(cx).update(cx, |transforms, cx| {
                        transforms.set_transform(entity, new_transform);
                    });

                    // Mark children as dirty since parent changed
                    if let Some(children) = ecs.hierarchy(cx).read(cx).get_children(entity) {
                        for child in children {
                            self.mark_dirty(*child);
                        }
                    }
                });
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
        cx: &mut Context<LunaEcs>,
    ) {
        self.mark_dirty(root);

        if let Some(children) = ecs.hierarchy(cx).read(cx).get_children(root) {
            for child in children.clone() {
                self.mark_branch_dirty(ecs, child, cx);
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_layout_system() {
//         let mut ecs = LunaEcs::new();
//         let mut layout_system = LayoutSystem::new();

//         // Create a test entity
//         let entity = ecs.create_entity();

//         // Add layout properties
//         let layout = LayoutProperties {
//             width: Some(100.0),
//             height: Some(50.0),
//             constraints: SizeConstraints {
//                 min_width: Some(50.0),
//                 max_width: Some(200.0),
//                 min_height: Some(25.0),
//                 max_height: Some(100.0),
//             },
//             margins: Margins {
//                 top: 10.0,
//                 right: 10.0,
//                 bottom: 10.0,
//                 left: 10.0,
//             },
//         };
//         ecs.layout_mut().set_layout(entity, layout);

//         // Mark entity for layout update
//         layout_system.mark_dirty(entity);

//         // Process layout updates
//         layout_system.process(&mut ecs);

//         // Verify transform was updated correctly
//         if let Some(transform) = ecs.transforms().get_transform(entity) {
//             assert_eq!(transform.scale.x, 100.0);
//             assert_eq!(transform.scale.y, 50.0);
//             assert_eq!(transform.position.x, 10.0); // Left margin
//             assert_eq!(transform.position.y, 10.0); // Top margin
//         }
//     }

//     #[test]
//     fn test_layout_constraints() {
//         let mut ecs = LunaEcs::new();
//         let mut layout_system = LayoutSystem::new();

//         let entity = ecs.create_entity();

//         // Add layout properties with constraints
//         let layout = LayoutProperties {
//             width: Some(250.0), // Exceeds max_width
//             height: Some(20.0), // Below min_height
//             constraints: SizeConstraints {
//                 min_width: Some(50.0),
//                 max_width: Some(200.0),
//                 min_height: Some(25.0),
//                 max_height: Some(100.0),
//             },
//             margins: Margins::default(),
//         };
//         ecs.layout_mut().set_layout(entity, layout);

//         // Process layout
//         layout_system.mark_dirty(entity);
//         layout_system.process(&mut ecs);

//         // Verify constraints were applied
//         if let Some(transform) = ecs.transforms().get_transform(entity) {
//             assert_eq!(transform.scale.x, 200.0); // Clamped to max_width
//             assert_eq!(transform.scale.y, 25.0); // Clamped to min_height
//         }
//     }
// }
