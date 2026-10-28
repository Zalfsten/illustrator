# Spec Delta

## ADDED Requirements

### Requirement: Capabilities are queryable by geometry identity

For a geometry identity resolvable in Document, the system SHALL report which capabilities its kind has: curve, endpoints, line, circular. The capability set of a kind SHALL be fixed for that kind and SHALL be declared together with the kind's capability factories, so the reported set cannot diverge from the factories the kind provides. Querying an identity that is not resolvable SHALL report that the geometry does not exist.

#### Scenario: Segment reports its capabilities

- **WHEN** the capabilities of a line segment's identity are queried
- **THEN** curve, endpoints and line are reported
- **AND** circular is not reported

#### Scenario: Arc reports its capabilities

- **WHEN** the capabilities of a circular arc's identity are queried
- **THEN** curve, endpoints and circular are reported
- **AND** line is not reported

#### Scenario: Reported set matches the factories

- **WHEN** any admitted kind reports a capability
- **THEN** the capability-scoped expressions of that capability can be built for geometries of that kind
- **AND** when it does not report a capability, those expressions cannot be built for it

### Requirement: Capability expressions can be built from a geometry identity

The system SHALL build capability-scoped expressions (start, end, direction, center, radius) from a geometry identity alone, without the caller naming the geometry kind. When the geometry lacks the required capability, or the identity is not resolvable, the system SHALL report that the expression is not available and SHALL NOT return an expression. An expression built this way SHALL evaluate identically to one built through the kind's own factory.

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

### Requirement: Tools decide applicability by capability

Tools and other callers that hold only geometry identities SHALL decide whether an operation applies by querying capabilities, and SHALL construct capability-scoped expressions through identity-based factories. They SHALL NOT branch on a concrete kind to decide applicability or to construct such expressions. An operation SHALL apply to every kind that has the capabilities it needs.

#### Scenario: Parallel tool accepts any line-capable geometry

- **WHEN** the user picks two geometries for a parallel or perpendicular relation
- **THEN** the tool accepts each pick whose geometry has the line capability
- **AND** builds the relation from direction expressions obtained by geometry identity

#### Scenario: Parallel tool rejects a geometry without line capability

- **WHEN** the user picks a circular arc for a parallel relation
- **THEN** the tool rejects the pick with feedback
- **AND** no relation is added

## MODIFIED Requirements

### Requirement: Geometry set includes curves with segment as proving kind

The system SHALL treat parameterized curves as the primary geometry kind for path-like primitives. A line segment SHALL be a curve in this sense and SHALL remain supported. A circular arc SHALL also be a supported curve kind under this capability. Document SHALL store geometries as a sum of kinds (of which line segment and circular arc are required kinds in this slice). Each stored geometry SHALL carry shared metadata (stable identity, result/construction role, and its State parameter block layout) alongside kind-specific payload. Shared geometry operations (parameter count, connectivity, reindex after removal, optional curve capability, role) SHALL apply uniformly across kinds. Kind-specific chart behavior and formula catalogs SHALL live with each kind (see *Geometry kinds own chart formulas*). Callers outside the kinds SHALL obtain capability-dependent behavior by querying capabilities and using identity-based capability factories (see *Capabilities are queryable by geometry identity*); dispatch on the kind discriminant SHALL be confined to the closed kind catalog. The core evaluate/solve path SHALL NOT use a privileged segment-only store API. Additional curve kinds (full circles, splines, function graphs) MAY be introduced in later changes that conform to the curve contract; this capability does not require them to be implemented yet. First-class point geometry entities and face/region geometries remain outside this requirement. Canvas paint and Mesh-backed pick SHALL use the display Mesh (or equivalent) rather than iterating document geometries by kind as the authoritative paint path.

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
