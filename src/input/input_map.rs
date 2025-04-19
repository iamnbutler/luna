use std::collections::HashMap;

use gpui::{App, AppContext, ElementId, Entity};

use super::TextInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputMapKey {
    PositionX,
    PositionY,
    Width,
    Height,
}

impl InputMapKey {
    pub fn to_element_id(&self) -> ElementId {
        match self {
            InputMapKey::PositionX => ElementId::from("input-x"),
            InputMapKey::PositionY => ElementId::from("input-y"),
            InputMapKey::Width => ElementId::from("input-width"),
            InputMapKey::Height => ElementId::from("input-height"),
        }
    }
}

impl std::fmt::Display for InputMapKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputMapKey::PositionX => write!(f, "X"),
            InputMapKey::PositionY => write!(f, "Y"),
            InputMapKey::Width => write!(f, "Width"),
            InputMapKey::Height => write!(f, "Height"),
        }
    }
}

pub struct InputMap {
    map: HashMap<InputMapKey, Entity<TextInput>>,
}

impl InputMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn new_input(
        mut self,
        key: InputMapKey,
        cx: &mut App,
        f: impl FnOnce(&mut TextInput, &mut App),
    ) -> Self {
        let element_id = key.to_element_id();
        let placeholder = key.to_string();
        let mut input = TextInput::new(element_id, placeholder, cx);
        f(&mut input, cx);
        self.map.insert(key, cx.new(|cx| input));
        self
    }

    pub fn get_input(&self, key: InputMapKey) -> Option<Entity<TextInput>> {
        self.map.get(&key).cloned()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity<TextInput>> {
        self.map.values()
    }
}
