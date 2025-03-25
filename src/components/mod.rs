//! Components module for Luna's ECS architecture.
//! Each component is a data container that can be attached to entities.

pub mod hierarchy;
pub mod layout;
pub mod render;
pub mod transform;

pub use hierarchy::HierarchyComponent;
pub use layout::{LayoutComponent, LayoutProperties, Margins, SizeConstraints};
pub use render::{ElementStyle, RenderComponent};
pub use transform::TransformComponent;
