//! Text input components for Luna.
//!
//! Provides single-line and multi-line text editing capabilities.
//!
//! # Example
//!
//! ```ignore
//! use gpui::Context;
//! use ui::input::{InputState, input, bind_input_keys};
//!
//! // Initialize keybindings (typically in app initialization)
//! bind_input_keys(cx, None);
//!
//! // Create an input state
//! let input_state = cx.new(|cx| InputState::new_singleline(cx));
//!
//! // In your render function:
//! input(&input_state, cx)
//!     .placeholder("Enter text...")
//!     .w(px(200.0))
//! ```

mod bidi;
pub mod bindings;
mod blink;
mod element;
mod handler;
mod state;

pub use bidi::{detect_base_direction, TextDirection};
pub use bindings::{bind_input_keys, InputBindings, INPUT_CONTEXT};
pub use blink::CursorBlink;
pub use element::{input, text_area, Input, InputColors};
pub use handler::{ElementInputHandler, EntityInputHandler};
pub use state::{InputLineLayout, InputState, InputStateEvent};
