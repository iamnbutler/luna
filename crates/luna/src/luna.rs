//! # Luna: A GPU-accelerated design canvas
//!
//! Luna is a modern design application built on the GPUI framework, providing a high-performance
//! canvas for creating and manipulating design elements.
//!
//! ## Architecture
//!
//! Luna is built around several common abstractions:
//!
//! - **Canvas**: The central drawing surface where elements are rendered and manipulated
//! - **SceneGraph**: Manages spatial relationships between nodes for efficient transformations
//! - **Elements**: Visual objects (rectangles, etc.) that can be created, selected, and modified
//! - **Tools**: Different interaction modes (selection, rectangle creation, hand tool, etc.)
//!
//! The application uses a combination of immediate and retained UI patterns, with a scene graph
//! for efficient spatial operations and a component-based architecture for the UI.

use assets::Assets;
use canvas::{
    canvas_element::CanvasElement,
    tools::{ActiveTool, GlobalTool, Tool},
    AppState, CanvasEvent, LunaCanvas,
};
use gpui::{
    actions, div, point, prelude::*, px, App, Application, Entity, FocusHandle, Focusable,
    IntoElement, KeyBinding, Menu, MenuItem, Subscription, TitlebarOptions, Window,
    WindowBackgroundAppearance, WindowOptions,
};
use project::{LunaProject, ProjectState};
use scene_graph::SceneGraph;
use std::path::PathBuf;
use std::sync::Arc;
use theme::{ActiveTheme, GlobalTheme, Theme};
use ui::{inspector::Inspector, sidebar::Sidebar};

mod assets;
mod serialization;

// Re-export commonly used items from external crates
pub use canvas::{canvas_element, tools};
pub use common::{color, coordinates, interactivity, keymap, util};
pub use node;
pub use scene_graph;
pub use theme;
pub use ui;

use crate::serialization::{deserialize_canvas, serialize_canvas};

actions!(
    luna,
    [
        Cancel,
        Copy,
        Cut,
        Delete,
        FrameTool,
        HandTool,
        NewFile,
        OpenFile,
        Paste,
        Quit,
        RectangleTool,
        ResetCurrentColors,
        SaveFile,
        SaveFileAs,
        SelectAll,
        SelectionTool,
        SwapCurrentColors,
        ToggleUI,
    ]
);

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
    /// Project state for file management
    project_state: Entity<ProjectState>,
    /// Subscriptions to various events
    _subscriptions: Vec<Subscription>,
}

impl Luna {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let app_state = cx.new(|cx| AppState {
            current_border_color: cx.theme().tokens.overlay0,
            current_background_color: cx.theme().tokens.surface0,
        });
        let focus_handle = cx.focus_handle();
        let scene_graph = cx.new(|_| SceneGraph::new());
        let theme = Theme::default();
        let canvas = cx.new(|cx| LunaCanvas::new(&app_state, &scene_graph, &theme, window, cx));
        let inspector = cx.new(|cx| Inspector::new(app_state.clone(), canvas.clone(), cx));
        let project_state = cx.new(|_| ProjectState::new());
        let sidebar = cx.new(|cx| Sidebar::new(canvas.clone(), project_state.clone(), cx));

        // Subscribe to canvas events to mark project as dirty
        let canvas_subscription = cx.subscribe(&canvas, Self::handle_canvas_event);

        Luna {
            app_state,
            canvas,
            scene_graph,
            focus_handle,
            inspector,
            sidebar,
            project_state,
            _subscriptions: vec![canvas_subscription],
        }
    }

    fn handle_canvas_event(
        &mut self,
        _canvas: Entity<LunaCanvas>,
        event: &CanvasEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            CanvasEvent::NodeAdded(_)
            | CanvasEvent::NodeRemoved(_)
            | CanvasEvent::ContentChanged => {
                // Mark the project as dirty when canvas content changes
                self.project_state.update(cx, |state, _| {
                    state.mark_dirty();
                });
            }
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
        self.canvas.update(cx, |canvas, _| {
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

        // Mark project as dirty after deleting nodes
        self.project_state.update(cx, |state, _| {
            state.mark_dirty();
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

    fn new_file(&mut self, _: &NewFile, _window: &mut Window, cx: &mut Context<Self>) {
        // Clear the canvas and create a new project
        self.project_state.update(cx, |state, _| {
            *state = ProjectState::new();
        });

        self.canvas.update(cx, |canvas, cx| {
            canvas.clear_all(cx);
        });

        cx.notify();
    }

    fn open_file(&mut self, _: &OpenFile, _window: &mut Window, cx: &mut Context<Self>) {
        let weak_project = self.project_state.downgrade();
        let weak_canvas = self.canvas.downgrade();
        let weak_scene = self.scene_graph.downgrade();
        let weak_app_state = self.app_state.downgrade();

        cx.spawn(async move |_, cx| {
            let paths = cx
                .update(|cx| {
                    cx.prompt_for_paths(gpui::PathPromptOptions {
                        files: true,
                        directories: false,
                        multiple: false,
                        prompt: None,
                    })
                })?
                .await??;

            if let Some(paths) = paths {
                if let Some(path) = paths.first() {
                    let project_data = project::LunaProject::load_from_file(path).await?;

                    cx.update(|cx| {
                        if let (Some(project_state), Some(canvas), Some(scene), Some(app_state)) = (
                            weak_project.upgrade(),
                            weak_canvas.upgrade(),
                            weak_scene.upgrade(),
                            weak_app_state.upgrade(),
                        ) {
                            project_state.update(cx, |state, _| {
                                state.project = project_data.clone();
                                state.file_path = Some(path.to_path_buf());
                                state.mark_clean();
                            });

                            deserialize_canvas(&project_data, &canvas, &scene, &app_state, cx)?;

                            // Force canvas to update and repaint
                            canvas.update(cx, |canvas, cx| {
                                canvas.mark_dirty(cx);
                                cx.notify();
                            });
                        }
                        anyhow::Ok(())
                    })??;
                }
            }
            Ok::<(), anyhow::Error>(())
        })
        .detach_and_log_err(cx);
    }

    fn save_file(&mut self, _: &SaveFile, window: &mut Window, cx: &mut Context<Self>) {
        let file_path = self.project_state.read(cx).file_path.clone();

        if let Some(path) = file_path {
            self.save_to_path(path, window, cx);
        } else {
            self.save_file_as(&SaveFileAs, window, cx);
        }
    }

    fn save_file_as(&mut self, _: &SaveFileAs, _window: &mut Window, cx: &mut Context<Self>) {
        let weak_project = self.project_state.downgrade();
        let weak_canvas = self.canvas.downgrade();
        let weak_scene = self.scene_graph.downgrade();
        let weak_app_state = self.app_state.downgrade();

        cx.spawn(async move |_, cx| {
            let path = cx
                .update(|cx| {
                    let home_dir = std::env::var("HOME")
                        .ok()
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|| std::path::PathBuf::from("/"));
                    cx.prompt_for_new_path(&home_dir, None)
                })?
                .await??;

            if let Some(mut path) = path {
                // Ensure the file has a .luna extension
                if path.extension().is_none()
                    || path.extension() != Some(std::ffi::OsStr::new("luna"))
                {
                    path.set_extension("luna");
                }

                cx.update(|cx| {
                    if let (Some(project_state), Some(canvas), Some(scene), Some(app_state)) = (
                        weak_project.upgrade(),
                        weak_canvas.upgrade(),
                        weak_scene.upgrade(),
                        weak_app_state.upgrade(),
                    ) {
                        let project = serialize_canvas(&canvas, &scene, &app_state, cx)?;

                        project_state.update(cx, |state, cx| {
                            state.project = project.clone();
                            state.file_path = Some(path.to_path_buf());
                            state.mark_clean();
                            cx.notify();
                        });

                        // Save to disk
                        let executor = cx.background_executor().clone();
                        executor
                            .spawn(async move { project.save_to_file(&path).await })
                            .detach_and_log_err(cx);
                    }
                    anyhow::Ok(())
                })??;
            }
            Ok::<(), anyhow::Error>(())
        })
        .detach_and_log_err(cx);
    }

    fn save_to_path(&mut self, mut path: PathBuf, _window: &mut Window, cx: &mut Context<Self>) {
        // Ensure the file has a .luna extension
        if path.extension().is_none() || path.extension() != Some(std::ffi::OsStr::new("luna")) {
            path.set_extension("luna");
        }

        let project = serialize_canvas(&self.canvas, &self.scene_graph, &self.app_state, cx)
            .unwrap_or_else(|_e| {
                // Failed to serialize canvas - return empty project
                LunaProject::new()
            });

        self.project_state.update(cx, |state, _| {
            state.project = project.clone();
            state.mark_clean();
        });

        cx.background_executor()
            .spawn(async move { project.save_to_file(&path).await })
            .detach_and_log_err(cx);
    }
}

impl Render for Luna {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get_global(cx);

        div()
            .id("Luna")
            .key_context("luna")
            .track_focus(&self.focus_handle)
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
            .map(|div| match *cx.active_tool().clone() {
                Tool::Hand => div.cursor_grab(),
                Tool::Frame | Tool::Line | Tool::TextCursor => div.cursor_crosshair(),
                _ => div.cursor_default(),
            })
            .on_action(cx.listener(Self::activate_hand_tool))
            .on_action(cx.listener(Self::activate_selection_tool))
            .on_action(cx.listener(Self::activate_rectangle_tool))
            .on_action(cx.listener(Self::activate_frame_tool))
            .on_action(cx.listener(Self::select_all_nodes))
            .on_action(cx.listener(Self::delete_selected_nodes))
            .on_action(cx.listener(Self::handle_cancel))
            .on_action(cx.listener(Self::new_file))
            .on_action(cx.listener(Self::open_file))
            .on_action(cx.listener(Self::save_file))
            .on_action(cx.listener(Self::save_file_as))
            .child(CanvasElement::new(&self.canvas, &self.scene_graph, cx))
            .child(self.inspector.clone())
            .child(self.sidebar.clone())
    }
}

impl Focusable for Luna {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

fn init_keymap(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("h", HandTool, None),
        KeyBinding::new("a", SelectionTool, None),
        KeyBinding::new("r", RectangleTool, None),
        KeyBinding::new("f", FrameTool, None),
        KeyBinding::new("escape", Cancel, None),
        KeyBinding::new("cmd-n", NewFile, None),
        KeyBinding::new("cmd-o", OpenFile, None),
        KeyBinding::new("cmd-s", SaveFile, None),
        KeyBinding::new("cmd-shift-s", SaveFileAs, None),
        KeyBinding::new("cmd-a", SelectAll, None),
        KeyBinding::new("cmd-v", Paste, None),
        KeyBinding::new("cmd-c", Copy, None),
        KeyBinding::new("cmd-x", Cut, None),
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("cmd-\\", ToggleUI, None),
        KeyBinding::new("x", SwapCurrentColors, None),
        KeyBinding::new("d", ResetCurrentColors, None),
        // Canvas
        KeyBinding::new("delete", Delete, None),
        KeyBinding::new("backspace", Delete, None),
        // Layer List
        KeyBinding::new("delete", Delete, Some("LayerList")),
        KeyBinding::new("backspace", Delete, Some("LayerList")),
    ]);
}

fn init_globals(cx: &mut App) {
    cx.set_global(GlobalTheme(Arc::new(Theme::default())));
    cx.set_global(GlobalTool(Arc::new(Tool::default())));
}

/// Application entry point
///
/// Initializes the GPUI application, sets up global state, defines menus,
/// and opens the main application window. This function is the starting point
/// for the entire Luna application.
fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        // Register global action handlers
        cx.on_action(quit);

        // Set up menus with File menu
        cx.set_menus(vec![
            Menu {
                name: "Luna".into(),
                items: vec![
                    MenuItem::action("About Luna", Quit), // TODO: Add About action
                    MenuItem::separator(),
                    MenuItem::action("Quit", Quit),
                ],
            },
            Menu {
                name: "File".into(),
                items: vec![
                    MenuItem::action("New", NewFile),
                    MenuItem::action("Open...", OpenFile),
                    MenuItem::separator(),
                    MenuItem::action("Save", SaveFile),
                    MenuItem::action("Save As...", SaveFileAs),
                ],
            },
            Menu {
                name: "Edit".into(),
                items: vec![
                    MenuItem::action("Copy", Copy),
                    MenuItem::action("Cut", Cut),
                    MenuItem::action("Paste", Paste),
                    MenuItem::separator(),
                    MenuItem::action("Select All", SelectAll),
                    MenuItem::action("Delete", Delete),
                ],
            },
            Menu {
                name: "Tools".into(),
                items: vec![
                    MenuItem::action("Selection Tool", SelectionTool),
                    MenuItem::action("Hand Tool", HandTool),
                    MenuItem::action("Rectangle Tool", RectangleTool),
                    MenuItem::action("Frame Tool", FrameTool),
                ],
            },
        ]);

        init_keymap(cx);
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
