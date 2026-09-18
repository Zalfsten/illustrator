# Spec Delta

## Purpose

Defines Model E: parameterized geometries expose derived point expressions, and relationships compile to equation objects that feed the solver — independent of UI sugar and of any concrete numerical algorithm.

## ADDED Requirements

### Requirement: Geometries are first-class parameterized objects

The system SHALL represent each geometric primitive as a first-class object with its own continuous parameters. A line segment SHALL be a geometry in this sense. The core model SHALL NOT require geometries to own references to independent point entities as their only representation of endpoints.

#### Scenario: Segment exists without shared point entities in the core

- **WHEN** a line segment is added to the document
- **THEN** the segment is stored as a parameterized geometry
- **AND** its endpoint locations are obtainable only through derived point expressions, not as mandatory separate core point entities

### Requirement: Endpoints yield point expressions

For every non-closed geometry that has endpoints, the system SHALL provide an `Endpoints` mapping that returns point expressions for those endpoints. A point expression SHALL evaluate to a location as a function of the geometry parameters and SHALL NOT be a snapshot of coordinates detached from those parameters.

#### Scenario: Endpoint expression tracks segment parameters

- **WHEN** a segment's parameters change
- **AND** an endpoint point expression of that segment is evaluated
- **THEN** the resulting location matches the endpoint implied by the new parameters

### Requirement: Relationships are equation factories

The system SHALL represent geometric relationships as operations that produce equation objects (residual contributions) suitable for inclusion in a global equation system. Assembling the document's structural relationships SHALL yield a combined residual vector over the collected geometry parameters.

#### Scenario: Structural equations assemble from relationships

- **WHEN** a document contains geometries and structural relationships
- **AND** the equation system is assembled
- **THEN** each structural relationship contributes one or more residual components
- **AND** the residual vector is a function of the geometry parameters

### Requirement: UI shared points desugar to congruence

Logical points shared in the user interface (for example a corner joining two segments) SHALL be represented in the core solely by congruence equations between the corresponding endpoint point expressions. The core SHALL treat UI shared-point identity as sugar for those equations, not as a second competing source of truth for location.

#### Scenario: Shared UI corner becomes congruence

- **WHEN** the user connects two segment ends as one shared UI point
- **THEN** the core model contains a congruence relationship between those endpoint expressions
- **AND** no separate authoritative core point entity is required to define that shared location

### Requirement: v1 geometry set is segments only

For this capability's v1 scope, the system SHALL support line segments as the only geometry kind. Circles, arcs, splines, function graphs, and other primitives are out of scope until specified elsewhere.

#### Scenario: Only segments are admitted in v1

- **WHEN** a v1 document is constructed under this capability
- **THEN** every geometry in the document is a line segment
