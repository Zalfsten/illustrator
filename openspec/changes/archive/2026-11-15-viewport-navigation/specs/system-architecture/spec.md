## ADDED Requirements

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
