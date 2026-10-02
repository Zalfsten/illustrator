# Spec Delta: geometry-ir

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

- **WHEN** Parallel is applied to tangent vector expressions of two line segments
- **THEN** those expressions are valid free-vector expressions under this capability
- **AND** no separate direction-expression factory is required

### Requirement: Curve quantities are exposed through expressions

For evaluate/solve and constraint assembly, derived locations and frame vectors on curves (endpoints where applicable, on-curve samples, tangent, normal) SHALL be obtained through **opaque point expressions** and **opaque vector expressions** that name geometries by stable identity. Implementations SHALL NOT require callers on the production evaluate/solve path to invoke geometry-kind-specific imperative sampling APIs that take State directly in place of those expressions. Geometry kinds SHALL construct the expressions via **capability-scoped factories**: every curve kind exposes on-curve, tangent, and normal factories; kinds with the endpoints capability additionally expose start and end factories; intrinsically periodic closed curves SHALL expose curve factories and SHALL NOT be required to expose start/end factories. Evaluation, connectivity, and analytical Jacobians for constraints SHALL go through the opaque expression API.

#### Scenario: Midpoint constraint uses an on-curve expression

- **WHEN** a structural relationship binds a location on a segment at normalized mid-parameter
- **THEN** the relationship references an on-curve point expression for that geometry
- **AND** assembly evaluates that expression against DocumentStructure and State rather than a separate geometry-only sampling entry point

#### Scenario: Closed curve omits endpoint factories

- **WHEN** an intrinsically periodic closed curve kind is admitted under this capability
- **THEN** it provides on-curve, tangent, and normal expression factories
- **AND** it is not required to provide start/end expression factories

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
