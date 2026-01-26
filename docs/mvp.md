# Luna MVP

## Shapes & Primitives

- [x] Rectangle
- [x] Ellipse
- [x] Frame (container with clipping)
- [ ] Text
- [ ] Vector path (pen tool)
- [ ] Line
- [ ] Polygon/Star
- [ ] Image
- [ ] Group (lightweight, non-Frame container)

## Selection & Transform

- [x] Single selection
- [x] Multi-selection (shift-click)
- [ ] Drag selection box (marquee)
- [x] Move shapes
- [x] Resize with handles
- [x] Proportional resize (shift)
- [ ] Rotation
- [ ] Flip horizontal/vertical
- [x] Duplicate (Cmd+D)
- [x] Delete

## Styling

- [x] Solid fill (HSLA)
- [x] Stroke (color, width)
- [x] Corner radius
- [ ] Opacity
- [ ] Multiple fills
- [ ] Multiple strokes
- [ ] Gradient fill (linear, radial)

## Effects

- [ ] Drop shadow
- [ ] Inner shadow

## Layout

- [x] Autolayout (flexbox-style)
- [x] Direction (horizontal/vertical)
- [x] Gap
- [x] Padding
- [x] Main axis alignment
- [x] Cross axis alignment
- [x] Sizing modes (fixed, fill, hug)

## Hierarchy

- [x] Parent/child nesting
- [x] Frame clipping
- [x] Layer list panel
- [ ] Drag reorder in layer list
- [ ] Lock/unlock layers
- [ ] Hide/show layers
- [ ] Rename layers

## Canvas

- [x] Infinite canvas
- [x] Pan (middle-click, hand tool)
- [x] Zoom (scroll, pinch)
- [ ] Zoom to fit
- [ ] Zoom to selection
- [ ] Zoom percentage control
- [ ] Rulers
- [ ] Guides
- [ ] Grid snapping
- [ ] Smart guides (alignment hints)

## Tools

- [x] Select tool
- [x] Pan tool
- [x] Rectangle tool
- [x] Ellipse tool
- [x] Frame tool
- [ ] Text tool
- [ ] Pen tool
- [ ] Line tool

## History

- [ ] Undo (canvas operations)
- [ ] Redo (canvas operations)
- [x] Undo (text input only)

## File Operations

- [x] Save (.luna format)
- [x] Load (.luna format)
- [ ] Export PNG
- [ ] Export SVG
- [ ] Copy/paste between files

## UI Panels

- [x] Tool rail
- [x] Properties panel
- [x] Layer list
- [x] Autolayout inspector
- [ ] Color picker (full)
- [ ] Assets panel
- [ ] Components panel

## Advanced Features (Post-MVP)

- [ ] Components/symbols
- [ ] Boolean operations
- [ ] Masks
- [ ] Multi-page documents
- [ ] Version history

## Performance & Architecture

- [ ] O(1) shape lookup (HashMap/SlotMap)
- [ ] Cached world positions
- [ ] Spatial index for hit testing
- [ ] Incremental rendering (dirty tracking)
- [ ] Shape count: 1000+ without degradation

## API & Extensibility

- [x] Command API (25+ commands)
- [x] Query API
- [ ] Scripting (JS/Lua)
