# Spec Delta

## MODIFIED Requirements

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
