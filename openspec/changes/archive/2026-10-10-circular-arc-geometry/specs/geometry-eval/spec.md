# Spec Delta: geometry-eval

## ADDED Requirements

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
