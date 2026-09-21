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

### Requirement: Display Mesh is the Canvas geometry source

The system SHALL provide a display Mesh (or equivalent view-layer edge list) derived from DocumentStructure plus a concrete State \(p\). The Mesh SHALL NOT be part of Document or Interact as authoritative structure or parameters. The Canvas SHALL obtain stroked sketch geometry from that Mesh. Until richer tessellation is needed, a Mesh whose edges correspond one-to-one with line segment geometries SHALL satisfy this requirement. Hit-testing against Mesh primitives MAY be used by tools; when used, hit results SHALL carry stable geometry identities consistent with the Mesh edge tags.

#### Scenario: Canvas paints via Mesh

- **WHEN** the App paints the sketch at the committed State
- **THEN** drawable edges come from a Mesh built from DocumentStructure and that State
- **AND** the Canvas does not own a second authoritative copy of structural relationships

#### Scenario: Mesh stays outside Document authority

- **WHEN** Interact commits a new State after a successful solve
- **THEN** DocumentStructure and committed State remain the sole authorities for parameters and constraints
- **AND** any Mesh used for display is rebuilt or invalidated from that Document rather than written back into DocumentStructure
