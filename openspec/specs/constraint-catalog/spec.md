# constraint-catalog

## Purpose

Defines the persistent v1 constraint catalog — Congruence, Parallel, Orthogonal, and FixPoint — including residual meaning and the rule that satisfied structural constraints are document invariants.

## Requirements

### Requirement: Congruence identifies two point expressions

The system SHALL provide a Congruence relationship between two point expressions. Congruence SHALL contribute exactly two residual components that are zero if and only if the two expressions evaluate to the same location (within solve tolerance).

#### Scenario: Congruent endpoints share a location after solve

- **WHEN** Congruence is applied to two endpoint expressions
- **AND** a successful solve completes
- **THEN** both expressions evaluate to the same location within solve tolerance

### Requirement: Parallel constrains segment directions

The system SHALL provide a Parallel relationship between two segments. Parallel SHALL contribute exactly one residual component that is zero if and only if the segments' direction vectors are linearly dependent (parallel or anti-parallel), including the degenerate case where either direction has zero length.

#### Scenario: Parallel segments keep aligned directions under edit

- **WHEN** two segments are constrained Parallel
- **AND** the document is edited through a successful solve
- **THEN** the segments' direction vectors remain linearly dependent within solve tolerance

### Requirement: Orthogonal constrains a right angle

The system SHALL provide an Orthogonal relationship between two segments. Orthogonal SHALL contribute exactly one residual component that is zero if and only if the segments' direction vectors are perpendicular (dot product zero), including the degenerate case where either direction has zero length.

#### Scenario: Orthogonal segments keep a right angle under edit

- **WHEN** two segments are constrained Orthogonal
- **AND** the document is edited through a successful solve
- **THEN** the segments' direction vectors remain perpendicular within solve tolerance

### Requirement: FixPoint anchors a point expression

The system SHALL provide a FixPoint relationship that anchors a point expression to a fixed target location. FixPoint SHALL contribute exactly two residual components that are zero if and only if the expression evaluates to that target (within solve tolerance).

#### Scenario: Fixed point does not move under successful solve of other edits

- **WHEN** a point expression is constrained with FixPoint to a target location
- **AND** another part of the figure is manipulated through a successful solve
- **THEN** the fixed point expression still evaluates to the target location within solve tolerance

### Requirement: Structural constraints are hard invariants

Congruence, Parallel, Orthogonal, and FixPoint SHALL be structural constraints. After every successful solve that commits a new document state, all structural residuals SHALL be satisfied within solve tolerance. The system SHALL NOT commit a document state from a solve that leaves structural residuals unsatisfied beyond that tolerance.

#### Scenario: Successful commit preserves structural satisfaction

- **WHEN** a solve succeeds and its result is committed as the document state
- **THEN** every structural constraint in the document is satisfied within solve tolerance

### Requirement: Degenerate segments do not crash the catalog

If a segment has zero length, Parallel and Orthogonal residuals involving that segment MAY be algebraically satisfied without conveying a meaningful angle. The system SHALL still accept such configurations without undefined behavior; improving or rejecting degeneracy is outside this catalog's normative behavior.

#### Scenario: Zero-length segment does not make assembly fail by itself

- **WHEN** a segment has coincident endpoints
- **AND** Parallel or Orthogonal involving that segment is present
- **THEN** equation assembly completes without runtime failure attributable solely to that degeneracy
