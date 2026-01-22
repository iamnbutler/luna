//! Luna commands - all operations that modify canvas state.
//!
//! Commands are intent-based: they describe what the user wants,
//! not how to achieve it. The execution layer handles:
//! - Propagation to children (future scene graph)
//! - Constraint satisfaction
//! - Undo/redo recording

use crate::Target;
use glam::Vec2;
use gpui::Hsla;
use node_2::ShapeKind;
use serde::{Deserialize, Serialize};

/// A command that modifies Luna canvas state.
///
/// Commands are serializable for:
/// - Recording macros/actions
/// - LLM generation
/// - Scripting
/// - Network sync (future)
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    // === Shape Creation ===
    /// Create a new shape.
    CreateShape {
        kind: ShapeKind,
        position: Vec2,
        size: Vec2,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fill: Option<ColorValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stroke: Option<StrokeValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        corner_radius: Option<f32>,
    },

    /// Duplicate target shapes with an offset.
    Duplicate {
        #[serde(default)]
        target: Target,
        #[serde(default = "default_duplicate_offset")]
        offset: Vec2,
    },

    /// Delete target shapes.
    Delete {
        #[serde(default)]
        target: Target,
    },

    // === Selection ===
    /// Select shapes, optionally adding to existing selection.
    Select {
        target: Target,
        #[serde(default)]
        add_to_selection: bool,
    },

    /// Clear the current selection.
    ClearSelection,

    /// Select all shapes.
    SelectAll,

    // === Transform ===
    /// Move shapes by a delta.
    Move {
        #[serde(default)]
        target: Target,
        delta: Vec2,
    },

    /// Set absolute position of shapes.
    SetPosition {
        #[serde(default)]
        target: Target,
        position: Vec2,
    },

    /// Resize shapes to a specific size.
    SetSize {
        #[serde(default)]
        target: Target,
        size: Vec2,
    },

    /// Scale shapes by a factor (relative resize).
    Scale {
        #[serde(default)]
        target: Target,
        factor: Vec2,
    },

    // === Style ===
    /// Set fill color.
    SetFill {
        #[serde(default)]
        target: Target,
        fill: Option<ColorValue>,
    },

    /// Set stroke style.
    SetStroke {
        #[serde(default)]
        target: Target,
        stroke: Option<StrokeValue>,
    },

    /// Set corner radius (rectangles).
    SetCornerRadius {
        #[serde(default)]
        target: Target,
        radius: f32,
    },

    // === Hierarchy ===
    /// Add a shape as a child of a frame.
    /// Converts the child's position to relative coordinates.
    AddChild {
        child: node_2::ShapeId,
        parent: node_2::ShapeId,
    },

    /// Remove shapes from their parent.
    /// Converts positions back to absolute coordinates.
    Unparent {
        #[serde(default)]
        target: Target,
    },

    /// Set whether a frame clips its children.
    SetClipChildren {
        #[serde(default)]
        target: Target,
        clip: bool,
    },

    // === Canvas ===
    /// Pan the viewport.
    Pan { delta: Vec2 },

    /// Zoom the viewport.
    Zoom {
        factor: f32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        center: Option<Vec2>,
    },

    /// Reset viewport to default.
    ResetView,

    // === Tool ===
    /// Switch the active tool.
    SetTool { tool: ToolKind },

    // === History (future) ===
    /// Undo the last command.
    Undo,

    /// Redo the last undone command.
    Redo,

    // === Batch ===
    /// Execute multiple commands in sequence.
    Batch { commands: Vec<Command> },
}

/// Color value for fill/stroke.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorValue {
    /// HSLA color.
    Hsla { h: f32, s: f32, l: f32, a: f32 },
    /// Hex color string (e.g., "#FF0000").
    Hex(HexColor),
}

/// Hex color wrapper for serde.
#[derive(Clone, Copy, Debug)]
pub struct HexColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Serialize for HexColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex = format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b);
        serializer.serialize_str(&hex)
    }
}

impl<'de> Deserialize<'de> for HexColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let s = s.trim_start_matches('#');
        if s.len() != 6 {
            return Err(serde::de::Error::custom("hex color must be 6 characters"));
        }
        let r = u8::from_str_radix(&s[0..2], 16).map_err(serde::de::Error::custom)?;
        let g = u8::from_str_radix(&s[2..4], 16).map_err(serde::de::Error::custom)?;
        let b = u8::from_str_radix(&s[4..6], 16).map_err(serde::de::Error::custom)?;
        Ok(HexColor { r, g, b })
    }
}

impl ColorValue {
    /// Convert to GPUI Hsla.
    pub fn to_hsla(self) -> Hsla {
        match self {
            ColorValue::Hsla { h, s, l, a } => gpui::hsla(h, s, l, a),
            ColorValue::Hex(hex) => {
                // Simple RGB to HSL conversion
                let r = hex.r as f32 / 255.0;
                let g = hex.g as f32 / 255.0;
                let b = hex.b as f32 / 255.0;

                let max = r.max(g).max(b);
                let min = r.min(g).min(b);
                let l = (max + min) / 2.0;

                if max == min {
                    return gpui::hsla(0.0, 0.0, l, 1.0);
                }

                let d = max - min;
                let s = if l > 0.5 {
                    d / (2.0 - max - min)
                } else {
                    d / (max + min)
                };

                let h = if max == r {
                    ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
                } else if max == g {
                    ((b - r) / d + 2.0) / 6.0
                } else {
                    ((r - g) / d + 4.0) / 6.0
                };

                gpui::hsla(h, s, l, 1.0)
            }
        }
    }
}

/// Stroke style value.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct StrokeValue {
    pub color: ColorValue,
    pub width: f32,
}

/// Tool kinds.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolKind {
    Select,
    Pan,
    Rectangle,
    Ellipse,
    Frame,
    // Future: Text, Pen, etc.
}

fn default_duplicate_offset() -> Vec2 {
    Vec2::new(20.0, 20.0)
}

/// Result of executing a command.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CommandResult {
    /// Command succeeded.
    Success {
        /// IDs of shapes created, if any.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        created: Vec<node_2::ShapeId>,
        /// IDs of shapes modified, if any.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        modified: Vec<node_2::ShapeId>,
        /// IDs of shapes deleted, if any.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        deleted: Vec<node_2::ShapeId>,
    },
    /// Command failed.
    Error {
        message: String,
    },
}

impl CommandResult {
    pub fn success() -> Self {
        Self::Success {
            created: vec![],
            modified: vec![],
            deleted: vec![],
        }
    }

    pub fn created(ids: Vec<node_2::ShapeId>) -> Self {
        Self::Success {
            created: ids,
            modified: vec![],
            deleted: vec![],
        }
    }

    pub fn modified(ids: Vec<node_2::ShapeId>) -> Self {
        Self::Success {
            created: vec![],
            modified: ids,
            deleted: vec![],
        }
    }

    pub fn deleted(ids: Vec<node_2::ShapeId>) -> Self {
        Self::Success {
            created: vec![],
            modified: vec![],
            deleted: ids,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_json_serialization() {
        // CreateShape command
        let cmd = Command::CreateShape {
            kind: ShapeKind::Rectangle,
            position: Vec2::new(100.0, 100.0),
            size: Vec2::new(50.0, 50.0),
            fill: Some(ColorValue::Hex(HexColor { r: 255, g: 0, b: 0 })),
            stroke: None,
            corner_radius: Some(8.0),
        };
        let json = serde_json::to_string_pretty(&cmd).unwrap();
        println!("CreateShape:\n{}", json);
        let _: Command = serde_json::from_str(&json).unwrap();

        // Move command with selection target
        let cmd = Command::Move {
            target: Target::Selection,
            delta: Vec2::new(10.0, 20.0),
        };
        let json = serde_json::to_string_pretty(&cmd).unwrap();
        println!("Move:\n{}", json);
        let _: Command = serde_json::from_str(&json).unwrap();

        // Batch command
        let cmd = Command::Batch {
            commands: vec![
                Command::CreateShape {
                    kind: ShapeKind::Rectangle,
                    position: Vec2::new(0.0, 0.0),
                    size: Vec2::new(100.0, 100.0),
                    fill: None,
                    stroke: None,
                    corner_radius: None,
                },
                Command::SetFill {
                    target: Target::Selection,
                    fill: Some(ColorValue::Hsla {
                        h: 0.5,
                        s: 1.0,
                        l: 0.5,
                        a: 1.0,
                    }),
                },
            ],
        };
        let json = serde_json::to_string_pretty(&cmd).unwrap();
        println!("Batch:\n{}", json);
        let _: Command = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_command_from_json_string() {
        // This is what an LLM might generate
        let json = r#"{
            "type": "create_shape",
            "kind": "Rectangle",
            "position": [100, 200],
            "size": [50, 50]
        }"#;
        let cmd: Command = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, Command::CreateShape { .. }));
    }
}
