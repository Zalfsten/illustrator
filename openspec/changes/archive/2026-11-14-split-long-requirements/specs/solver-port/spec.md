## MODIFIED Requirements

### Requirement: Common solver parameters

The solver port SHALL support configuration of common nonlinear-solve controls, including at least: convergence tolerance on residual norms (or an equivalent stopping criterion), maximum iterations, and algorithm/damping-related options applicable to the backend (for example step damping or trust-region / Levenberg–Marquardt style parameters when the backend uses them).

#### Scenario: Tolerance and iteration limit are configurable

- **WHEN** a caller sets a residual tolerance and a maximum iteration count on the solver port
- **THEN** the active backend uses those settings in its stopping criteria (or rejects the configuration if it cannot)
- **AND** a successful solve reports satisfaction within the configured residual tolerance for structural residuals

### Requirement: Evaluation fills first-class residual and Jacobian buffers

The evaluation interface used by the solver port SHALL accept the current parameter State \(p\) and fill a first-class residual buffer \(r\) and a first-class Jacobian buffer for \(J\). \(r\) and \(p\) SHALL be distinct buffers: residual length is the assembled residual arity, not the parameter count. Adapters for external solver libraries MAY copy between the library's buffers and the first-class buffers.

#### Scenario: Residual and Jacobian buffer kinds

- **WHEN** the solver requests a residual evaluation at \(p\)
- **THEN** assembly writes into a residual buffer of length equal to the number of residual rows
- **AND** that buffer is not the parameter State \(p\)
- **WHEN** the solver requests a Jacobian evaluation at \(p\)
- **THEN** assembly writes each contribution's derivative block into the contribution-blocked Jacobian buffer
- **AND** the represented global \(J\) has \(m\) rows and \(n\) columns

#### Scenario: Sparsity structure is fixed for an assembled system

- **WHEN** the solver requests Jacobian evaluations at two different states \(p_1\) and \(p_2\) of the same assembled system
- **THEN** the row and column mapping of every contribution block is identical for both evaluations
- **AND** only the numeric block entries differ

#### Scenario: Dense materialisation matches the sparse representation

- **WHEN** a dense \(m \times n\) materialisation of the Jacobian is requested at \(p\)
- **THEN** each entry equals the sum of all contribution-block entries mapped to that row and column
- **AND** entries not covered by any block are zero

#### Scenario: Scratch reuse within one solve

- **WHEN** a nonlinear solve performs multiple residual or Jacobian evaluations
- **THEN** the backend MAY refill previously allocated evaluation buffers instead of allocating new ones each call
- **AND** each evaluation still reflects the \(p\) supplied for that call

## ADDED Requirements

### Requirement: Unsupported solver parameters fail closed

Backends SHALL document which parameters they honor; unsupported parameters SHALL fail closed (reject or clearly ignore with observable signaling)—implementations MUST NOT silently claim success while ignoring a required tolerance setting.

#### Scenario: Unsupported parameter is not silently ignored

- **WHEN** a caller sets a solver parameter that the active backend does not honor
- **THEN** the backend rejects the configuration or signals observably that it ignores the parameter
- **AND** it does not report a successful solve as if the parameter had been honored

### Requirement: Jacobian buffer is contribution-blocked sparse

The Jacobian buffer SHALL be a **contribution-blocked sparse** representation: for each assembled contribution it holds that contribution's dense local derivative block together with the residual rows and parameter columns the block maps to, and the represented global \(J\) is the sum of those mapped blocks (see system-assembly). The sparsity structure (which rows and columns each block maps to) depends only on the assembled system, not on \(p\).

#### Scenario: Overlapping blocks add up

- **WHEN** two contribution blocks map entries to the same residual row and parameter column
- **THEN** the represented global \(J\) at that row and column is the sum of both block entries

### Requirement: Dense Jacobian materialisation is optional

Implementations MAY additionally offer a dense \(m \times n\) materialisation of the same \(J\) (for example for reference backends or tests). Such a materialisation SHALL represent the same \(J\) as the contribution-blocked buffer.

#### Scenario: Dense and sparse forms describe one Jacobian

- **WHEN** an implementation offers a dense materialisation and the Jacobian is evaluated at \(p\)
- **THEN** the dense matrix and the contribution-blocked buffer represent the same \(J\)

### Requirement: Evaluation buffers may be reused within a solve

Backends MAY allocate residual, Jacobian, and algorithm scratch once per solve and reuse those buffers across iterations within that solve. Reuse SHALL NOT change observable residuals/Jacobians at a given \(p\).

#### Scenario: Reused buffers give the same values

- **WHEN** the same \(p\) is evaluated twice within one solve, once into freshly allocated buffers and once into reused buffers
- **THEN** both evaluations give identical residuals and Jacobians
