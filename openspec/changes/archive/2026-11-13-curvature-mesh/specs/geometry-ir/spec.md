# Spec Delta: geometry-ir

## ADDED Requirements

### Requirement: Curves report their curvature

For every geometry with the curve capability, the system SHALL report its unsigned curvature at a normalized curve parameter t ∈ [0, 1] and a State: the reciprocal radius of the osculating circle, |γ′ × γ″| / |γ′|³, and 0 where the curve is straight. A caller SHALL obtain it by geometry identity without branching on the concrete kind.

#### Scenario: Segment has zero curvature

- **WHEN** the curvature of a line segment is queried at normalized t = 0, 0.5 and 1
- **THEN** each result is 0

#### Scenario: Arc has the reciprocal radius as curvature

- **WHEN** the curvature of a circular arc with radius R > 0 is queried at normalized t = 0, 0.5 and 1
- **THEN** each result equals 1 / R within floating-point tolerance

#### Scenario: Geometry without the curve capability has no curvature

- **WHEN** the curvature of a label is queried
- **THEN** the query reports that the geometry has no curvature

### Requirement: Curvature is an evaluation-only query

The curvature query SHALL serve evaluation (for example display tessellation). It SHALL NOT be a constraint operand, and it SHALL NOT require derivatives with respect to the State. It SHALL NOT change Document or the State it is evaluated at.

#### Scenario: Curvature leaves the drawing unchanged

- **WHEN** the curvature of an arc is queried at the committed State
- **THEN** Document and its committed State are unchanged afterwards
