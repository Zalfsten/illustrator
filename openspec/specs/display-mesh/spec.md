# display-mesh

## Purpose

Defines the display Mesh: a view-layer tessellation of Document evaluated at a concrete State into drawable edge primitives, including identity tags so hit-testing against those primitives returns stable geometry references for tools.

## Requirements

### Requirement: Mesh is derived from structure and state

The system SHALL provide a Mesh that is computed solely from Document and a concrete State \(p\). The Mesh SHALL NOT be an authoritative store of continuous parameters or structural constraints. Rebuilding the Mesh for the same structure and State SHALL yield equivalent drawable content (same edges up to ordering conventions documented by the implementation).

#### Scenario: Mesh rebuild tracks state

- **WHEN** Document is unchanged and State \(p\) changes endpoint coordinates of a line segment
- **AND** the Mesh is rebuilt from that structure and State
- **THEN** the Mesh edge corresponding to that geometry has the new endpoint locations
- **AND** Document itself is not mutated by the rebuild

#### Scenario: Mesh is not a second parameter store

- **WHEN** a caller holds a Mesh built at State \(p\)
- **THEN** editing Mesh edge coordinates (if mutable in an implementation) SHALL NOT be defined as updating the Document's committed State
- **AND** the authoritative continuous values remain the State vector \(p\)

### Requirement: Mesh exposes drawable edges tagged with geometry identity

The Mesh SHALL expose a collection of drawable edges. Each edge SHALL provide two endpoint locations (points) in **document/world coordinates** (the same space as State-evaluated geometry positions), suitable for stroking after a view transform, and SHALL carry the stable geometry identity of the geometry that produced it and that geometry's result/construction role (for presentation). For geometries tessellated into multiple edges in later slices, every edge of that geometry SHALL carry the same geometry identity. Mesh positions SHALL NOT be defined as window-pixel coordinates. Edge endpoint locations SHALL be obtained by evaluating the corresponding endpoint point expressions (or equivalent) at the State used to build the Mesh, not by bypassing the expression evaluate path with geometry-only State sampling APIs.

#### Scenario: Segment becomes one tagged edge

- **WHEN** a Document contains a single line segment geometry and the Mesh is built at the committed State
- **THEN** the Mesh contains one edge whose endpoints match the segment's evaluated endpoints in world coordinates
- **AND** that edge's geometry identity equals the segment's geometry identity

### Requirement: Circular arcs tessellate into polyline Mesh edges

When the Document contains a circular arc geometry that passes the Mesh visibility filter, building the Mesh at State \(p\) SHALL emit one or more drawable edges that approximate the arc as a polyline in world coordinates. Every edge produced for that arc SHALL carry the same geometry identity and role. The polyline SHALL begin at the arc's evaluated start endpoint and end at its evaluated end endpoint. Subdivision density MAY depend on the absolute sweep; a single chord MAY be used only when that remains a faithful display of a near-degenerate sweep. Segment geometries SHALL continue to contribute a single edge between their endpoints.

#### Scenario: Arc becomes multiple tagged edges

- **WHEN** a Document contains a single circular arc with a non-trivial sweep and the Mesh is built at the committed State
- **THEN** the Mesh contains more than one edge tagged with that arc's geometry identity
- **AND** the first polyline vertex matches the arc start endpoint and the last matches the arc end endpoint (within ordinary floating tolerance)

#### Scenario: Segment still one edge beside an arc

- **WHEN** a Document contains one line segment and one circular arc
- **AND** the Mesh is built at the committed State
- **THEN** the segment contributes exactly one edge
- **AND** the arc contributes one or more edges all tagged with the arc's identity

### Requirement: Mesh supports endpoint hit queries

The Mesh SHALL support a hit query that, given a query point and a radius in the **same world coordinate space** as Mesh positions, returns the nearest Mesh endpoint within that radius together with enough identity to form an endpoint reference (geometry identity and which end: start or end), or indicates no hit. Endpoint classification (start vs end) SHALL match the endpoints capability of the underlying geometry for geometries that expose endpoints. Converting a screen-pixel pick radius into world length is outside this Mesh requirement (App / Viewport concern).

#### Scenario: Hit near a segment start

- **WHEN** a Mesh is built from a document containing a line segment
- **AND** a hit query is issued near the segment's start endpoint within the hit radius (world units)
- **THEN** the query returns that geometry's identity and the start endpoint kind
- **AND** a query far from all Mesh endpoints returns no hit

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

Painting the sketch on the Canvas SHALL stroke Mesh edges (evaluated at the State being displayed, normally the committed State) rather than iterating Document geometries by kind. The Canvas MAY obtain logical handle points from point expressions separately from Mesh edges. Logical handle points shown for picking or decoration SHALL NOT include endpoints that belong only to geometries excluded by the same visibility filter used for the Mesh being painted (invisible ⇒ not presented as pick targets).

#### Scenario: Paint does not require segment iteration

- **WHEN** the App paints the sketch
- **THEN** stroked path geometry is taken from Mesh edges
- **AND** the paint path SHALL NOT require matching on line-segment geometry kinds in the Document

#### Scenario: Hidden construction is not presented as a handle

- **WHEN** the App paints in a mode that excludes construction geometries from the Mesh
- **AND** a point expression refers only to a construction geometry endpoint
- **THEN** that expression is not shown as a pickable handle in that paint
