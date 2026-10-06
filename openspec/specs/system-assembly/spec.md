# system-assembly

## Purpose

Defines FEM-style assembly: local residual and Jacobian contributions are scattered into the global residual and global Jacobian using local→global connectivity.

## Requirements

### Requirement: Global residual is assembled from local contributions

The system SHALL form the global residual vector \(r(p)\) by evaluating each active contribution’s local residual and scattering (accumulating) its entries into global residual indices according to connectivity.

#### Scenario: Multiple constraints fill one residual vector

- **WHEN** a sketch has several structural constraints
- **AND** the residual is evaluated at state \(p\)
- **THEN** each constraint’s local residual entries appear in the global residual at the positions dictated by connectivity
- **AND** the global residual length equals the sum of the arities of active residual contributions (for the structural system without soft terms)

### Requirement: Global Jacobian is assembled from local contributions

The system SHALL form the global Jacobian \(J(p) = \partial r / \partial p\) by placing each contribution’s derivative blocks into global residual rows and State columns via that contribution’s connectivity. Mathematically this is the scatter of a contribution Jacobian \(J_N\) whose columns follow connectivity order:

\(J[M_N^{\mathrm{row}}(i), M_N^{\mathrm{col}}(j)] \mathrel{+}= (J_N)_{ij}\) (up to residual-row indexing conventions).

Implementations MAY materialize a dense packed \(J_N\) buffer before scattering, or MAY write formula Jacobian blocks directly into the global matrix through the connectivity map. The observable contract is the assembled global \(J\).

#### Scenario: Local derivative lands at mapped global indices

- **WHEN** a contribution’s derivative for residual component \(i\) w.r.t. connectivity column \(j\) is assembled
- **THEN** it is added into the global Jacobian at the row and column corresponding to that residual component and the mapped global parameter index
- **AND** parameters not involved in that contribution receive no fill from that contribution

### Requirement: Assembly is re-evaluated at the current state

Because the system is nonlinear, residual and Jacobian assembly SHALL be performed at the current \(p\) whenever the solver requests an evaluation. The system SHALL NOT treat a previously assembled Jacobian as valid for a different \(p\) unless an explicit caching contract says otherwise.

#### Scenario: Solver iteration triggers fresh assembly

- **WHEN** the solver requests residual or Jacobian evaluation at a new \(p\)
- **THEN** assembly computes contributions using that \(p\)
- **AND** the returned residual/Jacobian correspond to that state

### Requirement: Contribution Jacobian columns follow connectivity order

Each structural contribution’s Jacobian columns SHALL follow the same order as that contribution’s parameter connectivity list (residual arity × connectivity length, whether packed locally or written straight into global \(J\)). Assembly SHALL map those columns to global State indices via that list. Implementations MAY fill contributions through typed Jacobian blocks (point/vector/scalar formula Jacobians) placed via a sequential column cursor; the observable contract is the scattered global \(J\).

#### Scenario: Joined contribution concatenates operand connectivities

- **WHEN** a contribution joins two expressions whose connectivities are \(C_a\) then \(C_b\)
- **AND** its Jacobian contribution is assembled
- **THEN** the contribution column count equals \(|C_a| + |C_b|\)
- **AND** the first \(|C_a|\) columns correspond to \(C_a\) and the remainder to \(C_b\)

### Requirement: Soft interaction terms assemble the same way

Temporary soft terms used during interaction (for example mouse-follow or stay-near-previous) SHALL, when active, contribute residual and Jacobian blocks that are assembled with the same connectivity/scatter rules, without becoming persistent document constraints. Soft residual scale relative to unweighted structural residuals SHALL be carried by the soft terms’ own weights (or equivalent per-term strength), not by a separate mandatory global rescale of structural rows during assembly.

#### Scenario: Soft term adds rows without persisting constraints

- **WHEN** soft interaction terms are active during a solve evaluation
- **THEN** their residual/Jacobian contributions are included in the assembled system for that evaluation
- **AND** after the interaction ends they are not retained as structural document constraints

#### Scenario: Soft strength is in the soft weights

- **WHEN** soft mouse-follow and stay-near-previous terms are assembled with structural residuals
- **THEN** the relative soft-vs-hard tradeoff is determined by those soft terms’ weights (or equivalent strengths)
- **AND** structural residual rows are not required to be globally rescaled solely because soft terms are present
