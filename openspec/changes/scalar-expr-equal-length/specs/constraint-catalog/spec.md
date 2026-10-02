# Spec Delta: constraint-catalog

## ADDED Requirements

### Requirement: EqualLength constrains two scalar expressions

The system SHALL provide an EqualLength relationship between two **opaque scalar expressions**. EqualLength SHALL contribute exactly one residual component that is zero if and only if the two evaluated scalars are equal (within solve tolerance). In the v1 catalog, tools SHALL typically supply curve length scalar expressions; the residual SHALL NOT hard-wire geometry kind outside those expressions.

#### Scenario: Equal segment lengths stay equal under edit

- **WHEN** EqualLength is applied to the length scalar expressions of two segments
- **AND** the document is edited through a successful solve
- **THEN** those lengths remain equal within solve tolerance

## MODIFIED Requirements

### Requirement: Structural constraints are hard invariants

Congruence, Parallel, Orthogonal, FixPoint, and EqualLength SHALL be structural constraints. After every successful solve that commits a new document state, all structural residuals SHALL be satisfied within solve tolerance. The system SHALL NOT commit a document state from a solve that leaves structural residuals unsatisfied beyond that tolerance.

#### Scenario: Successful commit preserves structural satisfaction

- **WHEN** a solve succeeds and its result is committed as the document state
- **THEN** every structural constraint in the document is satisfied within solve tolerance
