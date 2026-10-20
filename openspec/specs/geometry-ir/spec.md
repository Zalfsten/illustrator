# geometry-ir

## Purpose

Defines Model E: parameterized geometries expose derived point expressions, and relationships compile to equation objects that feed the solver — independent of UI sugar and of any concrete numerical algorithm.

## Requirements

### Requirement: Geometries are first-class parameterized objects

The system SHALL represent each geometric primitive as a first-class object with its own continuous parameters. A line segment SHALL be a geometry in this sense. The core model SHALL NOT require geometries to own references to independent point entities as their only representation of endpoints.

#### Scenario: Segment exists without shared point entities in the core

- **WHEN** a line segment is added to the document
- **THEN** the segment is stored as a parameterized geometry
- **AND** its endpoint locations are obtainable only through derived point expressions, not as mandatory separate core point entities

### Requirement: Endpoints yield point expressions

For every geometry that has the endpoints capability, the system SHALL provide an endpoints mapping that returns point expressions for those endpoints. A point expression SHALL evaluate to a location as a function of the geometry parameters and SHALL NOT be a snapshot of coordinates detached from those parameters. Geometries that are curves without the endpoints capability (intrinsically periodic closed curves) SHALL NOT be required to provide this mapping.

#### Scenario: Endpoint expression tracks segment parameters

- **WHEN** a segment's parameters change
- **AND** an endpoint point expression of that segment is evaluated
- **THEN** the resulting location matches the endpoint implied by the new parameters

#### Scenario: Periodic curve omits endpoints mapping

- **WHEN** an intrinsically periodic closed curve without join endpoints is queried for an endpoints mapping
- **THEN** the system does not treat missing geometric endpoints as a violation of the curve contract

### Requirement: Relationships are equation factories

The system SHALL represent geometric relationships as operations that produce equation objects (residual contributions) suitable for inclusion in a global equation system. Assembling the document's structural relationships SHALL yield a combined residual vector over the collected geometry parameters.

#### Scenario: Structural equations assemble from relationships

- **WHEN** a document contains geometries and structural relationships
- **AND** the equation system is assembled
- **THEN** each structural relationship contributes one or more residual components
- **AND** the residual vector is a function of the geometry parameters

### Requirement: UI shared points desugar to congruence

Logical points shared in the user interface (for example a corner joining two segments) SHALL be represented in the core solely by congruence equations between the corresponding endpoint point expressions. The core SHALL treat UI shared-point identity as sugar for those equations, not as a second competing source of truth for location.

#### Scenario: Shared UI corner becomes congruence

- **WHEN** the user connects two segment ends as one shared UI point
- **THEN** the core model contains a congruence relationship between those endpoint expressions
- **AND** no separate authoritative core point entity is required to define that shared location

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

Geometric start and end join points SHALL be provided only for curves that have an endpoints capability. That capability SHALL expose point expressions for start and end. Those expressions SHALL evaluate consistently with the curve position at normalized \(t = 0\) and \(t = 1\), and MAY use kind-efficient connectivity or Jacobians. Intrinsically periodic closed curves (for example a full circle) SHALL implement the curve position/tangent contract and SHALL NOT be required to expose geometric endpoints; their chart MAY use an origin for \(t = 0\) that is a parameterization gauge, not a join endpoint.

#### Scenario: Segment exposes endpoint expressions

- **WHEN** a line segment is queried for endpoints
- **THEN** start and end point expressions are available
- **AND** evaluating them matches position at normalized \(t = 0\) and \(t = 1\)

#### Scenario: Periodic curve needs no join endpoints

- **WHEN** an intrinsically periodic closed curve is present in the document
- **THEN** it still exposes normalized position, tangent, and normal for subdivision and constraints
- **AND** the core SHALL NOT require geometric start/end join endpoints for that curve

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

### Requirement: Cross-geometry constructions are expressions or constraints

Operations that involve more than one geometry (for example intersection of two curves) SHALL be represented as point or vector expressions and/or as constraint catalog entries that consume such expressions. The curve contract SHALL NOT require a binary intersect method as the sole representation of those operations. Full evaluation semantics for intersection and new tangency/incidence constraints MAY be specified in later changes; this capability requires only that such constructions are not modeled as mandatory first-class point geometry entities.

#### Scenario: Intersection is not a mandatory Point entity

- **WHEN** a construction needs the intersection of two curves
- **THEN** the intended core representation is a derived point expression (and/or a constraint over expressions)
- **AND** the core SHALL NOT require introducing a separate authoritative point geometry entity solely to name that intersection

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

### Requirement: Circular arc is an admitted curve kind with endpoints

The system SHALL admit a **circular arc** geometry kind: an open arc of a circle in the plane. A circular arc SHALL be a curve under this capability and SHALL expose the endpoints capability (geometric start and end join points). Document SHALL store circular arcs in the same closed sum of geometry kinds as line segments. Chart continuous parameters SHALL be: center location, radius length DOF, start **heading angle**, and **turn angle** sweep. In committed document State the radius SHALL be non-negative. A solver MAY hold a negative radius in intermediate iterates (the parameter is an unconstrained real during a solve); whenever a State is committed, an arc with radius \(R < 0\) SHALL be rewritten to radius \(-R\) and start heading \(\theta_{\mathrm{start}} + \pi\) with unchanged center and sweep, which describes the same curve. Evaluating the curve at normalized \(t \in [0,1]\) SHALL yield the point at polar angle \(\theta(t) = \theta_{\mathrm{start}} + t\,\Delta\theta\) on that circle (world +x as the heading null ray; increasing heading is counterclockwise in the y-up world frame). Start and end point expressions SHALL match on-curve samples at \(t = 0\) and \(t = 1\). Total arc length SHALL equal \(|R|\,|\Delta\theta|\). The radius expression of an arc SHALL evaluate to \(|R|\). For a circular arc of constant radius, `Normalized` and `Uniform` curve parameters SHALL yield the same point for the same numeric argument.

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

#### Scenario: Committed arc radius is non-negative

- **WHEN** an interactive solve ends with an arc whose radius parameter is negative and the result is committed
- **THEN** the committed State holds radius \(-R \ge 0\) and start heading \(\theta_0 + \pi\), with center and sweep unchanged
- **AND** start point, end point, on-curve samples, tangents, normals and length are equal (within floating-point tolerance) before and after the rewrite

#### Scenario: Adding an arc with negative radius stores the canonical form

- **WHEN** an arc is added to Document with a negative radius in its initial chart values
- **THEN** the stored State holds the non-negative radius and the start heading shifted by \(\pi\)
- **AND** the arc evaluates to the same curve as the given initial values

#### Scenario: Radius expression yields the magnitude

- **WHEN** the radius expression of an arc is evaluated at a State whose radius parameter is \(R\) (including a negative intermediate iterate)
- **THEN** it evaluates to \(|R|\)

### Requirement: Geometries have a result or construction role

Each geometry in Document SHALL have a role that is either **result** or **construction**. The role SHALL be part of Document (persistent sketch data), not solely transient UI state. Construction geometries SHALL remain parameterized geometries that participate in relationships, expression evaluation, and assembly like result geometries. The role SHALL NOT by itself remove a geometry from Document or from the State parameter vector.

#### Scenario: Construction geometry still constrains

- **WHEN** a construction line segment and a result line segment are related by a structural constraint
- **AND** the equation system is assembled and solved
- **THEN** the construction geometry's parameters participate in residuals like any other geometry
- **AND** the construction role does not exempt it from hard structural constraints

#### Scenario: Role is inspectable without evaluating State

- **WHEN** Document is inspected without reading State \(p\)
- **THEN** each geometry's result vs construction role is determined by Document alone

### Requirement: Geometry kinds own chart formulas; shared shell is kind-agnostic

Each document geometry SHALL combine **shared metadata** (stable identity, result/construction role, contiguous State parameter-block layout) with a **kind-specific payload** drawn from a closed sum of admitted geometry kinds (line segment today). Chart formulas that map that kind's local parameters to points or free vectors (endpoints where applicable, on-curve samples, tangents, normals, and similar) SHALL be defined with that kind. Opaque point and vector expressions SHALL evaluate those formulas without requiring constraint, assembly, or Interact code to match on geometry kind or on private formula kinds. Extending the admitted kind set SHALL extend the kind sum and the expression formula catalog; it SHALL NOT require changing shared identity or role types, and SHALL NOT reintroduce a privileged segment-only identity or store API on the evaluate/solve path.

#### Scenario: Adding a kind does not change identity or role types

- **WHEN** a future curve kind is admitted under this capability
- **THEN** geometries of that kind still use the same stable geometry identity type and result/construction role
- **AND** constraints that already consume opaque point or vector expressions continue to evaluate without matching on the new kind's formula variants

#### Scenario: Segment formulas are kind-owned

- **WHEN** a segment endpoint or on-curve sample is needed for constraints or Mesh
- **THEN** the corresponding point expression is constructed from the segment kind's formula catalog
- **AND** evaluation resolves the named geometry identity and applies that segment formula against State

### Requirement: Kind charts supply initial parameters; formulas use the shared block window

Each geometry kind SHALL define how its initial continuous parameters are packed for structural add (a kind-owned chart init). When those parameters are registered on the Document, the shared geometry metadata SHALL record the contiguous State block placement. Kind chart formulas that evaluate points or free vectors SHALL read State entries inside that recorded block window; they SHALL NOT rely on a separate authoritative global-index layout owned outside Document.

#### Scenario: Segment add packs endpoints into the new block

- **WHEN** a line segment is added with two endpoint locations as chart init
- **THEN** State receives four contiguous scalars in endpoint order for that geometry
- **AND** the geometry record's block layout identifies those scalars
- **AND** evaluating the segment's start and end point expressions at that State yields the init endpoints

#### Scenario: Formula evaluation uses the recorded block

- **WHEN** a kind-owned point or vector formula for a geometry is evaluated against Document and State
- **THEN** it resolves the geometry by stable identity
- **AND** it samples State only within the contiguous parameter window stored on that geometry's shared metadata

### Requirement: Heading angles and turn angles are distinct

The system SHALL distinguish **heading angles** (absolute orientations measured from a fixed planar null ray / angle origin) from **turn angles** (free angular displacements, independent of that origin). Implementations SHALL NOT overload a single angle quantity type for both roles. Algebra SHALL respect the distinction: the difference of two heading angles is a turn angle; adding a turn angle to a heading angle yields a heading angle; adding two turn angles yields a turn angle; adding two heading angles is not a meaningful operation. Both kinds of angle SHALL be expressible as continuous State degrees of freedom (radians) for chart packing when a geometry kind needs them. Wrapping or principal-value reduction at the type level is NOT required by this capability.

#### Scenario: Difference of headings is a turn

- **WHEN** two heading angles are subtracted in the domain model
- **THEN** the result is a turn angle
- **AND** it is not typed as a heading angle

#### Scenario: Heading plus turn is a heading

- **WHEN** a heading angle is combined with a turn angle by addition
- **THEN** the result is a heading angle

#### Scenario: Two headings cannot be added as a domain operation

- **WHEN** a caller attempts to treat the sum of two heading angles as a defined domain operation
- **THEN** the model does not admit that combination as a heading or turn result

### Requirement: Opaque heading-angle and turn-angle expressions

The system SHALL provide opaque **heading-angle expressions** and opaque **turn-angle expressions** that map State parameters (via named geometries and kind-owned formulas) to a heading angle or turn angle respectively. These families SHALL be distinct from each other, from length scalar expressions, from point expressions, and from free-vector expressions. Constraints and Interact SHALL NOT match on private formula kinds; they SHALL only evaluate, report parameter connectivity, and obtain analytical derivatives through the opaque expression API for each family. Kind-owned factories that construct such expressions MAY be introduced when a geometry kind needs them; this requirement does not itself admit a new geometry kind.

#### Scenario: Families stay separate

- **WHEN** a constraint or Interact path consumes a heading-angle expression
- **THEN** it does not treat that handle as a turn-angle expression or a length scalar expression
- **AND** evaluation does not require matching on private formula variants

#### Scenario: Evaluation names geometries by identity

- **WHEN** a heading-angle or turn-angle expression is evaluated at a State
- **THEN** it resolves named geometries by stable identity as needed
- **AND** the result is not a numeric snapshot detached from those parameters

### Requirement: Opaque scalar expressions

The system SHALL provide opaque **scalar expressions** that map State parameters (via named geometries and kind-owned formulas) to a single real-valued quantity in a **typed scalar family**. Typed families in this capability include at least **length** (physical length), **heading angle**, and **turn angle**. Scalar expressions SHALL evaluate as functions of the involved geometry parameters and SHALL NOT be numeric snapshots detached from those parameters. Constraints and Interact SHALL NOT match on private scalar formula kinds; they SHALL only evaluate, report parameter connectivity, and obtain analytical derivatives through the opaque scalar-expression API of the appropriate family. Scalar expressions SHALL be distinct from point expressions and free-vector expressions. An untyped catch-all scalar expression that erases length versus heading versus turn SHALL NOT be the production API for constraints.

#### Scenario: EqualLength consumes scalar expressions without formula matching

- **WHEN** EqualLength is applied to two length scalar expressions
- **THEN** assembly evaluates those expressions and forms the residual without matching on private scalar formula kinds
- **AND** each expression names geometries by stable identity as needed for evaluation

#### Scenario: Length is not interchangeable with angle families

- **WHEN** a length scalar expression is constructed for a curve
- **THEN** it is not admissible wherever a heading-angle or turn-angle expression is required
- **AND** conversely a heading-angle or turn-angle expression is not admissible wherever a length scalar expression is required

### Requirement: Curves expose a length scalar expression

Every curve kind SHALL expose a capability-scoped factory that constructs an opaque scalar expression for the curve's **total arc length** (physical length of the curve, not a normalized parameter). For a line segment, that length SHALL equal the Euclidean distance between its endpoints. Intrinsically periodic closed curves SHALL expose the same length factory (full closed length) and SHALL NOT require the endpoints capability. Length factories SHALL live with the curve capability, not the endpoints capability.

#### Scenario: Segment length matches endpoint distance

- **WHEN** a length scalar expression is constructed for a line segment
- **AND** it is evaluated at a given State
- **THEN** the value equals the Euclidean distance between the segment's start and end locations at that State

#### Scenario: Length factory does not require endpoints capability

- **WHEN** a curve kind without join endpoints is admitted under this capability
- **THEN** it is still required to expose a length scalar expression factory as part of the curve capability
- **AND** absence of start/end factories does not remove the length obligation

### Requirement: Curve quantities are exposed through expressions

For evaluate/solve and constraint assembly, derived locations, frame vectors, and curve length on curves (endpoints where applicable, on-curve samples, tangent, normal, total arc length) SHALL be obtained through **opaque point expressions**, **opaque vector expressions**, and **opaque scalar expressions** that name geometries by stable identity. Implementations SHALL NOT require callers on the production evaluate/solve path to invoke geometry-kind-specific imperative sampling APIs that take State directly in place of those expressions. Geometry kinds SHALL construct the expressions via **capability-scoped factories**: every curve kind exposes on-curve, tangent, normal, and length factories; kinds with the endpoints capability additionally expose start and end factories; intrinsically periodic closed curves SHALL expose curve factories (including length) and SHALL NOT be required to expose start/end factories. Evaluation, connectivity, and analytical Jacobians for constraints SHALL go through the opaque expression API.

#### Scenario: Midpoint constraint uses an on-curve expression

- **WHEN** a structural relationship binds a location on a segment at normalized mid-parameter
- **THEN** the relationship references an on-curve point expression for that geometry
- **AND** assembly evaluates that expression against Document and State rather than a separate geometry-only sampling entry point

#### Scenario: Closed curve omits endpoint factories

- **WHEN** an intrinsically periodic closed curve kind is admitted under this capability
- **THEN** it provides on-curve, tangent, normal, and length expression factories
- **AND** it is not required to provide start/end expression factories

#### Scenario: Equal length uses length scalar expressions

- **WHEN** EqualLength relates the lengths of two curve geometries
- **THEN** each side is a length scalar expression for that curve
- **AND** assembly evaluates those scalar expressions rather than a geometry-only length sampling entry point

### Requirement: Geometries are resolved by stable identity without scanning the full set

Each geometry in Document SHALL have a stable geometry identity. Document (or an equivalent read-only geometry resolution port implemented for it) SHALL resolve a geometry identity to the corresponding geometry metadata for expression evaluation, residual evaluation, and Jacobian assembly without scanning every geometry in the document for each lookup (lookup cost SHALL be independent of the number of geometries, aside from ordinary hash/map overhead). Constraints and expressions that name geometries SHALL use that identity. Removing a geometry SHALL drop the identity from the resolution map and invalidate or drop constraints that referenced it, consistent with existing structural-edit rules.

#### Scenario: Evaluate resolves a named geometry without a full scan

- **WHEN** a point or vector expression names a geometry identity
- **AND** that expression is evaluated against Document and a State
- **THEN** Document yields that geometry by identity resolution
- **AND** evaluation does not depend on linearly searching the entire geometry collection for a matching identity

#### Scenario: Remove drops identity resolution and orphan references

- **WHEN** a geometry is removed from Document
- **THEN** its identity is no longer resolvable
- **AND** structural constraints that referenced that identity are no longer present in the structure

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
