use gpui::actions;

mod input_map;
mod text_input;
pub use input_map::*;
pub use text_input::*;

actions!(
    text_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        ShowCharacterPalette,
        Paste,
        Cut,
        Copy,
        Enter,
    ]
);

// todo!("Don't leave any placeholders")
pub struct Placeholder;

/// Represents a user input structure
pub enum Input {
    TextInput(TextInput),
    SplitTextInput(Placeholder),
    Button(Placeholder),
    SplitButton(Placeholder),
    Dropdown(Placeholder),
}

/// Placeholder struct, will represent any input type and allow
/// dereferencing or downcasting to the underlying input type.
pub struct AnyInput(Box<Input>);
