# Luna Project Guidelines

## Commit Conventions

Use these prefixes for commit messages:

- `add` - something new (feature, file, component)
- `fix` - fixed some behavior or mistake
- `cleanup` - no functional change, just tidying
- `remove` - deleted something

Example: `add: input component for properties panel`

## Task Management with Spool

This project uses [spool](https://crates.io/crates/spool) for task management. Spool is a git-native, event-sourced task tracker.

### Key Commands

```bash
spool add "Task title" -p p1      # Add high priority task
spool add "Task title" -p p2      # Add normal priority task
spool list                        # List all tasks
spool complete <task-id>          # Mark task complete
spool show <task-id>              # Show task details
```

### When to Use Spool

- Create tasks for planned work before starting
- Update/complete tasks as work progresses
- Use priority levels: p1 (high), p2 (normal), p3 (low)

## Project Structure

Luna uses `_2` suffix crates for the simplified canvas implementation:

- `node_2` - Simplified shape model (Rectangle, Ellipse)
- `canvas_2` - Flat rendering, basic interactions
- `theme_2` - Minimal theming
- `ui_2` - UI components including input system

## GPUI Notes

- Element trait: `request_layout`, `prepaint`, `paint` methods take `window: &mut Window, cx: &mut App`
- Render trait uses `Context<Self>` not `App`
- Use `Entity<T>` for GPUI state management
- Use `EventEmitter<Event>` for component events
