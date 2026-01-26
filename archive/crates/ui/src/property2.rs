use std::ops::Range;
use std::str::FromStr;

use gpui::{
    div, prelude::*, px, App, ClickEvent, Context, ElementId, Entity, FocusHandle, Focusable, Hsla,
    IntoElement, KeyDownEvent, ParentElement, Render, SharedString, Styled, Window,
};
use smallvec::SmallVec;

use canvas::{AppState, LunaCanvas};
use node::{NodeCommon, NodeId};
use theme::{ActiveTheme, Theme};

pub enum ColorType {
    Hex(String),
}

/// Represents the number types and constraints for numeric properties
pub enum NumberType {
    /// Any numeric value (positive, negative, or zero)
    Any,

    /// Positive numbers only (>= 0), useful for dimensions, opacity, etc.
    Positive,

    /// Integer value only (no decimals)
    Integer,

    /// Positive integer only (>= 0)
    PositiveInteger,
}

/// Represents the type and validation rules for a property value in the design tool
pub enum PropertyValueType {
    Number(NumberType),
    Fraction, // can handle fractions, percentages and angles (in radians)
    Boolean,
    Color(ColorType),
    /// Text/string value (for font families, content, etc.)
    Text,
}

pub enum Property {
    X,
    Y,
    Width,
    Height,
}

pub struct PropertyInput {
    id: ElementId,
    focus_handle: FocusHandle,
    property: Property,
    tooltip: Option<SharedString>,
    value: Option<PropertyValueType>,
}
