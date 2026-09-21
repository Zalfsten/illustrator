# Spec Delta

## ADDED Requirements

### Requirement: Mesh respects presentation visibility filter

Building a Mesh for display or hit-testing SHALL accept a visibility filter that selects which geometries contribute edges. At minimum the filter SHALL support excluding construction-role geometries. Edges SHALL be emitted only for geometries that pass the filter. Mesh hit queries (`hit_endpoint`, `hit_geometry`, and equivalent) SHALL operate only on edges present in that Mesh; a geometry omitted by the filter SHALL NOT be returned by those queries.

#### Scenario: Result-only Mesh omits construction edges

- **WHEN** DocumentStructure contains a construction geometry and a result geometry
- **AND** a Mesh is built with a filter that excludes construction geometries
- **THEN** the Mesh contains no edge tagged with the construction geometry's identity
- **AND** a hit query near that construction geometry's evaluated location returns no hit for that geometry

#### Scenario: Include-construction Mesh keeps construction pickable

- **WHEN** the same document is meshed with a filter that includes construction geometries
- **THEN** the Mesh contains an edge for the construction geometry
- **AND** a hit query near that edge within radius can return that geometry's identity

## MODIFIED Requirements

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
