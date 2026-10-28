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

After every successful command the drawing SHALL satisfy all its structural constraints within solve tolerance; the session SHALL solve as part of the command where needed. If the constraints cannot be satisfied after the command, the command SHALL fail with error class `unsatisfiable`, reporting the remaining structural residual, and be rolled back.

#### Scenario: Contradicting constraint is rolled back

- **WHEN** `seg1` is fixed in place, `seg2` has the same length as `seg1`, and the two are parallel
- **AND** a perpendicular command is applied to their directions
- **THEN** the command fails with class `unsatisfiable`
- **AND** the drawing contains no perpendicular constraint and the previous values

### Requirement: Command errors are classified

A failed command SHALL report exactly one error class: unknown name, invalid or taken name, malformed or unresolvable address (including a missing capability), wrong address family, wrong object type (for example a role for a constraint), session busy (a drag is in progress), or `unsatisfiable`. Later capabilities MAY add classes without changing these.

#### Scenario: Unknown name

- **WHEN** a delete command names `nope` while no object is named `nope`
- **THEN** the command fails with the unknown-name class

#### Scenario: Missing capability

- **WHEN** a parallel command is applied to `arc1.direction`
- **THEN** the command fails because `arc1` has no line capability

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
