# Spec Delta

## MODIFIED Requirements

### Requirement: Addresses on named geometries

The system SHALL resolve an address of the form `<name>.<suffix>` on a named geometry to the corresponding opaque expression. It SHALL support at least: `start`, `end`, `center`, `origin`, `at(<t>)` (points); `direction`, `tangent(<t>)`, `normal(<t>)` (free vectors); `length`, `radius` (lengths). `<t>` is a normalized curve parameter in \([0, 1]\). Points and free vectors SHALL remain distinct result kinds.

#### Scenario: Endpoint address

- **WHEN** `seg1.end` is resolved
- **THEN** the result is a point expression that evaluates to the end point of `seg1`

#### Scenario: Vector address is not a point

- **WHEN** `seg1.direction` is resolved
- **THEN** the result is a free-vector expression, not a point expression

#### Scenario: Label origin and direction

- **WHEN** `lbl1.origin` and `lbl1.direction` are resolved for a label `lbl1`
- **THEN** the first is a point expression that evaluates to the label's origin
- **AND** the second is a free-vector expression that evaluates to the label's unit direction

#### Scenario: Label has no endpoints

- **WHEN** `lbl1.start` is resolved for a label `lbl1`
- **THEN** resolution fails because `lbl1` has no endpoints capability

#### Scenario: Segment has no origin

- **WHEN** `seg1.origin` is resolved
- **THEN** resolution fails because `seg1` has no anchor capability
