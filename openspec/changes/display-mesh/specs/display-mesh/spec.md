# Spec Delta

## Purpose

Defines the display Mesh: a view-layer tessellation of DocumentStructure evaluated at a concrete State into drawable edge primitives, including identity tags so hit-testing against those primitives returns stable geometry references for tools.

## ADDED Requirements

### Requirement: Mesh is derived from structure and state

The system SHALL provide a Mesh that is computed solely from DocumentStructure and a concrete State \(p\). The Mesh SHALL NOT be an authoritative store of continuous parameters or structural constraints. Rebuilding the Mesh for the same structure and State SHALL yield equivalent drawable content (same edges up to ordering conventions documented by the implementation).

#### Scenario: Mesh rebuild tracks state

- **WHEN** DocumentStructure is unchanged and State \(p\) changes endpoint coordinates of a line segment
- **AND** the Mesh is rebuilt from that structure and State
- **THEN** the Mesh edge corresponding to that geometry has the new endpoint locations
- **AND** DocumentStructure itself is not mutated by the rebuild

#### Scenario: Mesh is not a second parameter store

- **WHEN** a caller holds a Mesh built at State \(p\)
- **THEN** editing Mesh edge coordinates (if mutable in an implementation) SHALL NOT be defined as updating the Document's committed State
- **AND** the authoritative continuous values remain the State vector \(p\)

### Requirement: Mesh exposes drawable edges tagged with geometry identity

The Mesh SHALL expose a collection of drawable edges. Each edge SHALL provide two endpoint locations (points) suitable for stroking a line segment in the plane, and SHALL carry the stable geometry identity of the geometry that produced it. For geometries tessellated into multiple edges in later slices, every edge of that geometry SHALL carry the same geometry identity.

#### Scenario: Segment becomes one tagged edge

- **WHEN** a Document contains a single line segment geometry and the Mesh is built at the committed State
- **THEN** the Mesh contains one edge whose endpoints match the segment's evaluated endpoints
- **AND** that edge's geometry identity equals the segment's geometry identity

### Requirement: Mesh supports endpoint hit queries

The Mesh SHALL support a hit query that, given a query point and a radius in the same coordinate space as Mesh positions, returns the nearest Mesh endpoint within that radius together with enough identity to form an endpoint reference (geometry identity and which end: start or end), or indicates no hit. Endpoint classification (start vs end) SHALL match the endpoints capability of the underlying geometry for geometries that expose endpoints.

#### Scenario: Hit near a segment start

- **WHEN** a Mesh is built from a document containing a line segment
- **AND** a hit query is issued near the segment's start endpoint within the hit radius
- **THEN** the query returns that geometry's identity and the start endpoint kind
- **AND** a query far from all Mesh endpoints returns no hit

### Requirement: Mesh supports geometry hit queries via edges

The Mesh SHALL support a hit query that, given a query point and a radius, returns the geometry identity of the nearest Mesh edge whose distance to the query point is within the radius, or indicates no hit. Distance SHALL be measured to the edge as a finite line segment in the plane (closest point on the segment).

#### Scenario: Hit near the middle of an edge

- **WHEN** a Mesh contains an edge for a line segment geometry
- **AND** a hit query is issued near the midpoint of that edge within the hit radius
- **THEN** the query returns that edge's geometry identity

### Requirement: Canvas paints from Mesh edges

Painting the sketch on the Canvas SHALL stroke Mesh edges (evaluated at the State being displayed, normally the committed State) rather than iterating Document geometries by kind. The Canvas MAY obtain logical handle points from point expressions separately from Mesh edges.

#### Scenario: Paint does not require segment iteration

- **WHEN** the App paints the sketch
- **THEN** stroked path geometry is taken from Mesh edges
- **AND** the paint path SHALL NOT require matching on line-segment geometry kinds in the Document
