use crate::{
    canvas::Canvas,
    element::{LunaElement, LunaElementId},
    THEME_SELECTED,
};
use gpui::{prelude::FluentBuilder as _, *};

#[derive(IntoElement)]
pub struct LayerListElement {
    id: LunaElementId,
    element: LunaElement,
    canvas: WeakEntity<Canvas>,
}

impl LayerListElement {
    pub fn new(id: LunaElementId, element: LunaElement, canvas: WeakEntity<Canvas>) -> Self {
        Self {
            id,
            element,
            canvas,
        }
    }

    fn selected(&self, cx: &mut App) -> bool {
        self.canvas.upgrade().map_or(false, |canvas| {
            canvas.read(cx).selected_ids.contains(&self.id)
        })
    }
}

impl RenderOnce for LayerListElement {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let name = self.element.name.clone();
        let id = ElementId::Name(format!("layer-{}", self.id.0.to_string()).into());
        let canvas = self.canvas.upgrade().clone();

        div()
            .id(id)
            .flex()
            .flex_none()
            .items_center()
            .px_1()
            .h(px(24.))
            .when(self.selected(cx), |this| {
                this.bg(THEME_SELECTED.alpha(0.12))
            })
            .when_some(canvas, |this, canvas| {
                this.on_click(move |_, window, cx| {
                    canvas.update(cx, |canvas, cx| {
                        if canvas.selected_ids.contains(&self.id) {
                            canvas.deselect_element(self.id, cx);
                        } else {
                            canvas.select_element(self.id, cx);
                        }
                    });
                })
            })
            .child(name)
    }
}

pub struct LayerList {
    canvas: Entity<Canvas>,
}

impl LayerList {
    pub fn new(canvas: Entity<Canvas>, cx: &mut Context<Self>) -> Self {
        Self { canvas }
    }
}

impl Render for LayerList {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let elements = self.canvas.read(cx).elements().iter().collect::<Vec<_>>();
        let weak_canvas = self.canvas.downgrade().clone();

        div()
            .absolute()
            .w(px(200.))
            .h_full()
            .bg(rgb(0x2A2C31))
            .border_r_1()
            .border_color(rgb(0x3F434C))
            .flex()
            .flex_col()
            .children(elements.iter().map(|(&id, element)| {
                let element = element.read(cx);
                LayerListElement::new(id, element.clone(), weak_canvas.clone())
            }))
    }
}
