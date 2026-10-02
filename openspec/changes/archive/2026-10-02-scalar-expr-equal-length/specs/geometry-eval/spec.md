# Spec Delta: geometry-eval

## ADDED Requirements

### Requirement: Scalar expressions evaluate with analytical derivatives

Opaque scalar expressions that appear in structural constraints SHALL evaluate to a real number from DocumentStructure (or an equivalent geometry-resolution port) and State \(p\), applying kind-owned formulas of the named geometries. They SHALL provide analytical derivative information with respect to the formula-local parameters they depend on as a \(1 \times n\) Jacobian (row \(\partial s / \partial p_i\)), where \(n\) is the number of parameters the formula depends on and column order matches the expression's reported parameter-index list. The production Jacobian path SHALL use those analytical derivatives; finite-difference SHALL NOT be the production path for scalar-expression derivatives.

#### Scenario: Segment length Jacobian feeds EqualLength

- **WHEN** EqualLength depends on a segment length scalar expression
- **AND** assembly requests derivatives of that residual w.r.t. the segment's local parameters
- **THEN** analytical \(1 \times n\) derivative information for that scalar expression is available
- **AND** the local Jacobian block uses the expression's reported parameter ordering

#### Scenario: Segment length value is endpoint Euclidean distance

- **WHEN** a segment length scalar expression is evaluated at a State where the segment endpoints are known
- **THEN** the scalar equals \(\sqrt{(x_2-x_1)^2+(y_2-y_1)^2}\) from those endpoint coordinates in \(p\)
- **AND** the expression's parameter indices cover the parameters that length depends on
