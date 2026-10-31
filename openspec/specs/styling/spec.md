# styling Specification

## Purpose

Defines how the appearance of a drawing's geometries is described and drawn: named styles, per-geometry overrides, the stroke properties with their physical units, dash patterns and arrow heads. Appearance never affects constraints or solving.

## Requirements

### Requirement: A drawing has named styles

Every drawing SHALL hold a table of named styles. It SHALL always contain the style `default`. A named style SHALL hold a value for any subset of the style properties. Style names SHALL follow the object name syntax (object-naming) in a namespace of their own, separate from object names. The style table SHALL be part of the drawing, not of a view or a session.

#### Scenario: New drawing

- **WHEN** a new, empty drawing is created
- **THEN** its style table contains only `default`, with no property set

#### Scenario: Style and object may share a name

- **WHEN** a drawing has a segment named `axis` and a style named `axis` is defined
- **THEN** both exist and neither name is taken by the other

### Requirement: Style properties and built-in values

The style properties SHALL be:
- `width`: the stroke width in TeX points, greater than zero;
- `color`: an sRGB colour written `#rrggbb`;
- `dash`: one of `solid`, `dashed`, `dotted`, `dashdot`;
- `arrow`: one of `none`, `start`, `end`, `both`.

A property set nowhere SHALL take its built-in value: width 0.4, colour `#000000`, dash `solid`, arrow `none`.

#### Scenario: Built-in look

- **WHEN** a segment is created in a new drawing without any style information
- **THEN** it resolves to width 0.4 pt, colour `#000000`, dash `solid` and no arrow

### Requirement: A geometry refers to one style and may override properties

Every geometry SHALL refer to exactly one named style, `default` unless set otherwise. It SHALL also hold a value for any subset of the style properties, its overrides. A geometry SHALL NOT refer to a style that is not in the table.

#### Scenario: Created without style information

- **WHEN** a segment is created without a style or overrides
- **THEN** it refers to `default` and has no overrides

### Requirement: Properties resolve from override to built-in value

A geometry's value of a style property SHALL be, in this order:
1. its override;
2. else the value in its named style;
3. else the value in `default`;
4. else the built-in value.

Changing a named style SHALL change the resolved values of every geometry that refers to it and does not override that property.

#### Scenario: Override wins

- **WHEN** style `axis` has width 0.8 and arrow `end`
- **AND** segment `x` refers to `axis` and overrides colour `#c00000`
- **THEN** `x` resolves to width 0.8, colour `#c00000`, dash `solid` and arrow `end`

#### Scenario: Default fills the gaps

- **WHEN** `default` has width 0.6 and style `axis` has only arrow `end`
- **THEN** a geometry referring to `axis` resolves to width 0.6

#### Scenario: Changing a style changes its users

- **WHEN** segments `x` and `y` refer to `axis`, `y` overrides width 1.0, and `axis` changes its width from 0.8 to 1.2
- **THEN** `x` resolves to width 1.2 and `y` still to 1.0

### Requirement: Styles do not affect geometry

Styles, overrides and their changes SHALL NOT change parameter values, constraints, names, roles, components or the result of any solve. Arrow heads and dash gaps SHALL NOT change a curve's geometry.

#### Scenario: Restyling does not move anything

- **WHEN** a segment constrained to be parallel to another gets arrow `end` and width 2
- **THEN** every parameter value of the drawing is unchanged

### Requirement: Deleting a style

The style `default` SHALL NOT be deleted. A named style SHALL NOT be deleted while a geometry refers to it; the rejection SHALL name those geometries. Deleting an unused named style SHALL remove it from the table.

#### Scenario: Style in use

- **WHEN** segment `x` refers to `axis` and `axis` is deleted
- **THEN** the deletion is rejected, naming `x`
- **AND** the style table is unchanged

#### Scenario: Default stays

- **WHEN** deleting `default` is requested
- **THEN** the deletion is rejected

### Requirement: Style lengths are in TeX points

Stroke widths, dash lengths and arrow sizes SHALL be in TeX points (1 pt = 25.4/72.27 mm). They SHALL map to world units through the document scale (document-settings), so a printed figure shows them at their stated size.

#### Scenario: Width in world units

- **WHEN** the document scale is 10 mm per world unit and a segment has width 0.4 pt
- **THEN** its stroke is 0.4 × 25.4/72.27 / 10 ≈ 0.01406 world units wide

### Requirement: Stroke ends are butt

A stroke SHALL end flat at the end of the curve or of each dash, without a cap extending beyond it.

#### Scenario: Flat end

- **WHEN** a solid segment of width 2 pt from (0, 0) to (1, 0) is drawn
- **THEN** no part of its stroke lies beyond x = 1 or before x = 0

### Requirement: Dash patterns follow the curve

A dash pattern SHALL be laid along the curve's arc length, starting with a dash at the curve's start and continuing across the whole curve. The patterns, with w the stroke width:
- `solid`: no gaps;
- `dashed`: dash 3 pt, gap 3 pt;
- `dotted`: dash w, gap 2 pt;
- `dashdot`: dash 3 pt, gap 2 pt, dash w, gap 2 pt, repeated.

#### Scenario: Dashed segment

- **WHEN** a `dashed` segment is 12 pt long
- **THEN** it shows dashes at 0–3, 6–9 pt along its length and gaps at 3–6, 9–12 pt

#### Scenario: Pattern continues around an arc

- **WHEN** a `dashed` arc is drawn
- **THEN** its dashes are laid by arc length from the arc's start, independent of how the arc is tessellated for display

### Requirement: Arrow head geometry

An arrow head SHALL be a filled isosceles triangle, with w the stroke width:
- **tip:** exactly the curve's endpoint;
- **axis:** along the curve's tangent there, pointing away from the curve;
- **size:** length L = 3 pt + 4.5 w and base width 0.75 L.

`start` puts a head at the start, `end` at the end, `both` at both. It SHALL follow the exact curve, not a display approximation.

#### Scenario: Head on a segment

- **WHEN** segment (0, 0)–(1, 0) with width 0.4 pt and arrow `end` is drawn at a scale where 1 pt is 0.01 world units
- **THEN** its head is the triangle with tip (1, 0) and base corners (0.952, ±0.018)

#### Scenario: Head on an arc follows the tangent

- **WHEN** a quarter arc with center (0, 0), radius 1, start heading 0 and sweep +π/2 has arrow `end`
- **THEN** the head's tip is (0, 1) and its axis points in the direction (−1, 0)

### Requirement: Strokes end at the arrow base

Where a curve has an arrow head, its stroke SHALL end at the point at arc length L from that endpoint. A curve shorter than its arrow heads' lengths together SHALL be drawn with its heads and without a stroke.

#### Scenario: Shortened stroke

- **WHEN** a segment of length 20 pt with width 0.4 pt has arrow `end`
- **THEN** its stroke covers the first 15.2 pt and the head covers the last 4.8 pt

#### Scenario: Very short arrow

- **WHEN** a segment of length 3 pt with width 0.4 pt has arrow `both`
- **THEN** both heads are drawn and no stroke

### Requirement: Labels take the colour of their style

A label SHALL be drawn in the resolved colour of its style. Width, dash and arrow SHALL NOT apply to labels. A label's text size SHALL remain a label attribute (labels).

#### Scenario: Red label

- **WHEN** label `lbl1` refers to a style with colour `#c00000` and width 2
- **THEN** its typeset text is drawn in `#c00000` at its own size

### Requirement: The Canvas shows result geometry in its style

The Canvas SHALL draw each visible result geometry with its resolved style: colour, width at its physical size through the view's scale but never thinner than one pixel, dash pattern and arrow heads. Construction geometry SHALL be drawn in the editor's fixed construction look regardless of its style. Status displays for labels that are not typeset (label-typesetting) SHALL keep their own colours.

#### Scenario: Thin line stays visible

- **WHEN** a segment with width 0.1 pt is shown at a view scale where that is 0.2 pixels
- **THEN** it is drawn one pixel wide

#### Scenario: Construction ignores its style

- **WHEN** a construction segment refers to a style with colour `#c00000` and arrow `end` and construction geometry is shown
- **THEN** it is drawn in the construction look, without arrow head and not in `#c00000`
