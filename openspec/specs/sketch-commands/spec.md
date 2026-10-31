# sketch-commands Specification

## Purpose

Defines the command layer of a drawing session: the commands that change a drawing, how they address objects, their atomic and eagerly consistent semantics, their error classes, and the shared undo/redo history, so every front-end (GUI, scripts, AI clients) edits drawings the same way.

## Requirements

### Requirement: Commands address objects by name

Commands SHALL refer to existing objects by name and to points, free vectors and lengths by address, as defined by object-naming. A command SHALL NOT require internal identities. Every front-end of a session, including the interactive GUI, SHALL edit the drawing through these commands.

#### Scenario: GUI join issues a named command

- **WHEN** the user joins the end of `seg1` with the start of `seg2` in the GUI
- **THEN** the session receives a join command with the addresses `seg1.end` and `seg2.start`

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

### Requirement: Add constraint commands

The session SHALL provide one command per constraint kind: join (two point addresses), parallel and perpendicular (two free-vector addresses), fix (a point address and target coordinates), and equal length (two length addresses). Each SHALL accept an optional name and report the name of the created constraint. An address of the wrong family SHALL fail the command.

#### Scenario: Parallel lines by direction

- **WHEN** a parallel command is applied to `seg1.direction` and `seg2.direction`
- **THEN** the command succeeds and reports the new constraint's name
- **AND** after the command the two directions are parallel within tolerance

#### Scenario: Wrong address family is rejected

- **WHEN** a parallel command is applied to `seg1.start` and `seg2.direction`
- **THEN** the command fails because `seg1.start` is a point, not a free vector

### Requirement: Join within one congruence class is rejected

A join command whose two point addresses resolve to expressions of the same congruence class (including twice the same point) SHALL fail with error class `already joined` and leave the drawing unchanged. Points that only coincide in location without a Congruence chain SHALL be joinable.

#### Scenario: Joining a point with itself

- **WHEN** a join command is applied to `seg1.end` and `seg1.end`
- **THEN** the command fails with class `already joined`
- **AND** the drawing contains no new constraint

#### Scenario: Joining across an existing chain

- **WHEN** `seg1.end = seg2.start` and `seg2.start = seg3.start` are joined
- **AND** a join command is applied to `seg1.end` and `seg3.start`
- **THEN** the command fails with class `already joined`

#### Scenario: Coincident unjoined endpoints can be joined

- **WHEN** `seg1.end` and `seg2.start` evaluate to the same location without a Congruence chain between them
- **AND** a join command is applied to them
- **THEN** the command succeeds and the two points form one class

### Requirement: Delete, rename and set-role commands

The session SHALL provide commands to delete an object by name, to rename an object, and to set the role (result or construction) of a geometry. Deleting a geometry SHALL also delete the constraints that reference it, and the result SHALL report the names of all deleted objects.

#### Scenario: Deleting a geometry reports cascaded constraints

- **WHEN** `seg1` with constraints `par1` and `join1` on it is deleted
- **THEN** the result reports `seg1`, `par1` and `join1` as deleted

#### Scenario: Rename through a command

- **WHEN** a rename command renames `seg1` to `x_axis`
- **THEN** the geometry is named `x_axis` afterwards

### Requirement: Commands are atomic

A command SHALL either succeed completely or fail with no effect: on failure the drawing's structure, values and names SHALL be exactly as before the command, and the result SHALL state the error class.

#### Scenario: Failed create with a taken name changes nothing

- **WHEN** a segment is created with the name `seg1` while `seg1` exists
- **THEN** the command fails because the name is taken
- **AND** the drawing contains the same objects as before

### Requirement: Drawing stays structurally consistent

After every successful command every consistent component SHALL satisfy its structural constraints within solve tolerance; the session SHALL solve the components the command touches where needed. If a constraint command on consistent components leaves a component that cannot be satisfied, the command SHALL fail with error class `unsatisfiable`, reporting the remaining structural residual, and be rolled back. Conflicted components elsewhere SHALL NOT make a command fail.

#### Scenario: Contradicting constraint is rolled back

- **WHEN** `seg1` is fixed in place, `seg2` has the same length as `seg1`, and the two are parallel
- **AND** a perpendicular command is applied to their directions
- **THEN** the command fails with class `unsatisfiable`
- **AND** the drawing contains no perpendicular constraint and the previous values

#### Scenario: Contradiction elsewhere does not block commands

- **WHEN** `seg1` and `seg2` form a conflicted component
- **AND** `seg3` and `seg4` are in consistent components
- **AND** a parallel command is applied to `seg3.direction` and `seg4.direction`
- **THEN** the command succeeds
- **AND** `seg3` and `seg4` are parallel within solve tolerance
- **AND** the values of `seg1` and `seg2` are unchanged

### Requirement: Command errors are classified

A failed command SHALL report exactly one error class: unknown name, invalid or taken name, malformed or unresolvable address (including a missing capability), wrong address family, wrong object type (for example a role for a constraint), session busy (a drag is in progress), `already joined` (a join within one congruence class), `invalid value` (a non-finite number or a value outside its kind's range), `frozen` (an operand in a conflicted component), or `unsatisfiable`. Later capabilities MAY add classes without changing these.

#### Scenario: Unknown name

- **WHEN** a delete command names `nope` while no object is named `nope`
- **THEN** the command fails with the unknown-name class

#### Scenario: Missing capability

- **WHEN** a parallel command is applied to `arc1.direction`
- **THEN** the command fails because `arc1` has no line capability

#### Scenario: Invalid value

- **WHEN** a fix command has a target with an infinite coordinate
- **THEN** the command fails with the `invalid value` class

### Requirement: Undo and redo

Every successful command SHALL be one step in the session's history. Undo SHALL restore the drawing exactly as it was before the most recent step (structure, values, names); redo SHALL restore exactly the undone result. A failed command SHALL NOT create a step. Applying a command after undo SHALL discard the redo steps. Undo or redo with no step available SHALL report that and leave the drawing unchanged.

#### Scenario: Undo and redo a constraint

- **WHEN** a join command succeeds and undo is requested
- **THEN** the drawing has no join constraint and the values from before the command
- **AND** redo restores the join constraint and the values after the command

#### Scenario: Undo after a structural edit following a drag

- **WHEN** the user drags a point, then creates a segment, then requests undo twice
- **THEN** the first undo removes the segment
- **AND** the second undo restores the values from before the drag

#### Scenario: Nothing to undo

- **WHEN** undo is requested in a fresh session
- **THEN** the session reports that there is nothing to undo
- **AND** the drawing is unchanged

### Requirement: Drags share the command history

A completed drag that committed a new state SHALL be one step in the same history as commands. A cancelled or failed drag SHALL NOT create a step. Commands SHALL NOT be applied while a drag is in progress.

#### Scenario: Drag and command undo in order

- **WHEN** a join command succeeds and then a drag completes
- **THEN** one undo reverts the drag
- **AND** a second undo reverts the join

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

### Requirement: Kinds are selected by a kind tag

`create`, `constrain` and `set` SHALL carry a `kind` field that selects the geometry or constraint kind; its parameters or attributes are fields of the same object. Geometry tags SHALL be `segment`, `arc` and `label`; constraint tags SHALL be `join`, `parallel`, `perpendicular`, `fix` and `equal_length`. A kind tag SHALL be independent of the kind's name prefix. New kinds add tags without adding operations.

#### Scenario: Parallel constraint in interchange form

- **WHEN** the interchange text `{"op": "constrain", "kind": "parallel", "a": "seg1.direction", "b": "seg2.direction"}` is read
- **THEN** it is the parallel command on the two directions with an automatic name

#### Scenario: Unknown kind is rejected

- **WHEN** a `create` object with `"kind": "spline"` is read
- **THEN** reading fails because `spline` is not a geometry kind

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

### Requirement: Interchange round trip is lossless

Writing a command to its interchange form and reading it back SHALL yield the identical command, including every number bit for bit (negative zero included). Commands that cannot be written, because they contain a non-finite number, SHALL fail to write rather than produce text that reads back differently.

#### Scenario: Values survive exactly

- **WHEN** a segment command with start (0.1, 1/3) and end (1e-300, -0.0) is written and read back
- **THEN** the read command is identical to the written one

#### Scenario: Non-finite value cannot be written

- **WHEN** a segment command whose start has a NaN coordinate is written
- **THEN** writing fails with error class `invalid value`

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

### Requirement: Machine-readable command schema

The system SHALL provide a JSON Schema for the interchange form of all commands and a separate one for the declarative subset. Each SHALL list every operation and kind with its fields, required fields, field shapes and value ranges, and SHALL reject what reading rejects for unknown fields. The schemas SHALL be generated from the same definitions that read and write commands.

#### Scenario: Schema lists the kinds

- **WHEN** the schema for all commands is inspected
- **THEN** it contains each operation and, for `create`, `constrain` and `set`, each kind tag with its fields

#### Scenario: Subset schema requires names

- **WHEN** the schema for the declarative subset is inspected
- **THEN** it contains only `create` and `constrain`, each with `name` as a required field

### Requirement: Conflicted components are frozen

The values of a conflicted component (constraint-catalog) SHALL NOT change through commands or drags, except through a resolving solve after a delete. A constraint command with an operand in a conflicted component SHALL fail with error class `frozen` and leave the drawing unchanged. Delete, rename and set-role commands SHALL apply to objects in conflicted components as usual.

#### Scenario: Constraint inside a frozen component is rejected

- **WHEN** `seg1` and `seg2` form a conflicted component
- **AND** a parallel command is applied to `seg1.direction` and `seg2.direction`
- **THEN** the command fails with error class `frozen`
- **AND** the drawing is unchanged

#### Scenario: Joining to a frozen component is rejected

- **WHEN** `seg1` is in a conflicted component and `seg3` is in a consistent one
- **AND** a join command is applied to `seg3.start` and `seg1.end`
- **THEN** the command fails with error class `frozen`
- **AND** the drawing is unchanged

#### Scenario: Rename inside a frozen component succeeds

- **WHEN** `seg1` is in a conflicted component
- **AND** `seg1` is renamed to `base`
- **THEN** the command succeeds
- **AND** the values of the component are unchanged

### Requirement: Delete resolves contradictions

A delete command inside a conflicted component SHALL succeed. Afterwards, each remaining component that contained objects of that conflicted component SHALL get one structural solve: if it succeeds, its values SHALL be committed and the component is consistent; otherwise it SHALL keep its values and stay conflicted. Undo SHALL restore the conflicted state together with the drawing.

#### Scenario: Deleting the contradicting constraint unfreezes the component

- **WHEN** a component is conflicted only because of the constraint `orth1`
- **AND** `orth1` is deleted
- **THEN** the command succeeds
- **AND** the component is consistent and satisfies its structural constraints within solve tolerance

#### Scenario: Deleting an unrelated constraint keeps the component frozen

- **WHEN** a conflicted component still contains a contradiction after the constraint `eqlen1` is deleted
- **THEN** the delete succeeds
- **AND** the component stays conflicted with the values it had before

#### Scenario: Undo restores the conflict

- **WHEN** a delete has resolved a conflicted component
- **AND** undo is requested
- **THEN** the deleted constraint is back
- **AND** the component is conflicted again with its previous values

### Requirement: Session start from an existing drawing

A session started from an existing drawing SHALL give each conflicted component one structural solve. Components it solves SHALL be committed and consistent; the others SHALL keep their values and stay conflicted. Consistent components SHALL keep their values. Starting the session SHALL NOT create a history step.

#### Scenario: Solvable and contradicting components at start

- **WHEN** a drawing has one component whose values are off but whose constraints can be satisfied, and one component with contradicting constraints
- **AND** a session is started from it
- **THEN** the first component is solved and consistent
- **AND** the second keeps its values and is conflicted
- **AND** undo is not available

### Requirement: Conflicted components are listed

The session SHALL list its conflicted components, each with the names of its geometries and constraints and its structural residual. The list SHALL be empty when every component is consistent.

#### Scenario: One conflicted component is listed

- **WHEN** a session holds one conflicted component with segments `seg1`, `seg2` and constraints `par1`, `orth1`, and a consistent component with `seg3`
- **THEN** the list has exactly one entry
- **AND** it names `seg1`, `seg2`, `par1` and `orth1` and reports a residual above solve tolerance

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
