# Spec Delta: constraint-catalog

## MODIFIED Requirements

### Requirement: Parallel constrains two free-vector expressions

The system SHALL provide a Parallel relationship between two **opaque free-vector expressions**. Parallel SHALL contribute exactly one residual component that is zero if and only if the two evaluated free vectors are linearly dependent (parallel or anti-parallel), including the degenerate case where either vector has zero length. In the v1 catalog, tools SHALL typically supply curve tangent vector expressions (for line segments, tangent is constant along the chart); the residual SHALL NOT hard-wire segment identity outside those expressions.

#### Scenario: Parallel segment directions stay aligned under edit

- **WHEN** Parallel is applied to the tangent vector expressions of two segments
- **AND** the document is edited through a successful solve
- **THEN** those tangent vectors remain linearly dependent within solve tolerance

### Requirement: Orthogonal constrains two free-vector expressions

The system SHALL provide an Orthogonal relationship between two **opaque free-vector expressions**. Orthogonal SHALL contribute exactly one residual component that is zero if and only if the two evaluated free vectors are perpendicular (dot product zero), including the degenerate case where either vector has zero length. In the v1 catalog, tools SHALL typically supply curve tangent vector expressions (for line segments, tangent is constant along the chart); the residual SHALL NOT hard-wire segment identity outside those expressions.

#### Scenario: Orthogonal segment directions keep a right angle under edit

- **WHEN** Orthogonal is applied to the tangent vector expressions of two segments
- **AND** the document is edited through a successful solve
- **THEN** those tangent vectors remain perpendicular within solve tolerance

### Requirement: Degenerate zero vectors do not crash the catalog

If either free-vector expression involved in Parallel or Orthogonal evaluates to the zero vector (e.g. a zero-length segment whose tangent vanishes), the residual MAY be algebraically satisfied without conveying a meaningful angle. The system SHALL still accept such configurations without undefined behavior; improving or rejecting degeneracy is outside this catalog's normative behavior.

#### Scenario: Zero-length segment direction does not make assembly fail by itself

- **WHEN** a segment has coincident endpoints
- **AND** Parallel or Orthogonal is applied to vector expressions that include that segment's tangent
- **THEN** equation assembly completes without runtime failure attributable solely to that degeneracy
