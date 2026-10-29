# Spec Delta

## ADDED Requirements

### Requirement: Components of the constraint graph are enumerable

The system SHALL split a drawing into components: two geometries belong to the same component if a chain of structural constraints connects them, where each constraint connects all geometries it references. Each constraint belongs to the component of its geometries; a geometry without constraints is a component of its own. Components SHALL depend only on the constraints in Document, not on evaluated values.

#### Scenario: Joined segments form one component

- **WHEN** segments `seg1` and `seg2` are related by a Congruence and `seg3` has no constraints
- **THEN** `seg1`, `seg2` and the Congruence form one component
- **AND** `seg3` forms a component of its own

#### Scenario: Removing the only link splits a component

- **WHEN** a VectorsParallel is the only constraint between `seg1` and `seg2`
- **AND** that VectorsParallel is removed
- **THEN** `seg1` and `seg2` are in separate components

#### Scenario: Coincident unconstrained geometries stay separate

- **WHEN** two segments share an end location but no constraint relates them
- **THEN** they are in separate components

### Requirement: A component is consistent or conflicted

A component SHALL be conflicted if the structural residual of its constraints, evaluated at the committed values, exceeds solve tolerance; otherwise it SHALL be consistent. The classification SHALL be derived from structure and committed values only, so restoring a drawing restores its classification.

#### Scenario: Unsatisfied component is conflicted

- **WHEN** a drawing's committed values leave a FixPoint in one component away from its target beyond solve tolerance
- **THEN** that component is conflicted
- **AND** components whose constraints are satisfied are consistent

## MODIFIED Requirements

### Requirement: Structural constraints are hard invariants

Congruence, VectorsParallel, VectorsOrthogonal, FixPoint, and EqualLength SHALL be structural constraints. A solve SHALL cover a set of components and SHALL NOT change the values of other components. After every successful solve that commits a new document state, all structural residuals of the components it covers SHALL be satisfied within solve tolerance. The system SHALL NOT commit values for a component from a solve that leaves its structural residuals unsatisfied beyond that tolerance.

#### Scenario: Successful commit preserves structural satisfaction

- **WHEN** a solve succeeds and its result is committed as the document state
- **THEN** every structural constraint in the components the solve covers is satisfied within solve tolerance

#### Scenario: Other components keep their values

- **WHEN** a solve covers the component of `seg1`
- **AND** `seg2` is in another component
- **THEN** the values of `seg2` are unchanged after the solve, whether it succeeds or fails
