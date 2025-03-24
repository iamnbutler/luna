use crate::prelude::*;
use std::collections::HashMap;

/// Main ECS manager for Luna that handles entities and their components
pub struct LunaEcs {
    /// Map of active entities and their generation counters
    entities: HashMap<LunaEntityId, u32>,

    /// Storage for local transform components, mapping entities to their local position, scale, and rotation
    transforms: HashMap<LunaEntityId, LocalTransform>,
    /// Cached world transforms for each entity, computed from local transforms and the hierarchy
    pub world_transform_cache: HashMap<LunaEntityId, WorldTransform>,

    /// Hierarchy data mapping entities to their parent entity
    parents: HashMap<LunaEntityId, LunaEntityId>,
    /// Hierarchy data mapping entities to their list of child entities
    children: HashMap<LunaEntityId, Vec<LunaEntityId>>,

    /// Visual appearance properties for entities, including dimensions, colors, and styling
    render_properties: HashMap<LunaEntityId, RenderProperties>,
    /// Cached bounding boxes for entities, used for layout and hit testing
    bounds_cache: HashMap<LunaEntityId, BoundingBox>,

    /// Layout configuration for entities, including sizing constraints and margins
    layouts: HashMap<LunaEntityId, LayoutProperties>,

    /// Counter for generating unique entity IDs
    next_entity_id: u64,
}

impl LunaEcs {
    pub fn new() -> Self {
        LunaEcs {
            entities: HashMap::new(),
            transforms: HashMap::new(),
            world_transform_cache: HashMap::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
            render_properties: HashMap::new(),
            bounds_cache: HashMap::new(),
            layouts: HashMap::new(),
            next_entity_id: 1,
        }
    }

    /// Creates a new entity and returns its ID
    pub fn create_entity(&mut self) -> LunaEntityId {
        let entity_id = LunaEntityId::from(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.insert(entity_id, 0);
        entity_id
    }

    /// Removes an entity and all its components
    pub fn remove_entity(&mut self, entity: LunaEntityId) {
        // Remove from hierarchy
        if let Some(parent) = self.parents.remove(&entity) {
            if let Some(siblings) = self.children.get_mut(&parent) {
                siblings.retain(|&sibling| sibling != entity);
            }
        }
        self.children.remove(&entity);

        // Remove transform data
        self.transforms.remove(&entity);
        self.world_transform_cache.remove(&entity);

        // Remove render data
        self.render_properties.remove(&entity);
        self.bounds_cache.remove(&entity);

        // Remove layout data
        self.layouts.remove(&entity);

        // Remove entity itself
        self.entities.remove(&entity);
    }

    // Transform component methods
    pub fn set_transform(&mut self, entity: LunaEntityId, transform: LocalTransform) {
        self.transforms.insert(entity, transform);
        self.world_transform_cache.remove(&entity);
    }

    pub fn get_transform(&self, entity: LunaEntityId) -> Option<&LocalTransform> {
        self.transforms.get(&entity)
    }

    // Hierarchy component methods
    pub fn set_parent(&mut self, child: LunaEntityId, parent: LunaEntityId) {
        // Remove from old parent
        if let Some(old_parent) = self.parents.get(&child) {
            if let Some(old_siblings) = self.children.get_mut(old_parent) {
                old_siblings.retain(|&sibling| sibling != child);
            }
        }

        // Set new parent
        self.parents.insert(child, parent);
        self.children
            .entry(parent)
            .or_insert_with(Vec::new)
            .push(child);
    }

    pub fn get_parent(&self, entity: LunaEntityId) -> Option<LunaEntityId> {
        self.parents.get(&entity).copied()
    }

    pub fn get_children(&self, entity: LunaEntityId) -> Option<&Vec<LunaEntityId>> {
        self.children.get(&entity)
    }

    /// Gets the full chain of parents for an entity, from immediate parent to root
    pub fn get_parent_chain(&self, entity: LunaEntityId) -> Vec<LunaEntityId> {
        let mut chain = Vec::new();
        let mut current = entity;

        while let Some(parent) = self.parents.get(&current).copied() {
            chain.push(parent);
            current = parent;
        }

        chain
    }

    /// Adds a child to a parent entity
    pub fn add_child(&mut self, parent: LunaEntityId, child: LunaEntityId) {
        // Remove child from its current parent if it has one
        if let Some(old_parent) = self.parents.get(&child) {
            if let Some(old_siblings) = self.children.get_mut(old_parent) {
                old_siblings.retain(|&sibling| sibling != child);
            }
        }

        // Set new parent-child relationship
        self.parents.insert(child, parent);
        self.children
            .entry(parent)
            .or_insert_with(Vec::new)
            .push(child);
    }

    /// Removes a child from its parent
    pub fn remove_child(&mut self, child: LunaEntityId) {
        if let Some(parent) = self.parents.remove(&child) {
            if let Some(siblings) = self.children.get_mut(&parent) {
                siblings.retain(|&sibling| sibling != child);
            }
        }
    }

    /// Checks if an entity is a descendant of another entity
    pub fn is_descendant_of(&self, entity: LunaEntityId, ancestor: LunaEntityId) -> bool {
        let mut current = entity;
        while let Some(parent) = self.parents.get(&current) {
            if *parent == ancestor {
                return true;
            }
            current = *parent;
        }
        false
    }

    /// Removes an entity completely from the hierarchy (both as a parent and as a child)
    pub fn remove_from_hierarchy(&mut self, entity: LunaEntityId) {
        // Remove as a child from its parent
        self.remove_child(entity);

        // Remove its children (they become parentless)
        if let Some(children) = self.children.remove(&entity) {
            for child in children {
                self.parents.remove(&child);
            }
        }
    }

    /// Converts a point from local space to world space
    pub fn local_to_world(
        &mut self,
        local_point: LocalPosition,
        entity: LunaEntityId,
    ) -> Option<WorldPosition> {
        // Update world transforms to ensure we have current data
        self.update_world_transforms(entity);

        // Get the cached world transform
        let world_transform = self.world_transform_cache.get(&entity)?;

        Some(WorldPosition {
            x: world_transform.position.x + (local_point.x * world_transform.scale.x),
            y: world_transform.position.y + (local_point.y * world_transform.scale.y),
        })
    }

    /// Converts a point from world space to local space
    pub fn world_to_local(
        &mut self,
        world_point: WorldPosition,
        entity: LunaEntityId,
    ) -> Option<LocalPosition> {
        // Update world transforms to ensure we have current data
        self.update_world_transforms(entity);

        // Get the cached world transform
        let world_transform = self.world_transform_cache.get(&entity)?;

        Some(LocalPosition {
            x: (world_point.x - world_transform.position.x) / world_transform.scale.x,
            y: (world_point.y - world_transform.position.y) / world_transform.scale.y,
        })
    }

    // Render component methods
    pub fn set_render_properties(&mut self, entity: LunaEntityId, properties: RenderProperties) {
        self.render_properties.insert(entity, properties);
        self.bounds_cache.remove(&entity);
    }

    pub fn get_render_properties(&self, entity: LunaEntityId) -> Option<&RenderProperties> {
        self.render_properties.get(&entity)
    }

    pub fn compute_bounds(&self, entity: LunaEntityId, position: Vector2D) -> Option<BoundingBox> {
        let props = self.render_properties.get(&entity)?;

        let min = Vector2D {
            x: position.x,
            y: position.y,
        };
        let max = Vector2D {
            x: position.x + props.width,
            y: position.y + props.height,
        };

        Some(BoundingBox::new(min, max))
    }

    pub fn get_bounds(&self, entity: LunaEntityId) -> Option<&BoundingBox> {
        self.bounds_cache.get(&entity)
    }

    pub fn update_bounds(&mut self, entity: LunaEntityId, bounds: BoundingBox) {
        self.bounds_cache.insert(entity, bounds);
    }

    pub fn invalidate_bounds(&mut self, entity: LunaEntityId) {
        self.bounds_cache.remove(&entity);
    }

    // Layout component methods
    pub fn set_layout(&mut self, entity: LunaEntityId, layout: LayoutProperties) {
        self.layouts.insert(entity, layout);
    }

    pub fn get_layout(&self, entity: LunaEntityId) -> Option<&LayoutProperties> {
        self.layouts.get(&entity)
    }

    pub fn remove_layout(&mut self, entity: LunaEntityId) {
        self.layouts.remove(&entity);
    }

    /// Checks if an entity exists
    pub fn entity_exists(&self, entity: LunaEntityId) -> bool {
        self.entities.contains_key(&entity)
    }

    /// Updates the world transforms for an entity and its descendants
    pub fn update_world_transforms(&mut self, root: LunaEntityId) {
        if !self.entity_exists(root) {
            return;
        }

        // Get the parent chain for this entity
        let mut parent_chain = Vec::new();
        let mut current = root;
        while let Some(parent) = self.parents.get(&current).copied() {
            parent_chain.push(parent);
            current = parent;
        }

        // Compute world transform
        if let Some(local) = self.transforms.get(&root).copied() {
            let mut world = WorldTransform {
                position: WorldPosition {
                    x: local.position.x,
                    y: local.position.y,
                },
                scale: local.scale,
                rotation: local.rotation,
            };

            // Apply parent transforms
            for parent_id in parent_chain.iter().rev() {
                if let Some(parent_transform) = self.transforms.get(parent_id) {
                    // Apply parent transform
                    world.position.x =
                        parent_transform.position.x + (world.position.x * parent_transform.scale.x);
                    world.position.y =
                        parent_transform.position.y + (world.position.y * parent_transform.scale.y);
                    world.scale.x *= parent_transform.scale.x;
                    world.scale.y *= parent_transform.scale.y;
                    world.rotation += parent_transform.rotation;
                }
            }

            // Cache the computed world transform
            self.world_transform_cache.insert(root, world);

            // Update children recursively
            if let Some(children) = self.children.get(&root).cloned() {
                for child in children {
                    self.update_world_transforms(child);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_management() {
        let mut ecs = LunaEcs::new();

        // Create entity
        let entity = ecs.create_entity();
        assert!(ecs.entity_exists(entity));

        ecs.remove_entity(entity);
        assert!(!ecs.entity_exists(entity));
    }

    #[test]
    fn test_render_properties() {
        let mut ecs = LunaEcs::new();
        let entity = ecs.create_entity();

        let properties = RenderProperties {
            width: 200.0,
            height: 150.0,
            corner_radius: 5.0,
            fill_color: [1.0, 0.0, 0.0, 1.0],   // Red
            stroke_color: [0.0, 0.0, 0.0, 1.0], // Black
            stroke_width: 2.0,
        };

        ecs.set_render_properties(entity, properties);

        let retrieved = ecs.get_render_properties(entity).unwrap();
        assert_eq!(retrieved.width, 200.0);
        assert_eq!(retrieved.height, 150.0);
        assert_eq!(retrieved.corner_radius, 5.0);

        // Test bounds computation
        let position = Vector2D { x: 10.0, y: 20.0 };
        let bounds = ecs.compute_bounds(entity, position).unwrap();

        assert_eq!(bounds.min().x, 10.0);
        assert_eq!(bounds.min().y, 20.0);
        assert_eq!(bounds.max().x, 210.0); // x + width
        assert_eq!(bounds.max().y, 170.0); // y + height
    }

    #[test]
    fn test_layout_properties() {
        let mut ecs = LunaEcs::new();
        let entity = ecs.create_entity();

        let layout = LayoutProperties {
            width: Some(100.0),
            height: Some(50.0),
            constraints: SizeConstraints {
                min_width: Some(50.0),
                max_width: Some(200.0),
                min_height: Some(25.0),
                max_height: Some(100.0),
            },
            margins: Margins {
                top: 10.0,
                right: 10.0,
                bottom: 10.0,
                left: 10.0,
            },
        };

        ecs.set_layout(entity, layout.clone());

        let retrieved = ecs.get_layout(entity).unwrap();
        assert_eq!(retrieved, &layout);

        ecs.remove_layout(entity);
        assert!(ecs.get_layout(entity).is_none());
    }

    #[test]
    fn test_hierarchy_operations() {
        let mut ecs = LunaEcs::new();

        let parent = ecs.create_entity();
        let child1 = ecs.create_entity();
        let child2 = ecs.create_entity();
        let grandchild = ecs.create_entity();

        // Test add_child
        ecs.add_child(parent, child1);
        ecs.add_child(parent, child2);
        ecs.add_child(child1, grandchild);

        assert_eq!(ecs.get_parent(child1), Some(parent));
        assert_eq!(ecs.get_parent(child2), Some(parent));
        assert_eq!(ecs.get_parent(grandchild), Some(child1));

        // Test is_descendant_of
        assert!(ecs.is_descendant_of(grandchild, parent));
        assert!(ecs.is_descendant_of(grandchild, child1));
        assert!(!ecs.is_descendant_of(grandchild, child2));

        // Test remove_child
        ecs.remove_child(child1);
        assert!(ecs.get_parent(child1).is_none());
        assert!(ecs.get_parent(grandchild).is_some()); // Grandchild still has parent

        // Test remove_from_hierarchy
        ecs.remove_from_hierarchy(child1);
        assert!(ecs.get_parent(grandchild).is_none()); // Grandchild is now parentless
    }

    #[test]
    fn test_coordinate_conversion() {
        let mut ecs = LunaEcs::new();

        // Create a parent-child hierarchy
        let parent = ecs.create_entity();
        let child = ecs.create_entity();

        ecs.set_parent(child, parent);

        // Set up transforms
        ecs.set_transform(
            parent,
            LocalTransform {
                position: LocalPosition { x: 100.0, y: 100.0 },
                scale: Vector2D { x: 2.0, y: 2.0 },
                rotation: 0.0,
            },
        );

        ecs.set_transform(
            child,
            LocalTransform {
                position: LocalPosition { x: 50.0, y: 50.0 },
                scale: Vector2D { x: 1.0, y: 1.0 },
                rotation: 0.0,
            },
        );

        // Test local to world conversion
        let local_point = LocalPosition { x: 10.0, y: 10.0 };
        if let Some(world_point) = ecs.local_to_world(local_point, child) {
            // World position should be: parent_pos + (child_pos + local_point) * parent_scale
            assert_eq!(world_point.x, 220.0); // 100 + (50 + 10) * 2
            assert_eq!(world_point.y, 220.0); // 100 + (50 + 10) * 2
        } else {
            panic!("Failed to convert local to world coordinates");
        }

        // Test world to local conversion
        let world_point = WorldPosition { x: 220.0, y: 220.0 };
        if let Some(local_point) = ecs.world_to_local(world_point, child) {
            assert_eq!(local_point.x, 10.0);
            assert_eq!(local_point.y, 10.0);
        } else {
            panic!("Failed to convert world to local coordinates");
        }
    }
}
