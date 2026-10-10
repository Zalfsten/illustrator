# Spec Delta: geometry-ir

## ADDED Requirements

### Requirement: Circular arc is an admitted curve kind with endpoints

The system SHALL admit a **circular arc** geometry kind: an open arc of a circle in the plane. A circular arc SHALL be a curve under this capability and SHALL expose the endpoints capability (geometric start and end join points). Document SHALL store circular arcs in the same closed sum of geometry kinds as line segments. Chart continuous parameters SHALL be: center location, signed radius length DOF, start **heading angle**, and **turn angle** sweep. Evaluating the curve at normalized \(t \in [0,1]\) SHALL yield the point at polar angle \(\theta(t) = \theta_{\mathrm{start}} + t\,\Delta\theta\) on that circle (world +x as the heading null ray; increasing heading is counterclockwise in the y-up world frame). Start and end point expressions SHALL match on-curve samples at \(t = 0\) and \(t = 1\). Total arc length SHALL equal \(R\,|\Delta\theta|\) using the absolute value of the sweep turn and the absolute value of the radius when forming physical length (signed radius as a State DOF remains allowed; geometric length uses magnitude). For a circular arc of constant radius, `Normalized` and `Uniform` curve parameters SHALL yield the same point for the same numeric argument.

#### Scenario: Arc on-curve midpoint uses half sweep

- **WHEN** a circular arc with start heading \(\theta_0\) and sweep \(\Delta\theta\) is sampled at normalized \(t = 0.5\)
- **THEN** the point lies on the circle of the arc's center and radius
- **AND** its polar heading from the center is \(\theta_0 + 0.5\,\Delta\theta\)

#### Scenario: Arc endpoints match chart ends

- **WHEN** start and end endpoint expressions of a circular arc are evaluated
- **THEN** they equal the on-curve samples at normalized \(t = 0\) and \(t = 1\) respectively

#### Scenario: Arc length is radius times absolute sweep

- **WHEN** a length scalar expression for a circular arc is evaluated
- **THEN** the value equals \(|R|\,|\Delta\theta|\) from that arc's chart parameters at the State

#### Scenario: Arc exposes heading and sweep expressions

- **WHEN** a circular arc is queried for its start-heading and sweep expressions
- **THEN** those opaque heading-angle and turn-angle expressions evaluate to the arc's start heading and sweep turn at the State
- **AND** constraints may consume them without matching on private formula kinds

## MODIFIED Requirements

### Requirement: Geometry set includes curves with segment as proving kind

The system SHALL treat parameterized curves as the primary geometry kind for path-like primitives. A line segment SHALL be a curve in this sense and SHALL remain supported. A circular arc SHALL also be a supported curve kind under this capability. Document SHALL store geometries as a sum of kinds (of which line segment and circular arc are required kinds in this slice). Each stored geometry SHALL carry shared metadata (stable identity, result/construction role, and its State parameter block layout) alongside kind-specific payload. Shared geometry operations (parameter count, connectivity, reindex after removal, optional curve capability, role) SHALL apply uniformly across kinds. Kind-specific chart behavior and formula catalogs SHALL live with each kind (see *Geometry kinds own chart formulas*). Callers SHALL obtain kind-specific behavior by matching on kind or by querying a capability only where a kind discriminant is required (for example Mesh tessellation); the core evaluate/solve path SHALL NOT use a privileged segment-only store API. Additional curve kinds (full circles, splines, function graphs) MAY be introduced in later changes that conform to the curve contract; this capability does not require them to be implemented yet. First-class point geometry entities and face/region geometries remain outside this requirement. Canvas paint and Mesh-backed pick SHALL use the display Mesh (or equivalent) rather than iterating document geometries by kind as the authoritative paint path.

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
