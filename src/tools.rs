//! # Tool System
//!
//! This module implements the tool selection and interaction system for Luna.
//! It defines both the available tools and their UI representations.
//!
//! ## Architecture
//!
//! The tool system is built around three key components:
//!
//! - **ToolKind**: An enumeration representing the different interaction modes available in the app
//! - **ToolButton**: UI component for rendering individual tool buttons with appropriate styling
//! - **ToolStrip**: Container component that organizes tools into a vertical toolbar
//!
//! Tools are central to Luna's interaction model, determining how mouse and keyboard
//! input is interpreted when interacting with the canvas.

#![allow(unused, dead_code)]
use crate::canvas_element::CanvasElement;
use crate::GlobalState;
use crate::{canvas::LunaCanvas, theme::Theme};
use gpui::{
    actions, div, hsla, point, prelude::*, px, svg, App, Application, AssetSource, BoxShadow,
    ElementId, Entity, FocusHandle, Focusable, Global, Hsla, IntoElement, Keystroke, Menu,
    MenuItem, Modifiers, Pixels, Point, SharedString, TitlebarOptions, UpdateGlobal, WeakEntity,
    Window, WindowBackgroundAppearance, WindowOptions,
};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::{fs, path::PathBuf};
use strum::Display;

#[derive(Default, Debug, Display, Clone, Copy, PartialEq)]
pub enum Tool {
    /// Standard selection tool for clicking, dragging, and manipulating elements
    #[default]
    Selection,
    /// Tool for creating and editing connectors between elements
    ///
    /// Creates arrows that can either stand alone or connect elements while
    /// maintaining their connection when elements are moved.
    Arrow,
    /// Tool for creating organizational frames or artboards to group content
    Frame,
    /// Navigation tool for panning around the canvas by dragging
    Hand,
    /// Tool for inserting and manipulating images and image placeholders
    Image,
    /// Tool for drawing straight lines between two points
    Line,
    /// Vector tool for creating and editing bezier curves and paths
    Pen,
    /// Freehand tool for sketching and drawing with natural strokes
    Pencil,
    /// Tool for generating and modifying content using text prompts
    Prompt,
    /// Tool for quickly inserting saved elements such as icons, images and components
    ElementLibrary,
    /// Tool for drawing rectangles and squares of various dimensions
    Rectangle,
    /// Tool for adding, editing, and formatting text content
    TextCursor,
    /// Tool for increasing canvas magnification (zooming in)
    ZoomIn,
    /// Tool for decreasing canvas magnification (zooming out)
    ZoomOut,
}

impl Tool {
    pub fn src(self) -> SharedString {
        match self {
            Tool::Selection => "svg/arrow_pointer.svg".into(),
            Tool::Arrow => "svg/arrow_tool.svg".into(),
            Tool::Frame => "svg/frame.svg".into(),
            Tool::Hand => "svg/hand.svg".into(),
            Tool::Image => "svg/image.svg".into(),
            Tool::Line => "svg/line_tool.svg".into(),
            Tool::Pen => "svg/pen_tool.svg".into(),
            Tool::Pencil => "svg/pencil.svg".into(),
            Tool::Prompt => "svg/prompt.svg".into(),
            Tool::ElementLibrary => "svg/shapes.svg".into(),
            Tool::Rectangle => "svg/square.svg".into(),
            Tool::TextCursor => "svg/text_cursor.svg".into(),
            Tool::ZoomIn => "svg/zoom_in.svg".into(),
            Tool::ZoomOut => "svg/zoom_out.svg".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GlobalTool(pub Arc<Tool>);

impl Deref for GlobalTool {
    type Target = Arc<Tool>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for GlobalTool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Global for GlobalTool {}

pub trait ActiveTool {
    fn active_tool(&self) -> &Arc<Tool>;
}

impl ActiveTool for App {
    fn active_tool(&self) -> &Arc<Tool> {
        &self.global::<GlobalTool>().0
    }
}

/// Returns a [ToolButton]
pub fn tool_button(tool: Tool) -> ToolButton {
    ToolButton::new(tool)
}

/// UI component for rendering a tool selection button
///
/// ToolButton renders a selectable button with an icon corresponding to a specific tool.
/// It handles various visual states (normal, hover, selected, disabled) and provides
/// consistent styling across the application's tool interface.
///
/// The component adapts its appearance based on:
/// - The current selection state (whether this tool is currently active)
/// - Whether the tool is disabled (not yet implemented or currently unavailable)
/// - Theme-appropriate colors for each state
#[derive(IntoElement)]
pub struct ToolButton {
    /// The tool this button represents
    tool_kind: Tool,
    /// Whether this tool is currently unavailable
    disabled: bool,
}

impl ToolButton {
    pub fn new(tool: Tool) -> Self {
        ToolButton {
            tool_kind: tool,
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for ToolButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get_global(cx);
        let state = GlobalState::get(cx);
        let active_tool = cx.active_tool().clone();

        let tool_kind = self.tool_kind.clone();
        let selected = *active_tool == tool_kind;

        let icon_color = match (selected, self.disabled) {
            (true, true) => theme.tokens.active_border.alpha(0.5), // Use active_border for selected but disabled
            (true, false) => theme.tokens.active_border, // Use active_border for selected tools
            (false, true) => theme.tokens.overlay1,      // Use overlay1 for disabled tools
            (false, false) => theme.tokens.subtext0,     // Use subtext0 for normal tools
        };

        div()
            .id(ElementId::Name(tool_kind.to_string().into()))
            .size(px(25.))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .rounded(px(3.))
            .my_neg_1()
            .when(!self.disabled, |div| {
                div.hover(|div| div.bg(theme.tokens.surface1)) // Use surface1 for hover background
            })
            // .on_click(move |_, _, cx| {
            //     let tool_kind = tool_kind.clone();
            //     GlobalState::update_global(cx, |state, _| state.active_tool = tool_kind.clone())
            // })
            .child(
                svg()
                    .path(self.tool_kind.src())
                    .size(px(15.))
                    .text_color(icon_color),
            )
    }
}

// #[derive(IntoElement)]
// pub struct CurrentColorTool {}

// impl CurrentColorTool {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

// impl RenderOnce for CurrentColorTool {
//     fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
//         let theme = Theme::get_global(cx);
//         let state = GlobalState::get(cx);

//         div()
//             .id("current-color-tool")
//             .group("current-color-tool")
//             .relative()
//             .size(px(23.))
//             .mb_2()
//             .child(
//                 div()
//                     .id("current-forground-color")
//                     .absolute()
//                     .bottom_0()
//                     .right_0()
//                     .size(px(17.))
//                     .rounded(px(3.))
//                     .p_px()
//                     .bg(theme.background_color.blend(theme.foreground.alpha(0.32)))
//                     .shadow(smallvec::smallvec![BoxShadow {
//                         color: hsla(0.0, 0.0, 0.0, 0.24),
//                         offset: point(px(1.), px(0.)),
//                         blur_radius: px(0.),
//                         spread_radius: px(0.),
//                     }])
//                     .child(
//                         div()
//                             .rounded(px(2.))
//                             .size_full()
//                             .bg(state.current_border_color),
//                     ),
//             )
//             .child(
//                 div()
//                     .id("current-background-color")
//                     .absolute()
//                     .top_0()
//                     .left_0()
//                     .size(px(17.))
//                     .rounded(px(3.))
//                     .p_px()
//                     .bg(theme.background_color.blend(theme.foreground.alpha(0.32)))
//                     .shadow(smallvec::smallvec![BoxShadow {
//                         color: hsla(0.0, 0.0, 0.0, 0.36),
//                         offset: point(px(1.), px(1.)),
//                         blur_radius: px(0.),
//                         spread_radius: px(0.),
//                     }])
//                     .child(
//                         div()
//                             .rounded(px(2.))
//                             .size_full()
//                             .bg(state.current_background_color),
//                     ),
//             )
//     }
// }

/// Main toolbar component that organizes and displays available tools
///
/// ToolStrip creates a vertical strip of tool buttons, logically grouped with dividers
/// to create a cohesive and organized tool selection UI. It implements:
///
/// - Visual categorization of related tools (selection, drawing, shapes, etc.)
/// - Consistent spacing and alignment of tool buttons
/// - Theme-appropriate styling for the toolbar container
///
/// This component forms the primary tool selection interface in the application.
#[derive(IntoElement)]
pub struct ToolStrip {}

impl ToolStrip {
    /// Creates a new ToolStrip with default configuration
    pub fn new() -> Self {
        ToolStrip {}
    }
}

impl RenderOnce for ToolStrip {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get_global(cx);

        let tool_divider = || {
            div()
                .w_full()
                .flex()
                .items_center()
                .px(px(9.))
                .h(px(5.))
                .child(
                    div()
                        .h_px()
                        .w_full()
                        .rounded_full()
                        .bg(theme.tokens.overlay0), // Use overlay0 for dividers
                )
        };

        div()
            .id("tool_strip")
            .h_full()
            .w(px(35.))
            .flex()
            .flex_col()
            .items_center()
            .justify_between()
            .py(px(4.))
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(9.))
                    .child(tool_button(Tool::Selection))
                    .child(tool_button(Tool::Hand))
                    .child(tool_divider())
                    .child(tool_button(Tool::Prompt).disabled(true))
                    .child(tool_divider())
                    .child(tool_button(Tool::Pencil).disabled(true))
                    .child(tool_button(Tool::Pen).disabled(true))
                    .child(tool_button(Tool::TextCursor).disabled(true))
                    .child(tool_divider())
                    .child(tool_button(Tool::Frame).disabled(true))
                    .child(tool_button(Tool::Rectangle))
                    .child(tool_button(Tool::Line).disabled(true))
                    .child(tool_divider())
                    .child(tool_button(Tool::Image).disabled(true))
                    .child(tool_button(Tool::ElementLibrary).disabled(true))
                    .child(tool_divider())
                    .child(tool_button(Tool::Arrow).disabled(true)),
            )
            .child(
                div().w_full().flex().flex_col().items_center(), // .child(CurrentColorTool::new()),
            )
    }
}
