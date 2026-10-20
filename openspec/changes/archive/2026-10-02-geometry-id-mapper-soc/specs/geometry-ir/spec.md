# Spec Delta: geometry-ir

## ADDED Requirements

### Requirement: Stable geometry identities are allocated by the document registry

Stable geometry identities SHALL be allocated by the Document (or an equivalent registry authority that DocumentStructure uses) when a geometry is registered through a supported structural add. Chart formulas and kind pack/unpack SHALL NOT assign document identities. Callers MAY supply an explicit identity only for supported import/restore paths; in that case the registry SHALL observe the identity so later allocations do not collide with it. Implementations SHALL NOT require geometries to obtain identities from a process-global ambient generator during construction.

#### Scenario: Structural add assigns identity

- **WHEN** a parameterized geometry is added through the Document composite with kind-owned chart init
- **THEN** the registered geometry has a stable identity assigned by the document registry
- **AND** that identity was not required as an input to the kind's chart pack step

#### Scenario: Explicit identity is observed on import

- **WHEN** a supported import/restore path registers a geometry with an explicit stable identity
- **THEN** that identity is resolvable in DocumentStructure
- **AND** subsequent structural adds receive identities that do not collide with it

## MODIFIED Requirements

### Requirement: Geometries are resolved by stable identity without scanning the full set

Each geometry in DocumentStructure SHALL have a stable geometry identity. DocumentStructure (or an equivalent read-only geometry resolution port implemented for it) SHALL resolve a geometry identity to the corresponding geometry record for expression evaluation, residual evaluation, and Jacobian assembly without scanning every geometry in the document for each lookup (lookup cost SHALL be independent of the number of geometries, aside from ordinary hash/map overhead). Constraints and expressions that name geometries SHALL store and carry that identity as the durable handle. After identity resolution yields a geometry record, chart unpack and parameter-window reads for that evaluation step SHALL use the resolved record (together with State) and SHALL NOT require a second identity lookup inside the unpack step itself. Removing a geometry SHALL drop the identity from the resolution map and invalidate or drop constraints that referenced it, consistent with existing structural-edit rules.

#### Scenario: Evaluate resolves a named geometry without a full scan

- **WHEN** a point or vector expression names a geometry identity
- **AND** that expression is evaluated against DocumentStructure and a State
- **THEN** DocumentStructure yields that geometry by identity resolution
- **AND** evaluation does not depend on linearly searching the entire geometry collection for a matching identity

#### Scenario: Unpack uses the resolved geometry record

- **WHEN** a point or vector expression has resolved a geometry identity to a geometry record
- **AND** the expression unpacks that geometry's chart init from State
- **THEN** unpack reads the parameter window from that resolved record
- **AND** unpack does not perform another identity→geometry lookup

#### Scenario: Remove drops identity resolution and orphan references

- **WHEN** a geometry is removed from DocumentStructure
- **THEN** its identity is no longer resolvable
- **AND** structural constraints that referenced that identity are no longer present in the structure
