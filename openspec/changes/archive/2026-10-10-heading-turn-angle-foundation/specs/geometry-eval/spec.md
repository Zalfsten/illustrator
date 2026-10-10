# Spec Delta: geometry-eval

## ADDED Requirements

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
