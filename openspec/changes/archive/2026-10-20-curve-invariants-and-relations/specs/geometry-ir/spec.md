# Spec Delta

## ADDED Requirements

### Requirement: Line capability exposes a direction expression

Curves whose carrier is a straight line SHALL be able to expose a line capability. That capability SHALL provide an opaque free-vector **direction** expression that does not depend on a curve parameter. For a line segment it SHALL evaluate to end minus start. Curves without a straight carrier (for example circular arcs) SHALL NOT expose it, so a direction expression cannot be built for them. The direction expression SHALL provide analytical derivatives like other vector expressions.

#### Scenario: Segment direction is end minus start

- **WHEN** the direction expression of a line segment is evaluated at a State
- **THEN** it equals the segment's end point minus its start point
- **AND** it equals the segment's tangent expression at every normalized \(t\)

#### Scenario: Arcs offer no direction expression

- **WHEN** a circular arc is queried for the line capability
- **THEN** no direction expression is available for it

### Requirement: Circular capability exposes center and radius expressions

Curves whose carrier is a circle SHALL be able to expose a circular capability. That capability SHALL provide an opaque **center** point expression and an opaque **radius** length expression, both independent of a curve parameter. For a circular arc, the center SHALL evaluate to the arc's chart center and the radius to the non-negative radius \(|R|\). Curves without a circular carrier (for example line segments) SHALL NOT expose it. Both expressions SHALL provide analytical derivatives.

#### Scenario: Arc center and radius match the chart

- **WHEN** the center and radius expressions of a circular arc are evaluated at a State
- **THEN** the center equals the arc's chart center location
- **AND** the radius equals the magnitude of the arc's radius parameter
- **AND** every on-curve sample of the arc lies at that radius from that center

#### Scenario: Segments offer no center or radius expression

- **WHEN** a line segment is queried for the circular capability
- **THEN** no center or radius expression is available for it

#### Scenario: Equal radii via EqualLength on radius expressions

- **WHEN** EqualLength is applied to the radius expressions of two arcs
- **AND** the document is edited through a successful solve
- **THEN** both arcs have equal radius within solve tolerance

## MODIFIED Requirements

### Requirement: On-curve point expressions and frame vector expressions

The system SHALL provide point expressions that refer to a point on a named geometry (by stable geometry identity) at a stated curve parameter (`Normalized` or `Uniform`), provided that geometry is a curve. The system SHALL provide vector expressions that refer to the tangent free vector and the normal free vector of a named curve geometry at a stated curve parameter, with the length and orientation semantics of the curve tangent/normal maps. These expressions SHALL evaluate as functions of the involved geometry parameters and SHALL NOT be coordinate snapshots detached from those parameters. Existing endpoint point expressions remain valid and SHALL be consistent with on-curve samples at the corresponding normalized endpoints when the curve has endpoints. Endpoint expressions SHALL apply only to geometries that expose the endpoints capability; they SHALL NOT imply that every geometry is a segment. The system SHALL NOT require a separate segment-direction vector-expression factory; for line segments, tools MAY use a tangent vector expression at any chart parameter (the tangent is constant along the segment).

#### Scenario: Congruence can use an on-curve point expression

- **WHEN** a Congruence relationship binds an on-curve point expression at `Uniform(0.5)` on a segment to another point expression
- **AND** the equation system is assembled and solved successfully
- **THEN** the segment midpoint coincides with the other expression's location within solve tolerance

#### Scenario: Expressions name geometries by identity, not by kind-specific handles

- **WHEN** an on-curve or frame vector expression refers to a geometry
- **THEN** the reference is a stable geometry identity shared by all geometry kinds
- **AND** resolving that identity yields the geometry object used for evaluation
- **AND** the system SHALL NOT require a segment-only identity type for those expressions

#### Scenario: Parallel uses tangent free-vector expressions on segments

- **WHEN** VectorsParallel is applied to tangent vector expressions of two line segments
- **THEN** those expressions are valid free-vector expressions under this capability
- **AND** no separate direction-expression factory is required
