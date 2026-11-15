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

A curve geometry SHALL expose a position map, a tangent map, and a normal map over a normalized chart parameter \(t \in [0,1]\). Position SHALL evaluate to a point (via a point expression). Tangent and normal SHALL evaluate to free vectors (via vector expressions), not points.

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

### Requirement: Curve tangent is the chart derivative

The tangent at \(t\) SHALL be the chart derivative \(\gamma'(t)\) (parametric velocity); its length SHALL be the parametric speed and SHALL NOT be required to be one.

#### Scenario: Segment tangent has the segment's length

- **WHEN** the tangent of a segment from (0, 0) to (3, 4) is evaluated at t = 0.5
- **THEN** it is the free vector (3, 4), of length 5

### Requirement: Curve normal is the left-rotated tangent

The normal at \(t\) SHALL be a planar perpendicular to that tangent obtained by a fixed orientation convention (leftward relative to the curve's increasing-\(t\) direction: if tangent is \((v_x, v_y)\), normal is \((-v_y, v_x)\)); the normal's length SHALL equal the tangent's length at the same \(t\). The normal SHALL NOT be defined as a curvature-weighted vector; curvature as a separate quantity is out of scope for this requirement.

#### Scenario: Segment normal points left

- **WHEN** the normal of a segment from (0, 0) to (3, 4) is evaluated at any t
- **THEN** it is the free vector (−4, 3)

### Requirement: Normalized chart is the canonical parameterization

The normalized chart SHALL be the canonical parameterization of the curve's defining formula with its natural domain affinely mapped to \([0,1]\), unless the kind states that its normalized chart is the arc-length fraction (as the function curve does). Equal steps in \(t\) NEED NOT correspond to equal arc length except for curves whose chart already has constant speed (for example a line segment) or is the arc-length fraction.

#### Scenario: Quarter of a segment

- **WHEN** a segment from (0, 0) to (4, 0) is sampled at normalized t = 0.25
- **THEN** the sample is (1, 0)

### Requirement: Curve parameter kinds Normalized and Uniform

The system SHALL address a location on a curve using a curve parameter: `Normalized` (\(t \in [0,1]\) in the canonical chart), `Uniform` (\(u \in [0,1]\), the arc-length fraction: equal increments of \(u\) give equal arc length) or, on a curve with the abscissa capability, `Abscissa` (see *Abscissa curve parameter*). A physical arc-length domain \(s \in [0, L]\) SHALL NOT be required. For a line segment, `Normalized` and `Uniform` SHALL yield the same point for the same numeric argument.

#### Scenario: Uniform mid-parameter is arc-length midpoint on a segment

- **WHEN** a line segment is sampled with `Uniform` parameter \(u = 0.5\)
- **THEN** the resulting point coincides with the sample at `Normalized` \(t = 0.5\)
- **AND** that point is the arc-length midpoint of the segment

#### Scenario: Uniform is distinct from Normalized in general

- **WHEN** a future non-constant-speed curve (for example a cubic Bézier) is admitted under this capability
- **THEN** `Normalized(0.5)` and `Uniform(0.5)` NEED NOT evaluate to the same location
- **AND** both parameter kinds remain valid addresses on that curve

### Requirement: Endpoint capability is separate from Curve

Geometric start and end join points SHALL be provided only for curves that have an endpoints capability. That capability SHALL expose point expressions for start and end. Those expressions SHALL evaluate consistently with the curve position at normalized \(t = 0\) and \(t = 1\), and MAY use kind-efficient connectivity or Jacobians.

#### Scenario: Segment exposes endpoint expressions

- **WHEN** a line segment is queried for endpoints
- **THEN** start and end point expressions are available
- **AND** evaluating them matches position at normalized \(t = 0\) and \(t = 1\)

#### Scenario: Periodic curve needs no join endpoints

- **WHEN** an intrinsically periodic closed curve is present in the document
- **THEN** it still exposes normalized position, tangent, and normal for subdivision and constraints
- **AND** the core SHALL NOT require geometric start/end join endpoints for that curve

### Requirement: Closed curves need no endpoints

Intrinsically periodic closed curves (for example a full circle) SHALL implement the curve position/tangent contract and SHALL NOT be required to expose geometric endpoints; their chart MAY use an origin for \(t = 0\) that is a parameterization gauge, not a join endpoint.

#### Scenario: Curve without endpoints still has a position at t = 0

- **WHEN** a curve kind without the endpoints capability is asked for its start point expression
- **THEN** no start point expression can be built
- **AND** its position at normalized t = 0 is still evaluable

### Requirement: On-curve point expressions and frame vector expressions

The system SHALL provide point expressions that refer to a point on a named geometry (by stable geometry identity) at a stated curve parameter (`Normalized` or `Uniform`), provided that geometry is a curve. The system SHALL provide vector expressions that refer to the tangent free vector and the normal free vector of a named curve geometry at a stated curve parameter, with the length and orientation semantics of the curve tangent/normal maps.

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

### Requirement: Curve expressions track their parameters

These on-curve, tangent and normal expressions SHALL evaluate as functions of the involved geometry parameters and SHALL NOT be coordinate snapshots detached from those parameters.

#### Scenario: Expression follows a moved segment

- **WHEN** an on-curve expression at normalized t = 0.5 is built for a segment and the segment's end is then moved by a drag
- **THEN** the expression evaluates to the midpoint of the moved segment

### Requirement: Endpoint expressions agree with on-curve samples

Existing endpoint point expressions remain valid and SHALL be consistent with on-curve samples at the corresponding normalized endpoints when the curve has endpoints. Endpoint expressions SHALL apply only to geometries that expose the endpoints capability; they SHALL NOT imply that every geometry is a segment.

#### Scenario: Arc start equals its sample at t = 0

- **WHEN** an arc's start point expression and its on-curve expression at normalized t = 0 are evaluated
- **THEN** both give the same point

### Requirement: No separate segment-direction factory is required

The system SHALL NOT require a separate segment-direction vector-expression factory; for line segments, tools MAY use a tangent vector expression at any chart parameter (the tangent is constant along the segment).

#### Scenario: Segment tangent is constant

- **WHEN** a segment's tangent is evaluated at t = 0, 0.5 and 1
- **THEN** all three are the same free vector

### Requirement: Cross-geometry constructions are expressions or constraints

Operations that involve more than one geometry (for example intersection of two curves) SHALL be represented as point or vector expressions and/or as constraint catalog entries that consume such expressions. The curve contract SHALL NOT require a binary intersect method as the sole representation of those operations.

#### Scenario: Intersection is not a mandatory Point entity

- **WHEN** a construction needs the intersection of two curves
- **THEN** the intended core representation is a derived point expression (and/or a constraint over expressions)
- **AND** the core SHALL NOT require introducing a separate authoritative point geometry entity solely to name that intersection

### Requirement: Intersection semantics may come later

Full evaluation semantics for intersection and new tangency/incidence constraints MAY be specified in later changes. Such constructions SHALL NOT be modeled as mandatory first-class point geometry entities.

#### Scenario: No intersection point kind

- **WHEN** the admitted geometry kinds are listed
- **THEN** none of them is an intersection point entity

### Requirement: Geometry set includes curves with segment as proving kind

The system SHALL treat parameterized curves as the primary geometry kind for path-like primitives. A line segment SHALL be a curve in this sense and SHALL remain supported. Circular arcs and function curves SHALL also be supported curve kinds under this capability. Document SHALL store geometries as a sum of kinds (of which line segment, circular arc, function curve and label are required kinds in this slice). The curve capability SHALL be optional per kind: a label is a geometry without it.

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

### Requirement: Geometries share metadata and operations

Each stored geometry SHALL carry shared metadata (stable identity, result/construction role, and its State parameter block layout) alongside kind-specific payload. Shared geometry operations (parameter count, connectivity, reindex after removal, optional curve capability, role) SHALL apply uniformly across kinds.

#### Scenario: Three kinds, one set of operations

- **WHEN** a Document holds a segment, an arc and a label
- **THEN** each has an identity, a role and a parameter block
- **AND** parameter count, connectivity and role are queried the same way for all three

### Requirement: Capability dispatch stays in the kind catalog

Kind-specific chart behavior and formula catalogs SHALL live with each kind (see *Geometry kinds own chart formulas*). Callers outside the kinds SHALL obtain capability-dependent behavior by querying capabilities and using identity-based capability factories (see *Capabilities are queryable by geometry identity*); dispatch on the kind discriminant SHALL be confined to the closed kind catalog.

#### Scenario: Direction by identity

- **WHEN** a caller needs the direction of a geometry whose kind it does not know
- **THEN** it builds the direction through the identity-based line capability factory, which yields a direction for a segment or label and none for an arc

### Requirement: Curve code skips non-curves

Code that handles curves (tessellation, drag length floors, curve sampling) SHALL skip geometries without the curve capability rather than assume every geometry is a curve. The core evaluate/solve path SHALL NOT use a privileged segment-only store API.

#### Scenario: Drag floors only for curves

- **WHEN** a drag runs in a component with a segment and a label
- **THEN** a soft length floor exists for the segment and none for the label

### Requirement: Further curve kinds may follow

Additional curve kinds (full circles, splines, function graphs) MAY be introduced in later changes that conform to the curve contract; this capability does not require them to be implemented yet. First-class point geometry entities and face/region geometries remain outside this requirement. Each such kind SHALL conform to the curve contract.

#### Scenario: A new curve kind is a curve

- **WHEN** a further curve kind is admitted
- **THEN** it exposes position, tangent, normal and length like every curve

### Requirement: Paint and pick use the display Mesh

Canvas paint and Mesh-backed pick SHALL use the display Mesh (or equivalent) rather than iterating document geometries by kind as the authoritative paint path.

#### Scenario: Arc and segment from the Mesh

- **WHEN** the App paints a drawing with an arc and a segment
- **THEN** both are stroked from their Mesh edges

### Requirement: Circular arc is an admitted curve kind with endpoints

The system SHALL admit a **circular arc** geometry kind: an open arc of a circle in the plane. A circular arc SHALL be a curve under this capability and SHALL expose the endpoints capability (geometric start and end join points). Document SHALL store circular arcs in the same closed sum of geometry kinds as line segments. Chart continuous parameters SHALL be: center location, radius length DOF, start **heading angle**, and **turn angle** sweep.

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

### Requirement: Arc radius is non-negative when committed

In committed document State the radius SHALL be non-negative. A solver MAY hold a negative radius in intermediate iterates (the parameter is an unconstrained real during a solve); whenever a State is committed, an arc with radius \(R < 0\) SHALL be rewritten to radius \(-R\) and start heading \(\theta_{\mathrm{start}} + \pi\) with unchanged center and sweep, which describes the same curve.

#### Scenario: Negative radius is rewritten on commit

- **WHEN** a solve ends with an arc of radius −2 and start heading 0.3
- **THEN** the committed arc has radius 2 and start heading 0.3 + π, with the same center and sweep

### Requirement: Arc evaluation follows the polar chart

Evaluating the curve at normalized \(t \in [0,1]\) SHALL yield the point at polar angle \(\theta(t) = \theta_{\mathrm{start}} + t\,\Delta\theta\) on that circle (world +x as the heading null ray; increasing heading is counterclockwise in the y-up world frame). Start and end point expressions SHALL match on-curve samples at \(t = 0\) and \(t = 1\).

#### Scenario: Middle of a half circle

- **WHEN** an arc with center (1, 0), radius 1, start heading 0 and sweep π is evaluated at t = 0.5
- **THEN** the point is (1, 1)

### Requirement: Arc length and radius expressions

Total arc length SHALL equal \(|R|\,|\Delta\theta|\). The radius expression of an arc SHALL evaluate to \(|R|\).

#### Scenario: Negative iterate

- **WHEN** an arc iterate has radius −2 and sweep −π/2
- **THEN** its length expression evaluates to π and its radius expression to 2

### Requirement: Arc Normalized equals Uniform

For a circular arc of constant radius, `Normalized` and `Uniform` curve parameters SHALL yield the same point for the same numeric argument.

#### Scenario: Same point at 0.3

- **WHEN** an arc is sampled at `Normalized(0.3)` and at `Uniform(0.3)`
- **THEN** both samples are the same point

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

Each document geometry SHALL combine **shared metadata** (stable identity, result/construction role, style reference and style overrides (styling), contiguous State parameter-block layout) with a **kind-specific payload** drawn from a closed sum of admitted geometry kinds (line segment today). Chart formulas that map that kind's local parameters to points or free vectors (endpoints where applicable, on-curve samples, tangents, normals, and similar) SHALL be defined with that kind.

#### Scenario: Adding a kind does not change identity or role types

- **WHEN** a future curve kind is admitted under this capability
- **THEN** geometries of that kind still use the same stable geometry identity type and result/construction role
- **AND** constraints that already consume opaque point or vector expressions continue to evaluate without matching on the new kind's formula variants

#### Scenario: Segment formulas are kind-owned

- **WHEN** a segment endpoint or on-curve sample is needed for constraints or Mesh
- **THEN** the corresponding point expression is constructed from the segment kind's formula catalog
- **AND** evaluation resolves the named geometry identity and applies that segment formula against State

#### Scenario: Every kind carries a style

- **WHEN** a segment, an arc and a label are created without style information
- **THEN** each refers to the style `default` and has no overrides, without any kind-specific style field

### Requirement: Expressions hide kinds from constraints

Opaque point and vector expressions SHALL evaluate the kind-owned chart formulas without requiring constraint, assembly, or Interact code to match on geometry kind or on private formula kinds.

#### Scenario: Dragging an arc end

- **WHEN** Interact drags an arc's end point
- **THEN** it moves it through the arc's point expression, without code specific to arcs

### Requirement: Extending the kind set leaves the shell unchanged

Extending the admitted kind set SHALL extend the kind sum and the expression formula catalog; it SHALL NOT require changing shared identity or role types, and SHALL NOT reintroduce a privileged segment-only identity or store API on the evaluate/solve path.

#### Scenario: One identity type for all kinds

- **WHEN** a Document holds a segment, an arc and a label
- **THEN** their identities are of one type and resolve through the same lookup
- **AND** their roles are of one type

### Requirement: Kind charts supply initial parameters; formulas use the shared block window

Each geometry kind SHALL define how its initial continuous parameters are packed for structural add (a kind-owned chart init). When those parameters are registered on the Document, the shared geometry metadata SHALL record the contiguous State block placement.

#### Scenario: Segment add packs endpoints into the new block

- **WHEN** a line segment is added with two endpoint locations as chart init
- **THEN** State receives four contiguous scalars in endpoint order for that geometry
- **AND** the geometry record's block layout identifies those scalars
- **AND** evaluating the segment's start and end point expressions at that State yields the init endpoints

#### Scenario: Formula evaluation uses the recorded block

- **WHEN** a kind-owned point or vector formula for a geometry is evaluated against Document and State
- **THEN** it resolves the geometry by stable identity
- **AND** it samples State only within the contiguous parameter window stored on that geometry's shared metadata

#### Scenario: Attributes add no derivative columns

- **WHEN** the derivatives of a formula of a geometry with non-parameter attributes (a label's origin) are evaluated
- **THEN** they refer only to State entries inside that geometry's block window
- **AND** their number is the same as for the same formula with other attribute values

#### Scenario: Kinds whose geometry does not depend on attributes are unaffected by them

- **WHEN** the text attributes of a label are replaced and State is unchanged
- **THEN** the label's origin and direction expressions evaluate to the same values as before

#### Scenario: Replacing attributes keeps committed State canonical

- **WHEN** the attributes of a geometry are replaced while its committed State block holds non-canonical chart values (for example an arc with a negative radius)
- **THEN** the committed block afterwards holds the canonical chart values for the new attributes
- **AND** the geometry evaluates to the same curve as before

### Requirement: Kind formulas read their own block

Kind chart formulas that evaluate points, free vectors, scalars, exact shape or curvature SHALL read State entries inside the State block window recorded in the geometry's shared metadata; they SHALL NOT rely on a separate authoritative global-index layout owned outside Document.

#### Scenario: Segment after an arc

- **WHEN** a segment is added after an arc and its endpoints are evaluated
- **THEN** the evaluation reads only entries of the segment's own block

### Requirement: Kind formulas may read their own attributes

Besides its geometry's State block window, a kind formula MAY read the non-parameter attributes of the same geometry (for example a label's text attributes); it SHALL NOT read another geometry's attributes.

#### Scenario: Two labels, own attributes

- **WHEN** two labels with different attributes are evaluated
- **THEN** each label's formulas use only that label's attributes

### Requirement: Attributes are not State

Attributes SHALL NOT be State degrees of freedom: they contribute no derivative columns, and which State entries a formula reads SHALL NOT depend on them.

#### Scenario: Changing an attribute keeps the parameter layout

- **WHEN** a label's source attribute is changed
- **THEN** the label's parameter count and connectivity are unchanged
- **AND** its Jacobians have no column for the attribute

### Requirement: Heading angles and turn angles are distinct

The system SHALL distinguish **heading angles** (absolute orientations measured from a fixed planar null ray / angle origin) from **turn angles** (free angular displacements, independent of that origin). Implementations SHALL NOT overload a single angle quantity type for both roles.

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

### Requirement: Angle algebra respects heading and turn

Algebra SHALL respect the distinction: the difference of two heading angles is a turn angle; adding a turn angle to a heading angle yields a heading angle; adding two turn angles yields a turn angle; adding two heading angles is not a meaningful operation.

#### Scenario: Heading minus heading

- **WHEN** heading 0.5 is subtracted from heading 2.0
- **THEN** the result is the turn angle 1.5, and adding it to heading 0.5 gives heading 2.0

### Requirement: Angles are State degrees of freedom without wrapping

Both kinds of angle SHALL be expressible as continuous State degrees of freedom (radians) for chart packing when a geometry kind needs them. Wrapping or principal-value reduction at the type level is NOT required by this capability.

#### Scenario: Large start heading is kept

- **WHEN** an arc is added with start heading 7.0 radians
- **THEN** its State holds the start heading as a continuous value in radians

### Requirement: Opaque heading-angle and turn-angle expressions

The system SHALL provide opaque **heading-angle expressions** and opaque **turn-angle expressions** that map State parameters (via named geometries and kind-owned formulas) to a heading angle or turn angle respectively. These families SHALL be distinct from each other, from length scalar expressions, from point expressions, and from free-vector expressions.

#### Scenario: Families stay separate

- **WHEN** a constraint or Interact path consumes a heading-angle expression
- **THEN** it does not treat that handle as a turn-angle expression or a length scalar expression
- **AND** evaluation does not require matching on private formula variants

#### Scenario: Evaluation names geometries by identity

- **WHEN** a heading-angle or turn-angle expression is evaluated at a State
- **THEN** it resolves named geometries by stable identity as needed
- **AND** the result is not a numeric snapshot detached from those parameters

### Requirement: Angle expressions are used only through their API

Constraints and Interact SHALL NOT match on private formula kinds; they SHALL only evaluate, report parameter connectivity, and obtain analytical derivatives through the opaque expression API for each family. Kind-owned factories that construct such expressions MAY be introduced when a geometry kind needs them; this requirement does not itself admit a new geometry kind.

#### Scenario: Sweep through the API

- **WHEN** a constraint uses an arc's sweep turn-angle expression
- **THEN** it obtains value, connectivity and derivatives through the turn-angle expression API alone

### Requirement: Opaque scalar expressions

The system SHALL provide opaque **scalar expressions** that map State parameters (via named geometries and kind-owned formulas) to a single real-valued quantity in a **typed scalar family**. Typed families in this capability include at least **length** (physical length), **heading angle**, and **turn angle**. Scalar expressions SHALL be distinct from point expressions and free-vector expressions.

#### Scenario: EqualLength consumes scalar expressions without formula matching

- **WHEN** EqualLength is applied to two length scalar expressions
- **THEN** assembly evaluates those expressions and forms the residual without matching on private scalar formula kinds
- **AND** each expression names geometries by stable identity as needed for evaluation

#### Scenario: Length is not interchangeable with angle families

- **WHEN** a length scalar expression is constructed for a curve
- **THEN** it is not admissible wherever a heading-angle or turn-angle expression is required
- **AND** conversely a heading-angle or turn-angle expression is not admissible wherever a length scalar expression is required

### Requirement: Scalar expressions track their parameters

Scalar expressions SHALL evaluate as functions of the involved geometry parameters and SHALL NOT be numeric snapshots detached from those parameters.

#### Scenario: Length after a drag

- **WHEN** a segment's length expression is evaluated before and after a drag that lengthens the segment
- **THEN** each evaluation gives the segment's length at that time

### Requirement: Scalar expressions are used through their typed API

Constraints and Interact SHALL NOT match on private scalar formula kinds; they SHALL only evaluate, report parameter connectivity, and obtain analytical derivatives through the opaque scalar-expression API of the appropriate family. An untyped catch-all scalar expression that erases length versus heading versus turn SHALL NOT be the production API for constraints.

#### Scenario: Equal length takes lengths

- **WHEN** an EqualLength relationship is built
- **THEN** both of its operands are length expressions, not untyped scalars or angles

### Requirement: Curves expose a length scalar expression

Every curve kind SHALL expose a capability-scoped factory that constructs an opaque scalar expression for the curve's **total arc length** (physical length of the curve, not a normalized parameter). For a line segment, that length SHALL equal the Euclidean distance between its endpoints.

#### Scenario: Segment length matches endpoint distance

- **WHEN** a length scalar expression is constructed for a line segment
- **AND** it is evaluated at a given State
- **THEN** the value equals the Euclidean distance between the segment's start and end locations at that State

#### Scenario: Length factory does not require endpoints capability

- **WHEN** a curve kind without join endpoints is admitted under this capability
- **THEN** it is still required to expose a length scalar expression factory as part of the curve capability
- **AND** absence of start/end factories does not remove the length obligation

### Requirement: Length factory lives with the curve capability

Intrinsically periodic closed curves SHALL expose the same length factory (full closed length) and SHALL NOT require the endpoints capability. Length factories SHALL live with the curve capability, not the endpoints capability.

#### Scenario: Length without endpoints

- **WHEN** a curve kind has the curve capability but not the endpoints capability
- **THEN** its length expression can still be built

### Requirement: Curve quantities are exposed through expressions

For evaluate/solve and constraint assembly, derived locations, frame vectors, and curve length on curves (endpoints where applicable, on-curve samples, tangent, normal, total arc length) SHALL be obtained through **opaque point expressions**, **opaque vector expressions**, and **opaque scalar expressions** that name geometries by stable identity. Evaluation, connectivity, and analytical Jacobians for constraints SHALL go through the opaque expression API.

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

### Requirement: No kind-specific sampling on the solve path

Implementations SHALL NOT require callers on the production evaluate/solve path to invoke geometry-kind-specific imperative sampling APIs that take State directly in place of the opaque point, vector and scalar expressions.

#### Scenario: Residuals read curves through expressions

- **WHEN** residuals involving a segment and an arc are assembled
- **THEN** every curve quantity they use is obtained by evaluating an expression

### Requirement: Kinds construct expressions through capability factories

Geometry kinds SHALL construct the expressions via **capability-scoped factories**: every curve kind exposes on-curve, tangent, normal, and length factories; kinds with the endpoints capability additionally expose start and end factories; intrinsically periodic closed curves SHALL expose curve factories (including length) and SHALL NOT be required to expose start/end factories.

#### Scenario: Arc and label factories

- **WHEN** the factories of an arc and of a label are listed
- **THEN** the arc has on-curve, tangent, normal, length, start and end factories
- **AND** the label has none of the curve factories

### Requirement: Geometries are resolved by stable identity without scanning the full set

Each geometry in Document SHALL have a stable geometry identity. Document (or an equivalent read-only geometry resolution port implemented for it) SHALL resolve a geometry identity to the corresponding geometry metadata for expression evaluation, residual evaluation, and Jacobian assembly without scanning every geometry in the document for each lookup (lookup cost SHALL be independent of the number of geometries, aside from ordinary hash/map overhead).

#### Scenario: Evaluate resolves a named geometry without a full scan

- **WHEN** a point or vector expression names a geometry identity
- **AND** that expression is evaluated against Document and a State
- **THEN** Document yields that geometry by identity resolution
- **AND** evaluation does not depend on linearly searching the entire geometry collection for a matching identity

#### Scenario: Remove drops identity resolution and orphan references

- **WHEN** a geometry is removed from Document
- **THEN** its identity is no longer resolvable
- **AND** structural constraints that referenced that identity are no longer present in the structure

### Requirement: Geometry identities are used and dropped consistently

Constraints and expressions that name geometries SHALL use that identity. Removing a geometry SHALL drop the identity from the resolution map and invalidate or drop constraints that referenced it, consistent with existing structural-edit rules.

#### Scenario: Removed geometry no longer resolves

- **WHEN** a geometry with a constraint on it is removed
- **THEN** its identity no longer resolves
- **AND** the constraint on it is gone

### Requirement: Line capability exposes a direction expression

Curves whose carrier is a straight line SHALL be able to expose a line capability, and so SHALL labels, whose baseline is straight. That capability SHALL provide an opaque free-vector **direction** expression that does not depend on a curve parameter. For a line segment it SHALL evaluate to end minus start. For a label it SHALL evaluate to the unit vector of its heading.

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

### Requirement: Curves without a straight carrier have no direction

Curves without a straight carrier (for example circular arcs) SHALL NOT expose the line capability, so a direction expression cannot be built for them. The direction expression SHALL provide analytical derivatives like other vector expressions.

#### Scenario: No direction for an arc

- **WHEN** a direction expression is requested for an arc
- **THEN** none can be built

#### Scenario: Direction derivatives

- **WHEN** the Jacobian of a segment's direction expression is evaluated
- **THEN** it agrees with a finite-difference derivative within tolerance

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

### Requirement: Capabilities are queryable by geometry identity

For a geometry identity resolvable in Document, the system SHALL report which capabilities its kind has: curve, endpoints, line, circular, anchor, abscissa. The capability set of a kind SHALL be fixed for that kind and SHALL be declared together with the kind's capability factories, so the reported set cannot diverge from the factories the kind provides. Querying an identity that is not resolvable SHALL report that the geometry does not exist.

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

### Requirement: Function curve is an admitted curve kind

The system SHALL admit a **function curve** geometry kind: the graph of \(y = f(x)\) for \(x\) in a range \([x_0, x_1]\), \(x_0 < x_1\), drawn in a frame with origin location \(O\) and heading angle \(\theta\), scaled by \(s_u > 0\) along the frame's x axis and \(s_v > 0\) along its y axis. \(O\) and \(\theta\) SHALL be its continuous parameters. The expression \(f\) (expression-language, variable `x`), \(x_0\), \(x_1\), \(s_u\) and \(s_v\) SHALL be attributes.

#### Scenario: Sine over one period

- **WHEN** a function curve with \(f\) = `sin(x)`, range \([0, 2\pi]\), scales 1, origin (0, 0) and heading 0 is added
- **THEN** its start point is (0, 0) and its end point is \((2\pi, 0)\)
- **AND** its length expression evaluates to 7.6403955780 within \(10^{-9}\)

#### Scenario: The frame rotates the graph

- **WHEN** the same curve has heading \(\pi/2\)
- **THEN** its end point is \((0, 2\pi)\)

#### Scenario: Scales stretch the graph

- **WHEN** a function curve with \(f\) = `x^2`, range \([0, 1]\), scales \(s_u = 2\), \(s_v = 3\), origin (1, 1) and heading 0 is added
- **THEN** its end point is (3, 4)

### Requirement: Function curve normalized parameter is the arc-length fraction

For a function curve with total length \(L\), the point at normalized \(t\) SHALL be \(O + R(\theta)\,(s_u\,x,\; s_v\,f(x))\) for the \(x\) whose arc length from \(x_0\) along the scaled graph is \(t\,L\), within \(10^{-9} L\) in arc length. `Normalized` and `Uniform` SHALL yield the same point for the same numeric argument.

#### Scenario: Midpoint of a sine period

- **WHEN** the sine curve over \([0, 2\pi]\) with scales 1, origin (0, 0) and heading 0 is sampled at normalized \(t = 0.5\)
- **THEN** the sample is \((\pi, 0)\)

#### Scenario: Arc length up to a sample

- **WHEN** the curve \(f\) = `x^2` on \([0, 1]\) is sampled at normalized \(t = 0.3\)
- **THEN** the arc length of the graph from \(x_0\) to the sample is \(0.3\,L\) within \(10^{-9} L\)
- **AND** the sample at `Uniform(0.3)` is the same point

### Requirement: Function curve capabilities

A function curve SHALL expose the curve capability, the endpoints capability (start at \(x_0\), end at \(x_1\)), the anchor capability with origin \(O\), the line capability with direction \(R(\theta)\,(1, 0)\), the unit vector of the frame's x axis, and the abscissa capability. It SHALL NOT expose the circular capability.

#### Scenario: Origin and direction

- **WHEN** a function curve has origin (2, 1) and heading 0.3
- **THEN** its origin expression evaluates to (2, 1) and its direction expression to \((\cos 0.3, \sin 0.3)\)

#### Scenario: No radius

- **WHEN** a radius expression is requested for a function curve
- **THEN** none can be built because the curve has no circular capability

### Requirement: Function curves accept only smooth functions

Creating a function curve, or changing its expression or range, SHALL succeed only if \(f\), \(f'\) and \(f''\) are proven finite on the whole closed range \([x_0, x_1]\) (expression-language domain check). Otherwise it SHALL be rejected, and an existing curve SHALL keep its attributes. A range whose end has a vertical tangent is therefore rejected.

#### Scenario: Vertical tangent at a range end

- **WHEN** a function curve with \(f\) = `sqrt(x)` on \([0, 1]\) is created
- **THEN** creation is rejected

#### Scenario: Pole inside the range

- **WHEN** a function curve with \(f\) = `1/x` on \([-1, 1]\) is created
- **THEN** creation is rejected

#### Scenario: Smooth on a shifted range

- **WHEN** a function curve with \(f\) = `sqrt(x)` on \([1, 4]\) is created
- **THEN** creation succeeds

### Requirement: Function curve attributes move only the local curve

Changing a function curve's attributes SHALL NOT itself change \(O\) and \(\theta\); it SHALL change the curve's points accordingly. A solve, including the one that follows a geometry-changing `set` (sketch-commands), SHALL change only \(O\) and \(\theta\) of a function curve, never its attributes.

#### Scenario: Longer range, same origin

- **WHEN** the range of the sine curve with origin (0, 0) and heading 0 is changed from \([0, \pi]\) to \([0, 2\pi]\) on a curve without constraints
- **THEN** its origin and heading are unchanged
- **AND** its end point moves from \((\pi, 0)\) to \((2\pi, 0)\)

### Requirement: Abscissa curve parameter

On a function curve, the curve parameter `Abscissa(x)` SHALL address the point \(O + R(\theta)\,(s_u\,x,\; s_v\,f(x))\). The tangent at `Abscissa(x)` SHALL be the derivative with respect to \(x\), \(R(\theta)\,(s_u,\; s_v\,f'(x))\); the normal SHALL be that tangent turned a quarter to the left. Expressions with an abscissa parameter SHALL be point and free-vector expressions like any other, evaluated from the State.

#### Scenario: Point and tangent at x = 1

- **WHEN** the sine curve over \([0, 2\pi]\) with scales 1, origin (0, 0) and heading 0 is addressed at `Abscissa(1)`
- **THEN** the point is \((1, \sin 1)\) and the tangent is \((1, \cos 1)\)

#### Scenario: Same direction as the normalized tangent

- **WHEN** the tangent at `Abscissa(1)` and the tangent at the normalized parameter of the same point are evaluated
- **THEN** they point in the same direction and their lengths differ

#### Scenario: Abscissa stays at its x when the range changes

- **WHEN** the range of that curve changes from \([0, 2\pi]\) to \([0, \pi]\)
- **THEN** the point at `Abscissa(1)` is still \((1, \sin 1)\)

### Requirement: Abscissa parameters need the abscissa capability

An expression with an `Abscissa` parameter SHALL be buildable by geometry identity only for a geometry that has the abscissa capability. For any other geometry, building it SHALL fail with a missing capability.

#### Scenario: No abscissa on a segment

- **WHEN** an on-curve expression with `Abscissa(0.5)` is requested for a segment
- **THEN** none can be built because the segment has no abscissa capability

### Requirement: Abscissa continues beyond the range

An abscissa outside the function curve's range \([x_0, x_1]\) SHALL address the point of the graph's continuation at that \(x\), with the same formulas. A change of the range SHALL NOT change the point or tangent at an abscissa.

#### Scenario: Point beyond the drawn range

- **WHEN** the sine curve over \([0, 2\pi]\) with scales 1, origin (0, 0) and heading 0 is addressed at `Abscissa(8)`
- **THEN** the point is \((8, \sin 8)\)

### Requirement: Abscissa requires a defined function with finite slope

An abscissa SHALL be valid only where evaluating \(f\) reports no domain error (expression-language), so \(f\), \(f'\) and \(f''\) are finite there. Building an expression at an invalid abscissa SHALL fail; at a point with a vertical tangent the normalized parameter has to be used instead.

#### Scenario: Vertical tangent

- **WHEN** an expression at `Abscissa(0)` is requested on a curve with \(f\) = `sqrt(x)` on \([1, 4]\)
- **THEN** none can be built because the slope at \(x = 0\) is not finite

#### Scenario: Undefined function

- **WHEN** an expression at `Abscissa(-1)` is requested on a curve with \(f\) = `log(x)` on \([1, 2]\)
- **THEN** none can be built because \(f\) is not defined at \(x = -1\)

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

### Requirement: Shapes without closed-form pieces stay within the tolerance

A curve whose shape is not a segment's or a circular arc's SHALL describe it with cubic Bézier pieces, each starting and ending exactly on the curve. Pieces SHALL be refined until a deviation check finds each within the requested tolerance of the curve, both ways; the check MAY estimate from samples. The shape's total length SHOULD differ from the curve's length expression by at most the tolerance.

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

#### Scenario: A piece cap spreads the remaining error

- **WHEN** a curve needs more pieces than the cap to reach the tolerance
- **THEN** the pieces with the largest deviation were refined first, so no part of the curve keeps a coarse initial piece
- **AND** the caller is told that the cap stopped the refinement

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

The curvature query and the piece bound query SHALL serve evaluation (for example display tessellation). They SHALL NOT be constraint operands, and they SHALL NOT require derivatives with respect to the State. They SHALL NOT change Document or the State they are evaluated at.

#### Scenario: Curvature leaves the drawing unchanged

- **WHEN** the curvature of an arc is queried at the committed State
- **THEN** Document and its committed State are unchanged afterwards

#### Scenario: Piece bound leaves the drawing unchanged

- **WHEN** a piece bound of an arc is queried at the committed State
- **THEN** Document and its committed State are unchanged afterwards

### Requirement: Curves bound length and curvature over a parameter piece

For every geometry with the curve capability, the system SHALL report, for a normalized parameter piece [t0, t1] ⊆ [0, 1] with t0 < t1 and a State, an upper bound of the arc length of that piece and an upper bound of its unsigned curvature at every point of the piece. Neither bound SHALL be smaller than the true value; the length bound SHALL be exact for segments and arcs. A caller SHALL obtain both by geometry identity without branching on the concrete kind.

#### Scenario: Segment piece

- **WHEN** the piece [0.25, 0.75] of a line segment of length 4 is queried
- **THEN** its length is 2 and its curvature bound is 0

#### Scenario: Arc piece

- **WHEN** the piece [0, 0.5] of a circular arc with radius 2 and sweep π is queried
- **THEN** its length is π within floating-point tolerance and its curvature bound is 0.5

#### Scenario: Geometry without the curve capability has no piece bound

- **WHEN** a piece of a label is queried
- **THEN** the query reports that the geometry has no piece bound
