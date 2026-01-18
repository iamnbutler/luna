use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a shape.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShapeId(uuid::Uuid);

impl ShapeId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
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
