# Spec Delta: geometry-ir

## ADDED Requirements

### Requirement: Heading angles and turn angles are distinct

The system SHALL distinguish **heading angles** (absolute orientations measured from a fixed planar null ray / angle origin) from **turn angles** (free angular displacements, independent of that origin). Implementations SHALL NOT overload a single angle quantity type for both roles. Algebra SHALL respect the distinction: the difference of two heading angles is a turn angle; adding a turn angle to a heading angle yields a heading angle; adding two turn angles yields a turn angle; adding two heading angles is not a meaningful operation. Both kinds of angle SHALL be expressible as continuous State degrees of freedom (radians) for chart packing when a geometry kind needs them. Wrapping or principal-value reduction at the type level is NOT required by this capability.

#### Scenario: Difference of headings is a turn

- **WHEN** two heading angles are subtracted in the domain model
- **THEN** the result is a turn angle
- **AND** it is not typed as a heading angle

#### Scenario: Heading plus turn is a heading

- **WHEN** a heading angle is combined with a turn angle by addition
- **THEN** the result is a heading angle

#### Scenario: Two headings cannot be added as a domain operation

- **WHEN** a caller attempts to treat the sum of two heading angles as a defined domain operation
- **THEN** the model does not admit that combination as a heading or turn result

### Requirement: Opaque heading-angle and turn-angle expressions

The system SHALL provide opaque **heading-angle expressions** and opaque **turn-angle expressions** that map State parameters (via named geometries and kind-owned formulas) to a heading angle or turn angle respectively. These families SHALL be distinct from each other, from length scalar expressions, from point expressions, and from free-vector expressions. Constraints and Interact SHALL NOT match on private formula kinds; they SHALL only evaluate, report parameter connectivity, and obtain analytical derivatives through the opaque expression API for each family. Kind-owned factories that construct such expressions MAY be introduced when a geometry kind needs them; this requirement does not itself admit a new geometry kind.

#### Scenario: Families stay separate

- **WHEN** a constraint or Interact path consumes a heading-angle expression
- **THEN** it does not treat that handle as a turn-angle expression or a length scalar expression
- **AND** evaluation does not require matching on private formula variants

#### Scenario: Evaluation names geometries by identity

- **WHEN** a heading-angle or turn-angle expression is evaluated at a State
- **THEN** it resolves named geometries by stable identity as needed
- **AND** the result is not a numeric snapshot detached from those parameters

## MODIFIED Requirements

### Requirement: Opaque scalar expressions

The system SHALL provide opaque **scalar expressions** that map State parameters (via named geometries and kind-owned formulas) to a single real-valued quantity in a **typed scalar family**. Typed families in this capability include at least **length** (physical length), **heading angle**, and **turn angle**. Scalar expressions SHALL evaluate as functions of the involved geometry parameters and SHALL NOT be numeric snapshots detached from those parameters. Constraints and Interact SHALL NOT match on private scalar formula kinds; they SHALL only evaluate, report parameter connectivity, and obtain analytical derivatives through the opaque scalar-expression API of the appropriate family. Scalar expressions SHALL be distinct from point expressions and free-vector expressions. An untyped catch-all scalar expression that erases length versus heading versus turn SHALL NOT be the production API for constraints.

#### Scenario: EqualLength consumes scalar expressions without formula matching

- **WHEN** EqualLength is applied to two length scalar expressions
- **THEN** assembly evaluates those expressions and forms the residual without matching on private scalar formula kinds
- **AND** each expression names geometries by stable identity as needed for evaluation

#### Scenario: Length is not interchangeable with angle families

- **WHEN** a length scalar expression is constructed for a curve
- **THEN** it is not admissible wherever a heading-angle or turn-angle expression is required
- **AND** conversely a heading-angle or turn-angle expression is not admissible wherever a length scalar expression is required
