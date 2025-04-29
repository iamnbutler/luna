use gpui::{div, prelude::*, px, AnyElement, IntoElement, RenderOnce, SharedString};

use crate::typography::StyleTypographyExt as _;

pub mod inspector;

static ITEM_HEIGHT: f32 = 31.0;
static ROW_Y_PADDING: f32 = 5.0;
static ROW_GAP: f32 = 15.0;

#[derive(IntoElement)]
pub(super) struct SectionHeader {
    title: SharedString,
    end_slot: Option<AnyElement>,
}

impl SectionHeader {
    pub fn with_title(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            end_slot: None,
        }
    }
}

impl RenderOnce for SectionHeader {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        div()
            .flex()
            .gap(px(ROW_GAP))
            .w_full()
            .h(px(ITEM_HEIGHT))
            .items_center()
            .label_style()
            .child(self.title.to_uppercase())
    }
}
