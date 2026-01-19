//! Luna API - Command and query interface for Luna canvas operations.
//!
//! This crate defines the typed command language for all Luna operations.
//! Commands represent user intent and are:
//! - Serializable (for recording, LLM generation, scripting)
//! - Intent-based (what to do, not how to do it)
//! - Future-proof (system handles propagation, undo, etc.)
//!
//! # Example
//! ```ignore
//! use api::{Command, execute_command};
//!
//! let cmd = Command::CreateShape {
//!     kind: ShapeKind::Rectangle,
//!     position: Vec2::new(100.0, 100.0),
//!     size: Vec2::new(50.0, 50.0),
//!     fill: None,
//!     stroke: None,
//!     corner_radius: None,
//! };
//! let result = execute_command(&canvas, cmd, cx);
//! ```

mod command;
mod executor;
mod query;
mod target;

pub use command::*;
pub use executor::{execute_command, execute_query};
pub use query::*;
pub use target::*;
