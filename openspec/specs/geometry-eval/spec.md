# geometry-eval

## Purpose

Defines that geometries (and derived constraint contributions) expose evaluable quantities and analytical derivatives with respect to their local parameters for use in residual and Jacobian assembly.

## Requirements

### Requirement: Expressions evaluate geometry quantities from State

The production evaluate/solve path SHALL obtain geometric quantities needed by residuals and assembly (endpoint locations, on-curve samples, tangent and normal vectors, segment direction, and other chart-defined outputs) by evaluating **opaque point expressions** and **opaque vector expressions** against DocumentStructure (or an equivalent geometry-resolution port) and an explicit State \(p\). Those expressions SHALL map the involved geometries' local parameters (via connectivity into \(p\)) to points or free vectors by applying the **kind-owned** chart formulas of the named geometries. Point-valued outputs SHALL be distinguished from free-vector-valued outputs. Constraint and assembly code SHALL NOT match on private formula kinds.

#### Scenario: Endpoint expression depends on local parameters

- **WHEN** a point expression for a segment endpoint is evaluated at a given State
- **THEN** the result is computed from that segment’s local parameters in \(p\)
- **AND** those parameters are the ones referenced by the segment’s connectivity

#### Scenario: On-curve sample depends on local parameters

- **WHEN** an on-curve point expression on a line segment is evaluated at normalized \(t = 0.5\)
- **THEN** the result is computed from that segment’s local parameters in \(p\)
- **AND** the result is a point, not a free vector

### Requirement: Expressions expose derivatives w.r.t. local parameters

Point and vector expressions that appear in structural constraints SHALL provide analytical derivative information with respect to the local parameters they depend on (or equivalent information sufficient to form local Jacobian contributions). For a point expression, that information SHALL be a \(2 \times n\) Jacobian of the map from the expression's **formula-local** parameters to \((x,y)\), where \(n\) is the number of parameters the formula depends on (which MAY be a sparse subset of the owning geometry's full parameter block — e.g. a segment endpoint depends on two of four segment parameters). For a free-vector expression, that information SHALL be a \(2 \times n\) Jacobian of the map from formula-local parameters to \((v_x,v_y)\) with the same layout and column-order rules. Column order SHALL match the expression's reported parameter-index list. The production path for Jacobian assembly SHALL use those analytical derivatives. Finite-difference approximation SHALL NOT be used as a fallback or as the production Jacobian path. Formula-local Jacobians are small (chart DOFs); implementations MAY represent them as compact stack-friendly structures before scattering into constraint-local and global Jacobians. Structural constraints that combine two vector expressions (Parallel, Orthogonal) SHALL form residuals and local Jacobians by composing those vector evaluations and vector Jacobians — they SHALL NOT bypass the opaque vector-expression API with geometry-kind-specific direction tables.

#### Scenario: Local Jacobian contribution can be formed analytically

- **WHEN** a residual contribution depends on a quantity obtained from a point or vector expression
- **AND** assembly requests derivatives of that residual w.r.t. the involved geometries’ local parameters
- **THEN** the system can obtain those derivatives from analytical expression/constraint derivative information
- **AND** the resulting local Jacobian block is defined with respect to the involved local parameter ordering

#### Scenario: Endpoint point Jacobian is formula-sparse

- **WHEN** a segment start- or end-point expression reports its parameter indices and point Jacobian
- **THEN** both refer to the same two endpoint coordinate parameters
- **AND** the point Jacobian width is two, not the full four-parameter segment block
- **AND** the \(2 \times 2\) map is the identity on those coordinates

### Requirement: Constraints contribute residuals and local Jacobians

Each structural constraint SHALL define a residual contribution \(r_N\) as a function of the local parameters of the geometries it involves (via their quantities), and SHALL define the corresponding local Jacobian \(J_N = \partial r_N / \partial p_N\) (or blocks thereof) with respect to those local parameters.

#### Scenario: Orthogonal constraint yields residual and local derivatives

- **WHEN** an Orthogonal constraint between two free-vector expressions (v1: segment directions) is assembled at a given state
- **THEN** it produces a residual contribution of the specified arity
- **AND** it produces local partial derivatives of that residual w.r.t. the involved local parameters
- **AND** those derivatives are obtained by composing the vector expressions' analytical Jacobians

### Requirement: Curve frame expressions evaluate with derivatives

For curve geometries, on-curve point expressions and tangent/normal vector expressions at a stated curve parameter SHALL evaluate from local parameters in the supplied State using the length and orientation semantics of the curve contract (tangent = chart derivative; normal = leftward planar rotate of tangent with equal length). They SHALL provide analytical derivatives with respect to the involved local parameters sufficient to form Jacobian contributions when those expressions appear in residuals. The production Jacobian path for these quantities SHALL be analytical; finite-difference approximation SHALL NOT be used as a substitute for those derivatives.

#### Scenario: Segment tangent expression derivatives feed assembly

- **WHEN** a residual depends on a vector expression for a segment's tangent at a normalized parameter
- **AND** assembly requests derivatives w.r.t. that segment's local parameters
- **THEN** analytical derivative information for that vector expression is available
- **AND** the local Jacobian block uses the segment's local parameter ordering

#### Scenario: Segment normal matches rotate of tangent

- **WHEN** a line segment's tangent and normal vector expressions are evaluated at the same normalized \(t\) and State
- **THEN** the normal equals the leftward rotate of the tangent
- **AND** analytical derivatives of the normal expression are consistent with differentiating that rotate of the tangent

### Requirement: On-curve and Uniform parameters are evaluable

Point expressions that sample a curve at `Normalized` or `Uniform` parameters, and vector expressions for tangents and normals at those parameters, SHALL be evaluable against a state vector given DocumentStructure (or an equivalent geometry store that resolves geometry identities). Evaluation SHALL obtain the named geometry by identity resolution from that structure, then apply the curve (or endpoint) capability of that geometry. For line segments, evaluation at `Uniform(u)` SHALL match evaluation at `Normalized(t)` for \(t = u\), including derivatives w.r.t. the segment's local parameters. Implementations SHALL document how `Uniform` will be obtained for future non-constant-speed curves (closed form or inversion); this capability requires a correct segment path and a defined extension point, not a general numerical arc-length solver.

#### Scenario: Segment OnCurve Uniform matches Normalized

- **WHEN** an on-curve point expression on a segment uses `Uniform(0.25)`
- **AND** the same segment is evaluated at `Normalized(0.25)`
- **THEN** both evaluations yield the same point from the same state
- **AND** both yield matching analytical derivatives w.r.t. the segment's local parameters

#### Scenario: Assembly evaluates expressions via identity resolution

- **WHEN** residual or Jacobian assembly evaluates a constraint that names geometries by identity
- **THEN** each named geometry is obtained from DocumentStructure by identity resolution
- **AND** assembly SHALL NOT scan the full geometry list once per named identity to locate that geometry
