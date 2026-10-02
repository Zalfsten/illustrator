# Spec Delta: state-vector

## MODIFIED Requirements

### Requirement: Evaluation uses the geometry's parameter window from structure

When evaluating a quantity that depends on a geometry's local parameters against a State \(p\), the system SHALL obtain that geometry's contiguous parameter window (block offset and count, or equivalent) from the **resolved geometry record** in DocumentStructure (the record obtained by identity resolution, or an equivalent already-resolved handle). Kind-specific chart formulas MAY interpret slots inside that window; they SHALL NOT maintain a separate authoritative layout table of global indices disconnected from the geometry record. Chart unpack for evaluation SHALL take that resolved geometry record together with State \(p\) and SHALL NOT require a fresh identity lookup as part of the unpack operation itself.

#### Scenario: Formula reads through the structure window

- **WHEN** a point or vector expression for a geometry is evaluated at State \(p\)
- **THEN** the scalars it reads are entries of \(p\) in the window identified by that geometry's record in DocumentStructure
- **AND** changing those entries of \(p\) changes the evaluated quantity accordingly

#### Scenario: Unpack does not re-resolve identity

- **WHEN** evaluation has already resolved a geometry identity to a geometry record
- **AND** chart init is unpacked from State for that geometry
- **THEN** the unpack step uses the resolved record's parameter window
- **AND** the unpack step does not perform another identity→geometry lookup
