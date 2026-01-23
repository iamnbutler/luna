use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a shape.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShapeId(uuid::Uuid);

impl ShapeId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// Create a ShapeId from a string (UUID or short ID).
    /// If parsing fails, generates a new ID.
    pub fn from_str(s: &str) -> Self {
        // Try parsing as full UUID first
        if let Ok(uuid) = uuid::Uuid::parse_str(s) {
            return Self(uuid);
        }
        // Otherwise generate new (we don't store short form)
        Self::new()
    }

    /// Get the full UUID string.
    pub fn to_uuid_string(&self) -> String {
        self.0.to_string()
    }

    /// Create a ShapeId from a u128 (useful for tests).
    pub fn from_u128(value: u128) -> Self {
        Self(uuid::Uuid::from_u128(value))
    }
}

impl Default for ShapeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ShapeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ShapeId({})", &self.0.to_string()[..8])
    }
}

impl fmt::Display for ShapeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0.to_string()[..8])
    }
}
