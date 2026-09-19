# state-vector

## Purpose

Defines the global continuous state vector as the single source of truth for unknowns, with each geometry’s local parameters expressed as references into that vector.

## Requirements

### Requirement: Single global state vector

The system SHALL maintain one global continuous state vector \(p\) that holds all scalar unknowns of the active sketch used by the solver. Committed sketch state for solving SHALL be represented by the contents of this vector (together with discrete structure such as which geometries and constraints exist).

#### Scenario: Solver unknowns live in one vector

- **WHEN** a sketch with multiple geometries is prepared for solving
- **THEN** all continuous parameters participating in the solve are entries of a single state vector \(p\)
- **AND** the solver updates that vector (or an authorized working copy that is written back as \(p\)) rather than independent per-geometry parameter stores as the source of truth

### Requirement: Local parameters are references into the global vector

Each geometry SHALL associate its local parameters with entries of \(p\) via a connectivity mapping (local index → global index). Reading or writing a geometry’s local parameters SHALL mean accessing the referenced entries of \(p\). Geometries SHALL NOT own a separate authoritative copy of those scalars disconnected from \(p\).

#### Scenario: Geometry parameters resolve through connectivity

- **WHEN** a geometry evaluates a quantity that depends on its local parameters
- **THEN** those parameters are obtained from the corresponding entries of \(p\) identified by the geometry’s connectivity
- **AND** changing those entries of \(p\) changes the evaluated quantity accordingly

### Requirement: Connectivity is explicit for assembly

The local→global index mapping for each geometry (and for each constraint contribution that introduces residuals) SHALL be available as an explicit connectivity suitable for residual and Jacobian assembly. Implicit convenience views MAY exist for reading \(p\), but SHALL NOT replace connectivity as the rule for placing derivative contributions into the global Jacobian.

#### Scenario: Mapping answers where local DOFs land globally

- **WHEN** a geometry or constraint has local degrees of freedom
- **THEN** there exists a connectivity that maps each local degree of freedom to a global index in \(p\)
- **AND** that connectivity is what assembly uses to scatter local Jacobian entries into global positions
