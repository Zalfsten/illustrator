## MODIFIED Requirements

### Requirement: Geometry set includes curves with segment as proving kind

The system SHALL treat parameterized curves as the primary geometry kind for path-like primitives. A line segment SHALL be a curve in this sense and SHALL remain supported. Circular arcs and function curves SHALL also be supported curve kinds under this capability. Document SHALL store geometries as a sum of kinds (of which line segment, circular arc, function curve and label are required kinds in this slice). The curve capability SHALL be optional per kind: a label is a geometry without it.

#### Scenario: Segment remains an admitted curve

- **WHEN** a document under this capability contains geometry
- **THEN** line segments are valid curve geometries
- **AND** each segment exposes normalized position, tangent, and normal as required for curves

#### Scenario: Circular arc is an admitted curve

- **WHEN** a document contains a circular arc
- **THEN** that arc is a valid curve geometry under this capability
- **AND** it exposes normalized position, tangent, and normal as required for curves
- **AND** it exposes start and end endpoint expressions

#### Scenario: Non-segment curves are not yet mandatory

- **WHEN** a document is constructed with only line segments and circular arcs
- **THEN** the document satisfies this capability's geometry-set obligations
- **AND** absence of spline or full-circle kinds is not a violation

#### Scenario: Construction and result segments share the same kind API

- **WHEN** a document contains both a construction segment and a result segment
- **THEN** both are segment curve geometries under this capability
- **AND** they differ by role, not by a separate geometry kind

#### Scenario: Adding a kind does not touch capability callers

- **WHEN** a new curve kind with the line capability is admitted
- **THEN** tools and the Mesh handle it through capability queries
- **AND** no caller outside the kind catalog needs a kind-specific change for it

#### Scenario: Curve handling skips a label

- **WHEN** a document contains a segment and a label and a drag or a Mesh build runs over all geometries
- **THEN** the label is not treated as a curve
- **AND** the operation succeeds

### Requirement: Normalized chart is the canonical parameterization

The normalized chart SHALL be the canonical parameterization of the curve's defining formula with its natural domain affinely mapped to \([0,1]\), unless the kind states that its normalized chart is the arc-length fraction (as the function curve does). Equal steps in \(t\) NEED NOT correspond to equal arc length except for curves whose chart already has constant speed (for example a line segment) or is the arc-length fraction.

#### Scenario: Quarter of a segment

- **WHEN** a segment from (0, 0) to (4, 0) is sampled at normalized t = 0.25
- **THEN** the sample is (1, 0)

## ADDED Requirements

### Requirement: Function curve is an admitted curve kind

The system SHALL admit a **function curve** geometry kind: the graph of \(y = f(x)\) for \(x\) in a range \([x_0, x_1]\), \(x_0 < x_1\), drawn in a frame with origin location \(O\) and heading angle \(\theta\), scaled by \(s_u > 0\) along the frame's x axis and \(s_v > 0\) along its y axis. \(O\) and \(\theta\) SHALL be its continuous parameters. The expression \(f\) (expression-language, variable `x`), \(x_0\), \(x_1\), \(s_u\) and \(s_v\) SHALL be attributes.

#### Scenario: Sine over one period

- **WHEN** a function curve with \(f\) = `sin(x)`, range \([0, 2\pi]\), scales 1, origin (0, 0) and heading 0 is added
- **THEN** its start point is (0, 0) and its end point is \((2\pi, 0)\)
- **AND** its length expression evaluates to 7.6403955780 within \(10^{-9}\)

#### Scenario: The frame rotates the graph

- **WHEN** the same curve has heading \(\pi/2\)
- **THEN** its end point is \((0, 2\pi)\)

#### Scenario: Scales stretch the graph

- **WHEN** a function curve with \(f\) = `x^2`, range \([0, 1]\), scales \(s_u = 2\), \(s_v = 3\), origin (1, 1) and heading 0 is added
- **THEN** its end point is (3, 4)

### Requirement: Function curve normalized parameter is the arc-length fraction

For a function curve with total length \(L\), the point at normalized \(t\) SHALL be \(O + R(\theta)\,(s_u\,x,\; s_v\,f(x))\) for the \(x\) whose arc length from \(x_0\) along the scaled graph is \(t\,L\), within \(10^{-9} L\) in arc length. `Normalized` and `Uniform` SHALL yield the same point for the same numeric argument.

#### Scenario: Midpoint of a sine period

- **WHEN** the sine curve over \([0, 2\pi]\) with scales 1, origin (0, 0) and heading 0 is sampled at normalized \(t = 0.5\)
- **THEN** the sample is \((\pi, 0)\)

#### Scenario: Arc length up to a sample

- **WHEN** the curve \(f\) = `x^2` on \([0, 1]\) is sampled at normalized \(t = 0.3\)
- **THEN** the arc length of the graph from \(x_0\) to the sample is \(0.3\,L\) within \(10^{-9} L\)
- **AND** the sample at `Uniform(0.3)` is the same point

### Requirement: Function curve capabilities

A function curve SHALL expose the curve capability, the endpoints capability (start at \(x_0\), end at \(x_1\)), the anchor capability with origin \(O\), and the line capability with direction \(R(\theta)\,(1, 0)\), the unit vector of the frame's x axis. It SHALL NOT expose the circular capability.

#### Scenario: Origin and direction

- **WHEN** a function curve has origin (2, 1) and heading 0.3
- **THEN** its origin expression evaluates to (2, 1) and its direction expression to \((\cos 0.3, \sin 0.3)\)

#### Scenario: No radius

- **WHEN** a radius expression is requested for a function curve
- **THEN** none can be built because the curve has no circular capability

### Requirement: Function curves accept only smooth functions

Creating a function curve, or changing its expression or range, SHALL succeed only if \(f\), \(f'\) and \(f''\) are proven finite on the whole closed range \([x_0, x_1]\) (expression-language domain check). Otherwise it SHALL be rejected, and an existing curve SHALL keep its attributes. A range whose end has a vertical tangent is therefore rejected.

#### Scenario: Vertical tangent at a range end

- **WHEN** a function curve with \(f\) = `sqrt(x)` on \([0, 1]\) is created
- **THEN** creation is rejected

#### Scenario: Pole inside the range

- **WHEN** a function curve with \(f\) = `1/x` on \([-1, 1]\) is created
- **THEN** creation is rejected

#### Scenario: Smooth on a shifted range

- **WHEN** a function curve with \(f\) = `sqrt(x)` on \([1, 4]\) is created
- **THEN** creation succeeds

### Requirement: Function curve attributes move only the local curve

Changing a function curve's attributes SHALL keep \(O\) and \(\theta\) as they are and SHALL change the curve's points accordingly. A solve SHALL change only \(O\) and \(\theta\) of a function curve, never its attributes.

#### Scenario: Longer range, same origin

- **WHEN** the range of the sine curve with origin (0, 0) and heading 0 is changed from \([0, \pi]\) to \([0, 2\pi]\) on a curve without constraints
- **THEN** its origin and heading are unchanged
- **AND** its end point moves from \((\pi, 0)\) to \((2\pi, 0)\)
