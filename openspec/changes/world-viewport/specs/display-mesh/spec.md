# Spec Delta

## MODIFIED Requirements

### Requirement: Mesh exposes drawable edges tagged with geometry identity

The Mesh SHALL expose a collection of drawable edges. Each edge SHALL provide two endpoint locations (points) in **document/world coordinates** (the same space as State-evaluated geometry positions), suitable for stroking after a view transform, and SHALL carry the stable geometry identity of the geometry that produced it. For geometries tessellated into multiple edges in later slices, every edge of that geometry SHALL carry the same geometry identity. Mesh positions SHALL NOT be defined as window-pixel coordinates.

#### Scenario: Segment becomes one tagged edge

- **WHEN** a Document contains a single line segment geometry and the Mesh is built at the committed State
- **THEN** the Mesh contains one edge whose endpoints match the segment's evaluated endpoints in world coordinates
- **AND** that edge's geometry identity equals the segment's geometry identity

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
