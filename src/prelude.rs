use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Creates a new 2D vector.
pub fn vec2(x: f32, y: f32) -> Vector2D {
    Vector2D { x, y }
}

/// Represents a vector in 2D space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

impl From<(f32, f32)> for Vector2D {
    fn from(tuple: (f32, f32)) -> Self {
        Vector2D {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl From<Vector2D> for (f32, f32) {
    fn from(vec: Vector2D) -> Self {
        (vec.x, vec.y)
    }
}

impl From<[f32; 2]> for Vector2D {
    fn from(array: [f32; 2]) -> Self {
        Vector2D {
            x: array[0],
            y: array[1],
        }
    }
}

impl From<Vector2D> for [f32; 2] {
    fn from(vec: Vector2D) -> Self {
        [vec.x, vec.y]
    }
}

impl Default for Vector2D {
    fn default() -> Self {
        Vector2D { x: 0.0, y: 0.0 }
    }
}

/// Represents a position in world space coordinates.
///
/// World position is absolute within the entire canvas, independent of hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorldPosition {
    pub x: f32,
    pub y: f32,
}

impl Default for WorldPosition {
    fn default() -> Self {
        WorldPosition { x: 0.0, y: 0.0 }
    }
}

impl From<(f32, f32)> for WorldPosition {
    fn from(tuple: (f32, f32)) -> Self {
        WorldPosition {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl From<[f32; 2]> for WorldPosition {
    fn from(array: [f32; 2]) -> Self {
        WorldPosition {
            x: array[0],
            y: array[1],
        }
    }
}

impl From<Vector2D> for WorldPosition {
    fn from(vec: Vector2D) -> Self {
        WorldPosition { x: vec.x, y: vec.y }
    }
}

/// Represents a position in local space coordinates.
///
/// Local position is relative to the parent element in the hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LocalPosition {
    pub x: f32,
    pub y: f32,
}

impl Default for LocalPosition {
    fn default() -> Self {
        LocalPosition { x: 0.0, y: 0.0 }
    }
}

impl From<(f32, f32)> for LocalPosition {
    fn from(tuple: (f32, f32)) -> Self {
        LocalPosition {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl From<[f32; 2]> for LocalPosition {
    fn from(array: [f32; 2]) -> Self {
        LocalPosition {
            x: array[0],
            y: array[1],
        }
    }
}

impl From<Vector2D> for LocalPosition {
    fn from(vec: Vector2D) -> Self {
        LocalPosition { x: vec.x, y: vec.y }
    }
}

/// Represents a local transform, containing position, scale, and rotation relative to the parent.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LocalTransform {
    pub position: LocalPosition,
    pub scale: Vector2D,
    pub rotation: f32,
}

/// Represents a world transform, containing absolute position, scale, and rotation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorldTransform {
    pub position: WorldPosition,
    pub scale: Vector2D,
    pub rotation: f32,
}

/// An unrotated, rectangular bounding box (AABB) whose edges are parallel to the coordinate axes.
///
/// Used for efficient collision detection and spatial partitioning.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    min: Vector2D,
    max: Vector2D,
}

impl BoundingBox {
    pub fn new(min: Vector2D, max: Vector2D) -> Self {
        BoundingBox { min, max }
    }

    pub fn min(&self) -> Vector2D {
        self.min
    }

    pub fn max(&self) -> Vector2D {
        self.max
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn half_width(&self) -> f32 {
        self.width() / 2.0
    }

    pub fn half_height(&self) -> f32 {
        self.height() / 2.0
    }

    pub fn center(&self) -> Vector2D {
        vec2(
            self.min.x + self.width() / 2.0,
            self.min.y + self.height() / 2.0,
        )
    }
}

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

/// A unique identifier for entities in the Luna ECS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LunaEntityId(u64);

impl From<u64> for LunaEntityId {
    fn from(id: u64) -> Self {
        LunaEntityId(id)
    }
}

impl From<LunaEntityId> for u64 {
    fn from(id: LunaEntityId) -> Self {
        id.0
    }
}