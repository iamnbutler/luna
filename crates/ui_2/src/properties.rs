//! Properties panel for selected shapes.

use crate::components::{h_stack, panel, v_stack};
use canvas_2::Canvas;
use gpui::{
    div, px, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
};
use node_2::ShapeKind;
use theme_2::Theme;

/// Properties panel showing details of selected shapes.
pub struct PropertiesPanel {
    canvas: Entity<Canvas>,
    theme: Theme,
}

impl PropertiesPanel {
    pub fn new(canvas: Entity<Canvas>, theme: Theme) -> Self {
        Self { canvas, theme }
    }
}

impl Render for PropertiesPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let canvas = self.canvas.read(cx);
        let theme = &self.theme;

        // Get selected shapes
        let selected: Vec<_> = canvas
            .shapes
            .iter()
            .filter(|s| canvas.selection.contains(&s.id))
            .collect();

        let content = if selected.is_empty() {
            div()
                .text_sm()
                .text_color(theme.ui_text_muted)
                .child("No selection")
        } else if selected.len() == 1 {
            let shape = selected[0];
            let kind_name = match shape.kind {
                ShapeKind::Rectangle => "Rectangle",
                ShapeKind::Ellipse => "Ellipse",
            };

            v_stack()
                .gap(px(12.0))
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.ui_text)
                        .child(kind_name),
                )
                // Position
                .child(property_section(
                    "Position",
                    theme,
                    vec![
                        ("X", format!("{:.0}", shape.position.x)),
                        ("Y", format!("{:.0}", shape.position.y)),
                    ],
                ))
                // Size
                .child(property_section(
                    "Size",
                    theme,
                    vec![
                        ("W", format!("{:.0}", shape.size.x)),
                        ("H", format!("{:.0}", shape.size.y)),
                    ],
                ))
                // Fill
                .child(property_section_color(
                    "Fill",
                    theme,
                    shape.fill.as_ref().map(|f| f.color),
                ))
                // Stroke
                .child(property_section_stroke(
                    "Stroke",
                    theme,
                    shape.stroke.as_ref(),
                ))
        } else {
            div()
                .text_sm()
                .text_color(theme.ui_text_muted)
                .child(format!("{} shapes selected", selected.len()))
        };

        panel(theme)
            .w(px(200.0))
            .h_full()
            .child(
                div()
                    .text_xs()
                    .text_color(theme.ui_text_muted)
                    .pb(px(8.0))
                    .child("Properties"),
            )
            .child(content)
    }
}

fn property_section(
    label: &str,
    theme: &Theme,
    values: Vec<(&str, String)>,
) -> impl IntoElement {
    v_stack()
        .gap(px(4.0))
        .child(
            div()
                .text_xs()
                .text_color(theme.ui_text_muted)
                .child(label.to_string()),
        )
        .child(
            h_stack()
                .gap(px(8.0))
                .children(values.into_iter().map(|(name, value)| {
                    property_field(name, &value, theme)
                })),
        )
}

fn property_field(label: &str, value: &str, theme: &Theme) -> impl IntoElement {
    h_stack()
        .gap(px(4.0))
        .child(
            div()
                .text_xs()
                .text_color(theme.ui_text_muted)
                .w(px(16.0))
                .child(label.to_string()),
        )
        .child(
            div()
                .px(px(6.0))
                .py(px(2.0))
                .bg(theme_2::hsla(0.0, 0.0, 0.95, 1.0))
                .border_1()
                .border_color(theme.ui_border)
                .rounded(px(2.0))
                .text_xs()
                .text_color(theme.ui_text)
                .min_w(px(48.0))
                .child(value.to_string()),
        )
}

fn property_section_color(
    label: &str,
    theme: &Theme,
    color: Option<gpui::Hsla>,
) -> impl IntoElement {
    v_stack()
        .gap(px(4.0))
        .child(
            div()
                .text_xs()
                .text_color(theme.ui_text_muted)
                .child(label.to_string()),
        )
        .child(
            h_stack().gap(px(8.0)).child(if let Some(c) = color {
                h_stack()
                    .gap(px(4.0))
                    .child(
                        div()
                            .size(px(20.0))
                            .rounded(px(2.0))
                            .border_1()
                            .border_color(theme.ui_border)
                            .bg(c),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.ui_text)
                            .child(format_color(c)),
                    )
            } else {
                h_stack()
                    .child(div().text_xs().text_color(theme.ui_text_muted).child("None"))
            }),
        )
}

fn property_section_stroke(
    label: &str,
    theme: &Theme,
    stroke: Option<&node_2::Stroke>,
) -> impl IntoElement {
    v_stack()
        .gap(px(4.0))
        .child(
            div()
                .text_xs()
                .text_color(theme.ui_text_muted)
                .child(label.to_string()),
        )
        .child(if let Some(s) = stroke {
            h_stack()
                .gap(px(8.0))
                .child(
                    div()
                        .size(px(20.0))
                        .rounded(px(2.0))
                        .border_1()
                        .border_color(theme.ui_border)
                        .bg(s.color),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.ui_text)
                        .child(format!("{:.0}px", s.width)),
                )
        } else {
            h_stack().child(div().text_xs().text_color(theme.ui_text_muted).child("None"))
        })
}

fn format_color(c: gpui::Hsla) -> String {
    // Convert to hex-like display
    let r = (c.l * 255.0) as u8;
    format!("hsla({:.0}, {:.0}%, {:.0}%)", c.h * 360.0, c.s * 100.0, c.l * 100.0)
}
