# Spec Delta

## MODIFIED Requirements

### Requirement: Global Jacobian is assembled from local contributions

The system SHALL form the global Jacobian \(J(p) = \partial r / \partial p\) as the **sum** of each contribution's derivative block placed at that contribution's residual rows and connectivity columns:

\(J[M_N^{\mathrm{row}}(i), M_N^{\mathrm{col}}(j)] \mathrel{+}= (J_N)_{ij}\) (up to residual-row indexing conventions).

Entries that several block columns map to (including duplicate columns within one contribution's connectivity) SHALL accumulate; no block entry SHALL overwrite another. Implementations MAY store each \(J_N\) as a packed dense block next to its row and column mapping, without materialising a global matrix, or MAY write blocks into a global matrix through a connectivity scatter view. When a contribution concatenates several formula blocks, implementations MAY place successive column windows with same-kind matrix block slices. Formula Jacobians MAY be written as dense row-major blocks into the contribution window, optionally multiplied by a contribution scale (for example \(\pm 1\) for difference residuals). The observable contract is the global \(J\) defined by the sum above.

#### Scenario: Local derivative lands at mapped global indices

- **WHEN** a contribution's derivative for residual component \(i\) w.r.t. connectivity column \(j\) is assembled
- **THEN** it is added into the global Jacobian at the row and column corresponding to that residual component and the mapped global parameter index
- **AND** parameters not involved in that contribution receive no fill from that contribution

#### Scenario: Duplicate connectivity columns accumulate

- **WHEN** a contribution's connectivity lists the same global parameter index at two local columns \(j_1 \ne j_2\) (for example a Congruence between the start and end point of the same arc)
- **AND** its Jacobian is assembled at \(p\)
- **THEN** the global Jacobian entry for that parameter in each of the contribution's rows equals \((J_N)_{i j_1} + (J_N)_{i j_2}\)
- **AND** the result equals a finite-difference derivative of the assembled residual with respect to that parameter (within tolerance)

#### Scenario: Inactive one-sided contribution contributes zero

- **WHEN** a one-sided soft contribution is inactive at \(p\) (for example a length floor with \(L \ge \ell_{\min}\))
- **THEN** its block contributes zero to the global Jacobian at \(p\)
- **AND** it keeps the same row and column mapping as when active

### Requirement: Contribution Jacobian columns follow connectivity order

Each structural contribution's Jacobian columns SHALL follow the same order as that contribution's parameter connectivity list (residual arity × connectivity length, whether packed locally or written straight into global \(J\)). Assembly SHALL map those columns to solver columns via an assembly-owned column mapping. For an assembled system that does not reduce the parameter set, that mapping SHALL be the identity on global State indices, so the solver columns are the State indices. Implementations MAY fill contributions by writing point/vector/scalar formula Jacobians (formula-local columns) into the contribution matrix window, placing successive operand blocks via same-kind column block slices; the observable contract is the global \(J\).

#### Scenario: Joined contribution concatenates operand connectivities

- **WHEN** a contribution joins two expressions whose connectivities are \(C_a\) then \(C_b\)
- **AND** its Jacobian contribution is assembled
- **THEN** the contribution column count equals \(|C_a| + |C_b|\)
- **AND** the first \(|C_a|\) columns correspond to \(C_a\) and the remainder to \(C_b\)

#### Scenario: Formula Jacobians write with formula-local columns

- **WHEN** a contribution accumulates an expression's analytical Jacobian into the assembled system
- **THEN** that expression writes with formula-local column indices \(0..n-1\) matching its reported parameter-index list
- **AND** assembly/connectivity is responsible for mapping those columns into solver columns

#### Scenario: Unreduced system uses State indices as columns

- **WHEN** an assembled system is built without any parameter reduction
- **THEN** every contribution column maps to the global State index from its connectivity list
- **AND** the global \(J\) has one column per State parameter
