## MODIFIED Requirements

### Requirement: Curves are written from their exact shape

An export SHALL write each curve from its exact shape (geometry-ir), at the committed State. It SHALL NOT write the display mesh or any other display tessellation. A straight piece SHALL be written as a straight path piece. A circular-arc piece SHALL be written as a path piece of that circle with its exact center, radius and end points, in every format that has such a piece.

#### Scenario: Arc is not a polyline

- **WHEN** a drawing with one result circular arc is exported as SVG
- **THEN** the arc's path consists of elliptical-arc commands with equal radii equal to the arc's radius in output units
- **AND** it contains no straight pieces

#### Scenario: Segment is one straight piece

- **WHEN** a drawing with one solid result segment without arrows is exported
- **THEN** its path is one move to the start point and one straight piece to the end point

#### Scenario: Arc in PDF stays within 1 µm

- **WHEN** a drawing with a result circular arc of radius 10 world units at scale 10 (100 mm on paper) and sweep 3π/2 is exported as PDF
- **THEN** the arc's path consists of cubic Bézier pieces only, starting at the arc's start point and ending at its end point
- **AND** every point of those pieces lies within 1 µm of the arc's circle on paper

#### Scenario: Small arc needs few pieces

- **WHEN** a quarter arc of radius 0.1 world units at scale 10 (1 mm on paper) is exported as PDF
- **THEN** its path is a single cubic Bézier piece, which already stays within 1 µm

## ADDED Requirements

### Requirement: Arcs in formats without arc pieces

A format without a circular-arc path piece SHALL write a circular-arc piece as cubic Bézier pieces that start and end exactly at the arc's end points and deviate from the exact circle by at most 1 µm on paper at the export's physical size.

#### Scenario: Half arc in PDF

- **WHEN** a drawing with a result circular arc of radius 2 world units at scale 10 and sweep π is exported as PDF
- **THEN** its path consists of cubic Bézier pieces that start at the arc's start point and end at its end point
- **AND** every point of those pieces lies within 1 µm of the arc's circle on paper
