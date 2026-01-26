//! Simplified node types for Luna.
//!
//! This crate provides a flat, non-hierarchical shape model.
//! Shapes are rendered in z-order (index in the list).

pub mod coords;
pub mod layout;
pub mod layout_engine;
mod shape;
mod shape_id;

pub use coords::{CanvasDelta, CanvasPoint, CanvasSize, LocalPoint, ScreenPoint};
pub use layout::{
    ChildLayout, CrossAxisAlignment, FrameLayout, LayoutDirection, MainAxisAlignment, Padding,
    SizingMode,
};
pub use layout_engine::{compute_layout, LayoutInput, LayoutOutput};
pub use shape::{Fill, Shape, ShapeKind, Stroke};
pub use shape_id::ShapeId;
