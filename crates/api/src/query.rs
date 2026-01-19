//! Luna queries - read-only access to canvas state.
//!
//! Queries allow inspecting the canvas without modifying it.
//! Useful for agents to understand current state before issuing commands.

use crate::Target;
use glam::Vec2;
use gpui::Hsla;
use node_2::{ShapeId, ShapeKind};
use serde::{Deserialize, Serialize};

/// A query for canvas state (read-only).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Query {
    /// Get the current selection.
    GetSelection,

    /// Get all shapes.
    GetAllShapes,

    /// Get shapes matching a target.
    GetShapes { target: Target },

    /// Get a specific shape by ID.
    GetShape { id: ShapeId },

    /// Get the canvas bounds (bounding box of all shapes).
    GetCanvasBounds,

    /// Get the current viewport state.
    GetViewport,

    /// Get the current tool.
    GetTool,

    /// Get shape count.
    GetShapeCount,
}

/// Response to a query.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QueryResult {
    /// Selection result.
    Selection { ids: Vec<ShapeId> },

    /// Shapes result.
    Shapes { shapes: Vec<ShapeInfo> },

    /// Single shape result.
    Shape { shape: Option<ShapeInfo> },

    /// Bounds result.
    Bounds {
        min: Option<Vec2>,
        max: Option<Vec2>,
    },

    /// Viewport result.
    Viewport { offset: Vec2, zoom: f32 },

    /// Tool result.
    Tool { tool: String },

    /// Count result.
    Count { count: usize },

    /// Error result.
    Error { message: String },
}

/// Serializable shape information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShapeInfo {
    pub id: ShapeId,
    pub kind: ShapeKind,
    pub position: Vec2,
    pub size: Vec2,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<FillInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<StrokeInfo>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub corner_radius: f32,
}

fn is_zero(f: &f32) -> bool {
    *f == 0.0
}

/// Serializable fill info.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FillInfo {
    pub color: ColorInfo,
}

/// Serializable stroke info.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrokeInfo {
    pub color: ColorInfo,
    pub width: f32,
}

/// Serializable color info (always HSLA for consistency).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorInfo {
    pub h: f32,
    pub s: f32,
    pub l: f32,
    pub a: f32,
}

impl From<Hsla> for ColorInfo {
    fn from(c: Hsla) -> Self {
        Self {
            h: c.h,
            s: c.s,
            l: c.l,
            a: c.a,
        }
    }
}
