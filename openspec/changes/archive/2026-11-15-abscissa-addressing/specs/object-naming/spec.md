## MODIFIED Requirements

### Requirement: Address resolution reports why it fails

Resolving an address SHALL fail, returning no expression and a distinct reason, for: a malformed address, an unknown name, a name that denotes a constraint, a curve parameter outside \([0, 1]\), a geometry that lacks the capability the suffix needs, and an abscissa where the function is not defined or its slope is not finite.

#### Scenario: Missing capability is reported

- **WHEN** `arc1.direction` is resolved
- **THEN** resolution fails because `arc1` has no line capability
- **AND** no expression is returned

#### Scenario: Unknown name and constraint name

- **WHEN** `nope.start` is resolved while no object is named `nope`
- **THEN** resolution fails because the name is unknown
- **AND** resolving `par1.start` for a constraint `par1` fails because the name does not denote a geometry

#### Scenario: Curve parameter out of range

- **WHEN** `seg1.at(1.5)` is resolved
- **THEN** resolution fails because the parameter is outside \([0, 1]\)

## ADDED Requirements

### Requirement: Abscissa addresses

The system SHALL resolve `at(x=<x>)`, `tangent(x=<x>)` and `normal(x=<x>)` on a geometry with the abscissa capability to the point, tangent and normal at `Abscissa(<x>)` (geometry-ir), for any finite number \(x\). An argument without a keyword SHALL remain a normalized curve parameter. Abscissa expressions SHALL print in this form, so that resolving the printed address yields an equal expression.

#### Scenario: Tangent at x

- **WHEN** `fn1.tangent(x=1)` is resolved for a function curve `fn1`
- **THEN** the result is a free-vector expression that evaluates to the tangent at `Abscissa(1)`

#### Scenario: No keyword means normalized

- **WHEN** `fn1.at(0.4)` is resolved
- **THEN** the result is the point at normalized parameter 0.4

#### Scenario: Round trip of an abscissa address

- **WHEN** the point expression at `Abscissa(-2.5)` of `fn1` is printed
- **THEN** the result is `fn1.at(x=-2.5)`
- **AND** resolving it yields an equal expression

#### Scenario: Abscissa on a segment

- **WHEN** `seg1.at(x=1)` is resolved
- **THEN** resolution fails because `seg1` has no abscissa capability

#### Scenario: Invalid abscissa

- **WHEN** `fn1.at(x=0)` is resolved for `fn1` with \(f\) = `sqrt(x)` on \([1, 4]\)
- **THEN** resolution fails because the slope at \(x = 0\) is not finite
