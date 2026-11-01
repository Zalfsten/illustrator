# Spec Delta

## ADDED Requirements

### Requirement: Curves describe their exact shape

Every geometry with the curve capability SHALL describe its exact shape at a State as an ordered sequence of pieces. Each piece is a **straight piece** (start and end locations) or a **circular-arc piece** (center location, radius length ≥ 0, start heading angle, turn angle sweep). The vocabulary is fixed and does not depend on the kind. A caller SHALL obtain the shape by geometry identity without branching on the concrete kind.

#### Scenario: Segment shape

- **WHEN** the shape of a line segment is queried at a State
- **THEN** it is one straight piece from the segment's start point to its end point

#### Scenario: Arc shape

- **WHEN** the shape of a circular arc with center c, radius R, start heading θ₀ and sweep Δθ is queried
- **THEN** it is one circular-arc piece with center c, radius |R|, start heading θ₀ and sweep Δθ

#### Scenario: Geometry without the curve capability has no shape

- **WHEN** the shape of a label is queried
- **THEN** the query reports that the geometry has no curve shape

### Requirement: Exact shape agrees with the curve

The pieces of a curve's exact shape SHALL run from the curve's normalized position t = 0 to t = 1, each piece starting where the previous one ends. Every on-curve sample of the curve SHALL lie on the shape, and the shape's total length SHALL equal the curve's length expression, within floating-point tolerance.

#### Scenario: Samples lie on the shape

- **WHEN** a segment and a circular arc are sampled at normalized t = 0, 0.25, 0.5 and 1
- **THEN** each sample lies on that curve's shape
- **AND** the shape starts at the sample for t = 0 and ends at the sample for t = 1

#### Scenario: Shape length equals curve length

- **WHEN** the total length of an arc's shape is computed
- **THEN** it equals the arc's length expression |R| |Δθ|
