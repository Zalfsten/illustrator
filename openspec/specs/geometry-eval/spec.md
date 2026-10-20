# geometry-eval

## Purpose

Defines that geometries (and derived constraint contributions) expose evaluable quantities and analytical derivatives with respect to their local parameters for use in residual and Jacobian assembly.

## Requirements

### Requirement: Expressions evaluate geometry quantities from State

The production evaluate/solve path SHALL obtain geometric quantities needed by residuals and assembly (endpoint locations, on-curve samples, tangent and normal vectors, segment direction, and other chart-defined outputs) by evaluating **opaque point expressions** and **opaque vector expressions** against Document (or an equivalent geometry-resolution port) and an explicit State \(p\). Those expressions SHALL map the involved geometries' local parameters (via connectivity into \(p\)) to points or free vectors by applying the **kind-owned** chart formulas of the named geometries. Point-valued outputs SHALL be distinguished from free-vector-valued outputs. Constraint and assembly code SHALL NOT match on private formula kinds.

#### Scenario: Endpoint expression depends on local parameters

- **WHEN** a point expression for a segment endpoint is evaluated at a given State
- **THEN** the result is computed from that segment’s local parameters in \(p\)
- **AND** those parameters are the ones referenced by the segment’s connectivity

#### Scenario: On-curve sample depends on local parameters

- **WHEN** an on-curve point expression on a line segment is evaluated at normalized \(t = 0.5\)
- **THEN** the result is computed from that segment’s local parameters in \(p\)
- **AND** the result is a point, not a free vector

### Requirement: Expressions expose derivatives w.r.t. local parameters

Point and vector expressions that appear in structural constraints SHALL provide analytical derivative information with respect to the local parameters they depend on (or equivalent information sufficient to form local Jacobian contributions). For a point expression, that information SHALL be a \(2 \times n\) Jacobian of the map from the expression's **formula-local** parameters to \((x,y)\), where \(n\) is the number of parameters the formula depends on (which MAY be a sparse subset of the owning geometry's full parameter block — e.g. a segment endpoint depends on two of four segment parameters). For a free-vector expression, that information SHALL be a \(2 \times n\) Jacobian of the map from formula-local parameters to \((v_x,v_y)\) with the same layout and column-order rules. Column order SHALL match the expression's reported parameter-index list. Expressions SHALL be able to report that formula-local DOF count \(n\) without requiring callers to allocate the full parameter-index list when only the count is needed.

The production path SHALL write those analytical derivatives into the contribution Jacobian matrix buffer (formula-local columns), not via finite-difference. Writes MAY apply a contribution scale (for example \(+1\) / \(-1\) when a residual is a difference of two expressions). When a scalar residual depends on a free vector through a fixed contraction \(g^\top (\partial v/\partial p)\) (for example VectorsParallel or VectorsOrthogonal), the production path MAY derive that contracted \(1 \times n\) row from the vector expression’s \(2 \times n\) Jacobian (including via bounded stack scratch) rather than requiring each geometry kind to hand-write the contraction. Finite-difference approximation SHALL NOT be used as a fallback or as the production Jacobian path. Structural constraints that combine two vector expressions (VectorsParallel, VectorsOrthogonal) SHALL form residuals and local Jacobians by composing those vector evaluations and vector Jacobians — they SHALL NOT bypass the opaque vector-expression API with geometry-kind-specific direction tables.

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

- **WHEN** a VectorsOrthogonal constraint between two free-vector expressions (v1: segment directions) is assembled at a given state
- **THEN** it produces a residual contribution of the specified arity
- **AND** it produces local partial derivatives of that residual w.r.t. the involved local parameters
- **AND** those derivatives are obtained by composing the vector expressions' analytical Jacobians

### Requirement: Scalar expressions evaluate with analytical derivatives

Opaque scalar expressions that appear in structural constraints SHALL evaluate to a real number from Document (or an equivalent geometry-resolution port) and State \(p\), applying kind-owned formulas of the named geometries. They SHALL provide analytical derivative information with respect to the formula-local parameters they depend on as a \(1 \times n\) Jacobian (row \(\partial s / \partial p_i\)), where \(n\) is the number of parameters the formula depends on and column order matches the expression's reported parameter-index list. The production Jacobian path SHALL use those analytical derivatives; finite-difference SHALL NOT be the production path for scalar-expression derivatives.

#### Scenario: Segment length Jacobian feeds EqualLength

- **WHEN** EqualLength depends on a segment length scalar expression
- **AND** assembly requests derivatives of that residual w.r.t. the segment's local parameters
- **THEN** analytical \(1 \times n\) derivative information for that scalar expression is available
- **AND** the local Jacobian block uses the expression's reported parameter ordering

#### Scenario: Segment length value is endpoint Euclidean distance

- **WHEN** a segment length scalar expression is evaluated at a State where the segment endpoints are known
- **THEN** the scalar equals \(\sqrt{(x_2-x_1)^2+(y_2-y_1)^2}\) from those endpoint coordinates in \(p\)
- **AND** the expression's parameter indices cover the parameters that length depends on

### Requirement: Heading-angle and turn-angle expressions expose analytical 1×n Jacobians

Opaque heading-angle expressions and opaque turn-angle expressions that appear in structural constraints or other production evaluate paths SHALL evaluate to a heading angle or turn angle (radians) from Document (or an equivalent geometry-resolution port) and State \(p\), applying kind-owned formulas of the named geometries. They SHALL provide analytical derivative information with respect to the formula-local parameters they depend on as a \(1 \times n\) Jacobian (row \(\partial \theta / \partial p_i\)), where \(n\) is the number of parameters the formula depends on and column order matches the expression's reported parameter-index list. The production Jacobian path SHALL use those analytical derivatives; finite-difference SHALL NOT be the production path for those derivatives.

#### Scenario: Heading-angle Jacobian is analytical when a formula exists

- **WHEN** a heading-angle expression with a registered kind formula is evaluated for residual assembly
- **THEN** analytical \(1 \times n\) derivative information for that expression is available
- **AND** finite-difference is not used as the production derivative for that expression

#### Scenario: Turn-angle Jacobian is analytical when a formula exists

- **WHEN** a turn-angle expression with a registered kind formula is evaluated for residual assembly
- **THEN** analytical \(1 \times n\) derivative information for that expression is available
- **AND** column order matches the expression's reported parameter-index list

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

### Requirement: Circular arc expressions evaluate with analytical derivatives

For circular-arc geometries, on-curve point expressions, tangent/normal vector expressions, endpoint point expressions, length scalar expressions, start-heading expressions, and sweep turn-angle expressions SHALL evaluate from Document (or an equivalent geometry-resolution port) and State \(p\) using the arc chart (center, radius, start heading, sweep). They SHALL provide analytical derivatives with respect to formula-local parameters sufficient for Jacobian assembly. The production Jacobian path SHALL be analytical; finite-difference SHALL NOT be used as a substitute for those derivatives. Tangent at normalized \(t\) SHALL be the chart derivative \(\gamma'(t)\) (parametric velocity along increasing \(t\)); normal SHALL be the leftward planar rotate of that tangent with equal length.

#### Scenario: Arc on-curve Jacobian is analytical

- **WHEN** a residual depends on an on-curve point expression on a circular arc
- **AND** assembly requests derivatives w.r.t. that arc's local parameters
- **THEN** analytical derivative information for that point expression is available
- **AND** finite-difference is not the production path for that expression

#### Scenario: Arc tangent follows polar frame

- **WHEN** a circular arc's tangent vector expression is evaluated at a normalized \(t\)
- **THEN** the tangent is perpendicular to the radius vector from center to the on-curve point
- **AND** its orientation agrees with increasing \(t\) (sign of sweep)

#### Scenario: Arc heading and sweep Jacobians are analytical

- **WHEN** a residual depends on the arc's start-heading or sweep expression
- **THEN** analytical \(1 \times n\) derivative information for that expression is available
- **AND** column order matches the expression's reported parameter-index list

### Requirement: On-curve and Uniform parameters are evaluable

Point expressions that sample a curve at `Normalized` or `Uniform` parameters, and vector expressions for tangents and normals at those parameters, SHALL be evaluable against a state vector given Document (or an equivalent geometry store that resolves geometry identities). Evaluation SHALL obtain the named geometry by identity resolution from that structure, then apply the curve (or endpoint) capability of that geometry. For line segments, evaluation at `Uniform(u)` SHALL match evaluation at `Normalized(t)` for \(t = u\), including derivatives w.r.t. the segment's local parameters. Implementations SHALL document how `Uniform` will be obtained for future non-constant-speed curves (closed form or inversion); this capability requires a correct segment path and a defined extension point, not a general numerical arc-length solver.

#### Scenario: Segment OnCurve Uniform matches Normalized

- **WHEN** an on-curve point expression on a segment uses `Uniform(0.25)`
- **AND** the same segment is evaluated at `Normalized(0.25)`
- **THEN** both evaluations yield the same point from the same state
- **AND** both yield matching analytical derivatives w.r.t. the segment's local parameters

#### Scenario: Assembly evaluates expressions via identity resolution

- **WHEN** residual or Jacobian assembly evaluates a constraint that names geometries by identity
- **THEN** each named geometry is obtained from Document by identity resolution
- **AND** assembly SHALL NOT scan the full geometry list once per named identity to locate that geometry
