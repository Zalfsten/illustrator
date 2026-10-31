# Proposal

## Why

The product goal is figures with durable geometric relationships and free illustration styling. The constraint half exists. The styling half does not: every result curve is drawn as the same thin black line, without dashes, colours or arrows. Scientific and teaching figures cannot be drawn that way (axes, vectors, forces, hidden edges). Print export (#13) is blocked on this, because a print export without styles would export uniform hairlines. This change implements #10.

## What Changes

- **Named styles in the drawing.**
  - A drawing holds a table of named styles: a built-in `default`, which can be changed but not deleted, plus styles the user defines, such as `axis`, `vector` or `hidden`.
  - A named style lists only the properties that differ from `default`.
  - Styles belong to the drawing like the document settings. Changing them is a history step, and they are saved with the file.
- **Style and overrides per geometry.**
  - Every geometry refers to one named style (`default` unless set) and may override single properties.
  - A property resolves in this order: the object's override, then its named style, then `default`, then the built-in value.
  - Changing a named style changes every object that uses it.
- **Properties in this change:**
  - stroke width in TeX points;
  - colour `#rrggbb`;
  - dash pattern: one of `solid`, `dashed`, `dotted`, `dashdot`, with fixed lengths in pt;
  - arrow head at the curve's start, end, both or none.
  - Labels take only the colour; their size stays a label attribute.
- **Arrow heads have one shape.**
  - The head is a filled triangle whose size follows from the stroke width.
  - Its tip is exactly on the curve's endpoint, and it points along the exact curve tangent there.
  - The stroke ends at the base of the head.
  - Display and exports compute the head the same way.
- **Physical sizes.** Widths and dash lengths are in TeX points and map to the drawing through the document scale. The Canvas draws them at their physical size, never thinner than one pixel.
- **Construction geometry** keeps its style but is drawn in the editor's fixed construction look. Display modes (#52) may change this later.
- **Commands:**
  - `define_style` creates a named style or changes it;
  - `delete_style` deletes a style, and is rejected while a geometry uses it or for `default`;
  - `set_style` sets a geometry's style and overrides; `null` removes an override.
  - `create` accepts the same style fields.
  - The session reports the style table.
- **File format version 3.** A new envelope field `styles` holds the style table, and a geometry's `create` carries its style and overrides. Version 1 and 2 files load with only the default style. Saving always writes version 3.
- **Editor without a selection model.**
  - The toolbar holds a current style plus optional overrides, and new geometry gets both.
  - A brush tool applies them to a clicked curve.
  - A "Styles" window defines, changes and deletes named styles.
  - Editing a single object's overrides in a panel waits for the selection model (separate issue).

## Non-goals

- Fill of closed regions: the geometry model has no regions yet.
- Free dash lists, further arrow shapes, arrow sizes independent of the stroke width, transparency, named colours, line caps and joins chosen by the user.
- Renaming a style.
- Style inheritance between named styles other than through `default`.
- A selection model and an inspector panel.
- Brushing labels in the editor: there is no label pick yet (#63). Labels get styles through commands and files.
- SVG and PDF export: #9 and #13 consume the styles and the shared arrow and dash geometry defined here.
- Display modes and determination colouring (#52).

The style model, its resolution, the arrow and dash geometry, units and the file format are shared specs that every implementation follows, so that a figure prints the same everywhere. The toolbar, brush tool and style window are design decisions of the Rust implementation.

## Capabilities

### New Capabilities

- `styling`: what this capability covers:
  - named styles and the built-in default;
  - the style properties, their units and built-in values;
  - a geometry's style reference and overrides, and how a property resolves;
  - deleting styles;
  - dash patterns and arrow head geometry;
  - how styles are displayed on the Canvas.

### Modified Capabilities

- `sketch-commands`:
  - **New operations:** `define_style`, `delete_style` and `set_style`.
  - **Style fields on `create`:** `style`, `width`, `color`, `dash` and `arrow`, which become reserved field names.
  - **Declarative export:** the subset carries style and overrides in `create`.
  - **Queries:** the style table is queryable.
- `sketch-file`:
  - version 3 with the envelope field `styles`, one style per line;
  - version 1 and 2 files load with only the default style;
  - saving writes version 3.
- `geometry-ir`: the shared metadata of a geometry includes its style reference and overrides.

## Impact

- **`constraint_runtime`:**
  - the style model in the Document: a style table beside the settings, and style plus overrides per geometry;
  - resolution, plus platform-neutral stroke geometry (pt to world, arrow head polygon, trimming, dashing by arc length).
  - It stays serde-free.
- **`sketch_session`:** the new commands and the `create` style fields, the interchange form and schema, declarative export, and file version 3 with migration from versions 1 and 2.
- **`sketch_ui`:** styled strokes, dashes and arrow heads on the Canvas, label colour, the current-style toolbar, the brush tool and the Styles window.
- **Unchanged:** labels' typesetting, the solver and constraints. Styles never affect solving.
- **Files:** existing files load unchanged. Files saved by this version are version 3 and cannot be read by older builds.
