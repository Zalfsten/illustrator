# solver-port

## Purpose

Defines a swappable nonlinear solver port that consumes residual/Jacobian evaluations and exposes common solver controls, whether backed by an owned implementation or an external library.

## Requirements

### Requirement: Abstract solver port

The system SHALL expose an abstract solver port that drives updates to the global state vector \(p\) by requesting residual evaluations and Jacobian evaluations (or an agreed equivalent such as Jacobian-vector products) through callbacks or an equivalent evaluation interface—not by receiving a single static matrix as the sole problem definition for a nonlinear solve.

#### Scenario: Solver requests eval callbacks each iteration

- **WHEN** a nonlinear solve runs
- **THEN** the solver port repeatedly obtains residual information at the current \(p\)
- **AND** obtains Jacobian information (analytical assembly or an explicitly configured equivalent) at the current \(p\) as required by the chosen algorithm
- **AND** updates \(p\) toward satisfying the assembled structural residuals

### Requirement: Owned or external backend

Implementations SHALL be able to satisfy the solver port either with an owned numerical backend or by adapting an external solver library behind the same port. Spec-level behavior SHALL NOT require a particular library or algorithm name.

#### Scenario: Backend can be substituted

- **WHEN** one solver backend conforming to the port is replaced by another
- **THEN** structural solve semantics (invariants, tolerances as configured) remain expressible through the same port
- **AND** no requirement mandates a specific vendor library

### Requirement: Common solver parameters

The solver port SHALL support configuration of common nonlinear-solve controls, including at least: convergence tolerance on residual norms (or an equivalent stopping criterion), maximum iterations, and algorithm/damping-related options applicable to the backend (for example step damping or trust-region / Levenberg–Marquardt style parameters when the backend uses them). Backends SHALL document which parameters they honor; unsupported parameters SHALL fail closed (reject or clearly ignore with observable signaling)—implementations MUST NOT silently claim success while ignoring a required tolerance setting.

#### Scenario: Tolerance and iteration limit are configurable

- **WHEN** a caller sets a residual tolerance and a maximum iteration count on the solver port
- **THEN** the active backend uses those settings in its stopping criteria (or rejects the configuration if it cannot)
- **AND** a successful solve reports satisfaction within the configured residual tolerance for structural residuals

### Requirement: Evaluation fills first-class residual and Jacobian buffers

The evaluation interface used by the solver port SHALL accept the current parameter State \(p\) and fill a first-class residual buffer \(r\) and a first-class Jacobian buffer for \(J\). The Jacobian buffer SHALL be a **contribution-blocked sparse** representation: for each assembled contribution it holds that contribution's dense local derivative block together with the residual rows and parameter columns the block maps to, and the represented global \(J\) is the sum of those mapped blocks (see system-assembly). Implementations MAY additionally offer a dense \(m \times n\) materialisation of the same \(J\) (for example for reference backends or tests). The sparsity structure (which rows and columns each block maps to) depends only on the assembled system, not on \(p\). \(r\) and \(p\) SHALL be distinct buffers: residual length is the assembled residual arity, not the parameter count. Implementations MAY expose the same contract at a C ABI as opaque user data plus raw fill pointers; adapters MAY copy between foreign buffers and the first-class buffers.

Owned backends and FFI adapters MAY allocate residual, Jacobian, and algorithm scratch once per solve (or per attached evaluation handle) and reuse those buffers across iterations or callbacks within that solve. Reuse SHALL NOT change observable residuals/Jacobians at a given \(p\).

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
- **THEN** the backend or FFI adapter MAY refill previously allocated evaluation buffers instead of allocating new ones each call
- **AND** each evaluation still reflects the \(p\) supplied for that call

### Requirement: FFI-friendly evaluation contract

The evaluation interface used by the solver port SHALL be expressible in a form suitable for binding to foreign-language solvers (for example opaque user data plus residual/Jacobian fill callbacks), so that an external library can call back into assembly without requiring Rust-specific closure types at the ABI boundary. An adapter whose foreign library expects a different Jacobian layout (for example dense) SHALL convert from the contribution-blocked representation so that the foreign library sees the same global \(J\).

#### Scenario: External adapter can call assembly

- **WHEN** an external solver backend is attached
- **THEN** it can obtain residual and Jacobian fills at the current \(p\) through the port's evaluation contract
- **AND** assembly remains responsible for filling contribution blocks with their connectivity
- **AND** the adapter is responsible for converting that representation into the layout the foreign library expects

#### Scenario: Owned and external backends see the same Jacobian

- **WHEN** an owned sparse backend and an external dense backend evaluate the same assembled system at the same \(p\)
- **THEN** the global \(J\) each one consumes is identical up to floating-point rounding
