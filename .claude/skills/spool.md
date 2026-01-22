# Spool - Task Management

Spool is a git-native, event-sourced task management system used in this project. Tasks are stored in `.spool/events/` as JSONL files.

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

# Filter by stream
spool list --stream my-stream

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

# In a specific stream
spool add "Task title" --stream my-stream

# Full example
spool add "Fix authentication bug" -d "Users getting logged out unexpectedly" -p p0 -t bug -t auth -a @alice --stream backend
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

# Move to a different stream
spool update <task-id> --stream new-stream

# Update multiple fields
spool update <task-id> -t "New title" -d "New description" -p p0
```

### Streams

Streams let you group tasks into collections (e.g., by feature, sprint, or area).

```bash
# Create a task in a stream
spool add "Implement login" --stream auth

# List tasks in a stream
spool list --stream auth

# Move a task to a different stream
spool stream <task-id> new-stream

# Remove a task from its stream (move to no stream)
spool stream <task-id>

# You can also move tasks via update
spool update <task-id> --stream new-stream
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

# Archive completed tasks older than N days
spool archive <days>

# Validate event files
spool validate

# Start interactive shell mode
spool shell
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

### Working with Streams

```bash
# List all tasks in a stream
spool list --stream frontend

# Create tasks in a feature stream
spool add "Add dark mode toggle" --stream ui-redesign -p p2
spool add "Update color palette" --stream ui-redesign -p p2

# Move task to a different stream as priorities change
spool stream <task-id> urgent-fixes
```

## Priority Levels

- `p0` - Critical: Blocking issues, production down
- `p1` - High: Important work, should be done soon
- `p2` - Medium: Normal priority (default)
- `p3` - Low: Nice to have, backlog items

## Tips

- Task IDs are auto-generated and shown when you create a task
- Use `spool list -f ids` to get just IDs for scripting
- Use streams to organize tasks by feature, sprint, or area
- The `.spool/` directory should be committed to git
- Events are the source of truth; `.index.json` and `.state.json` are caches
