# display-mesh

## Purpose

Defines the display Mesh: a view-layer tessellation of Document evaluated at a concrete State into drawable edge primitives, including identity tags so hit-testing against those primitives returns stable geometry references for tools.

## Requirements

### Requirement: Mesh is derived from structure and state

The system SHALL provide a Mesh that is computed solely from Document, a concrete State \(p\) and the Mesh building options (the visibility filter and the chord tolerance). The Mesh SHALL NOT be an authoritative store of continuous parameters or structural constraints. Rebuilding the Mesh for the same structure, State and building options SHALL yield equivalent drawable content (same edges up to ordering conventions documented by the implementation).

#### Scenario: Mesh rebuild tracks state

- **WHEN** Document is unchanged and State \(p\) changes endpoint coordinates of a line segment
- **AND** the Mesh is rebuilt from that structure and State
- **THEN** the Mesh edge corresponding to that geometry has the new endpoint locations
- **AND** Document itself is not mutated by the rebuild

#### Scenario: Mesh is not a second parameter store

- **WHEN** a caller holds a Mesh built at State \(p\)
- **THEN** editing Mesh edge coordinates (if mutable in an implementation) SHALL NOT be defined as updating the Document's committed State
- **AND** the authoritative continuous values remain the State vector \(p\)

#### Scenario: Same options give the same Mesh

- **WHEN** the Mesh is built twice from the same Document and State with the same visibility filter and chord tolerance
- **THEN** both Meshes contain the same edges

### Requirement: Mesh exposes drawable edges tagged with geometry identity

The Mesh SHALL expose a collection of drawable edges. Each edge SHALL provide two endpoint locations (points) in **document/world coordinates** (the same space as State-evaluated geometry positions), suitable for stroking after a view transform, and SHALL carry the stable geometry identity of the geometry that produced it and that geometry's result/construction role (for presentation). For geometries tessellated into multiple edges in later slices, every edge of that geometry SHALL carry the same geometry identity. Mesh positions SHALL NOT be defined as window-pixel coordinates. Edge vertex locations SHALL be obtained by evaluating point expressions of the curve capability (on-curve samples, and endpoint expressions where the geometry has the endpoints capability) at the State used to build the Mesh, not by bypassing the expression evaluate path with geometry-only State sampling APIs. Tessellating a curve SHALL NOT require the endpoints capability. Geometries without the curve capability (labels) SHALL contribute no edges and no endpoint hit targets.

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

When the Document contains a circular arc geometry that passes the Mesh visibility filter, building the Mesh at State \(p\) SHALL emit one or more drawable edges that approximate the arc as a polyline in world coordinates. Every edge produced for that arc SHALL carry the same geometry identity and role. The polyline SHALL begin at the arc's evaluated start endpoint and end at its evaluated end endpoint. The edges of an arc SHALL be of equal length, and their number SHALL be the smallest that keeps the arc within the chord tolerance (see *Curve edges stay within the chord tolerance*), unless the per-curve edge cap is reached. Segment geometries SHALL continue to contribute a single edge between their endpoints.

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

The Mesh SHALL support a hit query that, given a query point and a radius in the **same world coordinate space** as Mesh positions, returns the nearest Mesh endpoint within that radius together with enough identity to form an endpoint reference (geometry identity and which end: start or end), or indicates no hit. Endpoint classification (start vs end) SHALL match the endpoints capability of the underlying geometry for geometries that expose endpoints. Geometries without the endpoints capability SHALL NOT contribute endpoint hit targets. Converting a screen-pixel pick radius into world length is outside this Mesh requirement (App / Viewport concern).

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

### Requirement: Mesh supports geometry hit queries via edges

The Mesh SHALL support a hit query that, given a query point and a radius in world coordinates, returns the geometry identity of the nearest Mesh edge whose distance to the query point is within the radius, or indicates no hit. Distance SHALL be measured to the edge as a finite line segment in the plane (closest point on the segment).

#### Scenario: Hit near a segment interior

- **WHEN** a Mesh is built from a document containing a line segment
- **AND** a hit query is issued near the midpoint of that segment within the hit radius (world units)
- **THEN** the query returns that segment's geometry identity
- **AND** a query farther than the radius from every Mesh edge returns no hit

### Requirement: Mesh respects presentation visibility filter

Building a Mesh for display or hit-testing SHALL accept a visibility filter that selects which geometries contribute edges. At minimum the filter SHALL support excluding construction-role geometries. Edges SHALL be emitted only for geometries that pass the filter. Mesh hit queries (`hit_endpoint`, `hit_geometry`, and equivalent) SHALL operate only on edges present in that Mesh; a geometry omitted by the filter SHALL NOT be returned by those queries.

#### Scenario: Result-only Mesh omits construction edges

- **WHEN** Document contains a construction geometry and a result geometry
- **AND** a Mesh is built with a filter that excludes construction geometries
- **THEN** the Mesh contains no edge tagged with the construction geometry's identity
- **AND** a hit query near that construction geometry's evaluated location returns no hit for that geometry

#### Scenario: Include-construction Mesh keeps construction pickable

- **WHEN** the same document is meshed with a filter that includes construction geometries
- **THEN** the Mesh contains an edge for the construction geometry
- **AND** a hit query near that edge within radius can return that geometry's identity

### Requirement: Canvas paints from Mesh edges

Painting the sketch on the Canvas SHALL stroke Mesh edges (evaluated at the State being displayed, normally the committed State) rather than iterating Document geometries by kind. Labels, which contribute no Mesh edges, SHALL be painted from their typesetting result and placement instead (label-typesetting). The Canvas MAY obtain logical handle points from point expressions separately from Mesh edges. Logical handle points shown for picking or decoration SHALL NOT include endpoints that belong only to geometries excluded by the same visibility filter used for the Mesh being painted (invisible ⇒ not presented as pick targets).

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

### Requirement: Mesh building takes a chord tolerance

Building a Mesh SHALL take a chord tolerance: a finite length greater than zero, in world units, that bounds how far the polyline of a curve may deviate from the curve. Converting a view scale (for example a fraction of a screen pixel) into this world length is outside this requirement (App / Viewport concern); Mesh positions stay in world coordinates.

#### Scenario: Finer tolerance gives at least as many edges

- **WHEN** the same Document and State are meshed with chord tolerances ε and ε / 4
- **THEN** every curve has at least as many edges in the second Mesh as in the first

#### Scenario: Tolerance does not change segments

- **WHEN** a Document with one line segment is meshed with any chord tolerance
- **THEN** the segment contributes exactly one edge between its endpoints

### Requirement: Curve edges stay within the chord tolerance

For every curve in a Mesh, every point of the curve between two consecutive polyline vertices SHALL lie within the chord tolerance of the edge joining them, unless the curve's edge count has reached the per-curve edge cap. The cap SHALL be documented by the implementation and SHALL be at least 256 edges per curve.

#### Scenario: Large arc stays within tolerance

- **WHEN** an arc with radius 20 and sweep 180° is meshed with chord tolerance 0.005
- **THEN** every on-curve sample of the arc lies within 0.005 of the edge between the vertices it falls between

#### Scenario: Cap bounds the edge count

- **WHEN** a curve would need more edges than the per-curve edge cap to stay within the chord tolerance
- **THEN** it contributes exactly the cap's number of edges
- **AND** its polyline still begins at the curve's start and ends at its end

### Requirement: Curve tessellation follows curvature

Curve edges SHALL be placed according to the curve's curvature along its whole length: shorter edges where the curvature is larger, longer edges where it is smaller, and a single edge where the curvature is zero. Edge placement SHALL NOT depend only on the curve's tangents at the ends of a parameter interval, nor only on curvature sampled at isolated points. Refining the edges of one part of a curve SHALL NOT add edges to parts that already stay within the chord tolerance.

#### Scenario: Zero curvature is one edge

- **WHEN** a line segment is meshed
- **THEN** it contributes one edge

#### Scenario: Equal tangents at the ends do not hide the curve

- **WHEN** an arc with sweep 360° minus a small angle is meshed, so that its tangents at both ends almost agree
- **THEN** it contributes as many edges as the chord tolerance requires for its curvature and length, not a single chord

#### Scenario: Straight stretch beside a curved part

- **WHEN** a curve that is straight on its first part and circular on its second part is meshed with a chord tolerance small against the circle's radius
- **THEN** its edge count stays far below the per-curve edge cap
- **AND** the straight part is covered by fewer edges than the circular part

#### Scenario: A narrow curvature spike is not missed

- **WHEN** a curve whose curvature is small except on a piece much narrower than its edges would otherwise be is meshed
- **THEN** every point of the curve lies within the chord tolerance of its edge, unless the edge cap is reached
