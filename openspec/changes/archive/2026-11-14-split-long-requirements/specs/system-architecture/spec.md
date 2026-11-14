## MODIFIED Requirements

### Requirement: Document holds discrete sketch plus committed state

The system SHALL treat a Document as owning the discrete sketch (geometries, structural constraints, and the connectivity schema that maps local parameters into indices of \(p\)) together with a committed State vector \(p\). The committed State SHALL hold the continuous values of those parameters. Document SHALL NOT depend on a GUI, renderer, or solver backend.

#### Scenario: Structure remains meaningful without evaluating p

- **WHEN** a Document's geometries and constraints are inspected without reading coordinate values
- **THEN** which geometries and structural relationships exist is fully determined by Document
- **AND** concrete endpoint locations are obtained only by evaluating expressions against a State \(p\)

### Requirement: Display Mesh is the Canvas geometry source

The system SHALL provide a display Mesh (or equivalent view-layer edge list) derived from Document plus a concrete State \(p\). The Mesh SHALL NOT be part of Document or Interact as authoritative structure or parameters. The Canvas SHALL obtain stroked sketch geometry from that Mesh. Until richer tessellation is needed, a Mesh whose edges correspond one-to-one with line segment geometries SHALL satisfy this requirement.

#### Scenario: Canvas paints via Mesh

- **WHEN** the App paints the sketch at the committed State
- **THEN** drawable edges come from a Mesh built from Document and that State
- **AND** the Canvas does not own a second authoritative copy of structural relationships

#### Scenario: Mesh stays outside Document authority

- **WHEN** Interact commits a new State after a successful solve
- **THEN** Document remains the sole authority for parameters and constraints
- **AND** any Mesh used for display is rebuilt or invalidated from that Document rather than written back into Document

### Requirement: App owns a Viewport for world–screen mapping

The App SHALL own a Viewport as UI session state that maps between **world coordinates** (document/Mesh space, y-up) and **screen coordinates** (Canvas pixels). Canvas paint SHALL transform Mesh edge positions from world to screen via the Viewport. The Viewport SHALL NOT be part of Document or committed State.

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

Each structural constraint in Document SHALL have a stable constraint identity, distinct from geometry identities and from positions in the State vector. Document SHALL resolve a constraint identity to that constraint without scanning every constraint in the document for each lookup (lookup cost SHALL be independent of the number of constraints, aside from ordinary hash/map overhead).

#### Scenario: Add returns a resolvable constraint identity

- **WHEN** a structural constraint is added to Document
- **THEN** the add operation yields a stable constraint identity
- **AND** Document resolves that identity to the added constraint

#### Scenario: Remove drops constraint identity resolution

- **WHEN** a structural constraint is removed from Document
- **THEN** its identity is no longer resolvable

### Requirement: Geometry–constraint incidence is queryable

Document SHALL answer which structural constraints reference a given geometry identity. Each structural constraint SHALL report the **deduplicated** set of geometry identities it depends on (each identity at most once; when several of its expressions name the same geometry, that identity appears once).

#### Scenario: Constraints on a geometry are listed after join

- **WHEN** Congruence joins an endpoint of geometry A to an endpoint of geometry B
- **AND** incidence for A is queried
- **THEN** that Congruence identity appears among the constraints on A
- **AND** incidence for B likewise includes that Congruence identity

#### Scenario: Same geometry twice still yields one incidence entry

- **WHEN** a structural constraint relates two expressions that resolve to the same geometry identity
- **AND** the geometries referenced by that constraint are reported
- **THEN** that identity appears once in the reported set
- **AND** incidence for that geometry lists the constraint once

#### Scenario: Orphan constraints disappear with their geometry

- **WHEN** a geometry is removed
- **THEN** no remaining constraint in Document references the removed identity
- **AND** incidence queries for other geometries no longer list those removed constraints

## ADDED Requirements

### Requirement: Evaluation takes Document plus an explicit State

Evaluate, assemble, and Mesh construction SHALL take Document for identity/connectivity resolution plus an explicit State (committed or working).

#### Scenario: Working State leaves the committed State alone

- **WHEN** geometries are evaluated at a working State during a drag
- **THEN** the Document's committed State is unchanged until the drag commits

### Requirement: Mesh hits carry geometry identity

Hit-testing against Mesh primitives MAY be used by tools; when used, hit results SHALL carry stable geometry identities consistent with the Mesh edge tags.

#### Scenario: Hit on an arc edge names the arc

- **WHEN** a tool hit-tests near an edge of an arc's Mesh
- **THEN** the hit carries the arc's geometry identity, the same identity the edge is tagged with

### Requirement: Pointer input maps through the Viewport

Pointer (or equivalent) input SHALL be mapped from screen to world via the Viewport before updating Document State, Interact soft targets, or issuing Mesh hit queries against world-space Mesh positions.

#### Scenario: Same click, different scale

- **WHEN** the same screen position is clicked under two Viewports with different scales
- **THEN** the world positions passed on differ according to the two Viewports

### Requirement: Constraint identities are fresh handles

Adding a structural constraint SHALL allocate a fresh identity; removing a constraint SHALL drop that identity from resolution. Constraints SHALL NOT be identified solely by their index in an unordered list when a stable handle is required (for example incidence queries or later UI selection).

#### Scenario: Removed identity is not handed out again

- **WHEN** a constraint is removed and another constraint is added afterwards
- **THEN** the new constraint's identity differs from the removed one's
- **AND** the identities of the other constraints are unchanged

### Requirement: Incidence stays consistent across edits

Removing a geometry SHALL remove every structural constraint that referenced that identity (orphan drop), consistent with geometry-ir structural-edit rules. Incidence answers SHALL remain consistent after add/remove of geometries and constraints.

#### Scenario: Removed constraint leaves incidence

- **WHEN** a Congruence between geometries A and B is removed
- **THEN** incidence for A and for B no longer lists that Congruence
