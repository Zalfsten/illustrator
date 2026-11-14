## MODIFIED Requirements

### Requirement: Document owns the discrete graph and a committed State

The system SHALL treat Document as owning the discrete sketch (geometries, structural constraints, and connectivity that maps each geometry's local parameters to indices in \(p\)) together with a **committed** State whose length matches the Document's parameter count. State SHALL be a first-class contiguous vector \(p\) of scalars (the sketch's continuous unknowns): the same State concept SHALL represent both the Document's committed values and any **working copy** used during solve or Interact.

#### Scenario: Structure and values can be reasoned about separately

- **WHEN** a caller holds Document and a State \(p\) of the correct length
- **THEN** evaluating geometries and assembling residuals is possible without the State living inside a GUI object
- **AND** replacing \(p\) with another State of the same length yields a different concrete figure under the same discrete graph

#### Scenario: Working copy is the same kind of State

- **WHEN** Interact or a solver iterates on a working copy of continuous parameters
- **THEN** that working copy is a State of the same kind as the Document's committed State
- **AND** a successful solve commits by replacing the committed State with that working State (or an equivalent write-back of its scalar entries)

### Requirement: Structural edits rebuild State and connectivity

When geometries are added or removed, the system SHALL update Document and rebuild or remap the State vector so that every remaining local parameter has a valid global index and \(p\)'s length equals the Document's parameter count. Those updates SHALL occur as atomic structural edits through Document (see *Structural edits are atomic on Document*).

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

### Requirement: Contiguous parameter block per geometry

Under the current Model E layout, each geometry SHALL own a **contiguous** block of scalar slots in State \(p\): there exists an offset \(o\) and a count \(n_G\) (the geometry's parameter count) such that local index \(i \in \{0,\ldots,n_G-1\}\) maps to global index \(o+i\). Connectivity for that geometry SHALL be exactly those \(n_G\) consecutive indices (or an equivalent derivation from \(o\) and \(n_G\)).

#### Scenario: Segment parameters form one block

- **WHEN** a line segment with four endpoint parameters is added
- **THEN** those four scalars occupy four consecutive indices in \(p\)
- **AND** the segment's connectivity is those four indices in local order

#### Scenario: Joining endpoints does not merge parameter slots

- **WHEN** two segment endpoints are made coincident via Congruence
- **THEN** each endpoint retains its own entries in \(p\)
- **AND** coincidence is enforced by the Congruence residual, not by a shared global index

### Requirement: Connectivity is explicit for assembly

The local→global index mapping for each geometry (and for each constraint contribution that introduces residuals) SHALL be part of Document (or derived solely from it) and available as explicit connectivity for residual and Jacobian assembly. Implicit convenience views MAY exist for reading \(p\), but SHALL NOT replace connectivity as the rule for placing derivative contributions into the global Jacobian.

#### Scenario: Mapping answers where local DOFs land globally

- **WHEN** a geometry or constraint has local degrees of freedom
- **THEN** there exists a connectivity that maps each local degree of freedom to a global index in \(p\)
- **AND** that connectivity is what assembly uses to scatter local Jacobian entries into global positions

## ADDED Requirements

### Requirement: Residual buffer is separate from State

The assembled residual \(r(p)\) SHALL use a separate contiguous buffer (not an alias of \(p\)); implementations MAY reuse the same owned vector *type* for \(p\) and \(r\) while keeping the roles distinct.

#### Scenario: Writing residuals leaves State unchanged

- **WHEN** residuals are assembled at a State \(p\)
- **THEN** the residual buffer has one entry per residual row
- **AND** \(p\) is unchanged afterwards

### Requirement: Evaluation takes Document plus an explicit State

Evaluate, assemble, and Mesh construction SHALL take Document for identity/connectivity resolution plus an explicit State (committed or working). Callers SHALL NOT treat an ad-hoc unstructured bag of scalars as a second competing authority for committed sketch values.

#### Scenario: One Document, two States

- **WHEN** the Mesh is built once from the committed State and once from a working State of the same Document
- **THEN** both Meshes have the same geometry identities and differ only in positions

### Requirement: Constraints follow geometries through structural edits

Constraints SHALL refer to geometries by stable geometry identity, not by raw positions in \(p\); global indices SHALL be derived via connectivity after the edit. Structural constraints themselves SHALL carry stable constraint identities (see system-architecture); removing a geometry SHALL drop orphan constraints that referenced that geometry identity.

#### Scenario: Constraint survives an index shift

- **WHEN** geometries A, B and C are added in that order, a constraint relates B and C, and A is removed
- **THEN** the constraint still relates B and C, with the same constraint identity
- **AND** it reads their parameters at their new indices in \(p\)

### Requirement: Geometries do not share State indices

Distinct geometries SHALL NOT share State indices; geometric coincidence and other relationships SHALL be expressed as constraints (residuals) over point/vector expressions, not by aliasing the same entries of \(p\) across geometries.

#### Scenario: Disjoint blocks

- **WHEN** a segment and an arc are added to a Document
- **THEN** the sets of indices in \(p\) owned by the two geometries are disjoint

### Requirement: Contiguous blocks until relaxed

This contiguous-block assumption MAY later be relaxed to an arbitrary per-geometry index list still owned by the geometry record in Document; until such a change is specified, implementations SHALL use contiguous blocks.

#### Scenario: Blocks stay contiguous across edits

- **WHEN** geometries are added and removed in any order
- **THEN** every remaining geometry's indices in \(p\) form one contiguous block

### Requirement: Block offsets still enumerate indices for assembly

Under the contiguous-block requirement, the local→global mapping MAY be stored as a block offset plus parameter count on the geometry; the assembly-visible connectivity SHALL still enumerate the global indices.

#### Scenario: Offset and count give the index list

- **WHEN** a segment's block has offset 4 and parameter count 4
- **THEN** its connectivity for assembly lists the global indices 4, 5, 6 and 7 in local order
