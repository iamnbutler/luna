//! Simplified node types for Luna.
//!
//! This crate provides a flat, non-hierarchical shape model.
//! Shapes are rendered in z-order (index in the list).

mod shape;
mod shape_id;

pub use shape::{Fill, Shape, ShapeKind, Stroke};
pub use shape_id::ShapeId;
