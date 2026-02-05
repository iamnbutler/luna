# Luna Project Status Report

**Generated:** 2026-02-04  
**Version:** 0.1.1  
**Last Commit:** 773e9a9 (2026-01-26)

---

## Overview

Luna is an in-development software design tool in the vein of Figma, Sketch, and old Framer. Built on top of [GPUI](https://www.gpui.rs/), the UI framework designed by the Zed Industries team to power Zed.

**Current Stage:** Toy editor / Early MVP  
**Lines of Code:** ~12,000 lines of Rust across 36 source files

---

## Project Structure

The project follows a workspace-based architecture with 9 crates:

### Core Crates

- **`luna/`** - Main application binary
  - Entry point and app initialization
  - Serialization/deserialization logic
  - Asset management integration

- **`canvas/`** - Canvas rendering & interactions
  - Viewport management (pan, zoom)
  - Element rendering
  - Canvas-level event handling

- **`node/`** - Shape data model
  - Shape types: Rectangle, Ellipse, Frame
  - Layout system (Autolayout/Flexbox)
  - Coordinate systems and transformations
  - Shape ID management

### UI & Theming

- **`ui/`** - UI components
  - Properties panel
  - Layer list
  - Tool rail
  - Text input component with bidi support

- **`theme/`** - Theming system
  - Color management
  - Theme definitions

- **`assets/`** - Embedded assets
  - Fonts, icons, and other resources
  - Built with rust-embed

### Extensibility

- **`api/`** - Command/query API for scripting
  - 25+ commands for canvas manipulation
  - Query API for inspecting state
  - Server infrastructure for external control
  - Command executor

- **`cli/`** - CLI tool
  - Command-line interface for Luna operations

- **`interchange/`** - File format (.luna)
  - KDL-based file format (v0.1.0)
  - Project serialization/deserialization

### Legacy Code

- **`archive/crates/`** - Archived legacy crates preserved for reference

---

## Implementation Status

### ✅ Completed Features

#### Shapes & Primitives
- ✅ Rectangle
- ✅ Ellipse
- ✅ Frame (container with clipping)
- ✅ Text

#### Selection & Transform
- ✅ Single selection
- ✅ Multi-selection (shift-click)
- ✅ Move shapes
- ✅ Resize with handles
- ✅ Proportional resize (shift)
- ✅ Duplicate (Cmd+D)
- ✅ Delete

#### Styling
- ✅ Solid fill (HSLA)
- ✅ Stroke (color, width)
- ✅ Corner radius

#### Layout
- ✅ Autolayout (flexbox-style)
- ✅ Direction (horizontal/vertical)
- ✅ Gap
- ✅ Padding
- ✅ Main axis alignment
- ✅ Cross axis alignment
- ✅ Sizing modes (fixed, fill, hug)

#### Hierarchy
- ✅ Parent/child nesting
- ✅ Frame clipping
- ✅ Layer list panel

#### Canvas
- ✅ Infinite canvas
- ✅ Pan (middle-click, hand tool)
- ✅ Zoom (scroll, pinch)

#### Tools
- ✅ Select tool
- ✅ Pan tool
- ✅ Rectangle tool
- ✅ Ellipse tool
- ✅ Frame tool
- ✅ Text tool

#### File Operations
- ✅ Save (.luna format using KDL)
- ✅ Load (.luna format)

#### UI Panels
- ✅ Tool rail
- ✅ Properties panel
- ✅ Layer list
- ✅ Autolayout inspector

#### API & Extensibility
- ✅ Command API (27+ commands)
- ✅ Query API

### 🚧 In Progress / Partial

- ⚠️ Undo/Redo (text input only, canvas operations pending)

### ❌ Not Yet Implemented

#### Shapes
- ❌ Vector path (pen tool)
- ❌ Line
- ❌ Polygon/Star
- ❌ Image
- ❌ Group (lightweight container)

#### Selection & Transform
- ❌ Drag selection box (marquee)
- ❌ Rotation
- ❌ Flip horizontal/vertical

#### Styling
- ❌ Opacity
- ❌ Multiple fills
- ❌ Multiple strokes
- ❌ Gradient fill (linear, radial)

#### Effects
- ❌ Drop shadow
- ❌ Inner shadow

#### Hierarchy
- ❌ Drag reorder in layer list
- ❌ Lock/unlock layers
- ❌ Hide/show layers
- ❌ Rename layers

#### Canvas
- ❌ Zoom to fit
- ❌ Zoom to selection
- ❌ Zoom percentage control
- ❌ Rulers
- ❌ Guides
- ❌ Grid snapping
- ❌ Smart guides (alignment hints)

#### Tools
- ❌ Pen tool
- ❌ Line tool

#### History
- ❌ Undo (canvas operations)
- ❌ Redo (canvas operations)

#### File Operations
- ❌ Export PNG
- ❌ Export SVG
- ❌ Copy/paste between files

#### UI Panels
- ❌ Color picker (full)
- ❌ Assets panel
- ❌ Components panel

#### Performance & Architecture
- ❌ O(1) shape lookup (HashMap/SlotMap)
- ❌ Cached world positions
- ❌ Spatial index for hit testing
- ❌ Incremental rendering (dirty tracking)
- ❌ Shape count: 1000+ without degradation

#### API & Extensibility
- ❌ Scripting (JS/Lua)

#### Advanced Features (Post-MVP)
- ❌ Components/symbols
- ❌ Boolean operations
- ❌ Masks
- ❌ Multi-page documents
- ❌ Version history

---

## File Format

**Format:** KDL (KDL Document Language)  
**Version:** 0.1.0  
**Status:** Implemented for basic shapes

### Current Format Example

```kdl
document version="0.1" {
  rect "uuid-here" x=100.0 y=100.0 width=150.0 height=100.0 {
    fill h=0.5 s=0.8 l=0.5 a=1.0
    stroke width=2.0 h=0.0 s=0.0 l=0.0 a=1.0
    radius 8.0
  }
  ellipse "uuid-here" x=300.0 y=150.0 width=120.0 height=120.0 {
    stroke width=2.0 h=0.0 s=0.0 l=0.0 a=1.0
  }
}
```

### Format Goals
1. **Bi-directional** - Perfect round-trip fidelity
2. **Human-readable** - Editable in text editor
3. **Tool-agnostic** - Other applications can implement support
4. **Simple** - Minimal complexity
5. **Extensible** - Support future features without breaking existing files

---

## Technology Stack

### Core Dependencies
- **GPUI** (v0.2.2) - UI framework with test support and inspector
- **Rust** (Edition 2021) - Systems programming language
- **KDL** (v6.5.0) - Document language for file format
- **Glam** (v0.30.5) - Math library for graphics
- **UUID** (v1.16.0) - Unique identifiers
- **Serde** (v1.0.221) - Serialization framework
- **Taffy** (v0.4.4) - Layout engine
- **Spool** (v1.0+) - Git-native task tracker

### Additional Libraries
- `palette` - Color manipulation
- `quadtree_rs` - Spatial indexing (planned)
- `slotmap` - Efficient entity storage
- `smol` - Async runtime
- `unicode-bidi` / `unicode-segmentation` - Text support

---

## Code Metrics

- **Total Source Files:** 36 Rust files
- **Total Lines:** ~12,000 lines
- **Crates:** 9 workspace members
- **Active Development:** Yes

### Crate Breakdown

| Crate | Purpose | Key Files |
|-------|---------|-----------|
| `luna` | Main app | luna.rs, serialization.rs, assets.rs |
| `canvas` | Canvas engine | canvas.rs, viewport.rs, element.rs |
| `node` | Data model | shape.rs, layout.rs, coords.rs, layout_engine.rs |
| `ui` | Components | properties.rs, layer_list.rs, tool_rail.rs, input/* |
| `theme` | Theming | lib.rs |
| `assets` | Resources | assets.rs |
| `api` | Scripting API | command.rs, query.rs, executor.rs, server.rs |
| `cli` | CLI tool | main.rs |
| `interchange` | File format | lib.rs, project.rs |

---

## Recent Activity

**Last Commit:** 773e9a9 (2026-01-26)  
**Message:** "Merge pull request #27 from iamnbutler/consolidate-crates"

The project recently underwent a major restructuring to consolidate crates, improving maintainability and reducing complexity.

---

## Development Priorities

Based on the MVP checklist, the next critical features to implement are:

### High Priority (Core Functionality)
1. **Canvas Undo/Redo** - Essential editing feature
2. **Text Enhancements** - In-place editing, wrapping, alignment
3. **Marquee Selection** - Drag selection box
4. **Export** - PNG/SVG export capabilities

### Medium Priority (UX Improvements)
1. **Layer Management** - Rename, lock, hide, reorder
2. **Zoom Controls** - Zoom to fit, zoom to selection, percentage control
3. **Opacity** - Shape opacity support
4. **Rotation** - Shape rotation with handles

### Low Priority (Polish)
1. **Grid & Guides** - Snapping, rulers, smart guides
2. **Effects** - Drop shadows, inner shadows
3. **Gradients** - Linear and radial gradient fills
4. **Color Picker** - Full-featured color selection UI

### Performance (Future)
1. **Optimized Hit Testing** - Spatial indexing with quadtree
2. **Cached Transforms** - World position caching
3. **Dirty Tracking** - Incremental rendering
4. **Scale Testing** - Support 1000+ shapes

---

## Architecture Notes

### GPUI Integration

Luna leverages GPUI's element system:
- `Element` trait: `request_layout`, `prepaint`, `paint` methods
- `Render` trait uses `Context<Self>` not `App`
- `Entity<T>` for state management
- `EventEmitter<Event>` for component events

### Data Model

```
Shape
├── id: UUID (8-char display format)
├── kind: Rectangle | Ellipse | Frame | Text
├── position: (x, y)
├── size: (width, height)
├── fill: Option<Color>
├── stroke: Option<{color, width}>
├── corner_radius: f32
├── text_content: Option<String>  # Text shapes only
├── font_size: Option<f32>         # Text shapes only
└── layout: Option<AutoLayout>

Color: HSLA (h: 0-1, s: 0-1, l: 0-1, a: 0-1)
```

### Command API

The API crate exposes 27+ commands including:
- Shape creation (create_rectangle, create_ellipse, create_frame, create_text)
- Manipulation (move_shape, resize_shape, delete_shape)
- Styling (set_fill, set_stroke, set_corner_radius)
- Text (set_text_content, set_font_size)
- Layout (set_autolayout, set_gap, set_padding)
- Selection (select_shape, deselect_all)

---

## Known Limitations

1. **No Undo/Redo** for canvas operations (only text input)
2. **Limited Shape Types** - Only Rectangle, Ellipse, Frame, Text
3. **Basic Text Support** - No in-place editing, wrapping, or rich formatting
4. **No Export** - Cannot output to PNG/SVG
5. **Performance Unoptimized** - No spatial indexing or caching
6. **Limited File Format** - Basic shapes only, no groups/paths
7. **Basic UI** - Missing many expected panels and controls

---

## Task Management

The project uses [Spool](https://crates.io/crates/spool) for git-native, event-sourced task tracking.

> **Note:** Spool binary not currently installed in environment. Task tracking would require installation via `cargo install spool`.

---

## Getting Started

### Prerequisites
- Rust toolchain (Edition 2021)
- GPUI dependencies

### Building
```bash
cargo build --release
```

### Running
```bash
cargo run --bin Luna
```

### CLI Tool
```bash
cargo run --bin luna-cli -- [command]
```

---

## Commit Conventions

The project follows these commit message prefixes:
- `add` - Something new (feature, file, component)
- `fix` - Fixed behavior or mistake
- `cleanup` - No functional change, tidying
- `remove` - Deleted something

Example: `add: input component for properties panel`

---

## Future Roadmap

### Short Term (Next Release)
- Complete undo/redo system
- Add text support
- Implement export functionality
- Improve layer management

### Medium Term
- Performance optimizations
- Component/symbol system
- Advanced styling (gradients, effects)
- Full keyboard shortcut support

### Long Term
- Plugin/scripting system (JS/Lua)
- Collaboration features
- Version history
- Multi-page documents
- Advanced vector editing

---

## Documentation

- `README.md` - Project overview
- `docs/mvp.md` - MVP feature checklist
- `docs/interchange-format.md` - File format specification
- `CLAUDE.md` - AI assistant guidelines

---

## Conclusion

Luna is in active early development with a solid foundation in place. Core features like basic shapes, autolayout, and file persistence are working. The next phase should focus on essential editing features (undo/redo, text, export) and UX improvements (layer management, zoom controls) to reach a usable MVP state.

The architecture is clean with well-separated concerns across crates, making it maintainable and extensible. The choice of GPUI as the UI framework provides a modern, performant base to build upon.

**Status:** Toy editor → MVP transition  
**Next Milestone:** Complete core editing features for practical use
