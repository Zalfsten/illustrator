# Spec Delta

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Command errors are classified

A failed command SHALL report exactly one error class: unknown name, invalid or taken name, malformed or unresolvable address (including a missing capability), wrong address family, wrong object type (for example a role for a constraint), session busy (a drag is in progress), `already joined` (a join within one congruence class), or `unsatisfiable`. Later capabilities MAY add classes without changing these.

#### Scenario: Unknown name

- **WHEN** a delete command names `nope` while no object is named `nope`
- **THEN** the command fails with the unknown-name class

#### Scenario: Missing capability

- **WHEN** a parallel command is applied to `arc1.direction`
- **THEN** the command fails because `arc1` has no line capability
