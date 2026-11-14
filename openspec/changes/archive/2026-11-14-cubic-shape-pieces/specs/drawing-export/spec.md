## MODIFIED Requirements

### Requirement: Export bounds are tight

The exported area SHALL be the smallest axis-aligned rectangle that contains all drawn shapes: strokes with their width, arrow heads and label outlines. A dashed stroke SHALL count as its undashed stroke. There SHALL be no margin. Each side SHALL touch a drawn shape within the file's numeric precision.

#### Scenario: Arc bounds use the arc, not the circle

- **WHEN** a quarter arc of radius 1 from heading 0 to heading π/2 around (0, 0) with width w and no arrows is exported
- **THEN** the exported area spans the quarter arc with half the width on each side of the curve
- **AND** it does not include the other three quarters of the circle

#### Scenario: Arrow head widens the bounds

- **WHEN** a horizontal segment with arrow `end` is exported
- **THEN** the exported height is the arrow head's base width 0.75 L, not the stroke width

#### Scenario: Cubic bounds use the curve, not its control points

- **WHEN** a stroke of width w along the cubic Bézier piece (0, 0), (1, 2), (2, 2), (3, 0) is exported
- **THEN** the top of the exported area is the curve's highest point 1.5 plus w/2
- **AND** it does not reach the control points' height 2

## ADDED Requirements

### Requirement: Cubic pieces are written as cubic path pieces

An export SHALL write a cubic Bézier shape piece as one cubic Bézier path piece with the piece's start point, control points and end point, in every format.

#### Scenario: Cubic piece in SVG

- **WHEN** a shape consisting of one cubic Bézier piece is exported as SVG
- **THEN** its path is one move to the piece's start point and one cubic command with the piece's control points and end point in output units

#### Scenario: Cubic piece in PDF

- **WHEN** a shape consisting of one cubic Bézier piece is exported as PDF
- **THEN** its path is one move to the piece's start point and one cubic curve operator with the piece's control points and end point in output units

### Requirement: Export asks for shapes within 1 µm on paper

An export SHALL query each curve's shape (geometry-ir) at a tolerance of at most 1 µm on paper at the export's physical size. A curve without closed-form pieces is then written within 1 µm of the exact curve; segments and circular arcs are written as specified for their pieces.

#### Scenario: Tolerance follows the document scale

- **WHEN** a drawing with scale 10 (1 world unit = 10 mm) is exported
- **THEN** every curve's shape is queried with a tolerance of at most 10⁻⁴ world units
