# Spec Delta

## MODIFIED Requirements

### Requirement: Circular arc is an admitted curve kind with endpoints

The system SHALL admit a **circular arc** geometry kind: an open arc of a circle in the plane. A circular arc SHALL be a curve under this capability and SHALL expose the endpoints capability (geometric start and end join points). Document SHALL store circular arcs in the same closed sum of geometry kinds as line segments. Chart continuous parameters SHALL be: center location, radius length DOF, start **heading angle**, and **turn angle** sweep. In committed document State the radius SHALL be non-negative. A solver MAY hold a negative radius in intermediate iterates (the parameter is an unconstrained real during a solve); whenever a State is committed, an arc with radius \(R < 0\) SHALL be rewritten to radius \(-R\) and start heading \(\theta_{\mathrm{start}} + \pi\) with unchanged center and sweep, which describes the same curve. Evaluating the curve at normalized \(t \in [0,1]\) SHALL yield the point at polar angle \(\theta(t) = \theta_{\mathrm{start}} + t\,\Delta\theta\) on that circle (world +x as the heading null ray; increasing heading is counterclockwise in the y-up world frame). Start and end point expressions SHALL match on-curve samples at \(t = 0\) and \(t = 1\). Total arc length SHALL equal \(|R|\,|\Delta\theta|\). The radius expression of an arc SHALL evaluate to \(|R|\). For a circular arc of constant radius, `Normalized` and `Uniform` curve parameters SHALL yield the same point for the same numeric argument.

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
