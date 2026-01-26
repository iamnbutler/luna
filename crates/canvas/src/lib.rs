//! Simplified canvas for Luna.
//!
//! Provides a flat canvas with basic shape rendering and interaction.

mod canvas;
mod element;
mod viewport;

pub use canvas::{Canvas, CanvasEvent, DragState, ResizeHandle, Tool};
pub use element::CanvasElement;
// Re-export coordinate types from node for convenience
pub use node::{CanvasDelta, CanvasPoint, CanvasSize, LocalPoint, ScreenPoint};
pub use viewport::Viewport;
