## MODIFIED Requirements

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

### Requirement: Cross-geometry constructions are expressions or constraints

Operations that involve more than one geometry (for example intersection of two curves) SHALL be represented as point or vector expressions and/or as constraint catalog entries that consume such expressions. The curve contract SHALL NOT require a binary intersect method as the sole representation of those operations.

#### Scenario: Intersection is not a mandatory Point entity

- **WHEN** a construction needs the intersection of two curves
- **THEN** the intended core representation is a derived point expression (and/or a constraint over expressions)
- **AND** the core SHALL NOT require introducing a separate authoritative point geometry entity solely to name that intersection

### Requirement: Geometry set includes curves with segment as proving kind

The system SHALL treat parameterized curves as the primary geometry kind for path-like primitives. A line segment SHALL be a curve in this sense and SHALL remain supported. A circular arc SHALL also be a supported curve kind under this capability. Document SHALL store geometries as a sum of kinds (of which line segment, circular arc and label are required kinds in this slice). The curve capability SHALL be optional per kind: a label is a geometry without it.

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

## ADDED Requirements

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

The normalized chart SHALL be the canonical parameterization of the curve's defining formula with its natural domain affinely mapped to \([0,1]\); equal steps in \(t\) NEED NOT correspond to equal arc length except for curves whose chart already has constant speed (for example a line segment).

#### Scenario: Quarter of a segment

- **WHEN** a segment from (0, 0) to (4, 0) is sampled at normalized t = 0.25
- **THEN** the sample is (1, 0)

### Requirement: Closed curves need no endpoints

Intrinsically periodic closed curves (for example a full circle) SHALL implement the curve position/tangent contract and SHALL NOT be required to expose geometric endpoints; their chart MAY use an origin for \(t = 0\) that is a parameterization gauge, not a join endpoint.

#### Scenario: Curve without endpoints still has a position at t = 0

- **WHEN** a curve kind without the endpoints capability is asked for its start point expression
- **THEN** no start point expression can be built
- **AND** its position at normalized t = 0 is still evaluable

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

### Requirement: Intersection semantics may come later

Full evaluation semantics for intersection and new tangency/incidence constraints MAY be specified in later changes; this capability requires only that such constructions are not modeled as mandatory first-class point geometry entities. Such constructions SHALL NOT be modeled as mandatory first-class point geometry entities.

#### Scenario: No intersection point kind

- **WHEN** the admitted geometry kinds are listed
- **THEN** none of them is an intersection point entity

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

### Requirement: Expressions hide kinds from constraints

Opaque point and vector expressions SHALL evaluate those formulas without requiring constraint, assembly, or Interact code to match on geometry kind or on private formula kinds.

#### Scenario: Dragging an arc end

- **WHEN** Interact drags an arc's end point
- **THEN** it moves it through the arc's point expression, without code specific to arcs

### Requirement: Extending the kind set leaves the shell unchanged

Extending the admitted kind set SHALL extend the kind sum and the expression formula catalog; it SHALL NOT require changing shared identity or role types, and SHALL NOT reintroduce a privileged segment-only identity or store API on the evaluate/solve path.

#### Scenario: One identity type for all kinds

- **WHEN** a Document holds a segment, an arc and a label
- **THEN** their identities are of one type and resolve through the same lookup
- **AND** their roles are of one type

### Requirement: Kind formulas read their own block

Kind chart formulas that evaluate points, free vectors, scalars, exact shape or curvature SHALL read State entries inside that recorded block window; they SHALL NOT rely on a separate authoritative global-index layout owned outside Document.

#### Scenario: Segment after an arc

- **WHEN** a segment is added after an arc and its endpoints are evaluated
- **THEN** the evaluation reads only entries of the segment's own block

### Requirement: Kind formulas may read their own attributes

Besides that block, a kind formula MAY read the non-parameter attributes of the same geometry (for example a label's text attributes); it SHALL NOT read another geometry's attributes.

#### Scenario: Two labels, own attributes

- **WHEN** two labels with different attributes are evaluated
- **THEN** each label's formulas use only that label's attributes

### Requirement: Attributes are not State

Attributes SHALL NOT be State degrees of freedom: they contribute no derivative columns, and which State entries a formula reads SHALL NOT depend on them.

#### Scenario: Changing an attribute keeps the parameter layout

- **WHEN** a label's source attribute is changed
- **THEN** the label's parameter count and connectivity are unchanged
- **AND** its Jacobians have no column for the attribute

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

### Requirement: Angle expressions are used only through their API

Constraints and Interact SHALL NOT match on private formula kinds; they SHALL only evaluate, report parameter connectivity, and obtain analytical derivatives through the opaque expression API for each family. Kind-owned factories that construct such expressions MAY be introduced when a geometry kind needs them; this requirement does not itself admit a new geometry kind.

#### Scenario: Sweep through the API

- **WHEN** a constraint uses an arc's sweep turn-angle expression
- **THEN** it obtains value, connectivity and derivatives through the turn-angle expression API alone

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

### Requirement: Length factory lives with the curve capability

Intrinsically periodic closed curves SHALL expose the same length factory (full closed length) and SHALL NOT require the endpoints capability. Length factories SHALL live with the curve capability, not the endpoints capability.

#### Scenario: Length without endpoints

- **WHEN** a curve kind has the curve capability but not the endpoints capability
- **THEN** its length expression can still be built

### Requirement: No kind-specific sampling on the solve path

Implementations SHALL NOT require callers on the production evaluate/solve path to invoke geometry-kind-specific imperative sampling APIs that take State directly in place of those expressions.

#### Scenario: Residuals read curves through expressions

- **WHEN** residuals involving a segment and an arc are assembled
- **THEN** every curve quantity they use is obtained by evaluating an expression

### Requirement: Kinds construct expressions through capability factories

Geometry kinds SHALL construct the expressions via **capability-scoped factories**: every curve kind exposes on-curve, tangent, normal, and length factories; kinds with the endpoints capability additionally expose start and end factories; intrinsically periodic closed curves SHALL expose curve factories (including length) and SHALL NOT be required to expose start/end factories.

#### Scenario: Arc and label factories

- **WHEN** the factories of an arc and of a label are listed
- **THEN** the arc has on-curve, tangent, normal, length, start and end factories
- **AND** the label has none of the curve factories

### Requirement: Geometry identities are used and dropped consistently

Constraints and expressions that name geometries SHALL use that identity. Removing a geometry SHALL drop the identity from the resolution map and invalidate or drop constraints that referenced it, consistent with existing structural-edit rules.

#### Scenario: Removed geometry no longer resolves

- **WHEN** a geometry with a constraint on it is removed
- **THEN** its identity no longer resolves
- **AND** the constraint on it is gone

### Requirement: Curves without a straight carrier have no direction

Curves without a straight carrier (for example circular arcs) SHALL NOT expose it, so a direction expression cannot be built for them. The direction expression SHALL provide analytical derivatives like other vector expressions.

#### Scenario: No direction for an arc

- **WHEN** a direction expression is requested for an arc
- **THEN** none can be built

#### Scenario: Direction derivatives

- **WHEN** the Jacobian of a segment's direction expression is evaluated
- **THEN** it agrees with a finite-difference derivative within tolerance
