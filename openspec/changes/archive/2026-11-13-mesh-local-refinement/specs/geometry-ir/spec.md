# Spec Delta: geometry-ir

## ADDED Requirements

### Requirement: Curves bound length and curvature over a parameter piece

For every geometry with the curve capability, the system SHALL report, for a normalized parameter piece [t0, t1] ⊆ [0, 1] with t0 < t1 and a State, the arc length of that piece and an upper bound of its unsigned curvature at every point of the piece. The bound SHALL NOT be smaller than the curvature anywhere on the piece. A caller SHALL obtain both by geometry identity without branching on the concrete kind.

#### Scenario: Segment piece

- **WHEN** the piece [0.25, 0.75] of a line segment of length 4 is queried
- **THEN** its length is 2 and its curvature bound is 0

#### Scenario: Arc piece

- **WHEN** the piece [0, 0.5] of a circular arc with radius 2 and sweep π is queried
- **THEN** its length is π within floating-point tolerance and its curvature bound is 0.5

#### Scenario: Geometry without the curve capability has no piece bound

- **WHEN** a piece of a label is queried
- **THEN** the query reports that the geometry has no piece bound

## MODIFIED Requirements

### Requirement: Curvature is an evaluation-only query

The curvature query and the piece bound query SHALL serve evaluation (for example display tessellation). They SHALL NOT be constraint operands, and they SHALL NOT require derivatives with respect to the State. They SHALL NOT change Document or the State they are evaluated at.

#### Scenario: Curvature leaves the drawing unchanged

- **WHEN** the curvature of an arc is queried at the committed State
- **THEN** Document and its committed State are unchanged afterwards

#### Scenario: Piece bound leaves the drawing unchanged

- **WHEN** a piece bound of an arc is queried at the committed State
- **THEN** Document and its committed State are unchanged afterwards
