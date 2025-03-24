//! Components module for Luna's ECS architecture.
//! Each component is a data container that can be attached to entities.

// Re-export common types from prelude instead of component files
// since we've moved to a flattened ECS architecture
pub use crate::prelude::{
    RenderProperties,
    LocalTransform, WorldTransform, LocalPosition, WorldPosition,
    LayoutProperties, SizeConstraints, Margins,
    BoundingBox, Vector2D
};

// Note: In our flattened ECS architecture, all component functionality has been
// moved directly into the LunaEcs struct. These types are re-exported here
// for backward compatibility.
