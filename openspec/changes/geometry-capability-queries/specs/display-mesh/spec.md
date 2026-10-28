# Spec Delta

## MODIFIED Requirements

### Requirement: Mesh exposes drawable edges tagged with geometry identity

The Mesh SHALL expose a collection of drawable edges. Each edge SHALL provide two endpoint locations (points) in **document/world coordinates** (the same space as State-evaluated geometry positions), suitable for stroking after a view transform, and SHALL carry the stable geometry identity of the geometry that produced it and that geometry's result/construction role (for presentation). For geometries tessellated into multiple edges in later slices, every edge of that geometry SHALL carry the same geometry identity. Mesh positions SHALL NOT be defined as window-pixel coordinates. Edge vertex locations SHALL be obtained by evaluating point expressions of the curve capability (on-curve samples, and endpoint expressions where the geometry has the endpoints capability) at the State used to build the Mesh, not by bypassing the expression evaluate path with geometry-only State sampling APIs. Tessellating a curve SHALL NOT require the endpoints capability.

#### Scenario: Segment becomes one tagged edge

- **WHEN** a Document contains a single line segment geometry and the Mesh is built at the committed State
- **THEN** the Mesh contains one edge whose endpoints match the segment's evaluated endpoints in world coordinates
- **AND** that edge's geometry identity equals the segment's geometry identity

#### Scenario: Curve without endpoints is tessellated

- **WHEN** a Document contains a curve geometry whose kind has the curve capability but not the endpoints capability
- **AND** the Mesh is built at the committed State
- **THEN** the Mesh contains one or more edges tagged with that geometry's identity
- **AND** their vertices are on-curve samples of that geometry

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
