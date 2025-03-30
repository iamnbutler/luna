#![allow(unused, dead_code)]
use anyhow::Result;
use assets::Assets;
use canvas::Canvas;
use canvas_element::CanvasElement;
use gpui::{
    actions, div, hsla, point, prelude::*, px, svg, App, Application, AssetSource, BoxShadow,
    ElementId, Entity, FocusHandle, Focusable, Global, Hsla, IntoElement, Keystroke, Menu,
    MenuItem, Modifiers, Pixels, Point, SharedString, TitlebarOptions, UpdateGlobal, WeakEntity,
    Window, WindowBackgroundAppearance, WindowOptions,
};
use node::NodeCommon;
use std::{fs, path::PathBuf};
use strum::Display;
use theme::Theme;
use tools::ToolKind;
use ui::{sidebar::Sidebar, Icon};
use util::keystroke_builder;

mod assets;
mod canvas;
mod canvas_element;
mod interactivity;
mod node;
mod scene_graph;
mod scene_node;
mod theme;
mod tools;
mod ui;
mod util;

actions!(
    luna,
    [
        Quit,
        ToggleUI,
        HandTool,
        SelectionTool,
        ResetCurrentColors,
        SwapCurrentColors,
        RectangleTool
    ]
);

// TODO: Most of this will get moved to Canvas
// TODO: The rest will move to Entity<AppState> on Luna
/// A temporary place to throw a grab bag of various states until
/// they can be organize and structured more clearly.
///
/// At the very least this will need to be refactored before adding
/// muliple windows, as the global state will apply to all windows.
struct GlobalState {
    active_tool: ToolKind,
    current_border_color: Hsla,
    current_background_color: Hsla,
    hide_sidebar: bool,
    sidebar_width: Pixels,

    // For panning the canvas with Hand tool
    drag_start_position: Option<Point<Pixels>>,
    scroll_start_position: Option<Point<f32>>,

    // For tracking mouse movement
    last_mouse_position: Option<Point<Pixels>>,
}

impl GlobalState {
    // Helper function to adjust a position for sidebar offset
    fn adjust_position(&self, position: Point<Pixels>) -> Point<Pixels> {
        let mut adjusted = position;
        if !self.hide_sidebar {
            adjusted.x -= self.sidebar_width;
        }
        adjusted
    }
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            active_tool: ToolKind::default(),
            current_border_color: gpui::white(),
            current_background_color: gpui::black(),
            hide_sidebar: false,
            sidebar_width: px(260.0),
            drag_start_position: None,
            scroll_start_position: None,
            last_mouse_position: None,
        }
    }

    pub fn get(cx: &App) -> &GlobalState {
        cx.global::<GlobalState>()
    }
}

impl Global for GlobalState {}

struct Luna {
    focus_handle: FocusHandle,
    canvas: Entity<Canvas>,
    sidebar: Entity<Sidebar>,
}

impl Luna {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let canvas = cx.new(|cx| Canvas::new(window, cx));
        let weak_canvas = canvas.downgrade();

        let sidebar = cx.new(|cx| Sidebar::new(weak_canvas));

        Luna {
            focus_handle,
            canvas,
            sidebar,
        }
    }

    pub fn toggle_ui(&mut self, _: &ToggleUI, _window: &mut Window, cx: &mut Context<Self>) {
        GlobalState::update_global(cx, |state, _| {
            let new_hide_state = !state.hide_sidebar;
            state.hide_sidebar = new_hide_state;
        });

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
        GlobalState::update_global(cx, |state, _| state.active_tool = ToolKind::Selection);
        cx.notify();
    }
    fn activate_rectangle_tool(
        &mut self,
        _: &RectangleTool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        GlobalState::update_global(cx, |state, _| state.active_tool = ToolKind::Rectangle);
        cx.notify();
    }

    fn swap_current_colors(
        &mut self,
        _: &SwapCurrentColors,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        GlobalState::update_global(cx, |state, cx| {
            let border_color = state.current_border_color;
            let background_color = state.current_background_color;

            state.current_border_color = background_color;
            state.current_background_color = border_color;
            cx.notify();
        });
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

        div()
            .id("Luna")
            .key_context("luna")
            .track_focus(&self.focus_handle(cx))
            .absolute()
            .top_0()
            .left_0()
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
                ToolKind::Frame | ToolKind::Rectangle | ToolKind::Line | ToolKind::TextCursor => {
                    div.cursor_crosshair()
                }
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
                let swap_colors = keystroke_builder("x");
                let rectangle_tool = keystroke_builder("r");

                if e.keystroke == toggle_ui {
                    this.toggle_ui(&ToggleUI::default(), window, cx);
                }

                if e.keystroke == hand_tool {
                    this.activate_hand_tool(&Default::default(), window, cx);
                }
                if e.keystroke == selection_tool {
                    this.activate_selection_tool(&Default::default(), window, cx);
                }
                if e.keystroke == swap_colors {
                    this.swap_current_colors(&Default::default(), window, cx);
                }
                if e.keystroke == rectangle_tool {
                    this.activate_rectangle_tool(&Default::default(), window, cx);
                }
            }))
            .child(CanvasElement::new(&self.canvas, cx))
            .child(self.sidebar.clone())
    }
}

impl Focusable for Luna {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
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
                |window, cx| cx.new(|cx| Luna::new(window, cx)),
            )
            .unwrap();
        });
}

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}
