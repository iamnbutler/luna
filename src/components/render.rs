use crate::prelude::*;

/// Visual properties for rendering an element
#[derive(Debug, Clone)]
pub struct RenderProperties {
    pub width: f32,
    pub height: f32,
    pub corner_radius: f32,
    pub fill_color: [f32; 4],   // RGBA
    pub stroke_color: [f32; 4], // RGBA
    pub stroke_width: f32,
}

impl Default for RenderProperties {
    fn default() -> Self {
        RenderProperties {
            width: 100.0,
            height: 100.0,
            corner_radius: 0.0,
            fill_color: [1.0, 1.0, 1.0, 1.0],   // White
            stroke_color: [0.0, 0.0, 0.0, 1.0], // Black
            stroke_width: 1.0,
        }
    }
}

/// Component that manages visual properties and computed bounds
pub struct RenderComponent {
    /// Map of entity IDs to their render properties
    properties: HashMap<LunaEntityId, RenderProperties>,
    /// Cache of computed bounding boxes
    bounds_cache: HashMap<LunaEntityId, BoundingBox>,
}

impl RenderComponent {
    pub fn new() -> Self {
        RenderComponent {
            properties: HashMap::new(),
            bounds_cache: HashMap::new(),
        }
    }

    /// Sets the render properties for an entity
    pub fn set_properties(&mut self, entity: LunaEntityId, properties: RenderProperties) {
        self.properties.insert(entity, properties);
        self.invalidate_bounds(entity);
    }

    /// Gets the render properties for an entity
    pub fn get_properties(&self, entity: LunaEntityId) -> Option<&RenderProperties> {
        self.properties.get(&entity)
    }

    /// Computes the bounding box for an entity based on its properties
    pub fn compute_bounds(&self, entity: LunaEntityId, position: Vector2D) -> Option<BoundingBox> {
        let props = self.properties.get(&entity)?;

        // Create bounding box from position and dimensions
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

    /// Gets the cached bounding box for an entity
    pub fn get_bounds(&self, entity: LunaEntityId) -> Option<&BoundingBox> {
        self.bounds_cache.get(&entity)
    }

    /// Updates the cached bounding box for an entity
    pub fn update_bounds(&mut self, entity: LunaEntityId, bounds: BoundingBox) {
        self.bounds_cache.insert(entity, bounds);
    }

    /// Invalidates the cached bounding box for an entity
    pub fn invalidate_bounds(&mut self, entity: LunaEntityId) {
        self.bounds_cache.remove(&entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_properties() {
        let mut render_component = RenderComponent::new();
        let entity = LunaEntityId::from(1);

        let properties = RenderProperties {
            width: 200.0,
            height: 150.0,
            corner_radius: 5.0,
            fill_color: [1.0, 0.0, 0.0, 1.0],   // Red
            stroke_color: [0.0, 0.0, 0.0, 1.0], // Black
            stroke_width: 2.0,
        };

        render_component.set_properties(entity, properties);

        let retrieved = render_component.get_properties(entity).unwrap();
        assert_eq!(retrieved.width, 200.0);
        assert_eq!(retrieved.height, 150.0);
        assert_eq!(retrieved.corner_radius, 5.0);
    }

    #[test]
    fn test_bounds_computation() {
        let mut render_component = RenderComponent::new();
        let entity = LunaEntityId::from(1);

        let properties = RenderProperties {
            width: 100.0,
            height: 50.0,
            ..Default::default()
        };

        render_component.set_properties(entity, properties);

        let position = Vector2D { x: 10.0, y: 20.0 };
        let bounds = render_component.compute_bounds(entity, position).unwrap();

        assert_eq!(bounds.min().x, 10.0);
        assert_eq!(bounds.min().y, 20.0);
        assert_eq!(bounds.max().x, 110.0);
        assert_eq!(bounds.max().y, 70.0);
    }
}
