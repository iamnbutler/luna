//! Luna 2: Simplified design canvas
//!
//! A streamlined version of Luna focused on basic shape drawing and manipulation.

use api::DebugServer;
use assets::Assets;
use canvas::{Canvas, CanvasElement, CanvasEvent, Tool};
use glam::Vec2;
use gpui::{
    actions, div, point, prelude::*, px, App, Application, Entity, FocusHandle, Focusable,
    IntoElement, KeyBinding, Menu, MenuItem, ParentElement, PathPromptOptions, Styled, Subscription,
    TitlebarOptions, Window, WindowBackgroundAppearance, WindowOptions,
};
use interchange::{Document, Project};
use node::Shape;
use std::path::PathBuf;
use std::sync::Arc;
use theme::Theme;
use ui::{bind_input_keys, LayerList, PropertiesPanel, ToolRail};

mod assets;

actions!(
    luna,
    [
        Cancel,
        Delete,
        Duplicate,
        EllipseTool,
        FrameTool,
        HandTool,
        NewFile,
        OpenProject,
        Quit,
        RectangleTool,
        SaveProject,
        SaveProjectAs,
        SelectAll,
        SelectionTool,
    ]
);

/// Main application component
struct Luna {
    canvas: Entity<Canvas>,
    tool_rail: Entity<ToolRail>,
    layer_list: Entity<LayerList>,
    properties: Entity<PropertiesPanel>,
    focus_handle: FocusHandle,
    theme: Theme,
    debug_server: Option<Arc<DebugServer>>,
    /// Current project path (for save-in-place)
    project_path: Option<PathBuf>,
    _subscriptions: Vec<Subscription>,
}

impl Luna {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let theme = Theme::light();
        let focus_handle = cx.focus_handle();
        let canvas = cx.new(|cx| Canvas::new(theme.clone(), cx));
        let tool_rail = cx.new(|_| ToolRail::new(canvas.clone(), theme.clone()));
        let layer_list = cx.new(|_| LayerList::new(canvas.clone(), theme.clone()));
        let properties = cx.new(|cx| PropertiesPanel::new(canvas.clone(), theme.clone(), cx));

        // Add some example shapes
        canvas.update(cx, |canvas, cx| {
            let rect = Shape::rectangle(Vec2::new(100.0, 100.0), Vec2::new(150.0, 100.0))
                .with_stroke(theme.default_stroke, 2.0);
            canvas.add_shape(rect, cx);

            let ellipse = Shape::ellipse(Vec2::new(300.0, 150.0), Vec2::new(120.0, 120.0))
                .with_stroke(theme.default_stroke, 2.0);
            canvas.add_shape(ellipse, cx);
        });

        let canvas_subscription = cx.subscribe(&canvas, Self::handle_canvas_event);

        // Start debug server if enabled
        let debug_server = if DebugServer::should_start() {
            let server = Arc::new(DebugServer::new());
            server.start();

            // Spawn a background task to poll for pending requests
            let server_clone = server.clone();
            cx.spawn(async move |this, cx| {
                loop {
                    // Check every 50ms for pending requests
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(50))
                        .await;

                    // If there are pending requests, trigger a re-render
                    if server_clone.has_pending() {
                        this.update(cx, |_, cx| {
                            cx.notify();
                        })
                        .ok();
                    }
                }
            })
            .detach();

            Some(server)
        } else {
            None
        };

        Luna {
            canvas,
            tool_rail,
            layer_list,
            properties,
            focus_handle,
            theme,
            debug_server,
            project_path: None,
            _subscriptions: vec![canvas_subscription],
        }
    }

    fn handle_canvas_event(
        &mut self,
        _canvas: Entity<Canvas>,
        event: &CanvasEvent,
        _cx: &mut Context<Self>,
    ) {
        match event {
            CanvasEvent::ShapeAdded(id) => {
                eprintln!("Shape added: {:?}", id);
            }
            CanvasEvent::ShapeRemoved(id) => {
                eprintln!("Shape removed: {:?}", id);
            }
            CanvasEvent::SelectionChanged => {
                eprintln!("Selection changed");
            }
            CanvasEvent::ContentChanged => {
                // Content changed
            }
        }
    }

    fn activate_hand_tool(&mut self, _: &HandTool, _window: &mut Window, cx: &mut Context<Self>) {
        self.canvas.update(cx, |canvas, _| {
            canvas.tool = Tool::Pan;
        });
        cx.notify();
    }

    fn activate_selection_tool(
        &mut self,
        _: &SelectionTool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.canvas.update(cx, |canvas, _| {
            canvas.tool = Tool::Select;
        });
        cx.notify();
    }

    fn activate_rectangle_tool(
        &mut self,
        _: &RectangleTool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.canvas.update(cx, |canvas, _| {
            canvas.tool = Tool::Rectangle;
        });
        cx.notify();
    }

    fn activate_ellipse_tool(
        &mut self,
        _: &EllipseTool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.canvas.update(cx, |canvas, _| {
            canvas.tool = Tool::Ellipse;
        });
        cx.notify();
    }

    fn activate_frame_tool(
        &mut self,
        _: &FrameTool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.canvas.update(cx, |canvas, _| {
            canvas.tool = Tool::Frame;
        });
        cx.notify();
    }

    fn delete_selected(&mut self, _: &Delete, _window: &mut Window, cx: &mut Context<Self>) {
        self.canvas.update(cx, |canvas, cx| {
            canvas.delete_selected(cx);
        });
    }

    fn duplicate_selected(&mut self, _: &Duplicate, _window: &mut Window, cx: &mut Context<Self>) {
        self.canvas.update(cx, |canvas, cx| {
            canvas.duplicate_selected(cx);
        });
    }

    fn handle_cancel(&mut self, _: &Cancel, _window: &mut Window, cx: &mut Context<Self>) {
        self.canvas.update(cx, |canvas, cx| {
            if canvas.tool != Tool::Select {
                canvas.tool = Tool::Select;
            } else {
                canvas.clear_selection(cx);
            }
        });
        cx.notify();
    }

    fn new_file(&mut self, _: &NewFile, _window: &mut Window, cx: &mut Context<Self>) {
        self.canvas.update(cx, |canvas, cx| {
            canvas.load_shapes(Vec::new(), cx);
        });
        self.project_path = None;
    }

    fn save_project(&mut self, _: &SaveProject, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(path) = &self.project_path {
            // Save to existing path
            self.save_to_path(path.clone(), cx);
        } else {
            // No path yet, prompt for one
            self.save_project_as(&SaveProjectAs, _window, cx);
        }
    }

    fn save_project_as(&mut self, _: &SaveProjectAs, _window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let path = cx
                .update(|cx| {
                    cx.prompt_for_new_path(
                        &std::env::current_dir().unwrap_or_default(),
                        Some("untitled.luna"),
                    )
                })?
                .await??;

            if let Some(path) = path {
                this.update(cx, |this, cx| {
                    this.project_path = Some(path.clone());
                    this.save_to_path(path, cx);
                })?;
            }
            anyhow::Ok(())
        })
        .detach_and_log_err(cx);
    }

    fn save_to_path(&self, path: PathBuf, cx: &mut Context<Self>) {
        let shapes: Vec<Shape> = self.canvas.read(cx).shapes.clone();
        let doc = Document::new(shapes);
        let project = Project::from_document("Untitled", doc);

        if let Err(e) = project.save(&path) {
            eprintln!("Failed to save project: {}", e);
        } else {
            eprintln!("Saved to {}", path.display());
        }
    }

    fn open_project(&mut self, _: &OpenProject, _window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let paths = cx
                .update(|cx| {
                    cx.prompt_for_paths(PathPromptOptions {
                        files: true,
                        directories: true,
                        multiple: false,
                        prompt: Some("Open Luna Project".into()),
                    })
                })?
                .await??;

            if let Some(paths) = paths {
                if let Some(path) = paths.first() {
                    this.update(cx, |this, cx| {
                        this.load_from_path(path.clone(), cx);
                    })?;
                }
            }
            anyhow::Ok(())
        })
        .detach_and_log_err(cx);
    }

    fn load_from_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        match Project::load(&path) {
            Ok(project) => {
                if let Some(doc) = project.default_page() {
                    self.canvas.update(cx, |canvas, cx| {
                        canvas.load_shapes(doc.shapes.clone(), cx);
                    });
                    self.project_path = Some(path.clone());
                    eprintln!("Loaded project from {}", path.display());
                }
            }
            Err(e) => {
                eprintln!("Failed to load project: {}", e);
            }
        }
    }
}

impl Render for Luna {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Process any pending debug server requests
        if let Some(ref server) = self.debug_server {
            server.process_pending(&self.canvas, cx);
        }

        div()
            .id("Luna")
            .key_context("luna")
            .track_focus(&self.focus_handle)
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .flex()
            .flex_row()
            .font_family("Berkeley Mono")
            .text_xs()
            .bg(self.theme.ui_background)
            .text_color(self.theme.ui_text)
            .border_1()
            .border_color(gpui::white().alpha(0.08))
            .rounded(px(16.))
            .overflow_hidden()
            .on_action(cx.listener(Self::activate_hand_tool))
            .on_action(cx.listener(Self::activate_selection_tool))
            .on_action(cx.listener(Self::activate_rectangle_tool))
            .on_action(cx.listener(Self::activate_ellipse_tool))
            .on_action(cx.listener(Self::activate_frame_tool))
            .on_action(cx.listener(Self::delete_selected))
            .on_action(cx.listener(Self::duplicate_selected))
            .on_action(cx.listener(Self::handle_cancel))
            .on_action(cx.listener(Self::new_file))
            .on_action(cx.listener(Self::save_project))
            .on_action(cx.listener(Self::save_project_as))
            .on_action(cx.listener(Self::open_project))
            // Far left: Tool rail
            .child(
                div()
                    .pt(px(32.0)) // Space for traffic lights
                    .child(self.tool_rail.clone()),
            )
            // Left: Layer list
            .child(
                div()
                    .p(px(8.0))
                    .pt(px(32.0)) // Space for traffic lights
                    .child(self.layer_list.clone()),
            )
            // Center: Canvas (takes remaining space)
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .child(CanvasElement::new(self.canvas.clone())),
            )
            // Right: Properties panel
            .child(
                div()
                    .p(px(8.0))
                    .pt(px(32.0)) // Space for traffic lights
                    .child(self.properties.clone()),
            )
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
        KeyBinding::new("v", SelectionTool, None),
        KeyBinding::new("r", RectangleTool, None),
        KeyBinding::new("o", EllipseTool, None),
        KeyBinding::new("f", FrameTool, None),
        KeyBinding::new("escape", Cancel, None),
        KeyBinding::new("cmd-n", NewFile, None),
        KeyBinding::new("cmd-s", SaveProject, None),
        KeyBinding::new("cmd-shift-s", SaveProjectAs, None),
        KeyBinding::new("cmd-o", OpenProject, None),
        KeyBinding::new("cmd-d", Duplicate, None),
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("delete", Delete, None),
        KeyBinding::new("backspace", Delete, None),
    ]);
}

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        cx.on_action(quit);

        cx.set_menus(vec![
            Menu {
                name: "Luna".into(),
                items: vec![
                    MenuItem::action("About Luna", Quit),
                    MenuItem::separator(),
                    MenuItem::action("Quit", Quit),
                ],
            },
            Menu {
                name: "File".into(),
                items: vec![
                    MenuItem::action("New", NewFile),
                    MenuItem::action("Open...", OpenProject),
                    MenuItem::separator(),
                    MenuItem::action("Save", SaveProject),
                    MenuItem::action("Save As...", SaveProjectAs),
                ],
            },
            Menu {
                name: "Edit".into(),
                items: vec![
                    MenuItem::action("Duplicate", Duplicate),
                    MenuItem::action("Delete", Delete),
                ],
            },
            Menu {
                name: "Tools".into(),
                items: vec![
                    MenuItem::action("Selection (V)", SelectionTool),
                    MenuItem::action("Hand (H)", HandTool),
                    MenuItem::action("Rectangle (R)", RectangleTool),
                    MenuItem::action("Ellipse (O)", EllipseTool),
                    MenuItem::action("Frame (F)", FrameTool),
                ],
            },
        ]);

        init_keymap(cx);
        bind_input_keys(cx, None);

        let window = cx
            .open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("Luna 2".into()),
                        appears_transparent: true,
                        traffic_light_position: Some(point(px(8.0), px(8.0))),
                    }),
                    window_background: WindowBackgroundAppearance::Transparent,
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| Luna::new(window, cx)),
            )
            .unwrap();

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
