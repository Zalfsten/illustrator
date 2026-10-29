# Spec Delta

## ADDED Requirements

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

## MODIFIED Requirements

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
