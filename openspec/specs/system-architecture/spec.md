# system-architecture

## Purpose

Defines the top-level shared component model for the illustrator: Document as structure plus committed state, App and Canvas for presentation and input, Interact for manipulation under constraints, and SolverPort as a replaceable numerical backend — independent of GUI toolkit and language.

## Requirements

### Requirement: Document holds discrete sketch plus committed state

The system SHALL treat a Document as owning the discrete sketch (geometries, structural constraints, and the connectivity schema that maps local parameters into indices of \(p\)) together with a committed State vector \(p\). The committed State SHALL hold the continuous values of those parameters. Document SHALL NOT depend on a GUI, renderer, or solver backend.

#### Scenario: Structure remains meaningful without evaluating p

- **WHEN** a Document's geometries and constraints are inspected without reading coordinate values
- **THEN** which geometries and structural relationships exist is fully determined by Document
- **AND** concrete endpoint locations are obtained only by evaluating expressions against a State \(p\)

### Requirement: Evaluation takes Document plus an explicit State

Evaluate, assemble, and Mesh construction SHALL take Document for identity/connectivity resolution plus an explicit State (committed or working).

#### Scenario: Working State leaves the committed State alone

- **WHEN** geometries are evaluated at a working State during a drag
- **THEN** the Document's committed State is unchanged until the drag commits

### Requirement: App owns presentation and edits through the command session

The system SHALL provide an App component that presents the active sketch of a command session (sketch-commands). The App SHALL host a Canvas that displays the sketch and delivers pointer (or equivalent) input. The App SHALL change the drawing only through the session's commands and drag operations, and SHALL NOT mutate the Document directly. Interact is hosted by the session. The App SHALL NOT embed solver algorithm details; interactive solving SHALL go through Interact and SolverPort.

#### Scenario: Canvas renders from Document state

- **WHEN** the App paints the sketch
- **THEN** geometry appearance is derived from the Document's structure evaluated at a concrete State (normally the committed \(p\))
- **AND** the Canvas does not own a second authoritative copy of structural relationships

#### Scenario: Tools edit through commands

- **WHEN** a GUI tool adds a geometry or a constraint
- **THEN** the change is applied as a session command
- **AND** it appears in the session's undo history

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

The system SHALL provide a display Mesh (or equivalent view-layer edge list) derived from Document plus a concrete State \(p\). The Mesh SHALL NOT be part of Document or Interact as authoritative structure or parameters. The Canvas SHALL obtain stroked sketch geometry from that Mesh. Until richer tessellation is needed, a Mesh whose edges correspond one-to-one with line segment geometries SHALL satisfy this requirement.

#### Scenario: Canvas paints via Mesh

- **WHEN** the App paints the sketch at the committed State
- **THEN** drawable edges come from a Mesh built from Document and that State
- **AND** the Canvas does not own a second authoritative copy of structural relationships

#### Scenario: Mesh stays outside Document authority

- **WHEN** Interact commits a new State after a successful solve
- **THEN** Document remains the sole authority for parameters and constraints
- **AND** any Mesh used for display is rebuilt or invalidated from that Document rather than written back into Document

### Requirement: Mesh hits carry geometry identity

Hit-testing against Mesh primitives MAY be used by tools; when used, hit results SHALL carry stable geometry identities consistent with the Mesh edge tags.

#### Scenario: Hit on an arc edge names the arc

- **WHEN** a tool hit-tests near an edge of an arc's Mesh
- **THEN** the hit carries the arc's geometry identity, the same identity the edge is tagged with

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

### Requirement: Pointer input maps through the Viewport

Pointer (or equivalent) input SHALL be mapped from screen to world via the Viewport before updating Document State, Interact soft targets, or issuing Mesh hit queries against world-space Mesh positions.

#### Scenario: Same click, different scale

- **WHEN** the same screen position is clicked under two Viewports with different scales
- **THEN** the world positions passed on differ according to the two Viewports

### Requirement: The view keeps its state across frames

The Viewport SHALL be a world center location and a scale (screen units per world unit) that persist across frames until the user navigates, a fit is requested, or the Canvas is resized. Edits of the drawing SHALL NOT change it.

#### Scenario: Edits do not move the view

- **WHEN** a segment is added or a point is dragged
- **THEN** the Viewport's center and scale are unchanged

### Requirement: Zoom keeps the anchor point fixed

Zooming by a factor around an anchor screen position SHALL multiply the scale by that factor and SHALL keep the world location under the anchor at the same screen position.

#### Scenario: Zoom in at the pointer

- **WHEN** the view is zoomed by factor 2 around a screen position whose world image is (3, 4)
- **THEN** the scale is doubled
- **AND** the world image of that screen position is still (3, 4)

### Requirement: Pan moves the view by a screen displacement

Panning by a screen displacement SHALL move every world location's screen image by that displacement and SHALL NOT change the scale.

#### Scenario: Pan right

- **WHEN** the view is panned by 50 screen units to the right
- **THEN** every world location appears 50 screen units further right at the same scale

### Requirement: Resizing the Canvas keeps center and scale

When the Canvas changes size, the Viewport SHALL keep the world location at the Canvas center and the scale; a larger Canvas SHALL show more of the drawing, not a larger drawing.

#### Scenario: Wider window shows more

- **WHEN** the Canvas becomes twice as wide
- **THEN** the scale is unchanged and the world location at the Canvas center is unchanged
- **AND** a world location that was just outside the left edge is now visible

### Requirement: The view can be fitted on demand

The user SHALL be able to fit the view to the drawing at any time, with the same rule as after New or Open (file-commands: The view fits a new or opened drawing): all visible geometry and the origins of visible labels inside the Canvas with a margin, or the default view for an empty drawing.

#### Scenario: Fit after zooming away

- **WHEN** the user has zoomed far into one corner and requests a fit
- **THEN** all visible geometry is inside the Canvas again

### Requirement: Zoom is bounded

The scale SHALL stay within a fixed range relative to the print-size scale. A zoom beyond that range SHALL stop at its bound and SHALL still keep the anchor point fixed.

#### Scenario: Zooming in stops

- **WHEN** the user zooms in by a large factor many times
- **THEN** the scale stops growing at the upper bound
- **AND** the world image of the anchor is unchanged

### Requirement: Navigation does not interrupt a drag

Zooming and panning SHALL be possible while a point is being dragged. They SHALL NOT end or cancel the drag; the dragged point SHALL follow the pointer's world image under the changed Viewport.

#### Scenario: Zoom during a drag

- **WHEN** the user zooms while dragging a segment end and then moves the pointer
- **THEN** the drag continues and the segment end follows the pointer's world image under the new scale
- **AND** releasing the pointer ends the drag as one history step

### Requirement: Print-size zoom and zoom display

The App SHALL offer a zoom step that shows the drawing at its physical print size (document-settings scale; 1 inch = 96 logical screen units) and SHALL display the current zoom as a percentage of that print size.

#### Scenario: One hundred percent

- **WHEN** the document scale is 10 mm per world unit and the user selects the print-size zoom
- **THEN** one world unit spans 10 × 96 / 25.4 ≈ 37.8 logical screen units
- **AND** the zoom display reads 100 %

### Requirement: Constraints are resolved by stable identity

Each structural constraint in Document SHALL have a stable constraint identity, distinct from geometry identities and from positions in the State vector. Document SHALL resolve a constraint identity to that constraint without scanning every constraint in the document for each lookup (lookup cost SHALL be independent of the number of constraints, aside from ordinary hash/map overhead).

#### Scenario: Add returns a resolvable constraint identity

- **WHEN** a structural constraint is added to Document
- **THEN** the add operation yields a stable constraint identity
- **AND** Document resolves that identity to the added constraint

#### Scenario: Remove drops constraint identity resolution

- **WHEN** a structural constraint is removed from Document
- **THEN** its identity is no longer resolvable

### Requirement: Constraint identities are fresh handles

Adding a structural constraint SHALL allocate a fresh identity; removing a constraint SHALL drop that identity from resolution. Constraints SHALL NOT be identified solely by their index in an unordered list when a stable handle is required (for example incidence queries or later UI selection).

#### Scenario: Removed identity is not handed out again

- **WHEN** a constraint is removed and another constraint is added afterwards
- **THEN** the new constraint's identity differs from the removed one's
- **AND** the identities of the other constraints are unchanged

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

### Requirement: Incidence stays consistent across edits

Removing a geometry SHALL remove every structural constraint that referenced that identity (orphan drop), consistent with geometry-ir structural-edit rules. Incidence answers SHALL remain consistent after add/remove of geometries and constraints.

#### Scenario: Removed constraint leaves incidence

- **WHEN** a Congruence between geometries A and B is removed
- **THEN** incidence for A and for B no longer lists that Congruence

### Requirement: World coordinates are y-up

Document State parameter blocks that encode planar point coordinates, and Mesh edge positions derived from them, SHALL use a right-handed sense in the plane with **+y upward** (mathematical/scientific convention). Screen toolkits that use +y downward SHALL obtain the flip only through the Viewport mapping, not by storing inverted y in committed State.

#### Scenario: Increasing world y moves toward the top of the Canvas after mapping

- **WHEN** two world points share the same x and the second has larger y
- **AND** both are mapped to screen through the Viewport
- **THEN** the second point's screen y is less than or equal to the first's when the Canvas uses +y downward

### Requirement: App presentation mode controls construction visibility and picking

The App SHALL provide a presentation mode that shows **result geometries only** or **includes construction geometries**. The mode SHALL set the visibility filter for the Mesh used in Canvas paint and Mesh-backed hit-testing. Construction geometries hidden by the mode SHALL NOT be pickable **on the Canvas**, neither via Mesh hits nor via logical handles under that filter; other views MAY still select them. Presentation mode is UI session state; geometry role remains in Document.

#### Scenario: Toggle hides and unpicks construction

- **WHEN** the user switches from include-construction mode to result-only mode
- **AND** the Canvas repaints
- **THEN** construction geometries are not stroked
- **AND** pointer hit-testing does not select those construction geometries

#### Scenario: Mode does not alter Document roles

- **WHEN** the user toggles presentation mode
- **THEN** each geometry's result vs construction role in Document is unchanged

#### Scenario: Hidden objects stay selectable by name

- **WHEN** construction geometries are hidden
- **AND** another view selects a construction geometry by name
- **THEN** the geometry is selected although it cannot be picked on the Canvas

### Requirement: Hiding objects deselects them

When a change of the presentation mode hides selected objects or objects in the active tool's slots, the App SHALL remove them from the selection and empty those slots, so that actions on the selection do not reach objects the user can no longer see on the Canvas. A logical point SHALL leave the selection when none of its class's endpoints remains visible.

#### Scenario: Switching to result only deselects construction

- **WHEN** a construction segment and a result segment are selected
- **AND** the user switches to result-only mode
- **THEN** only the result segment stays selected

### Requirement: Core is independent of host platform services

Document, expression evaluation, assembly, the SolverPort and its backends, Interact, the display Mesh, and App presentation logic SHALL NOT directly use host platform services such as file systems, process control, wall-clock timers, threads, windowing, or browser APIs. They SHALL exchange persistent or external data only as in-memory values (for example bytes or text). Each implementation SHALL be able to run the same core on every host platform it targets without changing core behavior.

#### Scenario: Same sketch behaves identically on different hosts

- **WHEN** the same Document is dragged with the same pointer inputs on two host platforms targeted by one implementation
- **THEN** the committed States after each step agree within the solver's numerical tolerance
- **AND** no core component's result depends on which host it ran on

#### Scenario: Core builds for a host without blocking I/O

- **WHEN** an implementation targets a host that offers no blocking file access and no additional threads
- **THEN** the core components still build and run on that host
- **AND** only the host boundary differs between that host and the others

### Requirement: Host services pass through a single host boundary

The App SHALL obtain every host platform service (for example opening or saving files, clipboard, time, sharing) through one replaceable host boundary supplied when the App starts. A service whose result is not available at once SHALL deliver it by completion (callback, message, or equivalent) and SHALL NOT block the interactive input-solve-paint loop while waiting.

#### Scenario: File open completes later without freezing interaction

- **WHEN** the user asks to open a file and the host shows a file picker
- **THEN** the Canvas keeps repainting and accepting input while the picker is open
- **AND** the file contents reach the App as an in-memory value once the host completes the request

#### Scenario: Host boundary is swappable at startup

- **WHEN** the App starts on a different host platform
- **THEN** only the supplied host boundary differs
- **AND** Document, Interact, and the SolverPort are created the same way as on any other host

### Requirement: Optional host capabilities are discoverable

The host boundary SHALL let the App ask at runtime whether an optional capability is available on the current host. When a capability is unavailable, the App SHALL hide or disable the UI that depends on it. Using the App SHALL NOT fail because an optional capability is missing. Label typesetting SHALL be such an optional capability, provided through the host boundary.

#### Scenario: Missing capability hides its command

- **WHEN** the current host does not provide an optional capability (for example system sharing)
- **THEN** the App does not offer the command that needs it
- **AND** all other commands keep working

#### Scenario: Available capability is offered

- **WHEN** the current host provides that optional capability
- **THEN** the App offers the corresponding command
- **AND** invoking it goes through the host boundary

#### Scenario: Typesetter comes from the host boundary

- **WHEN** the App starts on a host that provides a typesetter
- **THEN** the App obtains it from the host boundary supplied at start
- **AND** on a host without one, labels are shown in the not-typeset form and everything else works
