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

If you need more context on it's useage, see the [CLI Guide](https://github.com/iamnbutler/spool/blob/main/docs/CLI_GUIDE.md).

You have a [skill](.claude/skills/spool.md) for creating and managing spools and streams.

### When to Use Spool

- Always. You should always update the task manager.
  - Something changes? update the task manager.
  - Starting a new task that has no spool associated with it? add one then continue
  - abandoning an approach? update the related spools.
- Create tasks for planned work before starting
- Update/complete tasks as work progresses
- Use priority levels: p1 (high), p2 (normal), p3 (low)

## Project Structure

Luna uses `_2` suffix crates for the simplified canvas implementation:

- `node_2` - Simplified shape model (Rectangle, Ellipse)
- `canvas_2` - Flat rendering, basic interactions
- `theme_2` - Minimal theming
- `ui_2` - UI components including input system

These crates are temporary and will be mainlined when their implementations move past the original crates.

## GPUI Notes

- Element trait: `request_layout`, `prepaint`, `paint` methods take `window: &mut Window, cx: &mut App`
- Render trait uses `Context<Self>` not `App`
- Use `Entity<T>` for GPUI state management
- Use `EventEmitter<Event>` for component events
