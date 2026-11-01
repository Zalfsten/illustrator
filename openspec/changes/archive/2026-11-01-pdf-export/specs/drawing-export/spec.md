# Spec Delta

## MODIFIED Requirements

### Requirement: Curves are written from their exact shape

An export SHALL write each curve from its exact shape (geometry-ir), at the committed State. It SHALL NOT write the display mesh or any other display tessellation. A straight piece SHALL be written as a straight path piece. A circular-arc piece SHALL be written as a path piece of that circle with its exact center, radius and end points, in every format that has such a piece. A format without such a piece SHALL write a circular-arc piece as cubic Bézier pieces that start and end exactly at the arc's end points and deviate from the exact circle by at most 1 µm on paper at the export's physical size.

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

### Requirement: Export command

The app SHALL offer exporting the drawing as SVG and as PDF. It SHALL ask the host for a target, suggesting the drawing's file name with its sketch extension replaced by the format's extension (`.svg` or `.pdf`), or `untitled.svg` or `untitled.pdf` when the drawing has no file. A host that cannot write to a chosen file SHALL hand the bytes to the user as a download under the suggested name. If the user cancels the host's request, nothing SHALL be written. A write error SHALL be shown.

#### Scenario: Suggested name from the drawing's file

- **WHEN** a drawing whose file is `triangle.sketch.json` is exported
- **THEN** the host is asked for a target with suggested name `triangle.svg`

#### Scenario: Drawing without a file

- **WHEN** a drawing without a file is exported
- **THEN** the suggested name is `untitled.svg`

#### Scenario: Suggested PDF name

- **WHEN** a drawing whose file is `triangle.sketch.json` is exported as PDF
- **THEN** the host is asked for a target with suggested name `triangle.pdf`
- **AND** a drawing without a file suggests `untitled.pdf`

## ADDED Requirements

### Requirement: PDF form

The PDF form SHALL be one PDF document with exactly one page. The page's media box SHALL be the export's tight bounds at physical size, with no other content outside it. The page SHALL contain only vector paths, filled or stroked, in sRGB colours. The file SHALL embed no font and no image, refer to no external resource, and contain no creation date, modification date or file identifier.

#### Scenario: Page size is the drawing's physical size

- **WHEN** a drawing with scale 10 and one solid horizontal segment of length 3 world units, width 0.4 pt and no arrows is exported as PDF
- **THEN** the file has one page whose media box is 30 mm wide and 0.4 × 25.4/72.27 mm high

#### Scenario: Self-contained vector file

- **WHEN** a drawing with a segment, an arc and a label is exported as PDF
- **THEN** the file is a valid PDF with one page
- **AND** it contains no font, no image and no date or identifier

#### Scenario: Equal drawings give equal bytes

- **WHEN** the same drawing is exported as PDF twice
- **THEN** both files have identical bytes
