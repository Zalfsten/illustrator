## MODIFIED Requirements

### Requirement: Distinguish hard structural and soft manipulation conditions

The system SHALL distinguish structural conditions (persistent constraints from the catalog) from manipulation conditions introduced only for the active interaction. Structural conditions SHALL be treated as hard (must be satisfied for a successful commit). Manipulation conditions for mouse follow, stay-near-previous, and soft curve-length floor SHALL be treated as soft (best-effort toward the pointer / previous state / non-collapse without violating hard conditions).

#### Scenario: Drag does not permanently store mouse residuals

- **WHEN** the user finishes a drag gesture
- **THEN** the committed document contains only structural constraints as persistent relationships
- **AND** soft mouse-follow conditions from the gesture are not retained as document constraints

### Requirement: Soft length floor resists curve collapse during drag

During an interactive drag, the system SHALL include a soft, one-sided floor on each curve geometry's length expression so that collapse to zero length is not a soft minimum under mouse-follow. The floor residual SHALL be inactive when the current length \(L \ge \ell_{\min}\) and SHALL NOT become a persistent structural constraint. Soft length floors SHALL NOT alter VectorsParallel, VectorsOrthogonal, or other catalog constraint semantics.

#### Scenario: Drag toward a degenerate attractor keeps positive curve length

- **WHEN** a document has curve geometries under structural constraints that admit a near-zero-length configuration
- **AND** the user drags a logical UI point toward a pointer position that would otherwise collapse a curve
- **THEN** after each successful interactive solve during the drag, that curve's length remains at least on the order of \(f\,L_0\) for the configured fraction \(f\) and drag-start length \(L_0\)
- **AND** after the drag ends, no length-floor term remains as a document constraint

#### Scenario: Length floor scales with the figure

- **WHEN** two documents differ only by a uniform scale of their geometry and State
- **AND** the same relative drag gesture is applied with the same configured fraction \(f\)
- **THEN** the soft length floor thresholds scale with each document's drag-start curve lengths
- **AND** the floor is not a single absolute length shared across differently scaled figures

### Requirement: Logical UI points are congruence classes

A logical UI point that the user can grab SHALL correspond to one congruence class (see constraint-catalog) that contains at least one visible curve endpoint expression. Endpoints SHALL NOT be merged into one logical point because of their locations, only through Congruence; the set of logical points SHALL depend only on structure and visibility filter, not on State.

#### Scenario: Dragging a shared corner moves all congruent endpoints together

- **WHEN** two segment ends are joined as one logical UI point via Congruence
- **AND** the user drags that UI point through a successful solve
- **THEN** both endpoint expressions remain congruent
- **AND** both evaluate to the updated shared location

#### Scenario: Direct Congruence partners share the soft mouse target

- **WHEN** Congruence relates point expressions \(A\) and \(B\)
- **AND** the user drags the logical UI point corresponding to \(A\)
- **THEN** the soft mouse-follow terms for that evaluation include the State columns of both \(A\) and \(B\)
- **AND** stay-near-previous soft terms omit those columns for that evaluation

#### Scenario: The whole class shares the soft mouse target

- **WHEN** Congruences relate point expressions \(A\) to \(B\) and \(B\) to \(C\)
- **AND** the user drags the logical UI point corresponding to \(A\)
- **THEN** the soft mouse-follow terms for that evaluation include the State columns of \(A\), \(B\) and \(C\)
- **AND** stay-near-previous soft terms omit those columns for that evaluation

#### Scenario: A Join chain is one logical point

- **WHEN** three segment endpoints are related by two Congruences in a chain
- **THEN** the drawing presents exactly one logical UI point for them

#### Scenario: Coincident unjoined endpoints are separate logical points

- **WHEN** the end of one segment and the start of another evaluate to the same location
- **AND** no Congruence chain relates them
- **THEN** the drawing presents two logical UI points
- **AND** dragging one of them moves only the endpoints of its own class

#### Scenario: Logical points do not change during a drag

- **WHEN** the user drags an unjoined endpoint onto or through the location of another unjoined endpoint
- **THEN** the set of logical UI points stays the same throughout the drag

## ADDED Requirements

### Requirement: Soft terms live only in the interaction

Soft terms SHALL NOT be stored as document constraints; their residual strength relative to structural residuals SHALL be carried by soft-term weights (or equivalent per-term strengths) used only for the interaction evaluation.

#### Scenario: Weights are not part of the drawing

- **WHEN** a drag with soft terms of different weights has ended
- **THEN** neither the soft terms nor their weights are part of the document or of its saved file

### Requirement: Length floor is relative to the drag-start length

For each curve under the soft length floor, the floor \(\ell_{\min}\) SHALL be a configured fraction \(f\) of that curve's length \(L_0\) evaluated at the drag-start State (the stay-near snapshot), \(\ell_{\min} = f\,L_0\), and SHALL NOT be a fixed absolute length independent of the sketch scale.

#### Scenario: Floor stays fixed during the drag

- **WHEN** a curve has length 4 at drag start and the configured fraction is 0.1
- **THEN** its floor is 0.4 throughout the drag, also after the curve's length has changed

### Requirement: Dragging a logical point targets its whole class

Dragging a logical point SHALL apply the soft mouse target to every expression of its class; stay-near-previous soft terms SHALL NOT be applied to the State columns of those expressions for that evaluation (so stay does not fight the pointer).

#### Scenario: Unjoined endpoint keeps the other end staying

- **WHEN** the user drags the logical point of an unjoined segment start
- **THEN** the soft mouse-follow terms include the State columns of that start
- **AND** stay-near-previous soft terms omit those columns but still cover the segment's end
