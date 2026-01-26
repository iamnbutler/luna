//! Target specification for commands.
//!
//! Commands need to specify which shapes they operate on.
//! This module defines flexible targeting that works with
//! current selection, specific IDs, or queries.

use node::ShapeId;
use serde::{Deserialize, Serialize};

/// Specifies which shapes a command targets.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    /// The current selection (most common for user actions).
    Selection,

    /// Specific shape by ID.
    Shape(ShapeId),

    /// Multiple specific shapes by ID.
    Shapes(Vec<ShapeId>),

    /// All shapes on the canvas.
    All,

    /// Shapes matching a query (future: by name, type, property, etc.).
    Query(ShapeQuery),
}

impl Default for Target {
    fn default() -> Self {
        Self::Selection
    }
}

impl From<ShapeId> for Target {
    fn from(id: ShapeId) -> Self {
        Self::Shape(id)
    }
}

impl From<Vec<ShapeId>> for Target {
    fn from(ids: Vec<ShapeId>) -> Self {
        Self::Shapes(ids)
    }
}

/// Query to find shapes by properties.
/// Extensible for future scene graph features.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShapeQuery {
    /// Shapes of a specific kind.
    ByKind(ShapeKindFilter),

    /// Shapes with a specific name (future).
    ByName(String),

    /// Shapes within a bounding box.
    InBounds {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },

    /// Shapes that are children of a target (future: scene graph).
    ChildrenOf(Box<Target>),

    /// Shapes that are parents of a target (future: scene graph).
    ParentOf(Box<Target>),
}

/// Filter for shape kinds.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShapeKindFilter {
    Rectangle,
    Ellipse,
    Frame,
    // Future: Text, Path, Group, etc.
}
