# Spec Delta

## ADDED Requirements

### Requirement: Curves expose normalized position, tangent, and normal

A curve geometry SHALL expose a position map, a tangent map, and a normal map over a normalized chart parameter \(t \in [0,1]\). Position SHALL evaluate to a point (via a point expression). Tangent and normal SHALL evaluate to free vectors (via vector expressions), not points. The tangent at \(t\) SHALL be the chart derivative \(\gamma'(t)\) (parametric velocity); its length SHALL be the parametric speed and SHALL NOT be required to be one. The normal at \(t\) SHALL be a planar perpendicular to that tangent obtained by a fixed orientation convention (leftward relative to the curve's increasing-\(t\) direction: if tangent is \((v_x, v_y)\), normal is \((-v_y, v_x)\)); the normal's length SHALL equal the tangent's length at the same \(t\). The normal SHALL NOT be defined as a curvature-weighted vector; curvature as a separate quantity is out of scope for this requirement. The normalized chart SHALL be the canonical parameterization of the curve's defining formula with its natural domain affinely mapped to \([0,1]\); equal steps in \(t\) NEED NOT correspond to equal arc length except for curves whose chart already has constant speed (for example a line segment).

#### Scenario: Segment position at normalized mid-parameter

- **WHEN** a line segment curve is evaluated at normalized parameter \(t = 0.5\)
- **THEN** the resulting point is the midpoint of the segment's endpoints
- **AND** the tangent free vector is parallel to the segment direction
- **AND** the tangent length equals the segment length

#### Scenario: Segment normal is perpendicular with matching length

- **WHEN** a line segment curve is evaluated at any normalized \(t\)
- **THEN** the normal free vector is perpendicular to the tangent (dot product zero)
- **AND** the normal has the same length as the tangent
- **AND** the normal is non-zero whenever the segment has non-zero length

### Requirement: Curve parameter kinds Normalized and Uniform

The system SHALL address a location on a curve using a curve parameter that is either `Normalized` (\(t \in [0,1]\) in the canonical chart) or `Uniform` (\(u \in [0,1]\) such that equal increments of \(u\) correspond to equal arc length along the curve, i.e. arc-length fraction). A physical arc-length domain \(s \in [0, L]\) SHALL NOT be required in this capability. For a line segment, `Normalized` and `Uniform` SHALL yield the same point for the same numeric argument.

#### Scenario: Uniform mid-parameter is arc-length midpoint on a segment

- **WHEN** a line segment is sampled with `Uniform` parameter \(u = 0.5\)
- **THEN** the resulting point coincides with the sample at `Normalized` \(t = 0.5\)
- **AND** that point is the arc-length midpoint of the segment

#### Scenario: Uniform is distinct from Normalized in general

- **WHEN** a future non-constant-speed curve (for example a cubic Bézier) is admitted under this capability
- **THEN** `Normalized(0.5)` and `Uniform(0.5)` NEED NOT evaluate to the same location
- **AND** both parameter kinds remain valid addresses on that curve

### Requirement: Endpoint capability is separate from Curve

Geometric start and end join points SHALL be provided only for curves that have an endpoints capability. That capability SHALL expose point expressions for start and end. Those expressions MAY default to the curve position at normalized \(t = 0\) and \(t = 1\), and MAY be replaced by more efficient equivalents. Intrinsically periodic closed curves (for example a full circle) SHALL implement the curve position/tangent contract and SHALL NOT be required to expose geometric endpoints; their chart MAY use an origin for \(t = 0\) that is a parameterization gauge, not a join endpoint.

#### Scenario: Segment exposes endpoint expressions

- **WHEN** a line segment is queried for endpoints
- **THEN** start and end point expressions are available
- **AND** evaluating them matches position at normalized \(t = 0\) and \(t = 1\)

#### Scenario: Periodic curve needs no join endpoints

- **WHEN** an intrinsically periodic closed curve is present in the document
- **THEN** it still exposes normalized position, tangent, and normal for subdivision and constraints
- **AND** the core SHALL NOT require geometric start/end join endpoints for that curve

### Requirement: On-curve point expressions and frame vector expressions

The system SHALL provide point expressions that refer to a point on a named curve at a stated curve parameter (`Normalized` or `Uniform`). The system SHALL provide vector expressions that refer to the tangent free vector and the normal free vector of a named curve at a stated curve parameter, with the length and orientation semantics of the curve tangent/normal maps. These expressions SHALL evaluate as functions of the involved geometry parameters and SHALL NOT be coordinate snapshots detached from those parameters. Existing endpoint point expressions remain valid and SHALL be consistent with on-curve samples at the corresponding normalized endpoints when the curve has endpoints.

#### Scenario: Congruence can use an on-curve point expression

- **WHEN** a Congruence relationship binds an on-curve point expression at `Uniform(0.5)` on a segment to another point expression
- **AND** the equation system is assembled and solved successfully
- **THEN** the segment midpoint coincides with the other expression's location within solve tolerance

### Requirement: Cross-geometry constructions are expressions or constraints

Operations that involve more than one geometry (for example intersection of two curves) SHALL be represented as point or vector expressions and/or as constraint catalog entries that consume such expressions. The curve contract SHALL NOT require a binary intersect method as the sole representation of those operations. Full evaluation semantics for intersection and new tangency/incidence constraints MAY be specified in later changes; this capability requires only that such constructions are not modeled as mandatory first-class point geometry entities.

#### Scenario: Intersection is not a mandatory Point entity

- **WHEN** a construction needs the intersection of two curves
- **THEN** the intended core representation is a derived point expression (and/or a constraint over expressions)
- **AND** the core SHALL NOT require introducing a separate authoritative point geometry entity solely to name that intersection

### Requirement: Geometry set includes curves with segment as proving kind

The system SHALL treat parameterized curves as the primary geometry kind for path-like primitives. A line segment SHALL be a curve in this sense and SHALL remain supported. Additional curve kinds (circles, arcs, splines, function graphs) MAY be introduced in later changes that conform to the curve contract; this capability does not require them to be implemented yet. First-class point geometry entities and face/region geometries remain outside this requirement.

#### Scenario: Segment remains an admitted curve

- **WHEN** a document under this capability contains geometry
- **THEN** line segments are valid curve geometries
- **AND** each segment exposes normalized position, tangent, and normal as required for curves

#### Scenario: Non-segment curves are not yet mandatory

- **WHEN** a document is constructed with only line segment curves
- **THEN** the document satisfies this capability's geometry-set obligations
- **AND** absence of circle or spline kinds is not a violation

## MODIFIED Requirements

### Requirement: Endpoints yield point expressions

For every geometry that has the endpoints capability, the system SHALL provide an endpoints mapping that returns point expressions for those endpoints. A point expression SHALL evaluate to a location as a function of the geometry parameters and SHALL NOT be a snapshot of coordinates detached from those parameters. Geometries that are curves without the endpoints capability (intrinsically periodic closed curves) SHALL NOT be required to provide this mapping.

#### Scenario: Endpoint expression tracks segment parameters

- **WHEN** a segment's parameters change
- **AND** an endpoint point expression of that segment is evaluated
- **THEN** the resulting location matches the endpoint implied by the new parameters

#### Scenario: Periodic curve omits endpoints mapping

- **WHEN** an intrinsically periodic closed curve without join endpoints is queried for an endpoints mapping
- **THEN** the system does not treat missing geometric endpoints as a violation of the curve contract

## REMOVED Requirements

### Requirement: v1 geometry set is segments only

**Reason**: Replaced by the curve-centered geometry-set requirement so future primitives can conform to a shared curve contract without a hard segments-only gate.

**Migration**: Treat every existing line segment as a curve with endpoints; use normalized chart parameter \(t \in [0,1]\) for position/tangent; keep endpoint point expressions. Introduce further curve kinds only under the new curve requirements.
