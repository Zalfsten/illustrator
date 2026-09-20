# system-architecture

## Purpose

Defines the top-level shared component model for the illustrator: Document as structure plus committed state, App and Canvas for presentation and input, Interact for manipulation under constraints, and SolverPort as a replaceable numerical backend — independent of GUI toolkit and language.

## Requirements

### Requirement: Document is structure plus committed state

The system SHALL treat a Document as the composite of DocumentStructure and a committed State vector \(p\). DocumentStructure SHALL hold the discrete sketch: geometries, structural constraints, and the connectivity schema that maps local parameters into indices of \(p\). The committed State SHALL hold the continuous values of those parameters. DocumentStructure SHALL NOT depend on a GUI, renderer, or solver backend.

#### Scenario: Structure remains meaningful without evaluating p

- **WHEN** a Document's structure (geometries and constraints) is inspected without reading coordinate values
- **THEN** which geometries and structural relationships exist is fully determined by DocumentStructure
- **AND** concrete endpoint locations are obtained only by evaluating expressions against a State \(p\)

### Requirement: App owns presentation and hosts Interact

The system SHALL provide an App component that references a Document for the active sketch. The App SHALL host a Canvas that displays the sketch and delivers pointer (or equivalent) input. The App SHALL NOT embed solver algorithm details; interactive solving SHALL go through Interact and SolverPort.

#### Scenario: Canvas renders from Document state

- **WHEN** the App paints the sketch
- **THEN** geometry appearance is derived from the Document's structure evaluated at a concrete State (normally the committed \(p\))
- **AND** the Canvas does not own a second authoritative copy of structural relationships

### Requirement: Interact owns manipulation sessions

Interact SHALL run manipulation sessions (including drag) against DocumentStructure and a working State derived from the committed \(p\). On successful solve, Interact SHALL commit the working State into the Document's committed State. On failed solve, Interact SHALL leave the committed State unchanged. Soft manipulation conditions SHALL exist only for the session lifetime.

#### Scenario: Failed drag does not mutate committed state

- **WHEN** an interactive solve during a drag fails
- **THEN** the Document's committed State remains the last consistent committed State
- **AND** DocumentStructure is unchanged by that failed solve

### Requirement: SolverPort is used only through Interact

Nonlinear solves that update continuous parameters during interaction SHALL be performed through the SolverPort. Document and Canvas SHALL NOT call a concrete solver algorithm directly as part of their defined contracts.

#### Scenario: Drag solve goes through the port

- **WHEN** Interact needs a new State consistent with structural constraints and soft targets
- **THEN** it obtains that State via SolverPort evaluations of assembled residuals/Jacobians
- **AND** DocumentStructure is only read (plus connectivity) for assembly, not mutated by the solver itself

### Requirement: Display tessellation is a deferred view concern

Until geometries require tessellation beyond what the Canvas can draw directly from evaluated point expressions (for example only line segments), the system SHALL NOT require a first-class Mesh or View cache as part of Document or Interact. When such tessellation becomes necessary, it SHALL be introduced as a view layer derived from DocumentStructure plus a concrete State, not as a second source of truth for parameters or constraints.

#### Scenario: Segment-only sketch needs no architectural mesh

- **WHEN** a Document contains only line segments as geometries
- **THEN** the Canvas MAY draw by evaluating endpoint expressions at State \(p\) without a persistent Mesh component
- **AND** the absence of a Mesh does not violate this architecture capability
