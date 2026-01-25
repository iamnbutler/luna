#!/bin/bash
# Luna CLI Demo - Autolayout
# Demonstrates frame autolayout features
#
# Usage: ./scripts/cli/autolayout.sh
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

echo "=== Luna Autolayout Demo ==="
echo

# Clear canvas
echo "Clearing canvas..."
$CLI command '{"type": "select_all"}' > /dev/null
$CLI command '{"type": "delete", "target": "selection"}' > /dev/null
$CLI command '{"type": "reset_view"}' > /dev/null
sleep 0.3

# === Row Layout ===
echo "Creating frame with row layout..."
FRAME1=$($CLI command '{"type": "create_shape", "kind": "Frame", "position": [50, 50], "size": [400, 100], "fill": "#2C3E50"}')
FRAME1_ID=$(echo "$FRAME1" | grep -o '"created":\["\?[0-9]*"\?' | grep -o '[0-9]*' | head -1)

if [ -n "$FRAME1_ID" ]; then
    echo "  Frame created (ID: $FRAME1_ID)"

    # Enable autolayout with row direction
    $CLI command "{\"type\": \"set_layout\", \"target\": {\"shape\": $FRAME1_ID}, \"layout\": {\"direction\": \"Row\", \"gap\": 16, \"padding\": {\"top\": 16, \"right\": 16, \"bottom\": 16, \"left\": 16}, \"main_axis_alignment\": \"Start\", \"cross_axis_alignment\": \"Center\"}}" > /dev/null
    echo "  Autolayout enabled (Row, gap: 16, padding: 16)"

    # Create children - they'll be auto-positioned by layout
    for i in 1 2 3 4; do
        CHILD=$($CLI command "{\"type\": \"create_shape\", \"kind\": \"Rectangle\", \"position\": [0, 0], \"size\": [60, 60], \"fill\": \"#E74C3C\", \"corner_radius\": 8}")
        CHILD_ID=$(echo "$CHILD" | grep -o '"created":\["\?[0-9]*"\?' | grep -o '[0-9]*' | head -1)
        if [ -n "$CHILD_ID" ]; then
            $CLI command "{\"type\": \"add_child\", \"child\": $CHILD_ID, \"parent\": $FRAME1_ID}" > /dev/null
        fi
    done
    echo "  Added 4 children (auto-positioned by layout)"
fi
sleep 0.5

# === Column Layout ===
echo "Creating frame with column layout..."
FRAME2=$($CLI command '{"type": "create_shape", "kind": "Frame", "position": [500, 50], "size": [120, 350], "fill": "#34495E"}')
FRAME2_ID=$(echo "$FRAME2" | grep -o '"created":\["\?[0-9]*"\?' | grep -o '[0-9]*' | head -1)

if [ -n "$FRAME2_ID" ]; then
    echo "  Frame created (ID: $FRAME2_ID)"

    # Enable autolayout with column direction
    $CLI command "{\"type\": \"set_layout\", \"target\": {\"shape\": $FRAME2_ID}, \"layout\": {\"direction\": \"Column\", \"gap\": 12, \"padding\": {\"top\": 12, \"right\": 12, \"bottom\": 12, \"left\": 12}, \"main_axis_alignment\": \"Start\", \"cross_axis_alignment\": \"Stretch\"}}" > /dev/null
    echo "  Autolayout enabled (Column, gap: 12, cross: Stretch)"

    # Create children with different sizes - stretch will make them equal width
    COLORS=("#1ABC9C" "#3498DB" "#9B59B6" "#E67E22")
    HEIGHTS=(40 60 50 45)
    for i in 0 1 2 3; do
        COLOR=${COLORS[$i]}
        HEIGHT=${HEIGHTS[$i]}
        CHILD=$($CLI command "{\"type\": \"create_shape\", \"kind\": \"Rectangle\", \"position\": [0, 0], \"size\": [50, $HEIGHT], \"fill\": \"$COLOR\", \"corner_radius\": 6}")
        CHILD_ID=$(echo "$CHILD" | grep -o '"created":\["\?[0-9]*"\?' | grep -o '[0-9]*' | head -1)
        if [ -n "$CHILD_ID" ]; then
            $CLI command "{\"type\": \"add_child\", \"child\": $CHILD_ID, \"parent\": $FRAME2_ID}" > /dev/null
        fi
    done
    echo "  Added 4 children (stretched to frame width)"
fi
sleep 0.5

# === Space Between ===
echo "Creating frame with space-between alignment..."
FRAME3=$($CLI command '{"type": "create_shape", "kind": "Frame", "position": [50, 200], "size": [400, 80], "fill": "#1A252F"}')
FRAME3_ID=$(echo "$FRAME3" | grep -o '"created":\["\?[0-9]*"\?' | grep -o '[0-9]*' | head -1)

if [ -n "$FRAME3_ID" ]; then
    echo "  Frame created (ID: $FRAME3_ID)"

    # Enable autolayout with space-between
    $CLI command "{\"type\": \"set_layout\", \"target\": {\"shape\": $FRAME3_ID}, \"layout\": {\"direction\": \"Row\", \"gap\": 0, \"padding\": {\"top\": 16, \"right\": 16, \"bottom\": 16, \"left\": 16}, \"main_axis_alignment\": \"SpaceBetween\", \"cross_axis_alignment\": \"Center\"}}" > /dev/null
    echo "  Autolayout enabled (Row, SpaceBetween)"

    # Create children
    for i in 1 2 3; do
        CHILD=$($CLI command "{\"type\": \"create_shape\", \"kind\": \"Ellipse\", \"position\": [0, 0], \"size\": [48, 48], \"fill\": \"#F39C12\"}")
        CHILD_ID=$(echo "$CHILD" | grep -o '"created":\["\?[0-9]*"\?' | grep -o '[0-9]*' | head -1)
        if [ -n "$CHILD_ID" ]; then
            $CLI command "{\"type\": \"add_child\", \"child\": $CHILD_ID, \"parent\": $FRAME3_ID}" > /dev/null
        fi
    done
    echo "  Added 3 children (evenly spaced)"
fi
sleep 0.5

# === Centered Layout ===
echo "Creating frame with centered content..."
FRAME4=$($CLI command '{"type": "create_shape", "kind": "Frame", "position": [50, 320], "size": [400, 100], "fill": "#2C3E50"}')
FRAME4_ID=$(echo "$FRAME4" | grep -o '"created":\["\?[0-9]*"\?' | grep -o '[0-9]*' | head -1)

if [ -n "$FRAME4_ID" ]; then
    echo "  Frame created (ID: $FRAME4_ID)"

    # Enable autolayout centered
    $CLI command "{\"type\": \"set_layout\", \"target\": {\"shape\": $FRAME4_ID}, \"layout\": {\"direction\": \"Row\", \"gap\": 24, \"padding\": {\"top\": 0, \"right\": 0, \"bottom\": 0, \"left\": 0}, \"main_axis_alignment\": \"Center\", \"cross_axis_alignment\": \"Center\"}}" > /dev/null
    echo "  Autolayout enabled (Row, Center/Center)"

    # Create children
    SIZES=(30 50 40)
    COLORS=("#E74C3C" "#27AE60" "#3498DB")
    for i in 0 1 2; do
        SIZE=${SIZES[$i]}
        COLOR=${COLORS[$i]}
        CHILD=$($CLI command "{\"type\": \"create_shape\", \"kind\": \"Rectangle\", \"position\": [0, 0], \"size\": [$SIZE, $SIZE], \"fill\": \"$COLOR\", \"corner_radius\": 4}")
        CHILD_ID=$(echo "$CHILD" | grep -o '"created":\["\?[0-9]*"\?' | grep -o '[0-9]*' | head -1)
        if [ -n "$CHILD_ID" ]; then
            $CLI command "{\"type\": \"add_child\", \"child\": $CHILD_ID, \"parent\": $FRAME4_ID}" > /dev/null
        fi
    done
    echo "  Added 3 children (centered)"
fi
sleep 0.5

# Final stats
echo
echo "=== Stats ==="
$CLI count
$CLI command '{"type": "clear_selection"}' > /dev/null

echo
echo "=== Autolayout Features Demonstrated ==="
echo "  - Row layout with gap and padding"
echo "  - Column layout with cross-axis stretch"
echo "  - SpaceBetween alignment"
echo "  - Center/Center alignment"
echo "  - Auto-positioning of children"
echo
echo "Demo complete!"
