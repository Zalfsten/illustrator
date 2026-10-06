# system-architecture

## Purpose

Defines the top-level shared component model for the illustrator: Document as structure plus committed state, App and Canvas for presentation and input, Interact for manipulation under constraints, and SolverPort as a replaceable numerical backend — independent of GUI toolkit and language.

## Requirements

### Requirement: Document holds discrete sketch plus committed state

The system SHALL treat a Document as owning the discrete sketch (geometries, structural constraints, and the connectivity schema that maps local parameters into indices of \(p\)) together with a committed State vector \(p\). The committed State SHALL hold the continuous values of those parameters. Evaluate, assemble, and Mesh construction SHALL take Document for identity/connectivity resolution plus an explicit State (committed or working). Document SHALL NOT depend on a GUI, renderer, or solver backend.

#### Scenario: Structure remains meaningful without evaluating p

- **WHEN** a Document's geometries and constraints are inspected without reading coordinate values
- **THEN** which geometries and structural relationships exist is fully determined by Document
- **AND** concrete endpoint locations are obtained only by evaluating expressions against a State \(p\)

### Requirement: App owns presentation and hosts Interact

The system SHALL provide an App component that references a Document for the active sketch. The App SHALL host a Canvas that displays the sketch and delivers pointer (or equivalent) input. The App SHALL NOT embed solver algorithm details; interactive solving SHALL go through Interact and SolverPort.

#### Scenario: Canvas renders from Document state

- **WHEN** the App paints the sketch
- **THEN** geometry appearance is derived from the Document's structure evaluated at a concrete State (normally the committed \(p\))
- **AND** the Canvas does not own a second authoritative copy of structural relationships

### Requirement: Interact owns manipulation sessions

Interact SHALL run manipulation sessions (including drag) against Document and a working State of the same kind as the Document's committed State (derived from that committed \(p\)). On successful solve, Interact SHALL commit the working State into the Document's committed State. On failed solve, Interact SHALL leave the committed State unchanged. Soft manipulation conditions SHALL exist only for the session lifetime.

#### Scenario: Failed drag does not mutate committed state

- **WHEN** an interactive solve during a drag fails
- **THEN** the Document's committed State remains the last consistent committed State
- **AND** Document's geometries and constraints are unchanged by that failed solve

### Requirement: SolverPort is used only through Interact

Nonlinear solves that update continuous parameters during interaction SHALL be performed through the SolverPort. Document and Canvas SHALL NOT call a concrete solver algorithm directly as part of their defined contracts.

#### Scenario: Drag solve goes through the port

- **WHEN** Interact needs a new State consistent with structural constraints and soft targets
- **THEN** it obtains that State via SolverPort evaluations of assembled residuals/Jacobians
- **AND** Document is only read (plus connectivity) for assembly, not mutated by the solver itself

### Requirement: Display Mesh is the Canvas geometry source

The system SHALL provide a display Mesh (or equivalent view-layer edge list) derived from Document plus a concrete State \(p\). The Mesh SHALL NOT be part of Document or Interact as authoritative structure or parameters. The Canvas SHALL obtain stroked sketch geometry from that Mesh. Until richer tessellation is needed, a Mesh whose edges correspond one-to-one with line segment geometries SHALL satisfy this requirement. Hit-testing against Mesh primitives MAY be used by tools; when used, hit results SHALL carry stable geometry identities consistent with the Mesh edge tags.

#### Scenario: Canvas paints via Mesh

- **WHEN** the App paints the sketch at the committed State
- **THEN** drawable edges come from a Mesh built from Document and that State
- **AND** the Canvas does not own a second authoritative copy of structural relationships

#### Scenario: Mesh stays outside Document authority

- **WHEN** Interact commits a new State after a successful solve
- **THEN** Document remains the sole authority for parameters and constraints
- **AND** any Mesh used for display is rebuilt or invalidated from that Document rather than written back into Document

### Requirement: App owns a Viewport for world–screen mapping

The App SHALL own a Viewport as UI session state that maps between **world coordinates** (document/Mesh space, y-up) and **screen coordinates** (Canvas pixels). Canvas paint SHALL transform Mesh edge positions from world to screen via the Viewport. Pointer (or equivalent) input SHALL be mapped from screen to world via the Viewport before updating Document State, Interact soft targets, or issuing Mesh hit queries against world-space Mesh positions. The Viewport SHALL NOT be part of Document or committed State.

#### Scenario: Paint uses screen positions derived from world Mesh

- **WHEN** the App paints the sketch from a Mesh built at the committed State
- **THEN** stroked edge endpoints on the Canvas are the Viewport's screen images of the Mesh edge world positions
- **AND** Document State values remain unchanged by painting

#### Scenario: Pointer updates use world coordinates

- **WHEN** the user places a point or drags with the pointer on the Canvas
- **THEN** the location passed into Document / Interact is the Viewport's world image of the screen pointer position
- **AND** that world location is not required to equal the raw screen pixel coordinates

#### Scenario: Pixel hit radius maps to world length

- **WHEN** a tool issues a Mesh hit query with a radius specified in screen pixels
- **THEN** the App converts that radius to world length using the Viewport scale before calling the Mesh hit API
- **AND** a successful hit still returns stable geometry identity from the world-space Mesh

### Requirement: Constraints are resolved by stable identity

Each structural constraint in Document SHALL have a stable constraint identity, distinct from geometry identities and from positions in the State vector. Document SHALL resolve a constraint identity to that constraint without scanning every constraint in the document for each lookup (lookup cost SHALL be independent of the number of constraints, aside from ordinary hash/map overhead). Adding a structural constraint SHALL allocate a fresh identity; removing a constraint SHALL drop that identity from resolution. Constraints SHALL NOT be identified solely by their index in an unordered list when a stable handle is required (for example incidence queries or later UI selection).

#### Scenario: Add returns a resolvable constraint identity

- **WHEN** a structural constraint is added to Document
- **THEN** the add operation yields a stable constraint identity
- **AND** Document resolves that identity to the added constraint

#### Scenario: Remove drops constraint identity resolution

- **WHEN** a structural constraint is removed from Document
- **THEN** its identity is no longer resolvable

### Requirement: Geometry–constraint incidence is queryable

Document SHALL answer which structural constraints reference a given geometry identity. Each structural constraint SHALL report the set of geometry identities it depends on. Removing a geometry SHALL remove every structural constraint that referenced that identity (orphan drop), consistent with geometry-ir structural-edit rules. Incidence answers SHALL remain consistent after add/remove of geometries and constraints.

#### Scenario: Constraints on a geometry are listed after join

- **WHEN** Congruence joins an endpoint of geometry A to an endpoint of geometry B
- **AND** incidence for A is queried
- **THEN** that Congruence identity appears among the constraints on A
- **AND** incidence for B likewise includes that Congruence identity

#### Scenario: Orphan constraints disappear with their geometry

- **WHEN** a geometry is removed
- **THEN** no remaining constraint in Document references the removed identity
- **AND** incidence queries for other geometries no longer list those removed constraints

### Requirement: World coordinates are y-up

Document State parameter blocks that encode planar point coordinates, and Mesh edge positions derived from them, SHALL use a right-handed sense in the plane with **+y upward** (mathematical/scientific convention). Screen toolkits that use +y downward SHALL obtain the flip only through the Viewport mapping, not by storing inverted y in committed State.

#### Scenario: Increasing world y moves toward the top of the Canvas after mapping

- **WHEN** two world points share the same x and the second has larger y
- **AND** both are mapped to screen through the Viewport
- **THEN** the second point's screen y is less than or equal to the first's when the Canvas uses +y downward

### Requirement: App presentation mode controls construction visibility and picking

The App SHALL provide a presentation mode that either shows **result geometries only** or **includes construction geometries**. That mode SHALL determine the visibility filter used when building the Mesh for Canvas paint and for Mesh-backed hit-testing. When construction geometries are not shown in the current mode, they SHALL NOT be pickable via Mesh hits or via logical handles derived under the same filter. Presentation mode is UI session state; geometry role remains in Document.

#### Scenario: Toggle hides and unpicks construction

- **WHEN** the user switches from include-construction mode to result-only mode
- **AND** the Canvas repaints
- **THEN** construction geometries are not stroked
- **AND** pointer hit-testing does not select those construction geometries

#### Scenario: Mode does not alter Document roles

- **WHEN** the user toggles presentation mode
- **THEN** each geometry's result vs construction role in Document is unchanged
