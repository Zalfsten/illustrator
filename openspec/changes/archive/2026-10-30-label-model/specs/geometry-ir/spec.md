# Spec Delta

## ADDED Requirements

### Requirement: Anchor capability exposes an origin expression

Geometries with a placement frame SHALL be able to expose an anchor capability. That capability SHALL provide an opaque **origin** point expression that does not depend on a curve parameter and that has analytical derivatives. For a label it SHALL evaluate to the label's origin. Curves (line segments, circular arcs) SHALL NOT expose it.

#### Scenario: Label origin

- **WHEN** the origin expression of a label with origin (1, 2) is evaluated
- **THEN** it equals (1, 2)
- **AND** its derivatives with respect to the label's origin parameters form the identity

#### Scenario: Segment has no origin

- **WHEN** a line segment is queried for the anchor capability
- **THEN** no origin expression is available for it

### Requirement: Label is an admitted geometry kind without the curve capability

The system SHALL admit a **label** geometry kind in the same closed sum of kinds as curves. Its chart parameters SHALL be an origin location and a heading angle. It SHALL expose the anchor and line capabilities and SHALL NOT expose the curve, endpoints or circular capabilities.

#### Scenario: Label has no curve expressions

- **WHEN** an on-curve point, tangent, normal or length expression is requested for a label's identity
- **THEN** the system reports that the expression is not available

#### Scenario: Label is solved like other geometries

- **WHEN** a fix constraint is applied to a label's origin expression and the drawing is solved
- **THEN** the label's origin satisfies the fix within solve tolerance

## MODIFIED Requirements

### Requirement: Geometry set includes curves with segment as proving kind

The system SHALL treat parameterized curves as the primary geometry kind for path-like primitives. A line segment SHALL be a curve in this sense and SHALL remain supported. A circular arc SHALL also be a supported curve kind under this capability. Document SHALL store geometries as a sum of kinds (of which line segment, circular arc and label are required kinds in this slice). The curve capability SHALL be optional per kind: a label is a geometry without it. Each stored geometry SHALL carry shared metadata (stable identity, result/construction role, and its State parameter block layout) alongside kind-specific payload. Shared geometry operations (parameter count, connectivity, reindex after removal, optional curve capability, role) SHALL apply uniformly across kinds. Kind-specific chart behavior and formula catalogs SHALL live with each kind (see *Geometry kinds own chart formulas*). Callers outside the kinds SHALL obtain capability-dependent behavior by querying capabilities and using identity-based capability factories (see *Capabilities are queryable by geometry identity*); dispatch on the kind discriminant SHALL be confined to the closed kind catalog. Code that handles curves (tessellation, drag length floors, curve sampling) SHALL skip geometries without the curve capability rather than assume every geometry is a curve. The core evaluate/solve path SHALL NOT use a privileged segment-only store API. Additional curve kinds (full circles, splines, function graphs) MAY be introduced in later changes that conform to the curve contract; this capability does not require them to be implemented yet. First-class point geometry entities and face/region geometries remain outside this requirement. Canvas paint and Mesh-backed pick SHALL use the display Mesh (or equivalent) rather than iterating document geometries by kind as the authoritative paint path.

#### Scenario: Segment remains an admitted curve

- **WHEN** a document under this capability contains geometry
- **THEN** line segments are valid curve geometries
- **AND** each segment exposes normalized position, tangent, and normal as required for curves

#### Scenario: Circular arc is an admitted curve

- **WHEN** a document contains a circular arc
- **THEN** that arc is a valid curve geometry under this capability
- **AND** it exposes normalized position, tangent, and normal as required for curves
- **AND** it exposes start and end endpoint expressions

#### Scenario: Non-segment curves are not yet mandatory

- **WHEN** a document is constructed with only line segments and circular arcs
- **THEN** the document satisfies this capability's geometry-set obligations
- **AND** absence of spline or full-circle kinds is not a violation

#### Scenario: Construction and result segments share the same kind API

- **WHEN** a document contains both a construction segment and a result segment
- **THEN** both are segment curve geometries under this capability
- **AND** they differ by role, not by a separate geometry kind

#### Scenario: Adding a kind does not touch capability callers

- **WHEN** a new curve kind with the line capability is admitted
- **THEN** tools and the Mesh handle it through capability queries
- **AND** no caller outside the kind catalog needs a kind-specific change for it

#### Scenario: Curve handling skips a label

- **WHEN** a document contains a segment and a label and a drag or a Mesh build runs over all geometries
- **THEN** the label is not treated as a curve
- **AND** the operation succeeds

### Requirement: Line capability exposes a direction expression

Curves whose carrier is a straight line SHALL be able to expose a line capability, and so SHALL labels, whose baseline is straight. That capability SHALL provide an opaque free-vector **direction** expression that does not depend on a curve parameter. For a line segment it SHALL evaluate to end minus start. For a label it SHALL evaluate to the unit vector of its heading. Curves without a straight carrier (for example circular arcs) SHALL NOT expose it, so a direction expression cannot be built for them. The direction expression SHALL provide analytical derivatives like other vector expressions.

#### Scenario: Segment direction is end minus start

- **WHEN** the direction expression of a line segment is evaluated at a State
- **THEN** it equals the segment's end point minus its start point
- **AND** it equals the segment's tangent expression at every normalized \(t\)

#### Scenario: Arcs offer no direction expression

- **WHEN** a circular arc is queried for the line capability
- **THEN** no direction expression is available for it

#### Scenario: Label direction is the unit heading vector

- **WHEN** the direction expression of a label with heading θ is evaluated
- **THEN** it equals (cos θ, sin θ)

#### Scenario: Label parallel to a segment

- **WHEN** a parallel constraint on a label's direction and a segment's direction is solved
- **THEN** the label's direction is parallel to the segment within solve tolerance

### Requirement: Capabilities are queryable by geometry identity

For a geometry identity resolvable in Document, the system SHALL report which capabilities its kind has: curve, endpoints, line, circular, anchor. The capability set of a kind SHALL be fixed for that kind and SHALL be declared together with the kind's capability factories, so the reported set cannot diverge from the factories the kind provides. Querying an identity that is not resolvable SHALL report that the geometry does not exist.

#### Scenario: Segment reports its capabilities

- **WHEN** the capabilities of a line segment's identity are queried
- **THEN** curve, endpoints and line are reported
- **AND** circular and anchor are not reported

#### Scenario: Arc reports its capabilities

- **WHEN** the capabilities of a circular arc's identity are queried
- **THEN** curve, endpoints and circular are reported
- **AND** line and anchor are not reported

#### Scenario: Label reports its capabilities

- **WHEN** the capabilities of a label's identity are queried
- **THEN** anchor and line are reported
- **AND** curve, endpoints and circular are not reported

#### Scenario: Reported set matches the factories

- **WHEN** any admitted kind reports a capability
- **THEN** the capability-scoped expressions of that capability can be built for geometries of that kind
- **AND** when it does not report a capability, those expressions cannot be built for it

### Requirement: Capability expressions can be built from a geometry identity

The system SHALL build capability-scoped expressions (start, end, direction, center, radius, origin) from a geometry identity alone, without the caller naming the geometry kind. When the geometry lacks the required capability, or the identity is not resolvable, the system SHALL report that the expression is not available and SHALL NOT return an expression. An expression built this way SHALL evaluate identically to one built through the kind's own factory.

#### Scenario: Direction from a picked segment

- **WHEN** a direction expression is requested for a line segment's identity
- **THEN** a direction expression is returned
- **AND** it evaluates to the segment's end point minus its start point

#### Scenario: Direction from a picked arc is unavailable

- **WHEN** a direction expression is requested for a circular arc's identity
- **THEN** the system reports that no direction expression is available
- **AND** no expression is returned

#### Scenario: Center from a picked arc

- **WHEN** a center expression is requested for a circular arc's identity
- **THEN** a center point expression is returned that evaluates to the arc's chart center

#### Scenario: Origin from a label

- **WHEN** an origin expression is requested for a label's identity
- **THEN** an origin point expression is returned that evaluates to the label's origin
