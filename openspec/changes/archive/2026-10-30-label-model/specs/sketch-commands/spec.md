# Spec Delta

## ADDED Requirements

### Requirement: Set command changes object attributes

The session SHALL provide a `set` command that changes attributes of the named object: properties that are not parameters, such as a label's source or offset. It SHALL change only the attributes it carries, SHALL NOT change parameter values, names or roles, and SHALL NOT solve. Attributes of a conflicted component MAY be set.

#### Scenario: Change a label's source

- **WHEN** `set` is applied to label `lbl1` with source `$\alpha$`
- **THEN** the command succeeds and `lbl1` has source `$\alpha$`
- **AND** its other attributes and its origin and heading are unchanged

#### Scenario: Several attributes in one step

- **WHEN** `set` is applied to `lbl1` with halign `center` and offset [0, 3]
- **AND** the step is undone
- **THEN** both attributes return to their previous values

#### Scenario: Set on a constraint

- **WHEN** `set` with kind `label` names the constraint `par1`
- **THEN** the command fails with the wrong-object-type class

### Requirement: Set names the kind it expects

A `set` command SHALL carry the kind tag of the object it changes. Its attribute fields SHALL be those of that kind, each optional, at least one present. A kind without attributes SHALL NOT be a valid `set` kind. If the named geometry is of another kind, the command SHALL fail with the wrong-object-type class.

#### Scenario: Kind mismatch

- **WHEN** `{"op": "set", "kind": "label", "name": "seg1", "source": "x"}` is applied to segment `seg1`
- **THEN** the command fails with the wrong-object-type class
- **AND** the drawing is unchanged

#### Scenario: Set without attributes is rejected

- **WHEN** `{"op": "set", "kind": "label", "name": "lbl1"}` is read
- **THEN** reading fails because no attribute is given

### Requirement: Inherited size is restored by null

In a label `set`, a `size` of JSON `null` SHALL make the label inherit the document font size again. An absent `size` field SHALL leave the size unchanged.

#### Scenario: Back to the document size

- **WHEN** label `lbl1` has size 8 and `{"op": "set", "kind": "label", "name": "lbl1", "size": null}` is applied
- **THEN** `lbl1` inherits the document font size

### Requirement: Set document command

The session SHALL provide a `set_document` command that changes document settings (document-settings). Its fields `scale`, `font_size` and `preamble` SHALL each be optional, at least one present. It SHALL change only the settings it carries, SHALL NOT change any object, and SHALL NOT solve.

#### Scenario: Change the scale

- **WHEN** `{"op": "set_document", "scale": 5}` is applied
- **THEN** the scale is 5 and font size and preamble are unchanged

#### Scenario: Empty set_document is rejected

- **WHEN** `{"op": "set_document"}` is read
- **THEN** reading fails because no setting is given

### Requirement: Document settings are queryable

The session SHALL report the current document settings of its drawing.

#### Scenario: Settings after a change

- **WHEN** `set_document` with font size 12 succeeds and the settings are queried
- **THEN** font size 12 is reported

## MODIFIED Requirements

### Requirement: Create geometry commands

The session SHALL provide commands that create a line segment (start and end point), a circular arc (center, radius, start heading, sweep) and a label (origin, heading and its attributes, labels). Each SHALL accept an optional name and an optional role (result by default). On success the result SHALL report the name of the created geometry.

#### Scenario: Create a segment with an automatic name

- **WHEN** a segment is created without a name in an empty drawing
- **THEN** the command succeeds and reports the name `seg1`

#### Scenario: Create with an explicit name

- **WHEN** an arc is created with the name `fillet`
- **THEN** the command succeeds and the arc is named `fillet`

#### Scenario: Create a label with an automatic name

- **WHEN** a label is created without a name in an empty drawing
- **THEN** the command succeeds and reports the name `lbl1`

### Requirement: Commands have one interchange form

Every command SHALL have one interchange form, a JSON object whose `op` field names the operation. The operation set SHALL be closed: `create`, `constrain`, `delete`, `rename`, `set_role`, `set` and `set_document`. Files, the command protocol and clients that list commands SHALL use this form; front-ends MAY offer other syntaxes, but SHALL map them to these commands.

#### Scenario: A rename in interchange form

- **WHEN** the interchange text `{"op": "rename", "name": "seg1", "new_name": "x_axis"}` is read
- **THEN** it is the rename command from `seg1` to `x_axis`

#### Scenario: Unknown operation is rejected

- **WHEN** the interchange text `{"op": "parallel", "a": "seg1.direction", "b": "seg2.direction"}` is read
- **THEN** reading fails because `parallel` is not an operation

#### Scenario: A set in interchange form

- **WHEN** the interchange text `{"op": "set", "kind": "label", "name": "lbl1", "valign": "last"}` is read
- **THEN** it is the set command that changes only the vertical alignment of `lbl1`

### Requirement: Kinds are selected by a kind tag

`create`, `constrain` and `set` SHALL carry a `kind` field that selects the geometry or constraint kind; its parameters or attributes are fields of the same object. Geometry tags SHALL be `segment`, `arc` and `label`; constraint tags SHALL be `join`, `parallel`, `perpendicular`, `fix` and `equal_length`. A kind tag SHALL be independent of the kind's name prefix. New kinds add tags without adding operations.

#### Scenario: Parallel constraint in interchange form

- **WHEN** the interchange text `{"op": "constrain", "kind": "parallel", "a": "seg1.direction", "b": "seg2.direction"}` is read
- **THEN** it is the parallel command on the two directions with an automatic name

#### Scenario: Unknown kind is rejected

- **WHEN** a `create` object with `"kind": "spline"` is read
- **THEN** reading fails because `spline` is not a geometry kind

### Requirement: Interchange field shapes

Fields SHALL have these shapes: numbers as JSON numbers; points as two-element arrays `[x, y]`; angles in radians; names as strings; addresses of points, free vectors and lengths as address strings (object-naming); a role as `"result"` or `"construction"`. Label fields SHALL be: `origin` a point; `heading` an angle; `source` a string; `markup` `"latex"` or `"text"`; `halign` `"left"`, `"center"` or `"right"`; `valign` `"first"` or `"last"`; `offset` a two-element array of pt; `size` a number of pt. On `create`, a label SHALL require `origin`, `heading` and `source`; an absent `size` SHALL mean inherited. `name` SHALL be optional on `create` and `constrain`; `role` SHALL be optional on `create` and default to result.

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

### Requirement: Command values are validated

A command with a non-finite number (NaN or infinite), or with a number outside the range its kind admits, SHALL fail with error class `invalid value`, naming the field. A JSON number too large to represent as finite SHALL be rejected when read. For an arc, the radius SHALL be zero or positive; a negative radius SHALL be rejected, not rewritten. A label size and the document settings `scale` and `font_size` SHALL be greater than zero.

#### Scenario: Negative arc radius is rejected

- **WHEN** an arc is created with radius -1
- **THEN** the command fails with error class `invalid value` for `radius`
- **AND** the drawing is unchanged

#### Scenario: Zero radius is accepted

- **WHEN** an arc is created with radius 0
- **THEN** the command succeeds

#### Scenario: Out-of-range JSON number is rejected

- **WHEN** a segment `create` object with `"start": [1e999, 0]` is read
- **THEN** reading fails and no command results

#### Scenario: Zero label size is rejected

- **WHEN** `set` gives label `lbl1` size 0
- **THEN** the command fails with error class `invalid value` for `size`
- **AND** the drawing is unchanged

#### Scenario: Negative scale is rejected

- **WHEN** `set_document` gives scale -1
- **THEN** the command fails with error class `invalid value` for `scale`

### Requirement: Declarative subset

The declarative subset SHALL contain exactly `create` and `constrain` commands that each carry a `name`. It SHALL NOT contain delete, rename, set role, set, set document, drags, queries, undo or redo; a role, a name and attributes are expressed by the final `create`. A list in the declarative subset SHALL be checked for membership before any of it is executed.

#### Scenario: Name is required in the subset

- **WHEN** a `constrain` object without `name` is read as part of the declarative subset
- **THEN** reading fails because the name is missing

#### Scenario: Rename is outside the subset

- **WHEN** a `rename` object is read as part of the declarative subset
- **THEN** reading fails because rename is not declarative

#### Scenario: Set is outside the subset

- **WHEN** a `set` object is read as part of the declarative subset
- **THEN** reading fails because set is not declarative

### Requirement: Drawing as declarative command list

The session SHALL produce, for its current drawing, a declarative command list: one `create` per geometry in creation order, then one `constrain` per constraint in creation order, each with the object's current name, role, values, attributes and operand addresses. A label's `create` SHALL carry every attribute explicitly, except `size` when it is inherited. The list SHALL NOT contain document settings. Executing that list in a new, empty session with the same document settings SHALL give a drawing whose list is the same, with values equal within solve tolerance.

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

### Requirement: Machine-readable command schema

The system SHALL provide a JSON Schema for the interchange form of all commands and a separate one for the declarative subset. Each SHALL list every operation and kind with its fields, required fields, field shapes and value ranges, and SHALL reject what reading rejects for unknown fields. The schemas SHALL be generated from the same definitions that read and write commands.

#### Scenario: Schema lists the kinds

- **WHEN** the schema for all commands is inspected
- **THEN** it contains each operation and, for `create`, `constrain` and `set`, each kind tag with its fields

#### Scenario: Subset schema requires names

- **WHEN** the schema for the declarative subset is inspected
- **THEN** it contains only `create` and `constrain`, each with `name` as a required field
