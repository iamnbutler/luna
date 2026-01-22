//! Command and query execution against a Canvas.
//!
//! This module connects the abstract Command/Query types to the actual
//! Canvas implementation, executing operations and returning results.

use crate::{
    Command, CommandResult, Query, QueryResult, ShapeInfo, ShapeKindFilter, ShapeQuery, Target,
    ToolKind,
};
use canvas_2::{Canvas, Tool};
use glam::Vec2;
use gpui::{Context, Entity};
use node_2::{CanvasPoint, CanvasSize, Fill, Shape, ShapeId, ShapeKind, Stroke};

/// Execute a command against a canvas.
pub fn execute_command(
    canvas: &Entity<Canvas>,
    command: Command,
    cx: &mut gpui::App,
) -> CommandResult {
    canvas.update(cx, |canvas, cx| execute_command_inner(canvas, command, cx))
}

/// Execute a command against a canvas from within a view context.
/// Use this when you have access to &mut Context<T> rather than &mut App.
pub fn execute_command_in_context<T: 'static>(
    canvas: &Entity<Canvas>,
    command: Command,
    cx: &mut Context<T>,
) -> CommandResult {
    canvas.update(cx, |canvas, cx| execute_command_inner(canvas, command, cx))
}

fn execute_command_inner(canvas: &mut Canvas, command: Command, cx: &mut Context<Canvas>) -> CommandResult {
    match command {
        Command::CreateShape {
            kind,
            position,
            size,
            fill,
            stroke,
            corner_radius,
        } => {
            let mut shape = Shape::new(kind, CanvasPoint(position), CanvasSize(size));
            if let Some(fill) = fill {
                shape.fill = Some(Fill::new(fill.to_hsla()));
            }
            if let Some(stroke) = stroke {
                shape.stroke = Some(Stroke::new(stroke.color.to_hsla(), stroke.width));
            }
            if let Some(radius) = corner_radius {
                shape.corner_radius = radius;
            }
            let id = shape.id;
            canvas.add_shape(shape, cx);
            CommandResult::created(vec![id])
        }

        Command::Duplicate { target, offset } => {
            let ids = resolve_target(canvas, &target);
            if ids.is_empty() {
                return CommandResult::success();
            }

            let to_duplicate: Vec<_> = canvas
                .shapes
                .iter()
                .filter(|s| ids.contains(&s.id))
                .cloned()
                .collect();

            let mut created_ids = Vec::new();
            for mut shape in to_duplicate {
                shape.id = ShapeId::new();
                shape.position.0 += offset;
                created_ids.push(shape.id);
                canvas.shapes.push(shape);
            }

            // Select the new shapes
            canvas.selection.clear();
            for id in &created_ids {
                canvas.selection.insert(*id);
            }
            cx.notify();

            CommandResult::created(created_ids)
        }

        Command::Delete { target } => {
            let ids = resolve_target(canvas, &target);
            let deleted: Vec<_> = ids.iter().copied().collect();
            for id in ids {
                canvas.remove_shape(id, cx);
            }
            CommandResult::deleted(deleted)
        }

        Command::Select {
            target,
            add_to_selection,
        } => {
            let ids = resolve_target(canvas, &target);
            if !add_to_selection {
                canvas.selection.clear();
            }
            for id in &ids {
                canvas.selection.insert(*id);
            }
            cx.notify();
            CommandResult::success()
        }

        Command::ClearSelection => {
            canvas.clear_selection(cx);
            CommandResult::success()
        }

        Command::SelectAll => {
            canvas.selection.clear();
            for shape in &canvas.shapes {
                canvas.selection.insert(shape.id);
            }
            cx.notify();
            CommandResult::success()
        }

        Command::Move { target, delta } => {
            let ids = resolve_target(canvas, &target);
            let mut modified = Vec::new();
            for shape in &mut canvas.shapes {
                if ids.contains(&shape.id) {
                    shape.position.0 += delta;
                    modified.push(shape.id);
                }
            }
            cx.notify();
            CommandResult::modified(modified)
        }

        Command::SetPosition { target, position } => {
            let ids = resolve_target(canvas, &target);
            let mut modified = Vec::new();
            for shape in &mut canvas.shapes {
                if ids.contains(&shape.id) {
                    shape.position = CanvasPoint(position);
                    modified.push(shape.id);
                }
            }
            cx.notify();
            CommandResult::modified(modified)
        }

        Command::SetSize { target, size } => {
            let ids = resolve_target(canvas, &target);
            let mut modified = Vec::new();
            for shape in &mut canvas.shapes {
                if ids.contains(&shape.id) {
                    shape.size = CanvasSize(size);
                    modified.push(shape.id);
                }
            }
            cx.notify();
            CommandResult::modified(modified)
        }

        Command::Scale { target, factor } => {
            let ids = resolve_target(canvas, &target);
            let mut modified = Vec::new();
            for shape in &mut canvas.shapes {
                if ids.contains(&shape.id) {
                    shape.size.0 *= factor;
                    modified.push(shape.id);
                }
            }
            cx.notify();
            CommandResult::modified(modified)
        }

        Command::SetFill { target, fill } => {
            let ids = resolve_target(canvas, &target);
            let mut modified = Vec::new();
            for shape in &mut canvas.shapes {
                if ids.contains(&shape.id) {
                    shape.fill = fill.map(|f| Fill::new(f.to_hsla()));
                    modified.push(shape.id);
                }
            }
            cx.notify();
            CommandResult::modified(modified)
        }

        Command::SetStroke { target, stroke } => {
            let ids = resolve_target(canvas, &target);
            let mut modified = Vec::new();
            for shape in &mut canvas.shapes {
                if ids.contains(&shape.id) {
                    shape.stroke = stroke.map(|s| Stroke::new(s.color.to_hsla(), s.width));
                    modified.push(shape.id);
                }
            }
            cx.notify();
            CommandResult::modified(modified)
        }

        Command::SetCornerRadius { target, radius } => {
            let ids = resolve_target(canvas, &target);
            let mut modified = Vec::new();
            for shape in &mut canvas.shapes {
                if ids.contains(&shape.id) {
                    // Clamp to max valid radius
                    let max_radius = shape.size.width().min(shape.size.height()) / 2.0;
                    shape.corner_radius = radius.min(max_radius);
                    modified.push(shape.id);
                }
            }
            cx.notify();
            CommandResult::modified(modified)
        }

        Command::AddChild { child, parent } => {
            canvas.add_child(child, parent, cx);
            CommandResult::modified(vec![child, parent])
        }

        Command::Unparent { target } => {
            let ids = resolve_target(canvas, &target);
            let mut modified = Vec::new();
            for id in ids {
                // Check if shape has a parent before unparenting
                let has_parent = canvas.get_shape(id).map(|s| s.parent.is_some()).unwrap_or(false);
                if has_parent {
                    canvas.unparent(id, cx);
                    modified.push(id);
                }
            }
            CommandResult::modified(modified)
        }

        Command::SetClipChildren { target, clip } => {
            let ids = resolve_target(canvas, &target);
            let mut modified = Vec::new();
            for shape in &mut canvas.shapes {
                if ids.contains(&shape.id) && shape.kind == ShapeKind::Frame {
                    shape.clip_children = clip;
                    modified.push(shape.id);
                }
            }
            cx.notify();
            CommandResult::modified(modified)
        }

        Command::Pan { delta } => {
            canvas.viewport.pan(delta);
            cx.notify();
            CommandResult::success()
        }

        Command::Zoom { factor, center } => {
            let center = center.unwrap_or(Vec2::ZERO);
            canvas.viewport.zoom_at(gpui::point(center.x, center.y), factor);
            cx.notify();
            CommandResult::success()
        }

        Command::ResetView => {
            canvas.viewport.reset();
            cx.notify();
            CommandResult::success()
        }

        Command::SetTool { tool } => {
            canvas.tool = match tool {
                ToolKind::Select => Tool::Select,
                ToolKind::Pan => Tool::Pan,
                ToolKind::Rectangle => Tool::Rectangle,
                ToolKind::Ellipse => Tool::Ellipse,
                ToolKind::Frame => Tool::Frame,
            };
            cx.notify();
            CommandResult::success()
        }

        Command::Undo => {
            // TODO: Implement when undo/redo is added
            CommandResult::error("Undo not yet implemented")
        }

        Command::Redo => {
            // TODO: Implement when undo/redo is added
            CommandResult::error("Redo not yet implemented")
        }

        Command::Batch { commands } => {
            let mut all_created = Vec::new();
            let mut all_modified = Vec::new();
            let mut all_deleted = Vec::new();

            for cmd in commands {
                match execute_command_inner(canvas, cmd, cx) {
                    CommandResult::Success {
                        created,
                        modified,
                        deleted,
                    } => {
                        all_created.extend(created);
                        all_modified.extend(modified);
                        all_deleted.extend(deleted);
                    }
                    CommandResult::Error { message } => {
                        return CommandResult::error(format!("Batch failed: {}", message));
                    }
                }
            }

            CommandResult::Success {
                created: all_created,
                modified: all_modified,
                deleted: all_deleted,
            }
        }
    }
}

/// Execute a query against a canvas.
pub fn execute_query(canvas: &Entity<Canvas>, query: Query, cx: &gpui::App) -> QueryResult {
    let canvas = canvas.read(cx);
    execute_query_inner(canvas, query)
}

/// Execute a query against a canvas from within a view context.
pub fn execute_query_in_context<T: 'static>(
    canvas: &Entity<Canvas>,
    query: Query,
    cx: &Context<T>,
) -> QueryResult {
    let canvas = canvas.read(cx);
    execute_query_inner(canvas, query)
}

fn execute_query_inner(canvas: &Canvas, query: Query) -> QueryResult {
    match query {
        Query::GetSelection => QueryResult::Selection {
            ids: canvas.selection.iter().copied().collect(),
        },

        Query::GetAllShapes => QueryResult::Shapes {
            shapes: canvas.shapes.iter().map(shape_to_info).collect(),
        },

        Query::GetShapes { target } => {
            let ids = resolve_target_readonly(canvas, &target);
            QueryResult::Shapes {
                shapes: canvas
                    .shapes
                    .iter()
                    .filter(|s| ids.contains(&s.id))
                    .map(shape_to_info)
                    .collect(),
            }
        }

        Query::GetShape { id } => QueryResult::Shape {
            shape: canvas.shapes.iter().find(|s| s.id == id).map(shape_to_info),
        },

        Query::GetCanvasBounds => {
            if canvas.shapes.is_empty() {
                return QueryResult::Bounds {
                    min: None,
                    max: None,
                };
            }

            let mut min = Vec2::new(f32::MAX, f32::MAX);
            let mut max = Vec2::new(f32::MIN, f32::MIN);

            for shape in &canvas.shapes {
                let (shape_min, shape_max) = shape.bounds();
                min = min.min(shape_min.0);
                max = max.max(shape_max.0);
            }

            QueryResult::Bounds {
                min: Some(min),
                max: Some(max),
            }
        }

        Query::GetViewport => QueryResult::Viewport {
            offset: canvas.viewport.offset,
            zoom: canvas.viewport.zoom,
        },

        Query::GetTool => QueryResult::Tool {
            tool: format!("{:?}", canvas.tool),
        },

        Query::GetShapeCount => QueryResult::Count {
            count: canvas.shapes.len(),
        },
    }
}

/// Resolve a target to a list of shape IDs (mutable canvas access).
fn resolve_target(canvas: &Canvas, target: &Target) -> Vec<ShapeId> {
    resolve_target_readonly(canvas, target)
}

/// Resolve a target to a list of shape IDs (read-only).
fn resolve_target_readonly(canvas: &Canvas, target: &Target) -> Vec<ShapeId> {
    match target {
        Target::Selection => canvas.selection.iter().copied().collect(),
        Target::Shape(id) => vec![*id],
        Target::Shapes(ids) => ids.clone(),
        Target::All => canvas.shapes.iter().map(|s| s.id).collect(),
        Target::Query(query) => resolve_shape_query(canvas, query),
    }
}

/// Resolve a shape query to matching IDs.
fn resolve_shape_query(canvas: &Canvas, query: &ShapeQuery) -> Vec<ShapeId> {
    match query {
        ShapeQuery::ByKind(kind_filter) => {
            let kind = match kind_filter {
                ShapeKindFilter::Rectangle => ShapeKind::Rectangle,
                ShapeKindFilter::Ellipse => ShapeKind::Ellipse,
                ShapeKindFilter::Frame => ShapeKind::Frame,
            };
            canvas
                .shapes
                .iter()
                .filter(|s| s.kind == kind)
                .map(|s| s.id)
                .collect()
        }
        ShapeQuery::ByName(_name) => {
            // Future: when shapes have names
            vec![]
        }
        ShapeQuery::InBounds {
            x,
            y,
            width,
            height,
        } => {
            let bounds_min = Vec2::new(*x, *y);
            let bounds_max = Vec2::new(x + width, y + height);
            canvas
                .shapes
                .iter()
                .filter(|s| {
                    let (shape_min, shape_max) = s.bounds();
                    // Check if shape intersects bounds
                    shape_min.x() < bounds_max.x
                        && shape_max.x() > bounds_min.x
                        && shape_min.y() < bounds_max.y
                        && shape_max.y() > bounds_min.y
                })
                .map(|s| s.id)
                .collect()
        }
        ShapeQuery::ChildrenOf(target) => {
            let parent_ids = resolve_target_readonly(canvas, target);
            canvas
                .shapes
                .iter()
                .filter(|s| s.parent.map(|p| parent_ids.contains(&p)).unwrap_or(false))
                .map(|s| s.id)
                .collect()
        }
        ShapeQuery::ParentOf(target) => {
            let child_ids = resolve_target_readonly(canvas, target);
            let mut parent_ids = Vec::new();
            for shape in &canvas.shapes {
                if child_ids.contains(&shape.id) {
                    if let Some(parent_id) = shape.parent {
                        if !parent_ids.contains(&parent_id) {
                            parent_ids.push(parent_id);
                        }
                    }
                }
            }
            parent_ids
        }
    }
}

/// Convert a Shape to ShapeInfo for query results.
fn shape_to_info(shape: &Shape) -> ShapeInfo {
    use crate::{ColorInfo, FillInfo, StrokeInfo};

    ShapeInfo {
        id: shape.id,
        kind: shape.kind,
        position: shape.position.0,
        size: shape.size.0,
        fill: shape.fill.map(|f| FillInfo {
            color: ColorInfo::from(f.color),
        }),
        stroke: shape.stroke.map(|s| StrokeInfo {
            color: ColorInfo::from(s.color),
            width: s.width,
        }),
        corner_radius: shape.corner_radius,
        parent: shape.parent,
        children: shape.children.clone(),
        clip_children: shape.clip_children,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full tests require GPUI test context
    // These are basic unit tests for helper functions

    #[test]
    fn test_shape_to_info() {
        let shape = Shape::rectangle(Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0));
        let info = shape_to_info(&shape);
        assert_eq!(info.position, Vec2::new(10.0, 20.0));
        assert_eq!(info.size, Vec2::new(100.0, 50.0));
        assert!(matches!(info.kind, ShapeKind::Rectangle));
    }
}
