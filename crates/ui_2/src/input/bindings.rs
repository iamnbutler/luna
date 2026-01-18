//! Input keybinding configuration and actions.

use gpui::{actions, App, KeyBinding};

actions!(
    input,
    [
        Backspace,
        Delete,
        Escape,
        DeleteWordLeft,
        DeleteWordRight,
        DeleteToBeginningOfLine,
        DeleteToEndOfLine,
        Tab,
        Left,
        Right,
        Up,
        Down,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectDown,
        SelectAll,
        Home,
        End,
        SelectToBeginning,
        SelectToEnd,
        MoveToBeginning,
        MoveToEnd,
        Paste,
        Cut,
        Copy,
        Enter,
        WordLeft,
        WordRight,
        SelectWordLeft,
        SelectWordRight,
        Undo,
        Redo,
    ]
);

/// The key context used for input element keybindings.
pub const INPUT_CONTEXT: &str = "Input";

/// Keybindings configuration for input elements.
#[derive(Clone)]
pub struct InputBindings {
    pub backspace: Option<KeyBinding>,
    pub delete: Option<KeyBinding>,
    pub delete_word_left: Option<KeyBinding>,
    pub delete_word_right: Option<KeyBinding>,
    pub delete_to_beginning_of_line: Option<KeyBinding>,
    pub delete_to_end_of_line: Option<KeyBinding>,
    pub tab: Option<KeyBinding>,
    pub enter: Option<KeyBinding>,
    pub left: Option<KeyBinding>,
    pub right: Option<KeyBinding>,
    pub up: Option<KeyBinding>,
    pub down: Option<KeyBinding>,
    pub select_left: Option<KeyBinding>,
    pub select_right: Option<KeyBinding>,
    pub select_up: Option<KeyBinding>,
    pub select_down: Option<KeyBinding>,
    pub select_all: Option<KeyBinding>,
    pub home: Option<KeyBinding>,
    pub end: Option<KeyBinding>,
    pub move_to_beginning: Option<KeyBinding>,
    pub move_to_end: Option<KeyBinding>,
    pub select_to_beginning: Option<KeyBinding>,
    pub select_to_end: Option<KeyBinding>,
    pub word_left: Option<KeyBinding>,
    pub word_right: Option<KeyBinding>,
    pub select_word_left: Option<KeyBinding>,
    pub select_word_right: Option<KeyBinding>,
    pub copy: Option<KeyBinding>,
    pub cut: Option<KeyBinding>,
    pub paste: Option<KeyBinding>,
    pub undo: Option<KeyBinding>,
    pub redo: Option<KeyBinding>,
    pub escape: Option<KeyBinding>,
}

impl Default for InputBindings {
    fn default() -> Self {
        let context = Some(INPUT_CONTEXT);

        #[cfg(target_os = "macos")]
        {
            Self {
                backspace: Some(KeyBinding::new("backspace", Backspace, context)),
                delete: Some(KeyBinding::new("delete", Delete, context)),
                delete_word_left: Some(KeyBinding::new("alt-backspace", DeleteWordLeft, context)),
                delete_word_right: Some(KeyBinding::new("alt-delete", DeleteWordRight, context)),
                delete_to_beginning_of_line: Some(KeyBinding::new(
                    "cmd-backspace",
                    DeleteToBeginningOfLine,
                    context,
                )),
                delete_to_end_of_line: Some(KeyBinding::new("ctrl-k", DeleteToEndOfLine, context)),
                tab: Some(KeyBinding::new("tab", Tab, context)),
                enter: Some(KeyBinding::new("enter", Enter, context)),
                left: Some(KeyBinding::new("left", Left, context)),
                right: Some(KeyBinding::new("right", Right, context)),
                up: Some(KeyBinding::new("up", Up, context)),
                down: Some(KeyBinding::new("down", Down, context)),
                select_left: Some(KeyBinding::new("shift-left", SelectLeft, context)),
                select_right: Some(KeyBinding::new("shift-right", SelectRight, context)),
                select_up: Some(KeyBinding::new("shift-up", SelectUp, context)),
                select_down: Some(KeyBinding::new("shift-down", SelectDown, context)),
                select_all: Some(KeyBinding::new("cmd-a", SelectAll, context)),
                home: Some(KeyBinding::new("home", Home, context)),
                end: Some(KeyBinding::new("end", End, context)),
                move_to_beginning: Some(KeyBinding::new("cmd-up", MoveToBeginning, context)),
                move_to_end: Some(KeyBinding::new("cmd-down", MoveToEnd, context)),
                select_to_beginning: Some(KeyBinding::new(
                    "cmd-shift-up",
                    SelectToBeginning,
                    context,
                )),
                select_to_end: Some(KeyBinding::new("cmd-shift-down", SelectToEnd, context)),
                word_left: Some(KeyBinding::new("alt-left", WordLeft, context)),
                word_right: Some(KeyBinding::new("alt-right", WordRight, context)),
                select_word_left: Some(KeyBinding::new("alt-shift-left", SelectWordLeft, context)),
                select_word_right: Some(KeyBinding::new(
                    "alt-shift-right",
                    SelectWordRight,
                    context,
                )),
                copy: Some(KeyBinding::new("cmd-c", Copy, context)),
                cut: Some(KeyBinding::new("cmd-x", Cut, context)),
                paste: Some(KeyBinding::new("cmd-v", Paste, context)),
                undo: Some(KeyBinding::new("cmd-z", Undo, context)),
                redo: Some(KeyBinding::new("cmd-shift-z", Redo, context)),
                escape: Some(KeyBinding::new("escape", Escape, context)),
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Self {
                backspace: Some(KeyBinding::new("backspace", Backspace, context)),
                delete: Some(KeyBinding::new("delete", Delete, context)),
                delete_word_left: Some(KeyBinding::new("ctrl-backspace", DeleteWordLeft, context)),
                delete_word_right: Some(KeyBinding::new("ctrl-delete", DeleteWordRight, context)),
                delete_to_beginning_of_line: Some(KeyBinding::new(
                    "ctrl-shift-backspace",
                    DeleteToBeginningOfLine,
                    context,
                )),
                delete_to_end_of_line: Some(KeyBinding::new(
                    "ctrl-shift-delete",
                    DeleteToEndOfLine,
                    context,
                )),
                tab: Some(KeyBinding::new("tab", Tab, context)),
                enter: Some(KeyBinding::new("enter", Enter, context)),
                left: Some(KeyBinding::new("left", Left, context)),
                right: Some(KeyBinding::new("right", Right, context)),
                up: Some(KeyBinding::new("up", Up, context)),
                down: Some(KeyBinding::new("down", Down, context)),
                select_left: Some(KeyBinding::new("shift-left", SelectLeft, context)),
                select_right: Some(KeyBinding::new("shift-right", SelectRight, context)),
                select_up: Some(KeyBinding::new("shift-up", SelectUp, context)),
                select_down: Some(KeyBinding::new("shift-down", SelectDown, context)),
                select_all: Some(KeyBinding::new("ctrl-a", SelectAll, context)),
                home: Some(KeyBinding::new("home", Home, context)),
                end: Some(KeyBinding::new("end", End, context)),
                move_to_beginning: Some(KeyBinding::new("ctrl-home", MoveToBeginning, context)),
                move_to_end: Some(KeyBinding::new("ctrl-end", MoveToEnd, context)),
                select_to_beginning: Some(KeyBinding::new(
                    "ctrl-shift-home",
                    SelectToBeginning,
                    context,
                )),
                select_to_end: Some(KeyBinding::new("ctrl-shift-end", SelectToEnd, context)),
                word_left: Some(KeyBinding::new("ctrl-left", WordLeft, context)),
                word_right: Some(KeyBinding::new("ctrl-right", WordRight, context)),
                select_word_left: Some(KeyBinding::new("ctrl-shift-left", SelectWordLeft, context)),
                select_word_right: Some(KeyBinding::new(
                    "ctrl-shift-right",
                    SelectWordRight,
                    context,
                )),
                copy: Some(KeyBinding::new("ctrl-c", Copy, context)),
                cut: Some(KeyBinding::new("ctrl-x", Cut, context)),
                paste: Some(KeyBinding::new("ctrl-v", Paste, context)),
                undo: Some(KeyBinding::new("ctrl-z", Undo, context)),
                redo: Some(KeyBinding::new("ctrl-shift-z", Redo, context)),
                escape: Some(KeyBinding::new("escape", Escape, context)),
            }
        }
    }
}

impl InputBindings {
    /// Collects all `Some` bindings into a `Vec<KeyBinding>`.
    pub fn into_bindings(self) -> Vec<KeyBinding> {
        let mut bindings: Vec<Option<KeyBinding>> = vec![
            self.backspace,
            self.delete,
            self.delete_word_left,
            self.delete_word_right,
            self.delete_to_beginning_of_line,
            self.delete_to_end_of_line,
            self.tab,
            self.enter,
            self.left,
            self.right,
            self.up,
            self.down,
            self.select_left,
            self.select_right,
            self.select_up,
            self.select_down,
            self.select_all,
            self.home,
            self.end,
            self.move_to_beginning,
            self.move_to_end,
            self.select_to_beginning,
            self.select_to_end,
            self.word_left,
            self.word_right,
            self.select_word_left,
            self.select_word_right,
            self.copy,
            self.cut,
            self.paste,
            self.undo,
            self.redo,
            self.escape,
        ];

        #[cfg(target_os = "macos")]
        {
            let context = Some(INPUT_CONTEXT);
            bindings.push(Some(KeyBinding::new("cmd-left", Home, context)));
            bindings.push(Some(KeyBinding::new("cmd-right", End, context)));
        }

        bindings.into_iter().flatten().collect()
    }
}

/// Binds input keybindings to the application.
pub fn bind_input_keys(cx: &mut App, bindings: impl Into<Option<InputBindings>>) {
    let bindings = bindings.into().unwrap_or_default();
    cx.bind_keys(bindings.into_bindings());
}
