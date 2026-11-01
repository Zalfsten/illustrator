# drawing-export Specification

## Purpose

Defines how a drawing leaves the app as a standalone vector file for publications, slides and web pages: what is included, how exactly curves, styles and labels are written, the file's physical size and bounds, and the export command in the app.

## Requirements

### Requirement: Only visible result objects are exported

An export SHALL contain every geometry with the result role that has the curve capability, and every label with the result role. It SHALL NOT contain construction geometry, construction labels, constraint glyphs, handles, selection or hover highlights, or any other editor decoration.

#### Scenario: Construction geometry is left out

- **WHEN** a drawing with a result segment and a construction segment is exported
- **THEN** the file contains the result segment's stroke
- **AND** nothing of the construction segment

#### Scenario: Construction label is left out

- **WHEN** a drawing with a result label and a construction label is exported
- **THEN** the file contains the outlines of the result label only

### Requirement: Curves are written from their exact shape

An export SHALL write each curve from its exact shape (geometry-ir), at the committed State. It SHALL NOT write the display mesh or any other display tessellation. A straight piece SHALL be written as a straight path piece. A circular-arc piece SHALL be written as a path piece of that circle with its exact center, radius and end points, in every format that has such a piece.

#### Scenario: Arc is not a polyline

- **WHEN** a drawing with one result circular arc is exported as SVG
- **THEN** the arc's path consists of elliptical-arc commands with equal radii equal to the arc's radius in output units
- **AND** it contains no straight pieces

#### Scenario: Segment is one straight piece

- **WHEN** a drawing with one solid result segment without arrows is exported
- **THEN** its path is one move to the start point and one straight piece to the end point

### Requirement: Strokes are written in their resolved style

Each exported curve SHALL be drawn with its resolved style (styling) at its physical size. The style SHALL set the stroke colour and width, the flat stroke ends, the dash pattern laid along the arc length from the curve's start, and the filled arrow heads with the stroke ending at their base. These SHALL follow the exact curve.

#### Scenario: Width and colour at physical size

- **WHEN** a result segment with resolved width 0.8 pt and colour `#c00000` is exported
- **THEN** its stroke has colour `#c00000` and width 0.8 × 25.4/72.27 mm

#### Scenario: Arrow at the end

- **WHEN** a result segment from (0, 0) to (3, 0) with arrow `end` and width w is exported
- **THEN** the file contains a filled triangle with its tip at the segment's end point and length 3 pt + 4.5 w
- **AND** the segment's stroke ends at the triangle's base

#### Scenario: Dash phase starts at the curve start

- **WHEN** a dashed result segment with arrow `start` is exported
- **THEN** the dash pattern along the drawn stroke matches the pattern laid from the curve's start point, not from the arrow's base

### Requirement: Labels are written as outlines

An exported label SHALL be written as the outlines its typesetting delivers (label-typesetting), placed as in the Canvas (labels: frame, reference point, alignment, offset, document scale). Filled outlines SHALL be filled, stroked outlines SHALL be stroked with their width. Both SHALL use the label's resolved colour. The file SHALL NOT need fonts to show the labels.

#### Scenario: Label outlines in style colour

- **WHEN** a result label with source `$x$` and resolved colour `#0000ff` is exported
- **THEN** the file contains its glyph outlines filled with `#0000ff` at the label's placement
- **AND** the file contains no text element and refers to no font

#### Scenario: Empty label adds nothing

- **WHEN** a result label with an empty source is exported together with a segment
- **THEN** the file contains only the segment

### Requirement: Export waits for labels being typeset

When an export is requested while lines of exported labels are still being typeset, the export SHALL wait until all of them are delivered and only then produce the file. While it waits, the app SHALL stay usable and SHALL show that the export is waiting for labels.

#### Scenario: Export after typesetting finishes

- **WHEN** an export is requested while the typesetter is still loading
- **THEN** no file is produced yet and the app shows that the export waits for labels
- **AND** once all lines are delivered the export continues with the typeset outlines

### Requirement: Export fails for labels that cannot be typeset

When an exported label fails to typeset, when no typesetter is available, or when the typesetter failed to load, the export SHALL produce no file. It SHALL report an error that names each such label and the reason (label-typesetting).

#### Scenario: Unsupported command

- **WHEN** a drawing with a result label `lbl3` whose source uses an unsupported command is exported
- **THEN** no file is produced
- **AND** the error names `lbl3` and the unsupported command

#### Scenario: Construction label does not block

- **WHEN** only a construction label fails to typeset and the drawing is exported
- **THEN** the export succeeds without that label

### Requirement: Exports have physical size

An exported file SHALL state its size in millimetres. One world unit SHALL be the document scale in millimetres (document-settings), and lengths in pt SHALL be converted as 1 pt = 25.4/72.27 mm. The file SHALL keep the drawing's orientation: world +x to the right and world +y up on the page.

#### Scenario: Size follows the document scale

- **WHEN** a drawing with scale 10 and one solid horizontal segment of length 3 world units, width 0.4 pt and no arrows is exported
- **THEN** the file is 30 mm wide and 0.4 × 25.4/72.27 mm high

#### Scenario: Up stays up

- **WHEN** a drawing with a segment from (0, 0) to (0, 1) with arrow `end` is exported
- **THEN** the arrow head points to the top of the page

### Requirement: Export bounds are tight

The exported area SHALL be the smallest axis-aligned rectangle that contains all drawn shapes: strokes with their width, arrow heads and label outlines. A dashed stroke SHALL count as its undashed stroke. There SHALL be no margin. Each side SHALL touch a drawn shape within the file's numeric precision.

#### Scenario: Arc bounds use the arc, not the circle

- **WHEN** a quarter arc of radius 1 from heading 0 to heading π/2 around (0, 0) with width w and no arrows is exported
- **THEN** the exported area spans the quarter arc with half the width on each side of the curve
- **AND** it does not include the other three quarters of the circle

#### Scenario: Arrow head widens the bounds

- **WHEN** a horizontal segment with arrow `end` is exported
- **THEN** the exported height is the arrow head's base width 0.75 L, not the stroke width

### Requirement: Nothing visible is not exported

When a drawing has no exported object with drawn shapes, the export SHALL produce no file and SHALL report that there is nothing to export.

#### Scenario: Only construction geometry

- **WHEN** a drawing with only construction geometry is exported
- **THEN** no file is produced and the app reports that there is nothing to export

### Requirement: Exports are deterministic

Exporting the same drawing with the same typeset label lines SHALL produce the same bytes. The order of objects in the file SHALL be the order in which they were created.

#### Scenario: Export twice

- **WHEN** a drawing is exported twice without a change in between
- **THEN** both files have identical bytes

### Requirement: SVG form

The SVG form SHALL be one standalone SVG document. Its width and height SHALL be in millimetres, and its view box SHALL use millimetres as user units. It SHALL contain only path elements and their grouping. It SHALL contain no text elements, and it SHALL refer to no external resource such as a font, image or stylesheet.

#### Scenario: Standalone file

- **WHEN** a drawing with a segment, an arc and a label is exported as SVG
- **THEN** the file is a valid SVG document with width and height in `mm`
- **AND** every drawn shape is a path element, and the file refers to no external resource

### Requirement: Export command

The app SHALL offer exporting the drawing as SVG. It SHALL ask the host for a target, suggesting the drawing's file name with its sketch extension replaced by `.svg`, or `untitled.svg` when the drawing has no file. A host that cannot write to a chosen file SHALL hand the bytes to the user as a download under the suggested name. If the user cancels the host's request, nothing SHALL be written. A write error SHALL be shown.

#### Scenario: Suggested name from the drawing's file

- **WHEN** a drawing whose file is `triangle.sketch.json` is exported
- **THEN** the host is asked for a target with suggested name `triangle.svg`

#### Scenario: Drawing without a file

- **WHEN** a drawing without a file is exported
- **THEN** the suggested name is `untitled.svg`

### Requirement: Export leaves the drawing unchanged

An export SHALL NOT change the drawing, its history, its file, or its unsaved-changes state (file-commands). The file SHALL show the drawing as it was when the export was requested, even when the drawing is edited while the export waits.

#### Scenario: Unsaved state is kept

- **WHEN** a drawing with unsaved changes is exported successfully
- **THEN** it still has unsaved changes, and its file and title are unchanged

#### Scenario: Edit while waiting

- **WHEN** the user moves a segment while an export waits for labels
- **THEN** the exported file shows the segment where it was when the export was requested

### Requirement: Export respects drags and pending file requests

An export SHALL NOT start while a drag is in progress or while another file request of the app is pending. It SHALL NOT block input, solving or painting while it waits for labels or for the host.

#### Scenario: Export during a drag

- **WHEN** the user asks for an export while a drag is in progress
- **THEN** no export starts
