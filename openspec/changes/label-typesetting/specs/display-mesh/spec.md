# Spec Delta

## MODIFIED Requirements

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
