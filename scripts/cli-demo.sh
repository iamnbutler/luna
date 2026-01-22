#!/bin/bash
# Luna CLI Demo Script
# Demonstrates the debug API capabilities
#
# Usage: ./scripts/cli-demo.sh
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

echo "=== Luna CLI Demo ==="
echo

# Clear canvas
echo "Clearing canvas..."
$CLI command '{"type": "select_all"}' > /dev/null
$CLI command '{"type": "delete", "target": "selection"}' > /dev/null
sleep 0.3

# Create rainbow grid
echo "Creating rainbow grid..."
for i in 0 1 2 3 4 5; do
    for j in 0 1 2 3 4 5; do
        idx=$((i * 6 + j))
        hue=$(python3 -c "print(round($idx / 36.0, 2))")
        x=$((100 + j * 70))
        y=$((100 + i * 70))
        $CLI command "{\"type\": \"create_shape\", \"kind\": \"Rectangle\", \"position\": [$x, $y], \"size\": [60, 60], \"fill\": {\"h\": $hue, \"s\": 0.8, \"l\": 0.5, \"a\": 1.0}}" > /dev/null
    done
done
sleep 0.5

# Wave animation
echo "Wave animation..."
$CLI command '{"type": "select_all"}' > /dev/null
for i in 1 2 3 4 5; do
    $CLI command '{"type": "move", "target": "selection", "delta": [20, 0]}' > /dev/null
    sleep 0.05
done
for i in 1 2 3 4 5; do
    $CLI command '{"type": "move", "target": "selection", "delta": [-20, 0]}' > /dev/null
    sleep 0.05
done
sleep 0.3

# Add diagonal ellipses
echo "Adding ellipses..."
for i in 0 1 2 3 4 5; do
    x=$((130 + i * 70))
    y=$((130 + i * 70))
    $CLI command "{\"type\": \"create_shape\", \"kind\": \"Ellipse\", \"position\": [$x, $y], \"size\": [40, 40], \"fill\": {\"h\": 0.0, \"s\": 0.0, \"l\": 1.0, \"a\": 0.9}}" > /dev/null
done
sleep 0.3

# Bounce ellipses
echo "Bouncing ellipses..."
$CLI command '{"type": "select", "target": {"query": {"by_kind": "ellipse"}}}' > /dev/null
for i in 1 2 3 4; do
    $CLI command '{"type": "move", "target": "selection", "delta": [0, -25]}' > /dev/null
    sleep 0.04
done
for i in 1 2 3 4; do
    $CLI command '{"type": "move", "target": "selection", "delta": [0, 25]}' > /dev/null
    sleep 0.04
done
sleep 0.3

# Scale animation
echo "Scale animation..."
$CLI command '{"type": "select_all"}' > /dev/null
for i in 1 2 3 4 5; do
    $CLI command '{"type": "scale", "target": "selection", "factor": [1.08, 1.08]}' > /dev/null
    sleep 0.03
done
for i in 1 2 3 4 5; do
    $CLI command '{"type": "scale", "target": "selection", "factor": [0.926, 0.926]}' > /dev/null
    sleep 0.03
done
sleep 0.3

# Duplicate with offset
echo "Duplicating..."
$CLI command '{"type": "duplicate", "target": "selection", "offset": [15, 15]}' > /dev/null
sleep 0.3

# Final stats
echo
echo "=== Stats ==="
$CLI count
$CLI command '{"type": "clear_selection"}' > /dev/null

echo
echo "Demo complete!"
