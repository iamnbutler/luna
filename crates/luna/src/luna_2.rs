//! Luna 2: Simplified design canvas
//!
//! A streamlined version of Luna focused on basic shape drawing and manipulation.

use api::DebugServer;
use assets::Assets;
use canvas_2::{Canvas, CanvasElement, CanvasEvent, Tool};
use glam::Vec2;
use gpui::{
    actions, div, point, prelude::*, px, App, Application, Entity, FocusHandle, Focusable,
    IntoElement, KeyBinding, Menu, MenuItem, ParentElement, Styled, Subscription, TitlebarOptions,
    Window, WindowBackgroundAppearance, WindowOptions,
};
use node_2::Shape;
use std::sync::Arc;
use theme_2::Theme;
use ui_2::{bind_input_keys, LayerList, PropertiesPanel, ToolRail};

mod assets;

actions!(
    luna,
    [
        Cancel,
        Delete,
        Duplicate,
        EllipseTool,
        HandTool,
        NewFile,
        Quit,
        RectangleTool,
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
            canvas.shapes.clear();
            canvas.selection.clear();
            cx.notify();
        });
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
            .on_action(cx.listener(Self::delete_selected))
            .on_action(cx.listener(Self::duplicate_selected))
            .on_action(cx.listener(Self::handle_cancel))
            .on_action(cx.listener(Self::new_file))
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
        KeyBinding::new("escape", Cancel, None),
        KeyBinding::new("cmd-n", NewFile, None),
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
                items: vec![MenuItem::action("New", NewFile)],
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
