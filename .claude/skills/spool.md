# Spool - Task Management

Spool is a git-native, event-sourced task management system. Tasks are stored in `.spool/events/` as JSONL files.

## Installation

```bash
cargo install spool-cli    # CLI
cargo install spool-ui     # TUI (optional)
```

## TUI

Run `spool-ui` for a terminal interface with:

- Task list with priority coloring
- Detail panel with event history
- Filtering, sorting, search
- Stream navigation
- Inline task creation (`n`) and completion (`c`)

## Commands Reference

### Listing Tasks

```bash
# List open tasks (default)
spool list

# List all tasks including completed
spool list -s all

# List only completed tasks
spool list -s complete

# Filter by assignee
spool list -a @username

# Filter by tag
spool list -t bug

# Filter by priority
spool list -p p0

# Filter by stream (use stream ID)
spool list --stream <stream-id>

# List tasks without a stream (orphaned tasks)
spool list --no-stream

# Output as JSON
spool list -f json

# Output just IDs (useful for scripting)
spool list -f ids
```

### Creating Tasks

```bash
# Basic task
spool add "Task title"

# With description
spool add "Task title" -d "Detailed description of what needs to be done"

# With priority (p0=critical, p1=high, p2=medium, p3=low)
spool add "Task title" -p p1

# With tags (can use multiple -t flags)
spool add "Task title" -t bug -t urgent

# With assignee
spool add "Task title" -a @username

# In a specific stream (use stream ID)
spool add "Task title" --stream <stream-id>

# Full example
spool add "Fix authentication bug" -d "Users getting logged out unexpectedly" -p p0 -t bug -t auth -a @alice --stream <stream-id>
```

### Viewing Task Details

```bash
# Show task details
spool show <task-id>

# Show task with event history
spool show <task-id> --events
```

### Claiming & Assigning Tasks

```bash
# Claim a task for yourself (assigns to current git user)
spool claim <task-id>

# Assign to a specific user
spool assign <task-id> @username

# Unassign a task (free it up)
spool free <task-id>
```

### Updating Tasks

```bash
# Update title
spool update <task-id> -t "New title"

# Update description
spool update <task-id> -d "New description"

# Update priority
spool update <task-id> -p p1

# Move to a different stream (use stream ID)
spool update <task-id> --stream <stream-id>

# Remove from stream (use empty string)
spool update <task-id> --stream ""

# Update multiple fields
spool update <task-id> -t "New title" -d "New description" -p p0
```

### Streams

Streams are first-class entities for grouping tasks into collections (e.g., by feature, sprint, or area). Streams have IDs, names, and optional descriptions.

```bash
# Create a new stream
spool stream add "auth" -d "Authentication features"

# List all streams with task counts
spool stream list

# List streams as JSON
spool stream list -f json

# Show stream details and its tasks (by ID)
spool stream show <stream-id>

# Show stream by name
spool stream show --name "auth"

# Update stream metadata
spool stream update <stream-id> -n "new-name" -d "New description"

# Delete a stream (must have no tasks assigned)
spool stream delete <stream-id>
```

#### Working with Tasks and Streams

```bash
# Create a stream first
spool stream add "frontend" -d "Frontend development"
# Note the stream ID returned, e.g., "k8b2x-a1c3"

# Create a task in that stream
spool add "Implement login form" --stream k8b2x-a1c3

# List tasks in that stream
spool list --stream k8b2x-a1c3

# Move a task to a different stream
spool update <task-id> --stream <other-stream-id>

# Remove a task from its stream
spool update <task-id> --stream ""

# Find tasks without a stream
spool list --no-stream
```

### Completing & Reopening Tasks

```bash
# Mark task as complete (default resolution: done)
spool complete <task-id>

# Complete with specific resolution
spool complete <task-id> -r done      # Successfully completed
spool complete <task-id> -r wontfix   # Won't be fixed
spool complete <task-id> -r duplicate # Duplicate of another task
spool complete <task-id> -r obsolete  # No longer relevant

# Reopen a completed task
spool reopen <task-id>
```

### Maintenance Commands

```bash
# Rebuild index and state from events
spool rebuild

# Archive completed tasks older than N days (default: 30)
spool archive --days 30

# Preview what would be archived
spool archive --dry-run

# Validate event files
spool validate
```

## Workflow Examples

### Starting Work on a Task

1. List available tasks: `spool list`
2. Claim the task: `spool claim <task-id>`
3. View details: `spool show <task-id>`
4. Do the work
5. Complete: `spool complete <task-id>`

### Creating a Bug Report

```bash
spool add "Button doesn't respond to clicks" \
  -d "The submit button on the login form is unresponsive on mobile devices" \
  -p p1 \
  -t bug \
  -t ui
```

### Finding Your Tasks

```bash
# List tasks assigned to you
spool list -a @yourusername

# List high-priority bugs assigned to you
spool list -a @yourusername -p p0 -t bug
```

### Setting Up a New Feature Stream

```bash
# Create the stream
spool stream add "ui-redesign" -d "Q1 UI redesign project"

# Get the stream ID from output, then create tasks
spool add "Add dark mode toggle" --stream <stream-id> -p p2
spool add "Update color palette" --stream <stream-id> -p p2

# View all tasks in the stream
spool stream show --name "ui-redesign"
```

## Priority Levels

- `p0` - Critical: Blocking issues, production down
- `p1` - High: Important work, should be done soon
- `p2` - Medium: Normal priority (default)
- `p3` - Low: Nice to have, backlog items

## Tips

- Task IDs are auto-generated and shown when you create a task
- Stream IDs are also auto-generated - use `spool stream list` to see them
- Use `spool stream show --name "name"` if you know the name but not the ID
- Use `spool list -f ids` to get just IDs for scripting
- Use `spool list --no-stream` to find tasks that need organizing
- The `.spool/` directory should be committed to git
- Events are the source of truth; `.index.json` and `.state.json` are caches
