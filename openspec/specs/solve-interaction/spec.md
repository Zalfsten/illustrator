# solve-interaction

## Purpose

Defines interactive solve behavior for dragging logical UI points: structural constraints stay hard, mouse follow / stay-near / length-floor are soft, failed solves keep the last consistent state, and a drag gesture is one atomic edit.

## Requirements

### Requirement: Distinguish hard structural and soft manipulation conditions

The system SHALL distinguish structural conditions (persistent constraints from the catalog) from manipulation conditions introduced only for the active interaction. Structural conditions SHALL be treated as hard (must be satisfied for a successful commit). Manipulation conditions for mouse follow, stay-near-previous, and soft curve-length floor SHALL be treated as soft (best-effort toward the pointer / previous state / non-collapse without violating hard conditions). Soft terms SHALL NOT be stored as document constraints; their residual strength relative to structural residuals SHALL be carried by soft-term weights (or equivalent per-term strengths) used only for the interaction evaluation.

#### Scenario: Drag does not permanently store mouse residuals

- **WHEN** the user finishes a drag gesture
- **THEN** the committed document contains only structural constraints as persistent relationships
- **AND** soft mouse-follow conditions from the gesture are not retained as document constraints

### Requirement: Soft drag follows the pointer under invariants

While the user drags a logical UI point toward a pointer position, the system SHALL attempt to move that point toward the pointer while keeping all structural constraints satisfied. Exact coincidence with the pointer is NOT required when structural constraints forbid it.

#### Scenario: Orthogonal figure stays orthogonal while a free point is dragged

- **WHEN** a document has Orthogonal (and any other structural constraints) satisfied
- **AND** the user drags a logical UI point that is not fixed
- **THEN** after each successful interactive solve during the drag, structural constraints remain satisfied
- **AND** the dragged point's location moves toward the pointer to the extent the structural constraints allow

### Requirement: Prefer solutions near the previous state

When multiple structurally consistent solutions exist, interactive solves during and at the end of a drag SHALL prefer parameter values near the last consistent state (stay-near-previous), in addition to pursuing the soft mouse target.

#### Scenario: Drag does not jump to a distant alternate solution

- **WHEN** a successful interactive solve runs during a small pointer movement
- **THEN** the resulting configuration remains near the previous consistent configuration
- **AND** the system does not replace the figure with a distant alternate solution that also satisfies the structural constraints

### Requirement: Soft length floor resists curve collapse during drag

During an interactive drag, the system SHALL include a soft, one-sided floor on each curve geometry's length expression so that collapse to zero length is not a soft minimum under mouse-follow. For each such curve, the floor \(\ell_{\min}\) SHALL be a configured fraction \(f\) of that curve's length \(L_0\) evaluated at the drag-start State (the stay-near snapshot), \(\ell_{\min} = f\,L_0\), and SHALL NOT be a fixed absolute length independent of the sketch scale. The floor residual SHALL be inactive when the current length \(L \ge \ell_{\min}\) and SHALL NOT become a persistent structural constraint. Soft length floors SHALL NOT alter Parallel, Orthogonal, or other catalog constraint semantics.

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

### Requirement: Failed solve keeps last consistent state

If an interactive solve fails to produce a structurally consistent result within solve tolerance, the system SHALL keep the document (and the displayed committed state) at the last consistent state and SHALL make the failure observable to the user or host environment.

#### Scenario: Failed solve does not commit a broken figure

- **WHEN** an interactive solve fails during or at the end of a drag
- **THEN** the committed document state remains the last structurally consistent state
- **AND** a failure indication is observable

### Requirement: Drag gesture is atomic

A pointer-down on a logical UI point, followed by moves, followed by pointer-up, SHALL constitute a single edit action for history purposes. Intermediate moves SHALL NOT each create separate undo steps.

#### Scenario: Undo reverses an entire drag

- **WHEN** the user completes a drag from pointer-down through pointer-up that committed a new state
- **AND** the user requests undo once
- **THEN** the document returns to the state from before that pointer-down

### Requirement: Logical UI points are congruence classes

A logical UI point that the user can grab SHALL correspond to an equivalence class of endpoint expressions identified by Congruence. Dragging that UI point SHALL apply the soft mouse target to that shared location (the congruence class), not to only one segment's private endpoint parameters in isolation while leaving congruent partners unconstrained by Congruence. Soft mouse targets SHALL cover the dragged point expression and its **direct** Congruence partners (see constraint-catalog); stay-near-previous soft terms SHALL NOT be applied to State columns of those mouse-targeted expressions for that evaluation (so stay does not fight the pointer).

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

### Requirement: Solver backend is replaceable

The system SHALL expose solving behind a replaceable backend boundary so that different numerical algorithms can satisfy the same interaction contract. Spec-level behavior SHALL NOT depend on a particular algorithm name (for example Newton, Levenberg–Marquardt, or a GPU implementation).

#### Scenario: Behavior contract is algorithm-agnostic

- **WHEN** a backend that meets the solve-interaction contract is substituted for another
- **THEN** structural invariance, soft drag, stay-near-previous, soft length floor, failure retention, and atomic drag semantics remain as specified
- **AND** no requirement in this capability mandates a specific numerical method
