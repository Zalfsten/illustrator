## MODIFIED Requirements

### Requirement: Curves describe their exact shape

Every geometry with the curve capability SHALL describe its shape at a State and a positive tolerance length as an ordered sequence of pieces: **straight pieces** (start and end locations), **circular-arc pieces** (center location, radius length ≥ 0, start heading angle, turn angle sweep) or **cubic Bézier pieces** (start, two control and end locations). The vocabulary is fixed and kind-independent. A caller SHALL obtain the shape by geometry identity without branching on the concrete kind.

#### Scenario: Segment shape

- **WHEN** the shape of a line segment is queried at a State
- **THEN** it is one straight piece from the segment's start point to its end point

#### Scenario: Arc shape

- **WHEN** the shape of a circular arc with center c, radius R, start heading θ₀ and sweep Δθ is queried
- **THEN** it is one circular-arc piece with center c, radius |R|, start heading θ₀ and sweep Δθ

#### Scenario: Closed-form shapes ignore the tolerance

- **WHEN** the shape of a segment and of a circular arc is queried once with tolerance 1 and once with tolerance 10⁻⁹
- **THEN** both queries give the same single piece for each curve

#### Scenario: Geometry without the curve capability has no shape

- **WHEN** the shape of a label is queried
- **THEN** the query reports that the geometry has no curve shape

### Requirement: Exact shape agrees with the curve

The pieces of a curve's shape SHALL run from the curve's normalized position t = 0 to t = 1, each piece starting where the previous one ends. For a segment or a circular arc, every on-curve sample SHALL lie on the shape and the shape's total length SHALL equal the curve's length expression, within floating-point tolerance. Other curve kinds SHALL agree within the requested tolerance instead.

#### Scenario: Samples lie on the shape

- **WHEN** a segment and a circular arc are sampled at normalized t = 0, 0.25, 0.5 and 1
- **THEN** each sample lies on that curve's shape
- **AND** the shape starts at the sample for t = 0 and ends at the sample for t = 1

#### Scenario: Shape length equals curve length

- **WHEN** the total length of an arc's shape is computed
- **THEN** it equals the arc's length expression |R| |Δθ|

## ADDED Requirements

### Requirement: Shapes without closed-form pieces stay within the tolerance

A curve whose shape is not a segment's or a circular arc's SHALL describe it with cubic Bézier pieces. Every point of the shape SHALL lie within the requested tolerance of the curve, and every point of the curve within the tolerance of the shape. Each piece SHALL start and end exactly on the curve, and the shape's total length SHALL differ from the curve's length expression by at most the tolerance.

#### Scenario: Pieces stay within the tolerance

- **WHEN** the shape of a curve without closed-form pieces is queried with tolerance δ
- **THEN** it consists of cubic Bézier pieces
- **AND** every point of the shape lies within δ of the curve, and every point of the curve within δ of the shape

#### Scenario: Piece ends lie on the curve

- **WHEN** the shape of a curve without closed-form pieces is queried
- **THEN** the first piece starts at the curve's point for t = 0, the last piece ends at the point for t = 1, and every joint between pieces is a point of the curve

#### Scenario: Length agrees within the tolerance

- **WHEN** the shape of a curve without closed-form pieces is queried with tolerance δ
- **THEN** the total length of its pieces differs from the curve's length expression by at most δ
