## MODIFIED Requirements

### Requirement: Command errors are classified

A failed command SHALL report exactly one error class: unknown name, invalid or taken name, malformed or unresolvable address (including a missing capability), wrong address family, wrong object type (for example a role for a constraint), session busy (a drag is in progress), `already joined` (a join within one congruence class), `invalid value` (a non-finite number or a value outside its kind's range), `frozen` (an operand in a conflicted component), or `unsatisfiable`.

#### Scenario: Unknown name

- **WHEN** a delete command names `nope` while no object is named `nope`
- **THEN** the command fails with the unknown-name class

#### Scenario: Missing capability

- **WHEN** a parallel command is applied to `arc1.direction`
- **THEN** the command fails because `arc1` has no line capability

#### Scenario: Invalid value

- **WHEN** a fix command has a target with an infinite coordinate
- **THEN** the command fails with the `invalid value` class

### Requirement: Interchange field shapes

Fields SHALL have these shapes: numbers as JSON numbers; points as two-element arrays `[x, y]`; angles in radians; names as strings; addresses of points, free vectors and lengths as address strings (object-naming); a role as `"result"` or `"construction"`. `name` SHALL be optional on `create` and `constrain`; `role` SHALL be optional on `create` and default to result.

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

### Requirement: Drawing as declarative command list

The session SHALL produce, for its current drawing, a declarative command list: one `create` per geometry in creation order, then one `constrain` per constraint in creation order, each with the object's current name, role, values, attributes and operand addresses. Executing that list in a new, empty session with the same document settings and style table SHALL give a drawing whose list is the same, with values equal within solve tolerance.

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

### Requirement: Error classes are extensible

Later capabilities MAY add classes without changing these. A failure with one of the causes above SHALL keep reporting its class from the list above.

#### Scenario: Style commands add classes

- **WHEN** a style command fails because it names a style that does not exist
- **THEN** it reports the `unknown style` class added by the style commands
- **AND** a command failing for one of the classes above still reports that class

### Requirement: Label interchange fields

Label fields SHALL be: `origin` a point; `heading` an angle; `source` a string; `markup` `"latex"` or `"text"`; `halign` `"left"`, `"center"` or `"right"`; `valign` `"first"` or `"last"`; `offset` a two-element array of pt; `size` a number of pt. On `create`, a label SHALL require `origin`, `heading` and `source`; an absent `size` SHALL mean inherited.

#### Scenario: Label without size inherits

- **WHEN** a label `create` object without a `size` field is read
- **THEN** the label it creates inherits the document font size

#### Scenario: Unknown alignment is rejected

- **WHEN** a label `create` object has `"halign": "middle"`
- **THEN** reading fails because `halign` has a wrong value

### Requirement: Style interchange fields

Style fields SHALL be: `style` a style name; `width` a number of pt; `color` a string `#rrggbb` (hex digits in either case); `dash` `"solid"`, `"dashed"`, `"dotted"` or `"dashdot"`; `arrow` `"none"`, `"start"`, `"end"` or `"both"`. On `create` of every geometry kind they SHALL be optional; the property fields are overrides.

#### Scenario: Colour in either case

- **WHEN** a segment `create` object with `"color": "#C0c0C0"` is read
- **THEN** it creates a segment that overrides the colour with #c0c0c0

#### Scenario: Colour name is rejected

- **WHEN** a `create` object has `"color": "red"`
- **THEN** reading fails because `color` has a wrong value

### Requirement: Declarative list carries attributes and styles, not settings

A label's `create` SHALL carry every attribute explicitly, except `size` when it is inherited. A geometry's `create` SHALL carry `style` when it is not `default`, and each of its overrides. The list SHALL NOT contain document settings or style definitions.

#### Scenario: Settings and style definitions stay out

- **WHEN** a drawing with a defined style `axis` and font size 12 pt is exported as a declarative command list
- **THEN** the list contains no command that defines `axis` or sets the font size

#### Scenario: Default style is not written

- **WHEN** a segment with the default style and no overrides is exported as a declarative command list
- **THEN** its `create` has no `style` field and no style property fields

#### Scenario: Inherited label size is not written

- **WHEN** a label that inherits the document font size is exported
- **THEN** its `create` has every label attribute except `size`
