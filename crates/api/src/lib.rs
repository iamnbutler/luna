//! Luna API - Command and query interface for Luna canvas operations.
//!
//! This crate defines the typed command language for all Luna operations.
//! Commands represent user intent and are:
//! - Serializable (for recording, LLM generation, scripting)
//! - Intent-based (what to do, not how to do it)
//! - Future-proof (system handles propagation, undo, etc.)

mod command;
mod query;
mod target;

pub use command::*;
pub use query::*;
pub use target::*;
