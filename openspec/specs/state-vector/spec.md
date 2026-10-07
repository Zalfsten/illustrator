# state-vector

## Purpose

Defines the global continuous state vector as the single source of truth for unknowns, with each geometry’s local parameters expressed as references into that vector.

## Requirements

### Requirement: Document owns the discrete graph and a committed State

The system SHALL treat Document as owning the discrete sketch (geometries, structural constraints, and connectivity that maps each geometry's local parameters to indices in \(p\)) together with a **committed** State whose length matches the Document's parameter count. State SHALL be a first-class contiguous vector \(p\) of scalars (the sketch's continuous unknowns): the same State concept SHALL represent both the Document's committed values and any **working copy** used during solve or Interact. The assembled residual \(r(p)\) SHALL use a separate contiguous buffer (not an alias of \(p\)); implementations MAY reuse the same owned vector *type* for \(p\) and \(r\) while keeping the roles distinct. Evaluate, assemble, and Mesh construction SHALL take Document for identity/connectivity resolution plus an explicit State (committed or working). Callers SHALL NOT treat an ad-hoc unstructured bag of scalars as a second competing authority for committed sketch values.

#### Scenario: Structure and values can be reasoned about separately

- **WHEN** a caller holds Document and a State \(p\) of the correct length
- **THEN** evaluating geometries and assembling residuals is possible without the State living inside a GUI object
- **AND** replacing \(p\) with another State of the same length yields a different concrete figure under the same discrete graph

#### Scenario: Working copy is the same kind of State

- **WHEN** Interact or a solver iterates on a working copy of continuous parameters
- **THEN** that working copy is a State of the same kind as the Document's committed State
- **AND** a successful solve commits by replacing the committed State with that working State (or an equivalent write-back of its scalar entries)

### Requirement: Evaluation takes an explicit State

Geometry evaluation, residual assembly, and Jacobian assembly SHALL accept an explicit State \(p\) (committed or working copy). They SHALL NOT require mutating Document to try an alternate continuous configuration.

#### Scenario: Working copy is evaluated without committing

- **WHEN** Interact evaluates residuals at a working State during a drag
- **THEN** assembly uses that working State together with the current Document
- **AND** the Document's committed State is unchanged until an explicit successful commit

### Requirement: Structural edits are atomic on Document

Adding or removing a geometry SHALL update Document's discrete graph and committed State in one structural edit through Document (or an equivalent authority). Callers SHALL NOT independently append or remove State slots for a geometry without a matching geometry record (or the reverse) as a supported edit path. After any successful structural add or remove, State length SHALL equal the Document's parameter count and every remaining geometry's connectivity SHALL be valid for that State.

#### Scenario: Add extends graph and State together

- **WHEN** a parameterized geometry is added through Document with kind-owned initial parameters
- **THEN** Document gains the geometry record (including its contiguous block placement)
- **AND** the committed State gains exactly that geometry's parameter slots initialized to those values
- **AND** State length equals the Document's parameter count

#### Scenario: Unsupported split edit is not a supported path

- **WHEN** a caller would register a geometry record without extending State, or extend State without registering the geometry
- **THEN** that split is not a supported structural edit of the Document
- **AND** supported APIs keep the discrete graph and State matched

### Requirement: Evaluation uses the geometry's parameter window from structure

When evaluating a quantity that depends on a geometry's local parameters against a State \(p\), the system SHALL obtain that geometry's contiguous parameter window (block offset and count, or equivalent) from the geometry record in Document. Kind-specific chart formulas MAY interpret slots inside that window; they SHALL NOT maintain a separate authoritative layout table of global indices disconnected from the geometry record.

#### Scenario: Formula reads through the structure window

- **WHEN** a point or vector expression for a geometry is evaluated at State \(p\)
- **THEN** the scalars it reads are entries of \(p\) in the window identified by that geometry's record in Document
- **AND** changing those entries of \(p\) changes the evaluated quantity accordingly

### Requirement: Structural edits rebuild State and connectivity

When geometries are added or removed, the system SHALL update Document and rebuild or remap the State vector so that every remaining local parameter has a valid global index and \(p\)'s length equals the Document's parameter count. Those updates SHALL occur as atomic structural edits through Document (see *Structural edits are atomic on Document*). Constraints SHALL refer to geometries by stable geometry identity, not by raw positions in \(p\); global indices SHALL be derived via connectivity after the edit. Structural constraints themselves SHALL carry stable constraint identities (see system-architecture); removing a geometry SHALL drop orphan constraints that referenced that geometry identity.

#### Scenario: Adding a segment extends p

- **WHEN** a new parameterized geometry is added to the Document
- **THEN** State gains the corresponding scalar slots (initialized to the geometry's initial parameters)
- **AND** connectivity for that geometry maps its local parameters into those new indices
- **AND** Document and State remain length-matched after the edit

#### Scenario: Removing a geometry compacts p

- **WHEN** a geometry is removed from the Document
- **THEN** its parameter slots are removed from State (or equivalently State is rebuilt without them)
- **AND** remaining geometries' connectivity indices are updated to match the new State layout
- **AND** Document and State remain length-matched after the edit
- **AND** structural constraints that referenced the removed geometry identity are no longer present

### Requirement: Single global state vector

The system SHALL maintain one continuous State vector \(p\) per Document that holds all scalar unknowns of that sketch used by the solver. Committed continuous sketch values SHALL be exactly the Document's committed State. Discrete membership (which geometries and constraints exist) SHALL live in Document, not as entries of \(p\).

#### Scenario: Solver unknowns live in one vector

- **WHEN** a sketch with multiple geometries is prepared for solving
- **THEN** all continuous parameters participating in the solve are entries of a single State vector \(p\)
- **AND** the solver updates that vector (or an authorized working copy that is written back as the committed State) rather than independent per-geometry parameter stores as the source of truth

### Requirement: Local parameters are references into the global vector

Each geometry in Document SHALL associate its local parameters with entries of a State \(p\) via connectivity (local index → global index). Reading or writing a geometry's local parameters SHALL mean accessing the referenced entries of the State supplied to evaluation. Geometries SHALL NOT own a separate authoritative copy of those scalars disconnected from State.

#### Scenario: Geometry parameters resolve through connectivity

- **WHEN** a geometry evaluates a quantity that depends on its local parameters against a State \(p\)
- **THEN** those parameters are obtained from the corresponding entries of \(p\) identified by the geometry's connectivity
- **AND** changing those entries of \(p\) changes the evaluated quantity accordingly

### Requirement: Contiguous parameter block per geometry

Under the current Model E layout, each geometry SHALL own a **contiguous** block of scalar slots in State \(p\): there exists an offset \(o\) and a count \(n_G\) (the geometry's parameter count) such that local index \(i \in \{0,\ldots,n_G-1\}\) maps to global index \(o+i\). Connectivity for that geometry SHALL be exactly those \(n_G\) consecutive indices (or an equivalent derivation from \(o\) and \(n_G\)). Distinct geometries SHALL NOT share State indices; geometric coincidence and other relationships SHALL be expressed as constraints (residuals) over point/vector expressions, not by aliasing the same entries of \(p\) across geometries.

This contiguous-block assumption MAY later be relaxed to an arbitrary per-geometry index list still owned by the geometry record in Document; until such a change is specified, implementations SHALL use contiguous blocks.

#### Scenario: Segment parameters form one block

- **WHEN** a line segment with four endpoint parameters is added
- **THEN** those four scalars occupy four consecutive indices in \(p\)
- **AND** the segment's connectivity is those four indices in local order

#### Scenario: Joining endpoints does not merge parameter slots

- **WHEN** two segment endpoints are made coincident via Congruence
- **THEN** each endpoint retains its own entries in \(p\)
- **AND** coincidence is enforced by the Congruence residual, not by a shared global index

### Requirement: Connectivity is explicit for assembly

The local→global index mapping for each geometry (and for each constraint contribution that introduces residuals) SHALL be part of Document (or derived solely from it) and available as explicit connectivity for residual and Jacobian assembly. Implicit convenience views MAY exist for reading \(p\), but SHALL NOT replace connectivity as the rule for placing derivative contributions into the global Jacobian. Under the contiguous-block requirement, that mapping MAY be stored as a block offset plus parameter count on the geometry; the assembly-visible connectivity SHALL still enumerate the global indices.

#### Scenario: Mapping answers where local DOFs land globally

- **WHEN** a geometry or constraint has local degrees of freedom
- **THEN** there exists a connectivity that maps each local degree of freedom to a global index in \(p\)
- **AND** that connectivity is what assembly uses to scatter local Jacobian entries into global positions
