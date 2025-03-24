//! Components module for Luna's ECS architecture.
//! Each component is a data container that can be attached to entities.

pub mod hierarchy;
pub mod render;
pub mod transform;
pub mod layout;

pub use hierarchy::HierarchyComponent;
pub use render::{RenderComponent, RenderProperties};
pub use transform::TransformComponent;
pub use layout::{LayoutComponent, LayoutProperties, SizeConstraints, Margins};
