# Spec Delta: state-vector

## ADDED Requirements

### Requirement: Structural edits are atomic on the Document composite

Adding or removing a geometry SHALL update DocumentStructure and the Document's committed State in one structural edit through the Document composite (or an equivalent pairing authority). Callers SHALL NOT independently append or remove State slots for a geometry without a matching geometry record (or the reverse) as a supported edit path. After any successful structural add or remove, State length SHALL equal the structure's parameter count and every remaining geometry's connectivity SHALL be valid for that State.

#### Scenario: Add extends structure and State together

- **WHEN** a parameterized geometry is added through the Document composite with kind-owned initial parameters
- **THEN** DocumentStructure gains the geometry record (including its contiguous block placement)
- **AND** State gains exactly that geometry's parameter slots initialized to those values
- **AND** State length equals the structure's parameter count

#### Scenario: Unsupported split edit is not a supported path

- **WHEN** a caller would register a geometry record without extending State, or extend State without registering the geometry
- **THEN** that split is not a supported structural edit of the Document
- **AND** supported APIs keep structure and State matched

### Requirement: Evaluation uses the geometry's parameter window from structure

When evaluating a quantity that depends on a geometry's local parameters against a State \(p\), the system SHALL obtain that geometry's contiguous parameter window (block offset and count, or equivalent) from the geometry record in DocumentStructure. Kind-specific chart formulas MAY interpret slots inside that window; they SHALL NOT maintain a separate authoritative layout table of global indices disconnected from the geometry record.

#### Scenario: Formula reads through the structure window

- **WHEN** a point or vector expression for a geometry is evaluated at State \(p\)
- **THEN** the scalars it reads are entries of \(p\) in the window identified by that geometry's record in DocumentStructure
- **AND** changing those entries of \(p\) changes the evaluated quantity accordingly

## MODIFIED Requirements

### Requirement: Structural edits rebuild State and connectivity

When geometries are added or removed, the system SHALL update DocumentStructure and rebuild or remap the State vector so that every remaining local parameter has a valid global index and \(p\)'s length equals the structure's parameter count. Those updates SHALL occur as atomic structural edits through the Document composite (see *Structural edits are atomic on the Document composite*). Constraints SHALL refer to geometries by stable identity, not by raw positions in \(p\); global indices SHALL be derived via connectivity after the edit.

#### Scenario: Adding a segment extends p

- **WHEN** a new parameterized geometry is added to the Document
- **THEN** State gains the corresponding scalar slots (initialized to the geometry's initial parameters)
- **AND** connectivity for that geometry maps its local parameters into those new indices
- **AND** DocumentStructure and State remain length-matched after the edit

#### Scenario: Removing a geometry compacts p

- **WHEN** a geometry is removed from the Document
- **THEN** its parameter slots are removed from State (or equivalently State is rebuilt without them)
- **AND** remaining geometries' connectivity indices are updated to match the new State layout
- **AND** DocumentStructure and State remain length-matched after the edit
