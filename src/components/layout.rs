use crate::prelude::*;

/// Represents size constraints for an element
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SizeConstraints {
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
}

impl Default for SizeConstraints {
    fn default() -> Self {
        SizeConstraints {
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
        }
    }
}

/// Represents margins around an element
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Margins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Default for Margins {
    fn default() -> Self {
        Margins {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }
}

/// Represents the layout properties for an element
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutProperties {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub constraints: SizeConstraints,
    pub margins: Margins,
}

impl Default for LayoutProperties {
    fn default() -> Self {
        LayoutProperties {
            width: None,
            height: None,
            constraints: SizeConstraints::default(),
            margins: Margins::default(),
        }
    }
}

/// Component that handles layout properties and constraints for entities
pub struct LayoutComponent {
    /// Map of entity IDs to their layout properties
    layouts: HashMap<LunaEntityId, LayoutProperties>,
}

impl LayoutComponent {
    pub fn new() -> Self {
        LayoutComponent {
            layouts: HashMap::new(),
        }
    }

    /// Sets the layout properties for an entity
    pub fn set_layout(&mut self, entity: LunaEntityId, layout: LayoutProperties) {
        self.layouts.insert(entity, layout);
    }

    /// Gets the layout properties for an entity
    pub fn get_layout(&self, entity: LunaEntityId) -> Option<&LayoutProperties> {
        self.layouts.get(&entity)
    }

    /// Removes layout properties for an entity
    pub fn remove(&mut self, entity: LunaEntityId) {
        self.layouts.remove(&entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_component() {
        let mut layout_component = LayoutComponent::new();
        let entity = LunaEntityId::from(1);

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

        layout_component.set_layout(entity, layout.clone());

        let retrieved = layout_component.get_layout(entity).unwrap();
        assert_eq!(retrieved, &layout);

        layout_component.remove(entity);
        assert!(layout_component.get_layout(entity).is_none());
    }
}