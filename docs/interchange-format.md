# Luna Canvas Interchange Format (LCIF)

**Status:** Implemented (basic shapes)
**Version:** 0.1.0
**Format:** KDL (https://kdl.dev)

## Implemented Format

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

### Node types

- `document` - Root node with `version` property
- `rect` - Rectangle shape
- `ellipse` - Ellipse shape

### Shape properties

- First argument: UUID string (shape ID)
- `x`, `y` - Position (f64)
- `width`, `height` - Size (f64)

### Shape children

- `fill` - Fill color with `h`, `s`, `l`, `a` (HSLA, 0-1 range)
- `stroke` - Stroke with `width` and `h`, `s`, `l`, `a`
- `radius` - Corner radius (f64, positional argument)

---

## Goals

1. **Bi-directional**: Import and export with perfect round-trip fidelity
2. **Human-readable**: Editable in a text editor, easy to understand at a glance
3. **Tool-agnostic**: Other applications can implement support
4. **Simple**: Minimal complexity, easy to parse and generate
5. **Extensible**: Support future features without breaking existing files

## Current Data Model

From `node_2`:

```
Shape
├── id: UUID (8-char display format)
├── kind: Rectangle | Ellipse
├── position: (x, y)
├── size: (width, height)
├── fill: Option<Color>
├── stroke: Option<{color, width}>
└── corner_radius: f32

Color: HSLA (h: 0-1, s: 0-1, l: 0-1, a: 0-1)
```

---

## Format Options

### Option A: JSON Schema

The most universally supported format. Every language has a JSON parser.

```json
{
  "lcif": "0.1",
  "canvas": {
    "width": 800,
    "height": 600
  },
  "shapes": [
    {
      "id": "abc12345",
      "type": "rectangle",
      "x": 100,
      "y": 100,
      "width": 150,
      "height": 100,
      "fill": [0.5, 0.8, 0.5, 1.0],
      "stroke": {
        "color": [0, 0, 0, 1],
        "width": 2
      },
      "cornerRadius": 8
    },
    {
      "id": "def67890",
      "type": "ellipse",
      "x": 300,
      "y": 150,
      "width": 120,
      "height": 120,
      "stroke": {
        "color": [0, 0, 0, 1],
        "width": 2
      }
    }
  ]
}
```

**Pros:**
- Universal parser support
- Well-understood
- JSON Schema can provide validation

**Cons:**
- Verbose
- Noisy punctuation
- Colors as arrays aren't intuitive

---

### Option B: YAML

More human-friendly than JSON, same data model.

```yaml
lcif: "0.1"

canvas:
  width: 800
  height: 600

shapes:
  - id: abc12345
    type: rectangle
    x: 100
    y: 100
    width: 150
    height: 100
    fill: hsla(0.5, 0.8, 0.5, 1)
    stroke:
      color: black
      width: 2
    cornerRadius: 8

  - id: def67890
    type: ellipse
    x: 300
    y: 150
    width: 120
    height: 120
    stroke:
      color: black
      width: 2
```

**Pros:**
- More readable than JSON
- Good for hand-editing
- Supports comments

**Cons:**
- Whitespace-sensitive (error-prone)
- Multiple parser implementations vary in edge cases

---

### Option C: Custom DSL (CSS-inspired)

A domain-specific syntax optimized for canvas documents.

```
/* Luna Canvas Interchange Format */
@lcif 0.1;

@canvas {
  size: 800 600;
}

rectangle #abc12345 {
  position: 100 100;
  size: 150 100;
  fill: hsla(0.5 0.8 0.5 1);
  stroke: 2 black;
  corner-radius: 8;
}

ellipse #def67890 {
  position: 300 150;
  size: 120 120;
  stroke: 2 black;
}
```

**Pros:**
- Very readable
- Familiar to web developers
- Concise
- Supports comments

**Cons:**
- Requires custom parser
- Less universal tooling

---

### Option D: S-expressions

Lisp-style format. Unambiguous, simple grammar.

```lisp
(lcif "0.1"
  (canvas 800 600)

  (rectangle "abc12345"
    (at 100 100)
    (size 150 100)
    (fill (hsla 0.5 0.8 0.5 1))
    (stroke 2 (hsla 0 0 0 1))
    (corner-radius 8))

  (ellipse "def67890"
    (at 300 150)
    (size 120 120)
    (stroke 2 black)))
```

**Pros:**
- Trivial to parse (one grammar rule)
- Unambiguous
- Homoiconic (code is data)
- Easy to transform programmatically

**Cons:**
- Less familiar to most developers
- Parentheses can be visually noisy

---

### Option E: Line-oriented (INI-inspired)

Simple, line-based format.

```ini
[lcif]
version = 0.1

[canvas]
size = 800 600

[rectangle:abc12345]
position = 100 100
size = 150 100
fill = hsla(0.5 0.8 0.5 1)
stroke = 2 black
corner-radius = 8

[ellipse:def67890]
position = 300 150
size = 120 120
stroke = 2 black
```

**Pros:**
- Very simple to parse
- Familiar format
- Git-friendly (line-based diffs)

**Cons:**
- Limited nesting capability
- Less elegant for complex data

---

## Recommendation

**Primary format: JSON** with a well-defined schema (Option A)
- Universal compatibility
- Easy to implement in any language
- Can be generated/consumed by LLMs easily
- JSON Schema provides validation

**Secondary format: Custom DSL** (Option C) for human authoring
- `.lcif` extension for JSON
- `.luna` extension for DSL (optional, tooling can convert)

The DSL would be syntactic sugar that compiles to/from JSON.

---

## JSON Schema Draft

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Luna Canvas Interchange Format",
  "type": "object",
  "required": ["lcif", "shapes"],
  "properties": {
    "lcif": {
      "type": "string",
      "description": "Format version",
      "pattern": "^\\d+\\.\\d+$"
    },
    "canvas": {
      "type": "object",
      "properties": {
        "width": { "type": "number" },
        "height": { "type": "number" }
      }
    },
    "shapes": {
      "type": "array",
      "items": { "$ref": "#/definitions/shape" }
    }
  },
  "definitions": {
    "color": {
      "oneOf": [
        {
          "type": "array",
          "items": { "type": "number" },
          "minItems": 4,
          "maxItems": 4,
          "description": "HSLA as [h, s, l, a] where all values 0-1"
        },
        {
          "type": "string",
          "description": "Named color or hsla() function"
        }
      ]
    },
    "shape": {
      "type": "object",
      "required": ["id", "type", "x", "y", "width", "height"],
      "properties": {
        "id": { "type": "string" },
        "type": { "enum": ["rectangle", "ellipse"] },
        "x": { "type": "number" },
        "y": { "type": "number" },
        "width": { "type": "number" },
        "height": { "type": "number" },
        "fill": { "$ref": "#/definitions/color" },
        "stroke": {
          "type": "object",
          "properties": {
            "color": { "$ref": "#/definitions/color" },
            "width": { "type": "number" }
          }
        },
        "cornerRadius": { "type": "number" }
      }
    }
  }
}
```

---

## Color Representation

Colors use HSLA (Hue, Saturation, Lightness, Alpha):

| Component | Range | Description |
|-----------|-------|-------------|
| H (hue) | 0-1 | Color wheel position (0=red, 0.33=green, 0.67=blue) |
| S (saturation) | 0-1 | Color intensity (0=gray, 1=vivid) |
| L (lightness) | 0-1 | Brightness (0=black, 0.5=normal, 1=white) |
| A (alpha) | 0-1 | Opacity (0=transparent, 1=opaque) |

**Format options:**
- Array: `[0.5, 0.8, 0.5, 1.0]`
- Function: `"hsla(0.5, 0.8, 0.5, 1)"`
- Named: `"black"`, `"white"`, `"transparent"`

Named colors (optional):
- `black` = `[0, 0, 0, 1]`
- `white` = `[0, 0, 1, 1]`
- `transparent` = `[0, 0, 0, 0]`

---

## File Extensions

- `.lcif` - JSON format
- `.lcif.json` - Explicit JSON (for tooling)

---

## Example: Complete Document

```json
{
  "lcif": "0.1",
  "meta": {
    "created": "2024-01-20T10:30:00Z",
    "generator": "Luna 0.1.1"
  },
  "canvas": {
    "width": 800,
    "height": 600,
    "background": [0, 0, 1, 1]
  },
  "shapes": [
    {
      "id": "bg-rect",
      "type": "rectangle",
      "x": 50,
      "y": 50,
      "width": 200,
      "height": 150,
      "fill": [0.6, 0.7, 0.5, 1],
      "cornerRadius": 12
    },
    {
      "id": "accent",
      "type": "ellipse",
      "x": 300,
      "y": 100,
      "width": 100,
      "height": 100,
      "fill": [0.95, 0.8, 0.6, 1],
      "stroke": {
        "color": [0, 0, 0.2, 1],
        "width": 3
      }
    }
  ]
}
```

---

## Future Considerations

- **Groups**: Nested shape collections with transforms
- **References**: Reusable style definitions
- **Text**: Text nodes with font properties
- **Images**: Embedded or linked image references
- **Paths**: Arbitrary vector paths (bezier curves)
- **Constraints**: Layout relationships between shapes
- **Animations**: Keyframe-based motion

---

## Open Questions

1. Should IDs be optional or required?
2. Should we support relative positioning (parent-relative)?
3. How to handle vendor extensions? (`x-luna-*`?)
4. Should z-order be explicit or implicit (array order)?
5. Include viewport/zoom state or keep it document-only?
