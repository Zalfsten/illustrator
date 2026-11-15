## MODIFIED Requirements

### Requirement: Curve parameter kinds Normalized and Uniform

The system SHALL address a location on a curve using a curve parameter: `Normalized` (\(t \in [0,1]\) in the canonical chart), `Uniform` (\(u \in [0,1]\), the arc-length fraction: equal increments of \(u\) give equal arc length) or, on a curve with the abscissa capability, `Abscissa` (see *Abscissa curve parameter*). A physical arc-length domain \(s \in [0, L]\) SHALL NOT be required. For a line segment, `Normalized` and `Uniform` SHALL yield the same point for the same numeric argument.

#### Scenario: Uniform mid-parameter is arc-length midpoint on a segment

- **WHEN** a line segment is sampled with `Uniform` parameter \(u = 0.5\)
- **THEN** the resulting point coincides with the sample at `Normalized` \(t = 0.5\)
- **AND** that point is the arc-length midpoint of the segment

#### Scenario: Uniform is distinct from Normalized in general

- **WHEN** a future non-constant-speed curve (for example a cubic Bézier) is admitted under this capability
- **THEN** `Normalized(0.5)` and `Uniform(0.5)` NEED NOT evaluate to the same location
- **AND** both parameter kinds remain valid addresses on that curve

### Requirement: Capabilities are queryable by geometry identity

For a geometry identity resolvable in Document, the system SHALL report which capabilities its kind has: curve, endpoints, line, circular, anchor, abscissa. The capability set of a kind SHALL be fixed for that kind and SHALL be declared together with the kind's capability factories, so the reported set cannot diverge from the factories the kind provides. Querying an identity that is not resolvable SHALL report that the geometry does not exist.

#### Scenario: Segment reports its capabilities

- **WHEN** the capabilities of a line segment's identity are queried
- **THEN** curve, endpoints and line are reported
- **AND** circular and anchor are not reported

#### Scenario: Arc reports its capabilities

- **WHEN** the capabilities of a circular arc's identity are queried
- **THEN** curve, endpoints and circular are reported
- **AND** line and anchor are not reported

#### Scenario: Label reports its capabilities

- **WHEN** the capabilities of a label's identity are queried
- **THEN** anchor and line are reported
- **AND** curve, endpoints and circular are not reported

#### Scenario: Reported set matches the factories

- **WHEN** any admitted kind reports a capability
- **THEN** the capability-scoped expressions of that capability can be built for geometries of that kind
- **AND** when it does not report a capability, those expressions cannot be built for it

### Requirement: Function curve capabilities

A function curve SHALL expose the curve capability, the endpoints capability (start at \(x_0\), end at \(x_1\)), the anchor capability with origin \(O\), the line capability with direction \(R(\theta)\,(1, 0)\), the unit vector of the frame's x axis, and the abscissa capability. It SHALL NOT expose the circular capability.

#### Scenario: Origin and direction

- **WHEN** a function curve has origin (2, 1) and heading 0.3
- **THEN** its origin expression evaluates to (2, 1) and its direction expression to \((\cos 0.3, \sin 0.3)\)

#### Scenario: No radius

- **WHEN** a radius expression is requested for a function curve
- **THEN** none can be built because the curve has no circular capability

## ADDED Requirements

### Requirement: Abscissa curve parameter

On a function curve, the curve parameter `Abscissa(x)` SHALL address the point \(O + R(\theta)\,(s_u\,x,\; s_v\,f(x))\). The tangent at `Abscissa(x)` SHALL be the derivative with respect to \(x\), \(R(\theta)\,(s_u,\; s_v\,f'(x))\); the normal SHALL be that tangent turned a quarter to the left. Expressions with an abscissa parameter SHALL be point and free-vector expressions like any other, evaluated from the State.

#### Scenario: Point and tangent at x = 1

- **WHEN** the sine curve over \([0, 2\pi]\) with scales 1, origin (0, 0) and heading 0 is addressed at `Abscissa(1)`
- **THEN** the point is \((1, \sin 1)\) and the tangent is \((1, \cos 1)\)

#### Scenario: Same direction as the normalized tangent

- **WHEN** the tangent at `Abscissa(1)` and the tangent at the normalized parameter of the same point are evaluated
- **THEN** they point in the same direction and their lengths differ

#### Scenario: Abscissa stays at its x when the range changes

- **WHEN** the range of that curve changes from \([0, 2\pi]\) to \([0, \pi]\)
- **THEN** the point at `Abscissa(1)` is still \((1, \sin 1)\)

### Requirement: Abscissa parameters need the abscissa capability

An expression with an `Abscissa` parameter SHALL be buildable by geometry identity only for a geometry that has the abscissa capability. For any other geometry, building it SHALL fail with a missing capability.

#### Scenario: No abscissa on a segment

- **WHEN** an on-curve expression with `Abscissa(0.5)` is requested for a segment
- **THEN** none can be built because the segment has no abscissa capability

### Requirement: Abscissa continues beyond the range

An abscissa outside the function curve's range \([x_0, x_1]\) SHALL address the point of the graph's continuation at that \(x\), with the same formulas. A change of the range SHALL NOT change the point or tangent at an abscissa.

#### Scenario: Point beyond the drawn range

- **WHEN** the sine curve over \([0, 2\pi]\) with scales 1, origin (0, 0) and heading 0 is addressed at `Abscissa(8)`
- **THEN** the point is \((8, \sin 8)\)

### Requirement: Abscissa requires a defined function with finite slope

An abscissa SHALL be valid only where evaluating \(f\) reports no domain error (expression-language), so \(f\), \(f'\) and \(f''\) are finite there. Building an expression at an invalid abscissa SHALL fail; at a point with a vertical tangent the normalized parameter has to be used instead.

#### Scenario: Vertical tangent

- **WHEN** an expression at `Abscissa(0)` is requested on a curve with \(f\) = `sqrt(x)` on \([1, 4]\)
- **THEN** none can be built because the slope at \(x = 0\) is not finite

#### Scenario: Undefined function

- **WHEN** an expression at `Abscissa(-1)` is requested on a curve with \(f\) = `log(x)` on \([1, 2]\)
- **THEN** none can be built because \(f\) is not defined at \(x = -1\)
