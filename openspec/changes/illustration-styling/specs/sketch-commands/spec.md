# Spec Delta

## MODIFIED Requirements

### Requirement: Create geometry commands

The session SHALL provide commands that create a line segment (start and end point), a circular arc (center, radius, start heading, sweep) and a label (origin, heading and its attributes, labels). Each SHALL accept an optional name, an optional role (result by default), an optional style name (`default` by default) and optional style overrides (styling). On success the result SHALL report the name of the created geometry.

#### Scenario: Create a segment with an automatic name

- **WHEN** a segment is created without a name in an empty drawing
- **THEN** the command succeeds and reports the name `seg1`

#### Scenario: Create with an explicit name

- **WHEN** an arc is created with the name `fillet`
- **THEN** the command succeeds and the arc is named `fillet`

#### Scenario: Create a label with an automatic name

- **WHEN** a label is created without a name in an empty drawing
- **THEN** the command succeeds and reports the name `lbl1`

#### Scenario: Create with a style and an override

- **WHEN** a segment is created with style `axis` and colour `#c00000` while `axis` exists
- **THEN** the segment refers to `axis` and overrides its colour

#### Scenario: Create with an unknown style

- **WHEN** a segment is created with style `nope` while no style is named `nope`
- **THEN** the command fails with the `unknown style` class
- **AND** the drawing is unchanged

### Requirement: Commands have one interchange form

Every command SHALL have one interchange form, a JSON object whose `op` field names the operation. The operation set SHALL be closed: `create`, `constrain`, `delete`, `rename`, `set_role`, `set`, `set_document`, `define_style`, `delete_style` and `set_style`. Files, the command protocol and clients that list commands SHALL use this form; front-ends MAY offer other syntaxes, but SHALL map them to these commands.

#### Scenario: A rename in interchange form

- **WHEN** the interchange text `{"op": "rename", "name": "seg1", "new_name": "x_axis"}` is read
- **THEN** it is the rename command from `seg1` to `x_axis`

#### Scenario: Unknown operation is rejected

- **WHEN** the interchange text `{"op": "parallel", "a": "seg1.direction", "b": "seg2.direction"}` is read
- **THEN** reading fails because `parallel` is not an operation

#### Scenario: A set in interchange form

- **WHEN** the interchange text `{"op": "set", "kind": "label", "name": "lbl1", "valign": "last"}` is read
- **THEN** it is the set command that changes only the vertical alignment of `lbl1`

#### Scenario: A style definition in interchange form

- **WHEN** the interchange text `{"op": "define_style", "style": "axis", "width": 0.8, "arrow": "end"}` is read
- **THEN** it is the command that sets width 0.8 and arrow `end` in style `axis`

### Requirement: Interchange field shapes

Fields SHALL have these shapes: numbers as JSON numbers; points as two-element arrays `[x, y]`; angles in radians; names as strings; addresses of points, free vectors and lengths as address strings (object-naming); a role as `"result"` or `"construction"`. Label fields SHALL be: `origin` a point; `heading` an angle; `source` a string; `markup` `"latex"` or `"text"`; `halign` `"left"`, `"center"` or `"right"`; `valign` `"first"` or `"last"`; `offset` a two-element array of pt; `size` a number of pt. On `create`, a label SHALL require `origin`, `heading` and `source`; an absent `size` SHALL mean inherited. `name` SHALL be optional on `create` and `constrain`; `role` SHALL be optional on `create` and default to result. Style fields SHALL be: `style` a style name; `width` a number of pt; `color` a string `#rrggbb` (hex digits in either case); `dash` `"solid"`, `"dashed"`, `"dotted"` or `"dashdot"`; `arrow` `"none"`, `"start"`, `"end"` or `"both"`. On `create` of every geometry kind they SHALL be optional; the property fields are overrides.

#### Scenario: Segment in interchange form

- **WHEN** the interchange text `{"op": "create", "kind": "segment", "start": [0, 0], "end": [3, 0]}` is read
- **THEN** it is the create command for a segment from (0, 0) to (3, 0) with role result and an automatic name

#### Scenario: Arc and fix in interchange form

- **WHEN** `{"op": "create", "kind": "arc", "name": "fillet", "role": "construction", "center": [3, 1], "radius": 1, "start_heading": -1.5707963267948966, "sweep": 1.5707963267948966}` is read
- **AND** `{"op": "constrain", "kind": "fix", "point": "fillet.start", "target": [3, 0]}` is read
- **THEN** they are a construction arc named `fillet` and a fix of its start to (3, 0)

#### Scenario: Label in interchange form

- **WHEN** `{"op": "create", "kind": "label", "origin": [1, 0], "heading": 0, "source": "$x$", "offset": [2, 3]}` is read
- **THEN** it is the create command for a label at (1, 0) with heading 0, source `$x$`, offset [2, 3] and the defaults for the other attributes

#### Scenario: Label without source is rejected

- **WHEN** a label `create` object has no `source` field
- **THEN** reading fails because `source` is missing

#### Scenario: Styled segment in interchange form

- **WHEN** `{"op": "create", "kind": "segment", "start": [0, 0], "end": [3, 0], "style": "axis", "dash": "dashed"}` is read
- **THEN** it is the create command for a segment that refers to style `axis` and overrides the dash pattern with `dashed`

#### Scenario: Unknown dash pattern is rejected

- **WHEN** a `create` object has `"dash": "dashdotdot"`
- **THEN** reading fails because `dash` has a wrong value

### Requirement: Interchange reading is strict

Reading SHALL reject an object with an unknown field, a missing required field, a field of the wrong shape, or an unknown `op` or `kind`, and SHALL NOT fall back to defaults for misspelled fields. The field names `op`, `kind`, `name`, `role`, `style`, `width`, `color`, `dash` and `arrow` SHALL be reserved: no kind SHALL use them for its own parameters.

#### Scenario: Misspelled optional field is rejected

- **WHEN** a `create` object for a segment contains `"rol": "construction"`
- **THEN** reading fails because of the unknown field `rol`
- **AND** no segment with role result is created in its place

#### Scenario: Point with three coordinates is rejected

- **WHEN** a segment `create` object has `"start": [0, 0, 0]`
- **THEN** reading fails because `start` is not a point

#### Scenario: Misspelled colour is rejected

- **WHEN** a `create` object for a segment contains `"colour": "#c00000"`
- **THEN** reading fails because of the unknown field `colour`

### Requirement: Declarative subset

The declarative subset SHALL contain exactly `create` and `constrain` commands that each carry a `name`. It SHALL NOT contain delete, rename, set role, set, set document, define style, delete style, set style, drags, queries, undo or redo; a role, a name, attributes, a style and overrides are expressed by the final `create`. A list in the declarative subset SHALL be checked for membership before any of it is executed.

#### Scenario: Name is required in the subset

- **WHEN** a `constrain` object without `name` is read as part of the declarative subset
- **THEN** reading fails because the name is missing

#### Scenario: Rename is outside the subset

- **WHEN** a `rename` object is read as part of the declarative subset
- **THEN** reading fails because rename is not declarative

#### Scenario: Set is outside the subset

- **WHEN** a `set` object is read as part of the declarative subset
- **THEN** reading fails because set is not declarative

#### Scenario: Set style is outside the subset

- **WHEN** a `set_style` object is read as part of the declarative subset
- **THEN** reading fails because set style is not declarative

### Requirement: Drawing as declarative command list

The session SHALL produce, for its current drawing, a declarative command list: one `create` per geometry in creation order, then one `constrain` per constraint in creation order, each with the object's current name, role, values, attributes and operand addresses. A label's `create` SHALL carry every attribute explicitly, except `size` when it is inherited. A geometry's `create` SHALL carry `style` when it is not `default`, and each of its overrides. The list SHALL NOT contain document settings or style definitions. Executing that list in a new, empty session with the same document settings and style table SHALL give a drawing whose list is the same, with values equal within solve tolerance.

#### Scenario: Export and re-execute

- **WHEN** a drawing with segments `seg1`, `x_axis` (construction), arc `fillet`, a join `join1` of `seg1.end` and `fillet.start`, and `par1` on `seg1.direction` and `x_axis.direction` is exported
- **THEN** the list has the creates of `seg1`, `x_axis`, `fillet` in that order, then `join1` and `par1`, with those names, roles and addresses
- **AND** executing the list in an empty session and exporting again yields the same names, kinds, roles and addresses with values equal within solve tolerance

#### Scenario: Export after rename and delete

- **WHEN** `seg2` is renamed to `base` and `seg3` is deleted before export
- **THEN** the list contains a `create` named `base` and no command for `seg3` or its constraints

#### Scenario: Label attributes after set

- **WHEN** label `lbl1` is created with source `a` and then `set` to source `b` and markup `text`
- **THEN** the exported `create` of `lbl1` has source `b`, markup `text` and every other attribute except an inherited size

#### Scenario: Style after set style

- **WHEN** segment `seg1` is created with the default style and then `set_style` gives it style `axis` and colour `#c00000`
- **THEN** the exported `create` of `seg1` has style `axis` and colour `#c00000` and no other style field

## ADDED Requirements

### Requirement: Style error classes

Style commands SHALL add two error classes to those of the command layer: `unknown style` (a style name that is not in the style table) and `style in use` (deleting `default` or a style that a geometry refers to).

#### Scenario: Unknown style in set style

- **WHEN** `set_style` gives `seg1` the style `nope` while no style is named `nope`
- **THEN** the command fails with the `unknown style` class
- **AND** the drawing is unchanged

### Requirement: Define style command

The session SHALL provide a `define_style` command with a style name and any of the style properties (styling), at least one. It SHALL create the style if it does not exist, and SHALL otherwise change only the properties it carries. A property of JSON `null` SHALL remove that property from the style. It SHALL NOT change any object and SHALL NOT solve.

#### Scenario: Create a style

- **WHEN** `{"op": "define_style", "style": "vector", "width": 0.8, "arrow": "end"}` is applied and no style `vector` exists
- **THEN** the style table contains `vector` with width 0.8 and arrow `end`

#### Scenario: Change one property

- **WHEN** style `vector` has width 0.8 and arrow `end` and `{"op": "define_style", "style": "vector", "width": 1.2}` is applied
- **THEN** `vector` has width 1.2 and arrow `end`

#### Scenario: Remove a property

- **WHEN** `{"op": "define_style", "style": "default", "width": null}` is applied after `default` got width 0.6
- **THEN** `default` has no width and geometries resolve to the built-in width again

#### Scenario: Invalid style name

- **WHEN** a style named `2nd` is defined
- **THEN** the command fails with the invalid-or-taken-name class

### Requirement: Delete style command

The session SHALL provide a `delete_style` command with a style name. It SHALL delete the style as styling allows, and fail with the `unknown style` class for a style that does not exist and with the `style in use` class, naming the geometries that refer to it, for `default` or a style in use.

#### Scenario: Delete an unused style

- **WHEN** style `hidden` is not used and `{"op": "delete_style", "style": "hidden"}` is applied
- **THEN** `hidden` is no longer in the style table

#### Scenario: Delete a used style

- **WHEN** segment `x` refers to `axis` and `axis` is deleted
- **THEN** the command fails with the `style in use` class naming `x`

### Requirement: Set style command

The session SHALL provide a `set_style` command naming a geometry, with an optional `style` and any of the style properties, at least one field besides the name. It SHALL change only what it carries: `style` replaces the style reference, a property sets that override, and a property of JSON `null` removes that override. It SHALL NOT solve. Naming a constraint SHALL fail with the wrong-object-type class; naming a style that does not exist SHALL fail with the `unknown style` class.

#### Scenario: Assign a style

- **WHEN** `{"op": "set_style", "name": "seg1", "style": "axis"}` is applied
- **THEN** `seg1` refers to `axis` and keeps its overrides

#### Scenario: Remove an override

- **WHEN** `seg1` overrides colour `#c00000` and `{"op": "set_style", "name": "seg1", "color": null}` is applied
- **THEN** `seg1` has no colour override

#### Scenario: Set style on a constraint

- **WHEN** `set_style` names the constraint `par1`
- **THEN** the command fails with the wrong-object-type class

### Requirement: Style values are validated

A style command or a `create` with a width that is not finite or not greater than zero, or a colour that is not `#` followed by six hex digits, SHALL fail with the `invalid value` class naming the field, and SHALL leave the drawing unchanged.

#### Scenario: Zero width

- **WHEN** `define_style` gives style `axis` width 0
- **THEN** the command fails with the `invalid value` class for `width`

#### Scenario: Short colour

- **WHEN** `set_style` gives `seg1` the colour `#c00`
- **THEN** the command fails with the `invalid value` class for `color`

### Requirement: Style commands are history steps

`define_style`, `delete_style` and `set_style` SHALL each be one step in the session's history: undo SHALL restore the style table and every geometry's style and overrides as before the step, and redo SHALL restore the change.

#### Scenario: Undo a style change

- **WHEN** style `axis` changes its width from 0.8 to 1.2 and the step is undone
- **THEN** `axis` has width 0.8 again

### Requirement: Styles are queryable

The session SHALL report its drawing's style table and, for each geometry, its style name, its overrides and its resolved style properties.

#### Scenario: Resolved style of a geometry

- **WHEN** segment `x` refers to `axis` (width 0.8) and overrides colour `#c00000`, and its style is queried
- **THEN** the session reports style `axis`, the override colour `#c00000`, and resolved width 0.8, colour `#c00000`, dash `solid` and arrow `none`
