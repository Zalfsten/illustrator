# Spec Delta: display-mesh

## MODIFIED Requirements

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

## ADDED Requirements

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

Curve edges SHALL be placed according to the curve's curvature along its whole length: shorter edges where the curvature is larger, longer edges where it is smaller, and a single edge where the curvature is zero. Edge placement SHALL NOT depend only on the curve's tangents at the ends of a parameter interval.

#### Scenario: Zero curvature is one edge

- **WHEN** a line segment is meshed
- **THEN** it contributes one edge

#### Scenario: Equal tangents at the ends do not hide the curve

- **WHEN** an arc with sweep 360° minus a small angle is meshed, so that its tangents at both ends almost agree
- **THEN** it contributes as many edges as the chord tolerance requires for its curvature and length, not a single chord
