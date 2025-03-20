use gpui::{div, Div, Edges, Element, Hitbox, LayoutId};
use smallvec::SmallVec;

use crate::element::LunaElementId;

/// A position relative to a parent element.
pub struct RelativePosition {
    x: f32,
    y: f32,
}

impl Default for RelativePosition {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

#[derive(Debug, Clone)]
pub struct BorderEdge {
    width: f32,
    color: palette::Srgba<f32>,
}

impl Default for BorderEdge {
    fn default() -> Self {
        Self {
            width: 0.0,
            color: palette::Srgba::new(0.0, 0.0, 0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Fill {
    Solid(palette::Srgba<f32>),
}

impl Default for Fill {
    fn default() -> Self {
        Self::Solid(palette::Srgba::new(1.0, 1.0, 1.0, 1.0))
    }
}

/// A frame is a renderable vector element that can contain other elements.
pub struct Frame {
    base: Div,
    position: RelativePosition,
    width: f32,
    height: f32,
    fill: Fill,
    // NOTE: gpui currently only supports a single color/width
    // for the border, not one per edge.
    border: Edges<BorderEdge>,
    corner_radius: f32,
    rotation: f32,
    children: SmallVec<[LunaElementId; 2]>,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            base: div(),
            position: RelativePosition::default(),
            width: 64.0,
            height: 64.0,
            fill: Fill::default(),
            border: Edges::all(BorderEdge::default()),
            corner_radius: 0.0,
            rotation: 0.0,
            children: SmallVec::new(),
        }
    }
}

/// Frame state used by the [Frame] element after layout.
pub struct FramePrepaintState {
    hitbox: Hitbox,
    // layout: LayoutItemsResponse,
}

// impl Element for Frame {
//     type RequestLayoutState = ();

//     type PrepaintState = FramePrepaintState;

//     fn id(&self) -> Option<gpui::ElementId> {
//         todo!()
//     }

//     fn request_layout(
//         &mut self,
//         id: Option<&gpui::GlobalElementId>,
//         window: &mut gpui::Window,
//         cx: &mut gpui::App,
//     ) -> (gpui::LayoutId, Self::RequestLayoutState) {
//         todo!()
//     }

//     fn prepaint(
//         &mut self,
//         id: Option<&gpui::GlobalElementId>,
//         bounds: gpui::Bounds<gpui::Pixels>,
//         request_layout: &mut Self::RequestLayoutState,
//         window: &mut gpui::Window,
//         cx: &mut gpui::App,
//     ) -> Self::PrepaintState {
//         todo!()
//     }

//     fn paint(
//         &mut self,
//         id: Option<&gpui::GlobalElementId>,
//         bounds: gpui::Bounds<gpui::Pixels>,
//         request_layout: &mut Self::RequestLayoutState,
//         prepaint: &mut Self::PrepaintState,
//         window: &mut gpui::Window,
//         cx: &mut gpui::App,
//     ) {
//         todo!()
//     }
// }
