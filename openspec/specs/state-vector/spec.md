# state-vector

## Purpose

Defines the global continuous state vector as the single source of truth for unknowns, with each geometry’s local parameters expressed as references into that vector.

## Requirements

### Requirement: DocumentStructure and State are distinct

The system SHALL distinguish DocumentStructure from State. DocumentStructure SHALL own geometries, structural constraints, and connectivity that maps each geometry's local parameters to indices in a State vector of matching length. State SHALL be a single contiguous vector \(p\) of scalars. A Document SHALL be the pairing of one DocumentStructure with one committed State whose length matches that structure's parameter count.

#### Scenario: Structure and values can be reasoned about separately

- **WHEN** a caller holds DocumentStructure and a State \(p\) of the correct length
- **THEN** evaluating geometries and assembling residuals is possible without the State living inside a GUI object
- **AND** replacing \(p\) with another vector of the same length yields a different concrete figure under the same structure

### Requirement: Evaluation takes an explicit State

Geometry evaluation, residual assembly, and Jacobian assembly SHALL accept an explicit State \(p\) (committed or working copy). They SHALL NOT require mutating DocumentStructure to try an alternate continuous configuration.

#### Scenario: Working copy is evaluated without committing

- **WHEN** Interact evaluates residuals at a working State during a drag
- **THEN** assembly uses that working State together with the current DocumentStructure
- **AND** the Document's committed State is unchanged until an explicit successful commit

### Requirement: Structural edits rebuild State and connectivity

When geometries are added or removed, the system SHALL update DocumentStructure and rebuild or remap the State vector so that every remaining local parameter has a valid global index and \(p\)'s length equals the structure's parameter count. Constraints SHALL refer to geometries by stable identity, not by raw positions in \(p\); global indices SHALL be derived via connectivity after the edit.

#### Scenario: Adding a segment extends p

- **WHEN** a new parameterized geometry is added to DocumentStructure
- **THEN** State gains the corresponding scalar slots (initialized to the geometry's initial parameters)
- **AND** connectivity for that geometry maps its local parameters into those new indices

#### Scenario: Removing a geometry compacts p

- **WHEN** a geometry is removed from DocumentStructure
- **THEN** its parameter slots are removed from State (or equivalently State is rebuilt without them)
- **AND** remaining geometries' connectivity indices are updated to match the new State layout

### Requirement: Single global state vector

The system SHALL maintain one continuous State vector \(p\) per Document that holds all scalar unknowns of that sketch used by the solver. Committed continuous sketch values SHALL be exactly the Document's committed State. Discrete membership (which geometries and constraints exist) SHALL live in DocumentStructure, not as entries of \(p\).

#### Scenario: Solver unknowns live in one vector

- **WHEN** a sketch with multiple geometries is prepared for solving
- **THEN** all continuous parameters participating in the solve are entries of a single State vector \(p\)
- **AND** the solver updates that vector (or an authorized working copy that is written back as the committed State) rather than independent per-geometry parameter stores as the source of truth

### Requirement: Local parameters are references into the global vector

Each geometry in DocumentStructure SHALL associate its local parameters with entries of a State \(p\) via connectivity (local index → global index). Reading or writing a geometry's local parameters SHALL mean accessing the referenced entries of the State supplied to evaluation. Geometries SHALL NOT own a separate authoritative copy of those scalars disconnected from State.

#### Scenario: Geometry parameters resolve through connectivity

- **WHEN** a geometry evaluates a quantity that depends on its local parameters against a State \(p\)
- **THEN** those parameters are obtained from the corresponding entries of \(p\) identified by the geometry's connectivity
- **AND** changing those entries of \(p\) changes the evaluated quantity accordingly

### Requirement: Connectivity is explicit for assembly

The local→global index mapping for each geometry (and for each constraint contribution that introduces residuals) SHALL be part of DocumentStructure (or derived solely from it) and available as explicit connectivity for residual and Jacobian assembly. Implicit convenience views MAY exist for reading \(p\), but SHALL NOT replace connectivity as the rule for placing derivative contributions into the global Jacobian.

#### Scenario: Mapping answers where local DOFs land globally

- **WHEN** a geometry or constraint has local degrees of freedom
- **THEN** there exists a connectivity that maps each local degree of freedom to a global index in \(p\)
- **AND** that connectivity is what assembly uses to scatter local Jacobian entries into global positions
