## MODIFIED Requirements

### Requirement: Mesh exposes drawable edges tagged with geometry identity

The Mesh SHALL expose a collection of drawable edges. Each edge SHALL provide two endpoint locations (points) in **document/world coordinates** (the same space as State-evaluated geometry positions), suitable for stroking after a view transform, and SHALL carry the stable geometry identity of the geometry that produced it and that geometry's result/construction role (for presentation). Mesh positions SHALL NOT be defined as window-pixel coordinates.

#### Scenario: Segment becomes one tagged edge

- **WHEN** a Document contains a single line segment geometry and the Mesh is built at the committed State
- **THEN** the Mesh contains one edge whose endpoints match the segment's evaluated endpoints in world coordinates
- **AND** that edge's geometry identity equals the segment's geometry identity

#### Scenario: Curve without endpoints is tessellated

- **WHEN** a Document contains a curve geometry whose kind has the curve capability but not the endpoints capability
- **AND** the Mesh is built at the committed State
- **THEN** the Mesh contains one or more edges tagged with that geometry's identity
- **AND** their vertices are on-curve samples of that geometry

#### Scenario: Label contributes no edges

- **WHEN** a Document contains one segment and one label and the Mesh is built
- **THEN** the Mesh contains exactly the segment's edge
- **AND** no edge or endpoint hit target carries the label's identity

### Requirement: Circular arcs tessellate into polyline Mesh edges

When the Document contains a circular arc geometry that passes the Mesh visibility filter, building the Mesh at State \(p\) SHALL emit one or more drawable edges that approximate the arc as a polyline in world coordinates. Every edge produced for that arc SHALL carry the same geometry identity and role. The polyline SHALL begin at the arc's evaluated start endpoint and end at its evaluated end endpoint. Segment geometries SHALL continue to contribute a single edge between their endpoints.

#### Scenario: Arc becomes multiple tagged edges

- **WHEN** a Document contains a single circular arc whose sagitta over its whole sweep exceeds the chord tolerance, and the Mesh is built at the committed State
- **THEN** the Mesh contains more than one edge tagged with that arc's geometry identity
- **AND** the first polyline vertex matches the arc start endpoint and the last matches the arc end endpoint (within ordinary floating tolerance)

#### Scenario: Segment still one edge beside an arc

- **WHEN** a Document contains one line segment and one circular arc
- **AND** the Mesh is built at the committed State
- **THEN** the segment contributes exactly one edge
- **AND** the arc contributes one or more edges all tagged with the arc's identity

#### Scenario: Arc edges are equal and as few as the tolerance allows

- **WHEN** an arc with radius R and sweep Δθ is meshed with chord tolerance ε, where ε is small against R
- **THEN** all of its edges have the same length within floating tolerance
- **AND** each edge's sagitta R (1 − cos(Δθ / 2n)) for n edges is at most ε
- **AND** with one edge fewer the sagitta would exceed ε

#### Scenario: Larger radius gives more edges

- **WHEN** two arcs with the same sweep and radii 0.5 and 20 are meshed with the same chord tolerance
- **THEN** the arc with radius 20 has more edges than the arc with radius 0.5

#### Scenario: Edge count grows in single steps

- **WHEN** an arc of fixed radius is meshed with a fixed chord tolerance at sweeps increasing in small steps from 80° to 100°
- **THEN** the edge count never decreases
- **AND** consecutive edge counts differ by at most one

### Requirement: Mesh supports endpoint hit queries

The Mesh SHALL support a hit query that, given a query point and a radius in the **same world coordinate space** as Mesh positions, returns the nearest Mesh endpoint within that radius together with enough identity to form an endpoint reference (geometry identity and which end: start or end), or indicates no hit. Converting a screen-pixel pick radius into world length is outside this Mesh requirement (App / Viewport concern).

#### Scenario: Hit near a segment start

- **WHEN** a Mesh is built from a document containing a line segment
- **AND** a hit query is issued near the segment's start endpoint within the hit radius (world units)
- **THEN** the query returns that geometry's identity and the start endpoint kind
- **AND** a query far from all Mesh endpoints returns no hit

#### Scenario: Curve without endpoints yields no endpoint hit

- **WHEN** a Mesh is built from a document containing only a curve geometry without the endpoints capability
- **AND** an endpoint hit query is issued at the location of its normalized \(t = 0\) sample
- **THEN** the query returns no hit
- **AND** a geometry hit query at that location returns that geometry's identity

### Requirement: Canvas paints from Mesh edges

Painting the sketch on the Canvas SHALL stroke Mesh edges (evaluated at the State being displayed, normally the committed State) rather than iterating Document geometries by kind. Labels, which contribute no Mesh edges, SHALL be painted from their typesetting result and placement instead (label-typesetting).

#### Scenario: Paint does not require segment iteration

- **WHEN** the App paints the sketch
- **THEN** stroked path geometry is taken from Mesh edges
- **AND** the paint path SHALL NOT require matching on line-segment geometry kinds in the Document

#### Scenario: Hidden construction is not presented as a handle

- **WHEN** the App paints in a mode that excludes construction geometries from the Mesh
- **AND** a point expression refers only to a construction geometry endpoint
- **THEN** that expression is not shown as a pickable handle in that paint

#### Scenario: Labels are painted beside the Mesh

- **WHEN** the App paints a drawing with a segment and a label
- **THEN** the segment is stroked from its Mesh edge
- **AND** the label is painted from its typeset lines at its placement

## ADDED Requirements

### Requirement: All edges of a geometry share its identity

For geometries tessellated into multiple edges in later slices, every edge of that geometry SHALL carry the same geometry identity.

#### Scenario: Arc edges share one identity

- **WHEN** an arc is meshed into several edges
- **THEN** every one of those edges carries the arc's geometry identity

### Requirement: Mesh vertices come from point expressions

Edge vertex locations SHALL be obtained by evaluating point expressions of the curve capability (on-curve samples, and endpoint expressions where the geometry has the endpoints capability) at the State used to build the Mesh, not by bypassing the expression evaluate path with geometry-only State sampling APIs. Tessellating a curve SHALL NOT require the endpoints capability.

#### Scenario: Vertices follow a working State

- **WHEN** the Mesh of an arc is built from a working State that differs from the committed State
- **THEN** every vertex equals an on-curve or endpoint expression of the arc evaluated at that working State

### Requirement: Geometries without the curve capability have no Mesh presence

Geometries without the curve capability (labels) SHALL contribute no edges and no endpoint hit targets.

#### Scenario: Label alone gives an empty Mesh

- **WHEN** a Document contains only a label and the Mesh is built
- **THEN** the Mesh has no edges
- **AND** an endpoint hit query at the label's origin returns no hit

### Requirement: Arc edges are equal and minimal in number

The edges of an arc SHALL be of equal length, and their number SHALL be the smallest that keeps the arc within the chord tolerance (see *Curve edges stay within the chord tolerance*), unless the per-curve edge cap is reached.

#### Scenario: Half circle at tolerance 0.01

- **WHEN** a half circle of radius 1 is meshed with chord tolerance 0.01
- **THEN** it has 12 edges of equal length, since 11 edges would have a sagitta above 0.01

### Requirement: Endpoint hits follow the endpoints capability

Endpoint classification (start vs end) SHALL match the endpoints capability of the underlying geometry for geometries that expose endpoints. Geometries without the endpoints capability SHALL NOT contribute endpoint hit targets.

#### Scenario: Arc end is classified as its end

- **WHEN** a Mesh is built from a document containing an arc with negative sweep
- **AND** an endpoint hit query is issued at the location of the arc's end endpoint expression
- **THEN** the query returns the arc's identity and the end endpoint kind

### Requirement: Handles follow the Mesh visibility filter

The Canvas MAY obtain logical handle points from point expressions separately from Mesh edges. Logical handle points shown for picking or decoration SHALL NOT include endpoints that belong only to geometries excluded by the same visibility filter used for the Mesh being painted (invisible ⇒ not presented as pick targets).

#### Scenario: Point shared with a visible geometry stays a handle

- **WHEN** the end of a result segment is joined to the start of a construction segment
- **AND** the App paints in a mode that excludes construction geometries
- **THEN** the joined point is still presented as a handle, because it belongs to the visible result segment
