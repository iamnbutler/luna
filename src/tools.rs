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
use std::{fs, path::PathBuf};
use strum::Display;

#[derive(Default, Debug, Display, Clone, PartialEq)]
pub enum ToolKind {
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

impl ToolKind {
    pub fn src(self) -> SharedString {
        match self {
            ToolKind::Selection => "svg/arrow_pointer.svg".into(),
            ToolKind::Arrow => "svg/arrow_tool.svg".into(),
            ToolKind::Frame => "svg/frame.svg".into(),
            ToolKind::Hand => "svg/hand.svg".into(),
            ToolKind::Image => "svg/image.svg".into(),
            ToolKind::Line => "svg/line_tool.svg".into(),
            ToolKind::Pen => "svg/pen_tool.svg".into(),
            ToolKind::Pencil => "svg/pencil.svg".into(),
            ToolKind::Prompt => "svg/prompt.svg".into(),
            ToolKind::ElementLibrary => "svg/shapes.svg".into(),
            ToolKind::Rectangle => "svg/square.svg".into(),
            ToolKind::TextCursor => "svg/text_cursor.svg".into(),
            ToolKind::ZoomIn => "svg/zoom_in.svg".into(),
            ToolKind::ZoomOut => "svg/zoom_out.svg".into(),
        }
    }
}

/// Returns a [ToolButton]
pub fn tool_button(tool: ToolKind) -> ToolButton {
    ToolButton::new(tool)
}

#[derive(IntoElement)]
pub struct ToolButton {
    tool_kind: ToolKind,
    disabled: bool,
}

impl ToolButton {
    pub fn new(tool: ToolKind) -> Self {
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

        let tool_kind = self.tool_kind.clone();
        let selected = false;

        let icon_color = match (selected, self.disabled) {
            (true, true) => theme.selected.alpha(0.3),
            (true, false) => theme.selected,
            (false, true) => theme.foreground_disabled,
            (false, false) => theme.foreground_muted,
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
                div.hover(|div| div.bg(theme.foreground.opacity(0.05)))
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

#[derive(IntoElement)]
pub struct ToolStrip {}

impl ToolStrip {
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
                        .bg(theme.foreground.alpha(0.12)),
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
                    .child(tool_button(ToolKind::Selection))
                    .child(tool_button(ToolKind::Hand))
                    .child(tool_divider())
                    .child(tool_button(ToolKind::Prompt).disabled(true))
                    .child(tool_divider())
                    .child(tool_button(ToolKind::Pencil).disabled(true))
                    .child(tool_button(ToolKind::Pen).disabled(true))
                    .child(tool_button(ToolKind::TextCursor).disabled(true))
                    .child(tool_divider())
                    .child(tool_button(ToolKind::Frame).disabled(true))
                    .child(tool_button(ToolKind::Rectangle))
                    .child(tool_button(ToolKind::Line).disabled(true))
                    .child(tool_divider())
                    .child(tool_button(ToolKind::Image).disabled(true))
                    .child(tool_button(ToolKind::ElementLibrary).disabled(true))
                    .child(tool_divider())
                    .child(tool_button(ToolKind::Arrow).disabled(true)),
            )
            .child(
                div().w_full().flex().flex_col().items_center(), // .child(CurrentColorTool::new()),
            )
    }
}
