#!/bin/bash
# Luna CLI Demo 2 - New Features
# Demonstrates: frames, hex colors, corner radius, strokes, hierarchy, viewports
#
# Usage: ./scripts/cli-demo-2.sh
# Requires: Luna running with LUNA_DEBUG=1

set -e

CLI="./target/debug/luna-cli"

# Check if CLI exists
if [ ! -f "$CLI" ]; then
    echo "Building CLI..."
    cargo build --package luna-cli
fi

# Check if Luna is running
if ! $CLI list 2>/dev/null | grep -q "PID"; then
    echo "Error: No Luna instance found."
    echo "Start Luna with: LUNA_DEBUG=1 cargo run --bin Luna2"
    exit 1
fi

# Helper to extract UUID from JSON response
extract_id() {
    grep -o '"[0-9a-f-]\{36\}"' | head -1 | tr -d '"'
}

echo "=== Luna CLI Demo 2: New Features ==="
echo

# Clear canvas
echo "Clearing canvas..."
$CLI command '{"type": "select_all"}' > /dev/null
$CLI command '{"type": "delete", "target": "selection"}' > /dev/null
$CLI command '{"type": "reset_view"}' > /dev/null
sleep 0.3

# === Hex Colors & Corner Radius ===
echo "Creating rounded rectangles with hex colors..."
$CLI command '{"type": "create_shape", "kind": "Rectangle", "position": [100, 100], "size": [120, 80], "fill": "#FF6B6B", "corner_radius": 16}' > /dev/null
$CLI command '{"type": "create_shape", "kind": "Rectangle", "position": [240, 100], "size": [120, 80], "fill": "#4ECDC4", "corner_radius": 16}' > /dev/null
$CLI command '{"type": "create_shape", "kind": "Rectangle", "position": [380, 100], "size": [120, 80], "fill": "#45B7D1", "corner_radius": 16}' > /dev/null
$CLI command '{"type": "create_shape", "kind": "Rectangle", "position": [520, 100], "size": [120, 80], "fill": "#96CEB4", "corner_radius": 16}' > /dev/null
sleep 0.3

# === Strokes ===
echo "Adding shapes with strokes..."
$CLI command '{"type": "create_shape", "kind": "Ellipse", "position": [100, 220], "size": [100, 100], "fill": null, "stroke": {"color": "#FF6B6B", "width": 4}}' > /dev/null
$CLI command '{"type": "create_shape", "kind": "Ellipse", "position": [220, 220], "size": [100, 100], "fill": {"h": 0.5, "s": 0.8, "l": 0.6, "a": 0.5}, "stroke": {"color": "#4ECDC4", "width": 4}}' > /dev/null
$CLI command '{"type": "create_shape", "kind": "Rectangle", "position": [340, 220], "size": [100, 100], "fill": null, "stroke": {"color": "#45B7D1", "width": 3}, "corner_radius": 8}' > /dev/null
sleep 0.3

# === Frames with Hierarchy ===
echo "Creating frames with children..."

# Create a frame (container)
FRAME_RESULT=$($CLI command '{"type": "create_shape", "kind": "Frame", "position": [100, 360], "size": [200, 150], "fill": "#2C3E50"}')
FRAME_ID=$(echo "$FRAME_RESULT" | extract_id)

if [ -n "$FRAME_ID" ]; then
    echo "  Frame created (ID: $FRAME_ID)"

    # Create child shapes and add them to the frame
    CHILD1=$($CLI command '{"type": "create_shape", "kind": "Ellipse", "position": [120, 380], "size": [40, 40], "fill": "#E74C3C"}')
    CHILD1_ID=$(echo "$CHILD1" | extract_id)

    CHILD2=$($CLI command '{"type": "create_shape", "kind": "Rectangle", "position": [180, 380], "size": [40, 40], "fill": "#F39C12", "corner_radius": 6}')
    CHILD2_ID=$(echo "$CHILD2" | extract_id)

    CHILD3=$($CLI command '{"type": "create_shape", "kind": "Ellipse", "position": [240, 380], "size": [40, 40], "fill": "#27AE60"}')
    CHILD3_ID=$(echo "$CHILD3" | extract_id)

    # Add children to frame
    if [ -n "$CHILD1_ID" ]; then
        $CLI command "{\"type\": \"add_child\", \"child\": \"$CHILD1_ID\", \"parent\": \"$FRAME_ID\"}" > /dev/null
    fi
    if [ -n "$CHILD2_ID" ]; then
        $CLI command "{\"type\": \"add_child\", \"child\": \"$CHILD2_ID\", \"parent\": \"$FRAME_ID\"}" > /dev/null
    fi
    if [ -n "$CHILD3_ID" ]; then
        $CLI command "{\"type\": \"add_child\", \"child\": \"$CHILD3_ID\", \"parent\": \"$FRAME_ID\"}" > /dev/null
    fi

    # Enable clipping on the frame
    $CLI command "{\"type\": \"set_clip_children\", \"target\": {\"shape\": \"$FRAME_ID\"}, \"clip\": true}" > /dev/null
    echo "  Children added and clipping enabled"
fi
sleep 0.3

# === Set Position / Set Size ===
echo "Demonstrating absolute positioning..."
# Create a shape and then reposition it
$CLI command '{"type": "create_shape", "kind": "Rectangle", "position": [350, 360], "size": [60, 60], "fill": "#9B59B6", "corner_radius": 30}' > /dev/null
$CLI command '{"type": "select", "target": {"query": {"by_kind": "rectangle"}}}' > /dev/null

# Move specific shape using set_position
$CLI command '{"type": "clear_selection"}' > /dev/null
sleep 0.2

# === Query by Bounds ===
echo "Selecting shapes in a region..."
$CLI command '{"type": "select", "target": {"query": {"in_bounds": {"x": 80, "y": 80, "width": 200, "height": 200}}}}' > /dev/null
$CLI command '{"type": "set_fill", "target": "selection", "fill": "#E91E63"}' > /dev/null
sleep 0.3
$CLI command '{"type": "clear_selection"}' > /dev/null

# === Viewport Manipulation ===
echo "Viewport animations..."

# Zoom in
for i in 1 2 3 4; do
    $CLI command '{"type": "zoom", "factor": 1.1, "center": [300, 250]}' > /dev/null
    sleep 0.05
done

# Pan around
for i in 1 2 3 4 5; do
    $CLI command '{"type": "pan", "delta": [30, 0]}' > /dev/null
    sleep 0.03
done
for i in 1 2 3 4 5; do
    $CLI command '{"type": "pan", "delta": [-30, 0]}' > /dev/null
    sleep 0.03
done

# Zoom out
for i in 1 2 3 4; do
    $CLI command '{"type": "zoom", "factor": 0.91}' > /dev/null
    sleep 0.05
done

$CLI command '{"type": "reset_view"}' > /dev/null
sleep 0.3

# === Batch Commands ===
echo "Batch command: creating a row of shapes..."
$CLI command '{
    "type": "batch",
    "commands": [
        {"type": "create_shape", "kind": "Rectangle", "position": [100, 540], "size": [50, 50], "fill": "#1ABC9C", "corner_radius": 8},
        {"type": "create_shape", "kind": "Ellipse", "position": [170, 540], "size": [50, 50], "fill": "#3498DB"},
        {"type": "create_shape", "kind": "Rectangle", "position": [240, 540], "size": [50, 50], "fill": "#9B59B6", "corner_radius": 25},
        {"type": "create_shape", "kind": "Ellipse", "position": [310, 540], "size": [50, 50], "fill": "#E74C3C"},
        {"type": "create_shape", "kind": "Rectangle", "position": [380, 540], "size": [50, 50], "fill": "#F39C12", "corner_radius": 8}
    ]
}' > /dev/null
sleep 0.3

# === Corner Radius Animation ===
echo "Animating corner radius..."
$CLI command '{"type": "select", "target": {"query": {"by_kind": "rectangle"}}}' > /dev/null
for r in 0 4 8 12 16 20 24 20 16 12 8 4 0; do
    $CLI command "{\"type\": \"set_corner_radius\", \"target\": \"selection\", \"radius\": $r}" > /dev/null
    sleep 0.04
done
$CLI command '{"type": "clear_selection"}' > /dev/null
sleep 0.3

# === Tool Switching Demo ===
echo "Demonstrating tool switching..."
$CLI command '{"type": "set_tool", "tool": "rectangle"}' > /dev/null
sleep 0.2
$CLI command '{"type": "set_tool", "tool": "ellipse"}' > /dev/null
sleep 0.2
$CLI command '{"type": "set_tool", "tool": "frame"}' > /dev/null
sleep 0.2
$CLI command '{"type": "set_tool", "tool": "select"}' > /dev/null
sleep 0.2

# Final stats
echo
echo "=== Stats ==="
$CLI count
$CLI command '{"type": "clear_selection"}' > /dev/null

echo
echo "=== New Features Demonstrated ==="
echo "  - Hex colors (#FF6B6B format)"
echo "  - Corner radius on rectangles"
echo "  - Stroke styling (color + width)"
echo "  - Frames with child hierarchy"
echo "  - Clip children on frames"
echo "  - Query by bounds (in_bounds)"
echo "  - Viewport: pan, zoom, reset_view"
echo "  - Batch commands"
echo "  - Set corner radius dynamically"
echo "  - Tool switching"
echo
echo "Demo complete!"
