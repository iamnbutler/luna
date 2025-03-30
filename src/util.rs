#![allow(unused, dead_code)]

use gpui::{Keystroke, Modifiers, Pixels, Point};

/// Round a Pixels value to the nearest pixel
pub fn round_to_pixel(value: Pixels) -> Pixels {
    Pixels(value.0.round())
}

/// Create a Point where both x and y coordinates are rounded to pixels
pub fn rounded_point(x: Pixels, y: Pixels) -> Point<Pixels> {
    Point::new(round_to_pixel(x), round_to_pixel(y))
}

pub fn keystroke_builder(str: &str) -> Keystroke {
    let parts: Vec<&str> = str.split('-').collect();

    let mut modifiers = Modifiers {
        control: false,
        alt: false,
        shift: false,
        platform: false,
        function: false,
    };

    let mut key_char = None;

    // The last part is the key, everything before it is a modifier
    let key = if parts.is_empty() {
        ""
    } else {
        parts[parts.len() - 1]
    };

    for i in 0..parts.len() - 1 {
        match parts[i].to_lowercase().as_str() {
            "ctrl" | "control" => modifiers.control = true,
            "alt" | "option" => modifiers.alt = true,
            "shift" => modifiers.shift = true,
            "cmd" | "meta" | "command" | "platform" => modifiers.platform = true,
            "fn" | "function" => modifiers.function = true,
            _ => (),
        }
    }

    if !modifiers.control
        && !modifiers.alt
        && !modifiers.shift
        && !modifiers.platform
        && !modifiers.function
    {
        key_char = Some(key.to_string());
    }

    Keystroke {
        modifiers,
        key: key.into(),
        key_char,
    }
}
