use gpui::{App, AppContext as _};

use crate::prelude::*;

/// Main ECS manager for Luna that handles entities and their components
pub struct LunaEcs {
    /// Map of active entities and their generation counters
    entities: HashMap<LunaEntityId, u32>,
    /// Component storage
    transform_components: TransformComponent,
    hierarchy_components: HierarchyComponent,
    render_components: RenderComponent,
    layout_components: LayoutComponent,
    /// Counter for generating unique entity IDs
    next_entity_id: u64,
}

impl LunaEcs {
    pub fn new(cx: &mut Context<LunaEcs>) -> Self {
        let transform_components = TransformComponent::new();
        let hierarchy_components = HierarchyComponent::new();
        let render_components = RenderComponent::new();
        let layout_components = LayoutComponent::new();

        LunaEcs {
            entities: HashMap::new(),
            transform_components,
            hierarchy_components,
            render_components,
            layout_components,
            next_entity_id: 1,
        }
    }

    /// Returns a reference to the entity map
    pub fn entities(&self) -> &HashMap<LunaEntityId, u32> {
        &self.entities
    }

    /// Creates a new entity and returns its ID
    pub fn create_entity(&mut self) -> LunaEntityId {
        let entity_id = LunaEntityId::from(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.insert(entity_id, 0);
        entity_id
    }

    /// Removes an entity and all its components
    pub fn remove_entity(&mut self, entity: LunaEntityId, cx: &mut Context<Self>) {
        self.hierarchy_components.remove_child(entity);
        self.hierarchy_components.remove(entity);
        self.transform_components.remove(entity);
        self.render_components.remove(entity);
        self.layout_components.remove(entity);

        // Remove entity itself
        self.entities.remove(&entity);
    }

    /// Gets a reference to the transform component storage
    pub fn transforms(&self) -> &TransformComponent {
        &self.transform_components
    }

    /// Gets a mutable reference to the transform component storage
    pub fn transforms_mut(&mut self) -> &mut TransformComponent {
        &mut self.transform_components
    }

    /// Gets a reference to the hierarchy component storage
    pub fn hierarchy(&self) -> &HierarchyComponent {
        &self.hierarchy_components
    }

    /// Gets a mutable reference to the hierarchy component storage
    pub fn hierarchy_mut(&mut self) -> &mut HierarchyComponent {
        &mut self.hierarchy_components
    }

    /// Gets a reference to the render component storage
    pub fn render(&self) -> &RenderComponent {
        &self.render_components
    }

    /// Gets a mutable reference to the render component storage
    pub fn render_mut(&mut self) -> &mut RenderComponent {
        &mut self.render_components
    }

    /// Gets a reference to the layout component storage
    pub fn layout(&self) -> &LayoutComponent {
        &self.layout_components
    }

    /// Gets a mutable reference to the layout component storage
    pub fn layout_mut(&mut self) -> &mut LayoutComponent {
        &mut self.layout_components
    }

    /// Checks if an entity exists
    pub fn entity_exists(&self, entity: LunaEntityId) -> bool {
        self.entities.contains_key(&entity)
    }

    /// Updates the world transforms for an entity and its descendants
    pub fn update_world_transforms(&mut self, root: LunaEntityId, cx: &mut Context<Self>) {
        if !self.entity_exists(root) {
            return;
        }

        // Get the parent chain for this entity
        let parent_chain = self.hierarchy().get_parent_chain(root);

        // Update the world transform for this entity
        self.transforms_mut()
            .compute_world_transform(root, parent_chain);

        // Get any children
        // Get and clone the children to avoid borrowing issues
        let children = if let Some(children) = self.hierarchy().get_children(root) {
            children.clone()
        } else {
            Vec::new()
        };

        // Recursively update children
        for child in children {
            self.update_world_transforms(child, cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestAppContext;

    use super::*;
    use crate::prelude::*;

    #[gpui::test]
    fn test_entity_management(cx: &mut TestAppContext) {
        let ecs = cx.new(|cx| LunaEcs::new(cx));

        ecs.update(cx, |ecs, cx| {
            // Create entity
            let entity = ecs.create_entity();
            assert!(ecs.entity_exists(entity));

            ecs.remove_entity(entity, cx);
            assert!(!ecs.entity_exists(entity));
        });
    }

    // #[gpui::test]
    // fn test_component_access(cx: &mut TestAppContext) {
    //     let mut ecs = LunaEcs::new(cx);
    //     let entity = ecs.create_entity();

    //     // Add transform component
    //     let transform = LocalTransform {
    //         position: LocalPosition { x: 10.0, y: 20.0 },
    //         scale: Vector2D { x: 1.0, y: 1.0 },
    //         rotation: 0.0,
    //     };

    //     ecs.transforms_mut().set_transform(entity, transform);

    //     // Verify component exists
    //     assert!(ecs.transforms().get_transform(entity).is_some());
    // }
}
