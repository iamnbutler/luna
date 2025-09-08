//! # Luna: A GPU-accelerated design canvas
//!
//! Luna is a modern design application built on the GPUI framework, providing a high-performance
//! canvas for creating and manipulating design elements.
//!
//! ## Architecture
//!
//! Luna is built around several core abstractions:
//!
//! - **Canvas**: The central drawing surface where elements are rendered and manipulated
//! - **SceneGraph**: Manages spatial relationships between nodes for efficient transformations
//! - **Elements**: Visual objects (rectangles, etc.) that can be created, selected, and modified
//! - **Tools**: Different interaction modes (selection, rectangle creation, hand tool, etc.)
//!
//! The application uses a combination of immediate and retained UI patterns, with a scene graph
//! for efficient spatial operations and a component-based architecture for the UI.

#![allow(unused, dead_code)]
use anyhow::Result;
use assets::Assets;
use canvas::LunaCanvas;
use canvas_element::CanvasElement;
use gpui::{
    actions, div, hsla, point, prelude::*, px, svg, App, Application, AssetSource, BoxShadow,
    ElementId, Entity, FocusHandle, Focusable, Global, Hsla, IntoElement, KeyBinding, Keystroke,
    Menu, MenuItem, Modifiers, Pixels, Point, SharedString, TitlebarOptions, UpdateGlobal,
    WeakEntity, Window, WindowBackgroundAppearance, WindowOptions,
};
use node::{NodeCommon, NodeId};
use scene_graph::SceneGraph;
use std::{fs, path::PathBuf, sync::Arc};
use strum::Display;
use theme::{ActiveTheme, GlobalTheme, Theme};
use tools::{ActiveTool, GlobalTool, Tool};
use ui::{inspector::Inspector, sidebar::Sidebar, Icon};
use util::keystroke_builder;

mod assets;
mod canvas;
mod canvas_element;
mod color;
mod css_parser;
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
        Cancel,
        Copy,
        Cut,
        Delete,
        FrameTool,
        HandTool,
        Paste,
        Quit,
        RectangleTool,
        ResetCurrentColors,
        SelectAll,
        SelectionTool,
        SwapCurrentColors,
        ToggleUI,
    ]
);

/// Application-wide state accessible from any context
///
/// GlobalState provides access to application-level state that applies across
/// the entire application. It utilizes GPUI's global mechanism to make this
/// state available throughout the component hierarchy without explicit passing.
///
/// This includes UI configuration like sidebar state, canvas navigation state,
/// and input tracking. In a multi-window implementation, this would need to be
/// refactored to per-window state.
struct GlobalState {
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

/// Core application state shared between components
///
/// Unlike GlobalState, AppState is an Entity that can be updated and observed
/// through GPUI's reactive update mechanism. Components can subscribe to changes
/// in this state to update their rendering accordingly.
///
/// This state includes the currently active tool and the current element styling
/// properties that will be applied to newly created elements.
pub struct AppState {
    /// Current border color for new elements
    pub current_border_color: Hsla,
    /// Current background color for new elements
    pub current_background_color: Hsla,
}

/// Main application component that orchestrates the Luna design application
///
/// Luna is the root component of the application, responsible for:
/// - Managing core application entities (canvas, scene graph, app state)
/// - Handling tool activation and application-level event routing
/// - Coordinating between UI components (inspector, canvas, etc.)
/// - Rendering the main application layout
///
/// It serves as the connection point between the GPUI framework and Luna-specific
/// functionality, managing the overall application lifecycle.
struct Luna {
    /// Shared application state accessible by multiple components
    app_state: Entity<AppState>,
    /// The main canvas where elements are rendered and manipulated
    canvas: Entity<LunaCanvas>,
    /// Focus handle for keyboard event routing
    focus_handle: FocusHandle,
    /// Scene graph for managing spatial relationships between nodes
    scene_graph: Entity<SceneGraph>,
    /// Inspector panel for element properties and tools
    inspector: Entity<Inspector>,
    /// Sidebar for additional tools and controls
    sidebar: Entity<Sidebar>,
    /// Layer list showing all elements in the canvas
    layer_list: Entity<ui::layer_list::LayerList>,
}

impl Luna {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let app_state = cx.new(|cx| AppState {
            current_border_color: cx.theme().tokens.overlay0,
            current_background_color: cx.theme().tokens.surface0,
        });
        let focus_handle = cx.focus_handle();
        let scene_graph = cx.new(|cx| SceneGraph::new());
        let theme = Theme::default();
        let canvas = cx.new(|cx| LunaCanvas::new(&app_state, &scene_graph, &theme, window, cx));
        let inspector = cx.new(|cx| Inspector::new(app_state.clone(), canvas.clone()));
        let sidebar = cx.new(|cx| Sidebar::new(canvas.clone()));
        let layer_list = cx.new(|cx| ui::layer_list::LayerList::new(canvas.clone()));

        Luna {
            app_state,
            canvas,
            scene_graph,
            focus_handle,
            inspector,
            sidebar,
            layer_list,
        }
    }

    fn activate_hand_tool(&mut self, _: &HandTool, _window: &mut Window, cx: &mut Context<Self>) {
        cx.set_global(GlobalTool(Arc::new(Tool::Hand)));
        cx.notify();
    }

    fn activate_selection_tool(
        &mut self,
        _: &SelectionTool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.set_global(GlobalTool(Arc::new(Tool::Selection)));
        cx.notify();
    }

    fn activate_rectangle_tool(
        &mut self,
        _: &RectangleTool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.set_global(GlobalTool(Arc::new(Tool::Frame)));
        cx.notify();
    }

    fn activate_frame_tool(&mut self, _: &FrameTool, _window: &mut Window, cx: &mut Context<Self>) {
        cx.set_global(GlobalTool(Arc::new(Tool::Frame)));
        cx.notify();
    }

    fn select_all_nodes(&mut self, _: &SelectAll, _window: &mut Window, cx: &mut Context<Self>) {
        self.canvas.update(cx, |canvas, cx| {
            canvas.select_all_nodes();
        });
        cx.notify();
    }

    fn delete_selected_nodes(&mut self, _: &Delete, _window: &mut Window, cx: &mut Context<Self>) {
        self.canvas.update(cx, |canvas, cx| {
            let selected_nodes = canvas
                .get_root_nodes()
                .into_iter()
                .filter(|&node_id| canvas.is_node_selected(node_id))
                .collect::<Vec<_>>();

            for node_id in selected_nodes {
                canvas.remove_node(node_id, cx);
            }
            canvas.mark_dirty(cx);
        });
    }

    fn handle_cancel(&mut self, _: &Cancel, _window: &mut Window, cx: &mut Context<Self>) {
        let active_tool = *cx.active_tool().clone();

        if active_tool == Tool::Selection {
            self.canvas.update(cx, |canvas, cx| {
                canvas.deselect_all_nodes(cx);
                canvas.mark_dirty(cx);
            });
        } else {
            cx.dispatch_action(&SelectionTool);
        }
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
            .bg(theme.tokens.background)
            .text_color(theme.tokens.text)
            .border_1()
            .border_color(gpui::white().alpha(0.08))
            .rounded(px(16.))
            .overflow_hidden()
            .on_key_down(|event, _, _| {
                dbg!(event.keystroke.clone());
            })
            .map(|div| match *cx.active_tool().clone() {
                Tool::Hand => div.cursor_grab(),
                Tool::Frame | Tool::Frame | Tool::Line | Tool::TextCursor => div.cursor_crosshair(),
                _ => div.cursor_default(),
            })
            .on_action(cx.listener(Self::activate_hand_tool))
            .on_action(cx.listener(Self::activate_selection_tool))
            .on_action(cx.listener(Self::activate_rectangle_tool))
            .on_action(cx.listener(Self::activate_frame_tool))
            .on_action(cx.listener(Self::select_all_nodes))
            .on_action(cx.listener(Self::delete_selected_nodes))
            .on_action(cx.listener(Self::handle_cancel))
            .child(CanvasElement::new(&self.canvas, &self.scene_graph, cx))
            .child(self.inspector.clone())
            .child(self.sidebar.clone())
            .child(self.layer_list.clone())
    }
}

impl Focusable for Luna {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

fn init_globals(cx: &mut App) {
    cx.set_global(GlobalTheme(Arc::new(Theme::default())));
    cx.set_global(GlobalTool(Arc::new(Tool::default())));
    cx.set_global(GlobalState::new());
}

/// Application entry point
///
/// Initializes the GPUI application, sets up global state, defines menus,
/// and opens the main application window. This function is the starting point
/// for the entire Luna application.
fn main() {
    Application::new()
        .with_assets(Assets {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        })
        .run(|cx: &mut App| {
            cx.on_action(quit);
            cx.set_menus(vec![Menu {
                name: "Luna".into(),
                items: vec![MenuItem::action("Quit", Quit)],
            }]);

            cx.bind_keys([
                KeyBinding::new("h", HandTool, None),
                KeyBinding::new("a", SelectionTool, None),
                KeyBinding::new("r", RectangleTool, None),
                KeyBinding::new("f", FrameTool, None),
                KeyBinding::new("escape", Cancel, None),
                KeyBinding::new("delete", Delete, None),
                KeyBinding::new("backspace", Delete, None),
                KeyBinding::new("cmd-a", SelectAll, None),
                KeyBinding::new("cmd-v", Paste, None),
                KeyBinding::new("cmd-c", Copy, None),
                KeyBinding::new("cmd-x", Cut, None),
            ]);

            init_globals(cx);

            let window = cx
                .open_window(
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

            let view = window.update(cx, |_, _, cx| cx.entity()).unwrap();

            cx.on_keyboard_layout_change({
                move |cx| {
                    window.update(cx, |_, _, cx| cx.notify()).ok();
                }
            })
            .detach();

            window
                .update(cx, |view, window, cx| {
                    window.focus(&view.focus_handle(cx));
                    cx.activate(true);
                })
                .unwrap();
        });
}

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}
