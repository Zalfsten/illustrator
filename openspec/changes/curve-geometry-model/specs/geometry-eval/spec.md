# Spec Delta

## ADDED Requirements

### Requirement: Curves evaluate position, tangent, and normal with derivatives

A curve geometry SHALL evaluate its normalized position (as a point), tangent (as a free vector), and normal (as a free vector) at a requested normalized parameter \(t \in [0,1]\) from its local parameters in the supplied state, using the length and orientation semantics of the curve contract (tangent = chart derivative; normal = leftward planar rotate of tangent with equal length). It SHALL provide analytical derivatives of those quantities with respect to its local parameters sufficient to form Jacobian contributions when those quantities appear in residuals. The production Jacobian path for these quantities SHALL be analytical; finite-difference approximation SHALL NOT be used as a substitute for those derivatives.

#### Scenario: Segment tangent derivatives feed assembly

- **WHEN** a residual depends on a segment's tangent free vector at a normalized parameter
- **AND** assembly requests derivatives w.r.t. that segment's local parameters
- **THEN** analytical derivative information for the tangent map is available
- **AND** the local Jacobian block uses the segment's local parameter ordering

#### Scenario: Segment normal matches rotate of tangent

- **WHEN** a line segment's tangent and normal are evaluated at the same normalized \(t\)
- **THEN** the normal equals the leftward rotate of the tangent
- **AND** analytical derivatives of the normal are consistent with differentiating that rotate of the tangent

### Requirement: On-curve and Uniform parameters are evaluable

Point expressions that sample a curve at `Normalized` or `Uniform` parameters, and vector expressions for tangents and normals at those parameters, SHALL be evaluable against a state vector. For line segments, evaluation at `Uniform(u)` SHALL match evaluation at `Normalized(t)` for \(t = u\), including derivatives w.r.t. the segment's local parameters. Implementations SHALL document how `Uniform` will be obtained for future non-constant-speed curves (closed form or inversion); this capability requires a correct segment path and a defined extension point, not a general numerical arc-length solver.

#### Scenario: Segment OnCurve Uniform matches Normalized

- **WHEN** an on-curve point expression on a segment uses `Uniform(0.25)`
- **AND** the same segment is evaluated at `Normalized(0.25)`
- **THEN** both evaluations yield the same point from the same state
- **AND** both yield matching analytical derivatives w.r.t. the segment's local parameters

## MODIFIED Requirements

### Requirement: Geometries expose evaluation functions

A geometry SHALL provide evaluation functions that map its local parameters to geometric quantities needed by the system (for example endpoint locations, curve position, tangent, and normal at a normalized parameter, direction vectors, or other chart-defined outputs). These functions express results in terms of the geometry’s local parameterization. Point-valued outputs SHALL be distinguished from free-vector-valued outputs.

#### Scenario: Endpoint evaluation depends on local parameters

- **WHEN** a line segment geometry is asked for an endpoint location
- **THEN** the result is computed from that segment’s local parameters
- **AND** those local parameters are the ones referenced in the global state vector

#### Scenario: Curve mid-parameter evaluation depends on local parameters

- **WHEN** a line segment curve is asked for its position at normalized \(t = 0.5\)
- **THEN** the result is computed from that segment’s local parameters
- **AND** the result is a point, not a free vector

### Requirement: Geometries expose derivatives w.r.t. local parameters

A geometry SHALL provide analytical derivatives of its exposed quantities with respect to its local parameters (or equivalent analytical derivative information sufficient to form local Jacobian contributions). The production path for Jacobian assembly SHALL use those analytical derivatives. Finite-difference approximation SHALL NOT be used as a fallback or as the production Jacobian path.

#### Scenario: Local Jacobian contribution can be formed analytically

- **WHEN** a residual contribution depends on a geometry quantity
- **AND** assembly requests derivatives of that residual w.r.t. the geometry’s local parameters
- **THEN** the system can obtain those derivatives from analytical geometry/constraint derivative information
- **AND** the resulting local Jacobian block is defined with respect to the geometry’s local parameter ordering
