use std::{fs, path::PathBuf};

use gpui::{
    actions, div, hsla, point, prelude::*, px, svg, App, Application, AssetSource, BoxShadow,
    ElementId, FocusHandle, Focusable, Global, Hsla, IntoElement, Keystroke, Menu, MenuItem,
    Modifiers, Point, SharedString, Size, TitlebarOptions, UpdateGlobal, Window,
    WindowBackgroundAppearance, WindowOptions,
};

use anyhow::Result;
use strum::Display;

actions!(
    luna,
    [
        Quit,
        ToggleUI,
        HandTool,
        SelectionTool,
        ResetCurrentColors,
        SwapCurrentColors
    ]
);

const TITLEBAR_HEIGHT: f32 = 31.;

pub struct Theme {
    pub background_color: Hsla,
    pub canvas_color: Hsla,
    pub foreground: Hsla,
    pub foreground_muted: Hsla,
    pub foreground_disabled: Hsla,
    pub selected: Hsla,
}

impl Theme {
    pub fn new() -> Self {
        Theme {
            background_color: hsla(222.0 / 360.0, 0.12, 0.2, 1.0),
            canvas_color: hsla(224.0 / 360., 0.12, 0.19, 1.0),
            foreground: hsla(0.0, 1.0, 1.0, 1.0),
            foreground_muted: hsla(0.0, 1.0, 1.0, 0.6),
            foreground_disabled: hsla(0.0, 1.0, 1.0, 0.3),
            selected: hsla(210.0 / 360.0, 0.92, 0.65, 1.0),
        }
    }
}

impl Theme {
    pub fn get_global(cx: &App) -> &Theme {
        cx.global::<Theme>()
    }
}

impl Global for Theme {}

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

#[derive(Debug, Display, Clone, PartialEq)]
pub enum Icon {
    ArrowCounterClockwise,
    ArrowDownRight,
    Frame,
    Image,
    Path,
    Square,
    Text,
}

impl Icon {
    pub fn src(self) -> SharedString {
        match self {
            Icon::ArrowCounterClockwise => "svg/arrow_counter_clockwise.svg".into(),
            Icon::ArrowDownRight => "svg/arrow_down_right.svg".into(),
            Icon::Frame => "svg/frame.svg".into(),
            Icon::Image => "svg/image.svg".into(),
            Icon::Path => "svg/pen_tool.svg".into(),
            Icon::Square => "svg/square.svg".into(),
            Icon::Text => "svg/text_cursor.svg".into(),
        }
    }
}

#[derive(Default, Debug, Display, Clone, PartialEq)]
pub enum ToolKind {
    /// Standard selection tool for clicking, dragging, and manipulating elements
    #[default]
    ArrowPointer,
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
    Square,
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
            ToolKind::ArrowPointer => "svg/arrow_pointer.svg".into(),
            ToolKind::Arrow => "svg/arrow_tool.svg".into(),
            ToolKind::Frame => "svg/frame.svg".into(),
            ToolKind::Hand => "svg/hand.svg".into(),
            ToolKind::Image => "svg/image.svg".into(),
            ToolKind::Line => "svg/line_tool.svg".into(),
            ToolKind::Pen => "svg/pen_tool.svg".into(),
            ToolKind::Pencil => "svg/pencil.svg".into(),
            ToolKind::Prompt => "svg/prompt.svg".into(),
            ToolKind::ElementLibrary => "svg/shapes.svg".into(),
            ToolKind::Square => "svg/square.svg".into(),
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
        let selected = state.active_tool == tool_kind;

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
            .on_click(move |_, _, cx| {
                let tool_kind = tool_kind.clone();
                GlobalState::update_global(cx, |state, _| state.active_tool = tool_kind.clone())
            })
            .child(
                svg()
                    .path(self.tool_kind.src())
                    .size(px(15.))
                    .text_color(icon_color),
            )
    }
}

#[derive(IntoElement)]
pub struct CurrentColorTool {
    border: Hsla,
    background: Hsla,
}

impl CurrentColorTool {
    pub fn new() -> Self {
        Self {
            border: gpui::white(),
            background: gpui::black(),
        }
    }
}

impl RenderOnce for CurrentColorTool {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get_global(cx);

        div()
            .id("current-color-tool")
            .group("current-color-tool")
            .relative()
            .size(px(23.))
            .mb_2()
            .child(
                div()
                    .id("current-forground-color")
                    .absolute()
                    .bottom_0()
                    .right_0()
                    .size(px(17.))
                    .rounded(px(3.))
                    .p_px()
                    .bg(theme.background_color.blend(theme.foreground.alpha(0.32)))
                    .shadow(smallvec::smallvec![BoxShadow {
                        color: hsla(0.0, 0.0, 0.0, 0.24),
                        offset: point(px(1.), px(0.)),
                        blur_radius: px(0.),
                        spread_radius: px(0.),
                    }])
                    .child(div().rounded(px(2.)).size_full().bg(self.border)),
            )
            .child(
                div()
                    .id("current-background-color")
                    .absolute()
                    .top_0()
                    .left_0()
                    .size(px(17.))
                    .rounded(px(3.))
                    .p_px()
                    .bg(theme.background_color.blend(theme.foreground.alpha(0.32)))
                    .shadow(smallvec::smallvec![BoxShadow {
                        color: hsla(0.0, 0.0, 0.0, 0.36),
                        offset: point(px(1.), px(1.)),
                        blur_radius: px(0.),
                        spread_radius: px(0.),
                    }])
                    .child(div().rounded(px(2.)).size_full().bg(self.background)),
            )
    }
}

#[derive(IntoElement)]
struct ToolStrip {}

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
                    .child(tool_button(ToolKind::ArrowPointer))
                    .child(tool_button(ToolKind::Hand))
                    .child(tool_divider())
                    .child(tool_button(ToolKind::Prompt).disabled(true))
                    .child(tool_divider())
                    .child(tool_button(ToolKind::Pencil).disabled(true))
                    .child(tool_button(ToolKind::Pen).disabled(true))
                    .child(tool_button(ToolKind::TextCursor).disabled(true))
                    .child(tool_divider())
                    .child(tool_button(ToolKind::Frame).disabled(true))
                    .child(tool_button(ToolKind::Square).disabled(true))
                    .child(tool_button(ToolKind::Line).disabled(true))
                    .child(tool_divider())
                    .child(tool_button(ToolKind::Image).disabled(true))
                    .child(tool_button(ToolKind::ElementLibrary).disabled(true))
                    .child(tool_divider())
                    .child(tool_button(ToolKind::Arrow).disabled(true)),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(CurrentColorTool::new()),
            )
    }
}

#[derive(Clone, Debug)]
pub enum ElementKind {
    Frame,
    ShapeSquare,
    ShapeCircle,
    Text,
    Image,
    Path,
}

impl ElementKind {
    pub fn icon_src(&self) -> SharedString {
        match self {
            ElementKind::ShapeSquare => Icon::Square.src(),
            ElementKind::Text => Icon::Text.src(),
            ElementKind::Image => Icon::Image.src(),
            ElementKind::Path => Icon::Path.src(),
            _ => Icon::Frame.src(),
        }
    }
}

#[derive(IntoElement)]
pub struct LayerListItem {
    kind: ElementKind,
    name: SharedString,
    selected: bool,
}

impl LayerListItem {
    pub fn new(kind: ElementKind, name: impl Into<SharedString>) -> Self {
        Self {
            kind,
            name: name.into(),
            selected: false,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl RenderOnce for LayerListItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get_global(cx);

        let text_color = if self.selected {
            theme.foreground
        } else {
            theme.foreground_muted
        };

        div()
            .id(ElementId::Name(format!("layer-{}", self.name).into()))
            .pl(px(10.))
            .flex()
            .items_center()
            .rounded_tl(px(4.))
            .rounded_bl(px(4.))
            .when(self.selected, |div| div.bg(theme.selected.alpha(0.12)))
            .active(|div| div.bg(theme.foreground.opacity(0.05)))
            .text_color(text_color)
            .gap(px(10.))
            .child(
                svg()
                    .path(self.kind.icon_src())
                    .size(px(11.))
                    .text_color(text_color.alpha(0.8)),
            )
            .child(self.name)
    }
}

#[derive(IntoElement)]
struct LayerList {}

impl RenderOnce for LayerList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = GlobalState::get(cx);

        let mut layers = div().flex().flex_col().flex_1().pt_1();

        // Add all elements from GlobalState to the layer list
        for element in &state.elements {
            layers = layers.child(
                LayerListItem::new(element.kind.clone(), element.name.clone())
                    .selected(element.selected),
            );
        }

        layers
    }
}

#[derive(IntoElement)]
struct Sidebar {}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get_global(cx);

        let inner = div()
            .flex()
            .flex_col()
            .h_full()
            .w(px(260.))
            .rounded_tl(px(15.))
            .rounded_bl(px(15.))
            .bg(theme.background_color)
            .child(div().w_full().h(px(TITLEBAR_HEIGHT)))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .w_full()
                    .child(ToolStrip::new())
                    .child(LayerList {}),
            );

        div()
            .id("titlebar")
            .h_full()
            .w(px(261.))
            .border_r_1()
            .border_color(gpui::white().alpha(0.08))
            .rounded_tl(px(15.))
            .rounded_bl(px(15.))
            .bg(theme.background_color)
            .shadow(smallvec::smallvec![BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.24),
                offset: point(px(1.), px(0.)),
                blur_radius: px(0.),
                spread_radius: px(0.),
            }])
            .child(inner)
    }
}

#[derive(Clone, Debug)]
pub struct ElementStyles {
    border_width: f32,
    border_color: Hsla,
    background_color: Hsla,
    corner_radius: f32,
    size: Size<f32>,
    position: Point<f32>,
}

impl Default for ElementStyles {
    fn default() -> Self {
        ElementStyles {
            border_width: 1.0,
            border_color: gpui::black(),
            background_color: gpui::white(),
            corner_radius: 0.0,
            size: Size::new(100.0, 100.0),
            position: Point::new(0.0, 0.0),
        }
    }
}

#[derive(IntoElement)]
pub struct Shape {
    id: ElementId,
    style: ElementStyles,
}

impl Shape {
    pub fn new(id: ElementId) -> Self {
        Shape {
            id,
            style: ElementStyles::default(),
        }
    }

    pub fn with_style(mut self, style: ElementStyles) -> Self {
        self.style = style;
        self
    }
}

impl RenderOnce for Shape {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get_global(cx);

        div()
            .id(self.id)
            .absolute()
            .left(px(self.style.position.x))
            .top(px(self.style.position.y))
            .w(px(self.style.size.width))
            .h(px(self.style.size.height))
            .bg(self.style.background_color)
            .border(px(self.style.border_width))
            .border_color(self.style.border_color)
            .rounded(px(self.style.corner_radius))
    }
}

/// Represents a single element on the canvas
pub struct Element {
    pub id: String,
    pub kind: ElementKind,
    pub name: SharedString,
    pub styles: ElementStyles,
    pub selected: bool,
}

/// A temporary place to throw a grab bag of various states until
/// they can be organize and structured more clearly.
///
/// At the very least this will need to be refactored before adding
/// muliple windows, as the global state will apply to all windows.
struct GlobalState {
    active_tool: ToolKind,
    current_border_color: Hsla,
    current_background_color: Hsla,
    elements: Vec<Element>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            active_tool: ToolKind::default(),
            current_border_color: gpui::white(),
            current_background_color: gpui::black(),
            elements: vec![
                // Add some initial elements for testing
                Element {
                    id: "rect1".to_string(),
                    kind: ElementKind::ShapeSquare,
                    name: "Rectangle 1".into(),
                    styles: ElementStyles {
                        position: Point::new(50.0, 50.0),
                        size: Size::new(100.0, 100.0),
                        background_color: gpui::red(),
                        ..Default::default()
                    },
                    selected: false,
                },
                Element {
                    id: "rect2".to_string(),
                    kind: ElementKind::ShapeSquare,
                    name: "Rectangle 2".into(),
                    styles: ElementStyles {
                        position: Point::new(200.0, 100.0),
                        size: Size::new(150.0, 80.0),
                        background_color: gpui::blue(),
                        ..Default::default()
                    },
                    selected: true,
                },
            ],
        }
    }

    pub fn get(cx: &App) -> &GlobalState {
        cx.global::<GlobalState>()
    }

    pub fn add_element(&mut self, element: Element) {
        self.elements.push(element);
    }
}

impl Global for GlobalState {}

/// Canvas is responsible for rendering all elements
#[derive(IntoElement)]
pub struct Canvas {}

impl Canvas {
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderOnce for Canvas {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get_global(cx);
        let state = GlobalState::get(cx);

        // Container for all canvas elements
        let mut canvas = div()
            .id("canvas")
            .size_full()
            .flex_1()
            .relative()
            .bg(theme.canvas_color);

        // Add all elements from GlobalState to the canvas
        for element in &state.elements {
            canvas = canvas.child(
                Shape::new(ElementId::Name(element.id.clone().into()))
                    .with_style(element.styles.clone()),
            );
        }

        canvas
    }
}

struct Luna {
    hide_sidebar: bool,
    focus_handle: FocusHandle,
}

impl Luna {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        Luna {
            hide_sidebar: false,
            focus_handle,
        }
    }

    pub fn hide_sidebar(&mut self, hide: bool) -> &mut Self {
        self.hide_sidebar = hide;
        self
    }

    pub fn toggle_ui(&mut self, _: &ToggleUI, _window: &mut Window, cx: &mut Context<Self>) {
        self.hide_sidebar(!self.hide_sidebar);
        cx.notify();
    }

    fn activate_hand_tool(&mut self, _: &HandTool, _window: &mut Window, cx: &mut Context<Self>) {
        GlobalState::update_global(cx, |state, _| state.active_tool = ToolKind::Hand);
        cx.notify();
    }

    fn activate_selection_tool(
        &mut self,
        _: &SelectionTool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        GlobalState::update_global(cx, |state, _| state.active_tool = ToolKind::ArrowPointer);
        cx.notify();
    }

    fn swap_current_colors(
        &mut self,
        _: &SwapCurrentColors,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        GlobalState::update_global(cx, |state, _| {
            std::mem::swap(
                &mut state.current_border_color,
                &mut state.current_background_color,
            );
        });
        cx.notify();
    }

    fn reset_current_colors(
        &mut self,
        _: &ResetCurrentColors,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        GlobalState::update_global(cx, |state, _| {
            state.current_border_color = gpui::white();
            state.current_background_color = gpui::black();
        });
        cx.notify();
    }
}

impl Render for Luna {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get_global(cx);
        let state = GlobalState::get(cx);

        // Create the canvas component
        let canvas = Canvas::new();

        div()
            .id("Luna")
            .key_context("luna")
            .track_focus(&self.focus_handle(cx))
            .relative()
            .size_full()
            .flex()
            .font_family("Berkeley Mono")
            .text_xs()
            .bg(theme.background_color)
            .text_color(theme.foreground)
            .border_1()
            .border_color(gpui::white().alpha(0.08))
            .rounded(px(16.))
            .overflow_hidden()
            .map(|div| match state.active_tool {
                ToolKind::Hand => div.cursor_grab(),
                _ => div.cursor_default(),
            })
            .on_action(cx.listener(Self::toggle_ui))
            .on_action(cx.listener(Self::activate_hand_tool))
            .on_action(cx.listener(Self::activate_selection_tool))
            .on_action(cx.listener(Self::reset_current_colors))
            .on_action(cx.listener(Self::swap_current_colors))
            .on_key_down(cx.listener(|this, e: &gpui::KeyDownEvent, window, cx| {
                let toggle_ui = keystroke_builder("cmd-.");
                let selection_tool = keystroke_builder("v");
                let hand_tool = keystroke_builder("h");

                if e.keystroke == toggle_ui {
                    this.toggle_ui(&ToggleUI::default(), window, cx);
                }

                if e.keystroke == hand_tool {
                    this.activate_hand_tool(&Default::default(), window, cx);
                }
                if e.keystroke == selection_tool {
                    this.activate_selection_tool(&Default::default(), window, cx);
                }

                cx.stop_propagation();
            }))
            .when(!self.hide_sidebar, |this| this.child(Sidebar {}))
            .child(canvas)
    }
}

impl Focusable for Luna {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

struct Assets {
    base: PathBuf,
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        fs::read(self.base.join(path))
            .map(|data| Some(std::borrow::Cow::Owned(data)))
            .map_err(|err| err.into())
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        fs::read_dir(self.base.join(path))
            .map(|entries| {
                entries
                    .filter_map(|entry| {
                        entry
                            .ok()
                            .and_then(|entry| entry.file_name().into_string().ok())
                            .map(SharedString::from)
                    })
                    .collect()
            })
            .map_err(|err| err.into())
    }
}

fn init_globals(cx: &mut App) {
    cx.set_global(Theme::new());
    cx.set_global(GlobalState::new());
}

fn main() {
    Application::new()
        .with_assets(Assets {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        })
        .run(|cx: &mut App| {
            cx.activate(true);
            cx.on_action(quit);
            cx.set_menus(vec![Menu {
                name: "Luna".into(),
                items: vec![MenuItem::action("Quit", Quit)],
            }]);

            init_globals(cx);

            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("Luna".into()),
                        appears_transparent: true,
                        traffic_light_position: Some(point(px(8.0), px(8.0))),
                    }),
                    window_background: WindowBackgroundAppearance::Transparent,
                    ..Default::default()
                },
                |_window, cx| cx.new(|cx| Luna::new(cx)),
            )
            .unwrap();
        });
}

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}
