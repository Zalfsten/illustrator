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

- **WHEN** a document has VectorsOrthogonal (and any other structural constraints) satisfied
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

A logical UI point that the user can grab SHALL correspond to one congruence class (see constraint-catalog) that contains at least one visible curve endpoint expression. Endpoints SHALL NOT be merged into one logical point because of their locations, only through Congruence; the set of logical points SHALL depend only on structure and visibility filter, not on State. Dragging a logical point SHALL apply the soft mouse target to every expression of its class; stay-near-previous soft terms SHALL NOT be applied to the State columns of those expressions for that evaluation (so stay does not fight the pointer).

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

### Requirement: Solver backend is replaceable

The system SHALL expose solving behind a replaceable backend boundary so that different numerical algorithms can satisfy the same interaction contract. Spec-level behavior SHALL NOT depend on a particular algorithm name (for example Newton, Levenberg–Marquardt, or a GPU implementation).

#### Scenario: Behavior contract is algorithm-agnostic

- **WHEN** a backend that meets the solve-interaction contract is substituted for another
- **THEN** structural invariance, soft drag, stay-near-previous, soft length floor, failure retention, and atomic drag semantics remain as specified
- **AND** no requirement in this capability mandates a specific numerical method

### Requirement: A drag solves only the dragged component

Interactive solves during and at the end of a drag SHALL cover only the component (constraint-catalog) that contains the dragged logical UI point. Only that component's structural residuals SHALL decide whether a solve succeeds, and other components SHALL keep their values. A conflicted component elsewhere SHALL NOT make a drag fail.

#### Scenario: Drag next to a conflicted component

- **WHEN** a drawing holds a conflicted component and a separate consistent component with a free endpoint
- **AND** the user drags that endpoint through successful solves
- **THEN** the endpoint moves toward the pointer under its component's constraints
- **AND** the values of the conflicted component are unchanged

### Requirement: Frozen components cannot be dragged

A drag SHALL NOT start on a logical UI point whose congruence class lies in a conflicted component. The attempt SHALL leave values unchanged, SHALL NOT create a history step, and SHALL report that the component is frozen to the host, which decides how to present it.

#### Scenario: Pointer-down on a frozen point

- **WHEN** a logical UI point belongs to a conflicted component
- **AND** a drag is started on it
- **THEN** no drag is active
- **AND** the drawing and the history are unchanged
- **AND** the host receives the frozen indication
