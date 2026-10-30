# Spec Delta

## Purpose

Defines what typesetting a label delivers and how labels are displayed: how a source is split into lines and how the lines are stacked, how markup and the preamble are applied, how failures are reported, and how a label appears in the drawing whether or not it could be typeset.

## ADDED Requirements

### Requirement: Typesetting delivers lines in the label frame

Typesetting a label SHALL deliver its lines in order. Each line SHALL carry its outlines in pt in the label frame (x along the baseline, y a quarter turn counterclockwise, origin at the line's left end on its baseline), its width along the baseline as set, and its visible height above and depth below the baseline. Typesetting SHALL NOT change the label's parameters or attributes.

#### Scenario: Single line metrics

- **WHEN** the `text` label `Ag` is typeset at 10 pt
- **THEN** it delivers one line with a width greater than zero
- **AND** that line's height above and depth below the baseline are both greater than zero

### Requirement: Lines are split at explicit breaks

A label's source SHALL be split into lines at its explicit breaks only: in `latex` markup at each `\\` that is not inside a group, an environment or math; in `text` markup at each newline character. An empty source SHALL deliver no lines.

#### Scenario: Break inside a matrix stays

- **WHEN** the `latex` label `a\\$\begin{pmatrix}1\\2\end{pmatrix}$` is typeset
- **THEN** it delivers two lines, and the matrix with both rows is on the second

#### Scenario: Newline in text markup

- **WHEN** the `text` label `x` + newline + `y` is typeset
- **THEN** it delivers two lines

### Requirement: Line spacing follows LaTeX

The distance between the baselines of two consecutive lines SHALL be the larger of 1.2 × the label size and the upper line's depth plus the lower line's height plus 1 pt. Height and depth are the lines' visible extents.

#### Scenario: Plain lines

- **WHEN** the `text` label `a` + newline + `b` is typeset at 10 pt
- **THEN** its two baselines are 12 pt apart

#### Scenario: Tall line pushes apart

- **WHEN** a `latex` label at 10 pt has a first line `$\displaystyle\sum_{i=1}^{n} i$` and a second line `x`
- **THEN** its baselines are more than 12 pt apart
- **AND** the gap between the first line's visible bottom and the second line's visible top is 1 pt within 0.01 pt

### Requirement: Markup and preamble are applied

In `latex` markup, text and `$...$` math SHALL be typeset together on a line, and macros defined in the document preamble SHALL be available to every label. In `text` markup, every character SHALL be set as itself, and the preamble SHALL NOT apply.

#### Scenario: Macro from the preamble

- **WHEN** the preamble is `\newcommand{\R}{\mathbb{R}}`
- **THEN** a `latex` label `$x \in \R$` typesets with the same outlines as `$x \in \mathbb{R}$`

#### Scenario: Text markup is literal

- **WHEN** the `text` label `$\alpha$` is typeset
- **THEN** its outlines are those of the characters `$`, `\`, `a`, `l`, `p`, `h`, `a`, `$`, not of the Greek letter

### Requirement: Typesetting failures are reported

Typesetting SHALL report a failure instead of lines when the source is not valid in its markup (for example an unclosed `$`), and SHALL report an unsupported construct, naming it, when the source uses a command or environment the typesetter does not provide. A failure SHALL NOT affect other labels.

#### Scenario: Unknown command is named

- **WHEN** the `latex` label `$\foo{x}$` is typeset
- **THEN** typesetting reports an unsupported construct naming `\foo`

#### Scenario: Unclosed math

- **WHEN** the `latex` label `$x` is typeset
- **THEN** typesetting reports a failure and delivers no lines

### Requirement: Labels are displayed in world space

The App SHALL paint each typeset label at its placement (labels: frame, reference point, alignment, offset, document scale) so that zooming the view scales labels and geometry together. The horizontal alignment SHALL use each line's width as set. Labels with the construction role SHALL follow the same presentation filter as construction geometries.

#### Scenario: Label follows its geometry

- **WHEN** a label's origin is joined to a segment's end and the segment's end is dragged
- **THEN** after the drag the label is painted at the new position of the segment's end

#### Scenario: Zoom keeps proportions

- **WHEN** the view is zoomed by a factor of two
- **THEN** a label's painted size and its distance to the geometry both double on screen

#### Scenario: Hidden construction label

- **WHEN** the App shows result geometries only
- **THEN** a label with the construction role is not painted

### Requirement: Labels that cannot be typeset stay visible

A label whose source fails to typeset, or that is shown where no typesetter is available, SHALL remain visible at its placement and SHALL be marked as not typeset, so that it can be told apart from a typeset label. Its source text SHALL be readable in that form.

#### Scenario: Failure is marked in place

- **WHEN** a label's source is `$\foo$`
- **THEN** the label is shown at its placement, marked as not typeset, with its source readable

#### Scenario: No typesetter available

- **WHEN** the App runs without a typesetter
- **THEN** every label is shown marked as not typeset, at its placement

### Requirement: Typesetting is a replaceable backend

Typesetting SHALL be reached through one port that takes source, markup, size and preamble and returns lines or a failure. The drawing model, the command session and label placement SHALL NOT depend on a particular typesetting engine. An implementation MAY provide different typesetters on different hosts, or none.

#### Scenario: Placement without an engine

- **WHEN** label placement is computed from lines supplied by a test typesetter with fixed metrics
- **THEN** the placed positions follow the labels spec for those metrics, without any typesetting engine present
