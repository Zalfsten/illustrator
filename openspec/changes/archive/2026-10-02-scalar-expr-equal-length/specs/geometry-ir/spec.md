# Spec Delta: geometry-ir

## ADDED Requirements

### Requirement: Opaque scalar expressions

The system SHALL provide opaque **scalar expressions** that map State parameters (via named geometries and kind-owned formulas) to a single real number. Scalar expressions SHALL evaluate as functions of the involved geometry parameters and SHALL NOT be numeric snapshots detached from those parameters. Constraints and Interact SHALL NOT match on private scalar formula kinds; they SHALL only evaluate, report parameter connectivity, and obtain analytical derivatives through the opaque scalar-expression API. Scalar expressions SHALL be distinct from point expressions and free-vector expressions.

#### Scenario: EqualLength consumes scalar expressions without formula matching

- **WHEN** EqualLength is applied to two scalar expressions
- **THEN** assembly evaluates those expressions and forms the residual without matching on private scalar formula kinds
- **AND** each expression names geometries by stable identity as needed for evaluation

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

## MODIFIED Requirements

### Requirement: Curve quantities are exposed through expressions

For evaluate/solve and constraint assembly, derived locations, frame vectors, and curve length on curves (endpoints where applicable, on-curve samples, tangent, normal, total arc length) SHALL be obtained through **opaque point expressions**, **opaque vector expressions**, and **opaque scalar expressions** that name geometries by stable identity. Implementations SHALL NOT require callers on the production evaluate/solve path to invoke geometry-kind-specific imperative sampling APIs that take State directly in place of those expressions. Geometry kinds SHALL construct the expressions via **capability-scoped factories**: every curve kind exposes on-curve, tangent, normal, and length factories; kinds with the endpoints capability additionally expose start and end factories; intrinsically periodic closed curves SHALL expose curve factories (including length) and SHALL NOT be required to expose start/end factories. Evaluation, connectivity, and analytical Jacobians for constraints SHALL go through the opaque expression API.

#### Scenario: Midpoint constraint uses an on-curve expression

- **WHEN** a structural relationship binds a location on a segment at normalized mid-parameter
- **THEN** the relationship references an on-curve point expression for that geometry
- **AND** assembly evaluates that expression against DocumentStructure and State rather than a separate geometry-only sampling entry point

#### Scenario: Closed curve omits endpoint factories

- **WHEN** an intrinsically periodic closed curve kind is admitted under this capability
- **THEN** it provides on-curve, tangent, normal, and length expression factories
- **AND** it is not required to provide start/end expression factories

#### Scenario: Equal length uses length scalar expressions

- **WHEN** EqualLength relates the lengths of two curve geometries
- **THEN** each side is a length scalar expression for that curve
- **AND** assembly evaluates those scalar expressions rather than a geometry-only length sampling entry point
