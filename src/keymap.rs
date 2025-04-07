use gpui::{App, KeyBinding};

use crate::{
    Cancel, Copy, Cut, Delete, FrameTool, HandTool, Paste, RectangleTool, SelectAll, SelectionTool,
};

pub fn init_keymap(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("h", HandTool, None),
        KeyBinding::new("a", SelectionTool, None),
        KeyBinding::new("r", RectangleTool, None),
        KeyBinding::new("f", FrameTool, None),
        KeyBinding::new("escape", Cancel, None),
        KeyBinding::new("cmd-a", SelectAll, None),
        KeyBinding::new("cmd-v", Paste, None),
        KeyBinding::new("cmd-c", Copy, None),
        KeyBinding::new("cmd-x", Cut, None),
        // Canvas
        KeyBinding::new("delete", Delete, None),
        KeyBinding::new("backspace", Delete, None),
        // Layer List
        KeyBinding::new("delete", Delete, Some("LayerList")),
        KeyBinding::new("backspace", Delete, Some("LayerList")),
    ]);
}
