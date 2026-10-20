# Spec Delta

## MODIFIED Requirements

### Requirement: Soft drag follows the pointer under invariants

While the user drags a logical UI point toward a pointer position, the system SHALL attempt to move that point toward the pointer while keeping all structural constraints satisfied. Exact coincidence with the pointer is NOT required when structural constraints forbid it.

#### Scenario: Orthogonal figure stays orthogonal while a free point is dragged

- **WHEN** a document has VectorsOrthogonal (and any other structural constraints) satisfied
- **AND** the user drags a logical UI point that is not fixed
- **THEN** after each successful interactive solve during the drag, structural constraints remain satisfied
- **AND** the dragged point's location moves toward the pointer to the extent the structural constraints allow

### Requirement: Soft length floor resists curve collapse during drag

During an interactive drag, the system SHALL include a soft, one-sided floor on each curve geometry's length expression so that collapse to zero length is not a soft minimum under mouse-follow. For each such curve, the floor \(\ell_{\min}\) SHALL be a configured fraction \(f\) of that curve's length \(L_0\) evaluated at the drag-start State (the stay-near snapshot), \(\ell_{\min} = f\,L_0\), and SHALL NOT be a fixed absolute length independent of the sketch scale. The floor residual SHALL be inactive when the current length \(L \ge \ell_{\min}\) and SHALL NOT become a persistent structural constraint. Soft length floors SHALL NOT alter VectorsParallel, VectorsOrthogonal, or other catalog constraint semantics.

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
