use gpui::{prelude::FluentBuilder as _, *};
use schemars_derive::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::canvas::Canvas;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct LunaElementId(pub(crate) usize);

impl LunaElementId {
    pub fn element_id(&self) -> ElementId {
        ElementId::Integer(self.0)
    }
}

impl Into<ElementId> for LunaElementId {
    fn into(self) -> ElementId {
        ElementId::Integer(self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LunaElement {
    pub(crate) id: LunaElementId,
    pub(crate) name: SharedString,
    pub(crate) style: ElementStyle,
    focus_handle: FocusHandle,
    canvas: WeakEntity<Canvas>,
}

impl LunaElement {
    pub fn new(
        id: LunaElementId,
        name: Option<SharedString>,
        style: ElementStyle,
        canvas: WeakEntity<Canvas>,
        cx: &mut App,
    ) -> Entity<Self> {
        let focus_handle = cx.focus_handle();
        cx.new(|cx| Self {
            id,
            name: name.unwrap_or_else(|| SharedString::from("Untitled")),
            style,
            focus_handle,
            canvas,
        })
    }

    pub fn selected(&self, cx: &mut Context<Self>) -> bool {
        self.canvas
            .upgrade()
            .map(|canvas| canvas.read(cx).selected_ids.contains(&self.id))
            .unwrap_or(false)
    }
}

impl Render for LunaElement {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let style = self.style.clone();
        let position = self.style.position.expect("Canvas must have a position");
        let dragging = if let Some(canvas) = self.canvas.upgrade() {
            canvas.read(cx).dragging.is_some()
        } else {
            false
        };

        div()
            .id(self.id.element_id())
            .track_focus(&self.focus_handle.clone())
            .absolute()
            .top(position.y)
            .left(position.x)
            .w(style.size.width)
            .h(style.size.height)
            .border_1()
            .border_color(gpui::transparent_black())
            .hover(|this| {
                if !dragging {
                    this.border_color(rgb(0x0C8CE9))
                } else {
                    this
                }
            })
            .child(
                div()
                    .size_full()
                    .bg(style.background_color)
                    .border(style.border_width)
                    .border_color(style.border_color),
            )
    }
}

impl Focusable for LunaElement {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ElementStyle {
    pub(crate) size: Size<Pixels>,
    pub(crate) border_width: Pixels,
    pub(crate) border_color: Hsla,
    pub(crate) background_color: Hsla,
    pub(crate) position: Option<Point<Pixels>>,
}

impl ElementStyle {
    pub fn new(cx: &mut App) -> Self {
        Self {
            size: Size::new(px(48.), px(48.)),
            border_width: px(1.),
            border_color: rgb(0x3F434C).into(),
            background_color: rgb(0x292C32).into(),
            position: None,
        }
    }

    pub fn size(mut self, size: Size<Pixels>) -> Self {
        self.size = size;
        self
    }
}
