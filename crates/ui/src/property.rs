//! Property editing components for the inspector panel.
//!
//! Provides reusable UI components for displaying and editing
//! element properties, with support for mixed-value states.

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

/// Input field for numeric property values with support for mixed states
///
/// Displays property values with appropriate formatting based on selection state:
/// - No value: Empty field
/// - Single value: Shows the exact value
/// - Multiple different values: Shows "Mixed"
#[derive(IntoElement)]
pub struct PropertyInput {
    value: Option<Vec<f32>>,
    icon: SharedString,
}

impl PropertyInput {
    pub fn new(value: Option<Vec<f32>>, icon: impl Into<SharedString>) -> Self {
        Self {
            value,
            icon: icon.into(),
        }
    }
}

impl RenderOnce for PropertyInput {
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = Theme::default();

        // Convert Option<Vec<f32>> to display string
        let display_value = match &self.value {
            None => SharedString::from(""),
            Some(values) if values.is_empty() => SharedString::from("Mixed"),
            Some(values) if values.len() == 1 => SharedString::from(format!("{}", values[0])),
            Some(_) => SharedString::from("Mixed"),
        };

        let no_value = display_value.is_empty();
        let mixed = display_value == "Mixed";

        div().flex().flex_row().child(
            div()
                .flex()
                .items_center()
                .flex_none()
                .pl(px(6.))
                .pr(px(4.))
                .w(px(84.))
                .rounded(px(4.))
                .bg(theme.tokens.surface0)
                .text_color(theme.tokens.text)
                .when(no_value || mixed, |this| {
                    this.text_color(theme.tokens.text.alpha(0.5))
                })
                .text_size(px(11.))
                .child(div().flex_1().child(display_value))
                .child(
                    div()
                        .flex()
                        .justify_center()
                        .flex_none()
                        .overflow_hidden()
                        .w(px(11.))
                        .h_full()
                        .child(self.icon),
                ),
        )
    }
}

/// Types of properties that can be edited
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropertyType {
    X,
    Y,
    Width,
    Height,
    BorderWidth,
    CornerRadius,
}

/// Interactive input field for numeric property values
pub struct InteractivePropertyInput {
    value: Option<Vec<f32>>,
    icon: SharedString,
    property: PropertyType,
    selected_nodes: Vec<NodeId>,
    canvas: Entity<LunaCanvas>,
    focus_handle: FocusHandle,
    content: String,
    selected_range: Range<usize>,
    is_selecting: bool,
    was_focused: bool,
}

impl InteractivePropertyInput {
    pub fn new(
        value: Option<Vec<f32>>,
        icon: impl Into<SharedString>,
        property: PropertyType,
        selected_nodes: Vec<NodeId>,
        canvas: Entity<LunaCanvas>,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();

        // Convert value to string for display
        let content = match &value {
            None => String::new(),
            Some(values) if values.is_empty() => String::new(),
            Some(values) if values.len() == 1 => format!("{}", values[0]),
            Some(_) => String::from("Mixed"),
        };

        Self {
            value,
            icon: icon.into(),
            property,
            selected_nodes,
            canvas,
            focus_handle,
            content: content.clone(),
            selected_range: 0..0,
            is_selecting: false,
            was_focused: false,
        }
    }

    pub fn update_value(
        &mut self,
        value: Option<Vec<f32>>,
        selected_nodes: Vec<NodeId>,
        cx: &mut Context<Self>,
    ) {
        self.value = value;
        self.selected_nodes = selected_nodes;

        // Update content to reflect new value
        self.content = match &self.value {
            None => String::new(),
            Some(values) if values.is_empty() => String::new(),
            Some(values) if values.len() == 1 => format!("{}", values[0]),
            Some(_) => String::from("Mixed"),
        };

        cx.notify();
    }

    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }

    fn apply_value(&mut self, cx: &mut Context<Self>) {
        // Try to parse the content as a float
        if let Ok(new_value) = self.content.parse::<f32>() {
            // Update the canvas with the new value
            self.canvas.update(cx, |canvas, cx| {
                for &node_id in &self.selected_nodes {
                    if let Some(node) = canvas.get_node_mut(node_id) {
                        match self.property {
                            PropertyType::X => {
                                let layout = node.layout_mut();
                                layout.x = new_value;
                            }
                            PropertyType::Y => {
                                let layout = node.layout_mut();
                                layout.y = new_value;
                            }
                            PropertyType::Width => {
                                let layout = node.layout_mut();
                                layout.width = new_value.max(1.0);
                            }
                            PropertyType::Height => {
                                let layout = node.layout_mut();
                                layout.height = new_value.max(1.0);
                            }
                            PropertyType::BorderWidth => {
                                let color = node.border_color();
                                node.set_border(color, new_value.max(0.0));
                            }
                            PropertyType::CornerRadius => {
                                node.set_corner_radius(new_value.max(0.0));
                            }
                        }
                    }
                }
                canvas.mark_dirty(cx);
            });

            // Update our stored value
            self.value = Some(vec![new_value]);
        }
    }

    fn insert_text(&mut self, text: &str) {
        // Only allow numeric input (digits, decimal point, minus sign)
        let filtered: String = text
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect();

        if !filtered.is_empty() {
            self.content
                .replace_range(self.selected_range.clone(), &filtered);
            let new_cursor = self.selected_range.start + filtered.len();
            self.selected_range = new_cursor..new_cursor;
        }
    }

    fn backspace(&mut self) {
        if self.selected_range.is_empty() && self.selected_range.start > 0 {
            let start = self.selected_range.start - 1;
            self.content.remove(start);
            self.selected_range = start..start;
        } else if !self.selected_range.is_empty() {
            self.content.replace_range(self.selected_range.clone(), "");
            let cursor = self.selected_range.start;
            self.selected_range = cursor..cursor;
        }
    }

    fn select_all(&mut self) {
        self.selected_range = 0..self.content.len();
    }

    fn move_cursor_left(&mut self) {
        if !self.selected_range.is_empty() {
            self.selected_range = self.selected_range.start..self.selected_range.start;
        } else if self.selected_range.start > 0 {
            let new_pos = self.selected_range.start - 1;
            self.selected_range = new_pos..new_pos;
        }
    }

    fn move_cursor_right(&mut self) {
        if !self.selected_range.is_empty() {
            self.selected_range = self.selected_range.end..self.selected_range.end;
        } else if self.selected_range.end < self.content.len() {
            let new_pos = self.selected_range.end + 1;
            self.selected_range = new_pos..new_pos;
        }
    }

    fn on_click(&mut self, _event: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.focus_handle.is_focused(window) {
            window.focus(&self.focus_handle);
            // Select all when first focusing
            self.select_all();
        }
        cx.notify();
    }
}

impl Focusable for InteractivePropertyInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for InteractivePropertyInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::default();
        let is_focused = self.focus_handle.is_focused(window);
        let no_value = self.content.is_empty();
        let mixed = self.content == "Mixed";

        // Check if focus state changed
        if self.was_focused && !is_focused {
            // Lost focus - apply value
            self.apply_value(cx);
            self.selected_range = 0..0;
        } else if !self.was_focused && is_focused {
            // Gained focus - select all
            self.selected_range = 0..self.content.len();
        }
        self.was_focused = is_focused;

        div()
            .id(ElementId::Name(
                format!("property-input-{:?}", self.property).into(),
            ))
            .key_context("PropertyInput")
            .track_focus(&self.focus_handle)
            .on_click(cx.listener(|this, event: &ClickEvent, window, cx| {
                this.on_click(event, window, cx);
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "enter" => {
                        this.apply_value(cx);
                        window.blur();
                    }
                    "escape" => {
                        // Reset to original value
                        this.content = match &this.value {
                            None => String::new(),
                            Some(values) if values.is_empty() => String::new(),
                            Some(values) if values.len() == 1 => format!("{}", values[0]),
                            Some(_) => String::from("Mixed"),
                        };
                        this.selected_range = 0..0;
                        window.blur();
                    }
                    "backspace" => this.backspace(),
                    "left" => this.move_cursor_left(),
                    "right" => this.move_cursor_right(),
                    "a" if event.keystroke.modifiers.platform => this.select_all(),
                    key => {
                        // Handle regular character input
                        if let Some(key_char) = &event.keystroke.key_char {
                            if !event.keystroke.modifiers.control
                                && !event.keystroke.modifiers.platform
                            {
                                // Filter to only allow numeric input
                                let filtered: String = key_char
                                    .chars()
                                    .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                                    .collect();

                                if !filtered.is_empty() {
                                    this.insert_text(&filtered);
                                }
                            }
                        }
                    }
                }
                cx.notify();
            }))
            .flex()
            .flex_row()
            .child(
                div()
                    .id(ElementId::Name(
                        format!("property-input-field-{:?}", self.property).into(),
                    ))
                    .flex()
                    .items_center()
                    .flex_none()
                    .pl(px(6.))
                    .pr(px(4.))
                    .w(px(84.))
                    .rounded(px(4.))
                    .bg(if is_focused {
                        theme.tokens.surface1
                    } else {
                        theme.tokens.surface0
                    })
                    .border_1()
                    .border_color(if is_focused {
                        theme.tokens.active_border
                    } else {
                        gpui::transparent_black()
                    })
                    .text_color(theme.tokens.text)
                    .when(no_value || mixed, |this| {
                        this.text_color(theme.tokens.text.alpha(0.5))
                    })
                    .text_size(px(11.))
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .relative()
                            .child(
                                div()
                                    .when(is_focused && !self.selected_range.is_empty(), |d| {
                                        d.bg(theme.tokens.selected.alpha(0.3))
                                    })
                                    .child(self.content.clone()),
                            )
                            .when(is_focused && self.selected_range.is_empty(), |d| {
                                // Show a simple cursor at the end when focused with no selection
                                d.child(
                                    div()
                                        .absolute()
                                        .right_0()
                                        .w(px(1.))
                                        .h_full()
                                        .bg(theme.tokens.text),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_center()
                            .flex_none()
                            .overflow_hidden()
                            .w(px(11.))
                            .h_full()
                            .child(self.icon.clone()),
                    ),
            )
    }
}

#[derive(IntoElement)]
pub struct ColorInput {
    value: Option<SharedString>,
    opacity: Option<f32>,
    icon: SharedString,
}

impl ColorInput {
    pub fn new(value: Option<SharedString>, icon: SharedString) -> Self {
        Self {
            value,
            opacity: None,
            icon,
        }
    }

    pub fn parse_color(&self) -> Option<Hsla> {
        if let Some(color_str) = &self.value {
            core::color::parse_color(color_str)
        } else {
            None
        }
    }
}

impl RenderOnce for ColorInput {
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = cx.theme();

        let parsed_color = self.parse_color();
        let mut display_value = String::new();
        if let Some(color) = parsed_color {
            display_value = color.to_string();
        };

        div().flex().flex_row().child(
            div()
                .flex()
                .items_center()
                .flex_none()
                .pl(px(6.))
                .pr(px(4.))
                .w_full()
                .rounded(px(4.))
                .bg(theme.tokens.surface0)
                .text_color(theme.tokens.text)
                .text_size(px(11.))
                .when_some(parsed_color, |this, color| {
                    this.child(
                        div()
                            .size(px(9.))
                            .border_color(cx.theme().tokens.inactive_border)
                            .border_1()
                            .rounded(px(2.))
                            .bg(color),
                    )
                })
                .when(!display_value.is_empty(), |this| {
                    this.child(div().flex_1().ml(px(4.)).child(display_value))
                })
                .child(div().flex_1())
                .child(
                    div()
                        .flex()
                        .justify_center()
                        .flex_none()
                        .overflow_hidden()
                        .w(px(11.))
                        .h_full()
                        .child(self.icon),
                ),
        )
    }
}
