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

The session SHALL provide commands that create a line segment (start and end point) and a circular arc (center, radius, start heading, sweep). Each SHALL accept an optional name and an optional role (result by default). On success the result SHALL report the name of the created geometry.

#### Scenario: Create a segment with an automatic name

- **WHEN** a segment is created without a name in an empty drawing
- **THEN** the command succeeds and reports the name `seg1`

#### Scenario: Create with an explicit name

- **WHEN** an arc is created with the name `fillet`
- **THEN** the command succeeds and the arc is named `fillet`

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

Every command SHALL have one interchange form, a JSON object whose `op` field names the operation. The operation set SHALL be closed: `create`, `constrain`, `delete`, `rename` and `set_role`. Files, the command protocol and clients that list commands SHALL use this form; front-ends MAY offer other syntaxes, but SHALL map them to these commands.

#### Scenario: A rename in interchange form

- **WHEN** the interchange text `{"op": "rename", "name": "seg1", "new_name": "x_axis"}` is read
- **THEN** it is the rename command from `seg1` to `x_axis`

#### Scenario: Unknown operation is rejected

- **WHEN** the interchange text `{"op": "parallel", "a": "seg1.direction", "b": "seg2.direction"}` is read
- **THEN** reading fails because `parallel` is not an operation

### Requirement: Kinds are selected by a kind tag

`create` and `constrain` SHALL carry a `kind` field that selects the geometry or constraint kind; its parameters are fields of the same object. Geometry tags SHALL be `segment` and `arc`; constraint tags SHALL be `join`, `parallel`, `perpendicular`, `fix` and `equal_length`. A kind tag SHALL be independent of the kind's name prefix. New kinds add tags without adding operations.

#### Scenario: Parallel constraint in interchange form

- **WHEN** the interchange text `{"op": "constrain", "kind": "parallel", "a": "seg1.direction", "b": "seg2.direction"}` is read
- **THEN** it is the parallel command on the two directions with an automatic name

#### Scenario: Unknown kind is rejected

- **WHEN** a `create` object with `"kind": "spline"` is read
- **THEN** reading fails because `spline` is not a geometry kind

### Requirement: Interchange field shapes

Fields SHALL have these shapes: numbers as JSON numbers; points as two-element arrays `[x, y]`; angles in radians; names as strings; addresses of points, free vectors and lengths as address strings (object-naming); a role as `"result"` or `"construction"`. `name` SHALL be optional on `create` and `constrain`; `role` SHALL be optional on `create` and default to result.

#### Scenario: Segment in interchange form

- **WHEN** the interchange text `{"op": "create", "kind": "segment", "start": [0, 0], "end": [3, 0]}` is read
- **THEN** it is the create command for a segment from (0, 0) to (3, 0) with role result and an automatic name

#### Scenario: Arc and fix in interchange form

- **WHEN** `{"op": "create", "kind": "arc", "name": "fillet", "role": "construction", "center": [3, 1], "radius": 1, "start_heading": -1.5707963267948966, "sweep": 1.5707963267948966}` is read
- **AND** `{"op": "constrain", "kind": "fix", "point": "fillet.start", "target": [3, 0]}` is read
- **THEN** they are a construction arc named `fillet` and a fix of its start to (3, 0)

### Requirement: Interchange reading is strict

Reading SHALL reject an object with an unknown field, a missing required field, a field of the wrong shape, or an unknown `op` or `kind`, and SHALL NOT fall back to defaults for misspelled fields. The field names `op`, `kind`, `name` and `role` SHALL be reserved: no kind SHALL use them for its own parameters.

#### Scenario: Misspelled optional field is rejected

- **WHEN** a `create` object for a segment contains `"rol": "construction"`
- **THEN** reading fails because of the unknown field `rol`
- **AND** no segment with role result is created in its place

#### Scenario: Point with three coordinates is rejected

- **WHEN** a segment `create` object has `"start": [0, 0, 0]`
- **THEN** reading fails because `start` is not a point

### Requirement: Interchange round trip is lossless

Writing a command to its interchange form and reading it back SHALL yield the identical command, including every number bit for bit (negative zero included). Commands that cannot be written, because they contain a non-finite number, SHALL fail to write rather than produce text that reads back differently.

#### Scenario: Values survive exactly

- **WHEN** a segment command with start (0.1, 1/3) and end (1e-300, -0.0) is written and read back
- **THEN** the read command is identical to the written one

#### Scenario: Non-finite value cannot be written

- **WHEN** a segment command whose start has a NaN coordinate is written
- **THEN** writing fails with error class `invalid value`

### Requirement: Command values are validated

A command with a non-finite number (NaN or infinite), or with a number outside the range its kind admits, SHALL fail with error class `invalid value`, naming the field. A JSON number too large to represent as finite SHALL be rejected when read. For an arc, the radius SHALL be zero or positive; a negative radius SHALL be rejected, not rewritten.

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

### Requirement: Declarative subset

The declarative subset SHALL contain exactly `create` and `constrain` commands that each carry a `name`. It SHALL NOT contain delete, rename, set role, drags, queries, undo or redo; a role and a name are expressed by the final `create`. A list in the declarative subset SHALL be checked for membership before any of it is executed.

#### Scenario: Name is required in the subset

- **WHEN** a `constrain` object without `name` is read as part of the declarative subset
- **THEN** reading fails because the name is missing

#### Scenario: Rename is outside the subset

- **WHEN** a `rename` object is read as part of the declarative subset
- **THEN** reading fails because rename is not declarative

### Requirement: Drawing as declarative command list

The session SHALL produce, for its current drawing, a declarative command list: one `create` per geometry in creation order, then one `constrain` per constraint in creation order, each with the object's current name, role, values and operand addresses. Executing that list in a new, empty session SHALL give a drawing whose list is the same, with values equal within solve tolerance.

#### Scenario: Export and re-execute

- **WHEN** a drawing with segments `seg1`, `x_axis` (construction), arc `fillet`, a join `join1` of `seg1.end` and `fillet.start`, and `par1` on `seg1.direction` and `x_axis.direction` is exported
- **THEN** the list has the creates of `seg1`, `x_axis`, `fillet` in that order, then `join1` and `par1`, with those names, roles and addresses
- **AND** executing the list in an empty session and exporting again yields the same names, kinds, roles and addresses with values equal within solve tolerance

#### Scenario: Export after rename and delete

- **WHEN** `seg2` is renamed to `base` and `seg3` is deleted before export
- **THEN** the list contains a `create` named `base` and no command for `seg3` or its constraints

### Requirement: Machine-readable command schema

The system SHALL provide a JSON Schema for the interchange form of all commands and a separate one for the declarative subset. Each SHALL list every operation and kind with its fields, required fields, field shapes and value ranges, and SHALL reject what reading rejects for unknown fields. The schemas SHALL be generated from the same definitions that read and write commands.

#### Scenario: Schema lists the kinds

- **WHEN** the schema for all commands is inspected
- **THEN** it contains each operation and, for `create` and `constrain`, each kind tag with its fields

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
