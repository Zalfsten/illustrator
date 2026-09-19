# Design

## Context

Greenfield repository with OpenSpec only — no application code yet. Product goal and the one-spec / many-implementations rule live in `openspec/config.yaml`; see `proposal.md` for why this slice exists. Specs define Model E (`geometry-ir`), the v1 catalog (`constraint-catalog`), and drag/solve behavior (`solve-interaction`). This design chooses the **first** implementation stack and the internal shape of the owned solver; those choices are not shared requirements.

## Goals / Non-Goals

**Goals:**

- Deliver a minimal interactive sketch loop that proves the specs: create segments, apply Congruence / Parallel / Orthogonal / FixPoint, drag logical UI points without breaking structural constraints.
- Keep Model E as the in-memory IR; lower UI shared points to Congruence; assemble residuals; solve with an owned, swappable backend.
- Structure the codebase so a second solver algorithm (and later GPU) can plug in behind the same residual API.
- Prefer a compiled Linux-desktop path for the first implementation (language choice below).

**Non-Goals:**

- Labels, LaTeX, styles, export, function graphs, circles/splines, equal-length constraints.
- Using PlaneGCS/SolveSpace/OpenCASCADE as the document model or primary solver (may be studied later; not dependencies for v1).
- Production-grade DOF analysis UI, conflict explanation, or maximal consistent subsets.
- Multi-platform packaging beyond what is needed to run the first desktop binary on Linux.

## Decisions

### D1: First implementation language — Rust

**Choice:** Rust for the first Linux desktop implementation.

**Rationale:** Fits an owned numerical core, replaceable backends, and later FFI/GPU experiments; avoids C++ as the application language while still allowing optional C bindings later if needed. Strong fit for long-lived interactive apps versus Go (GC) or Julia (deployment/GUI).

**Alternatives considered:**

- **Julia** — excellent for numerics prototyping; weak as a shipping desktop host. Rejected for impl #1; acceptable later for offline algorithm spikes.
- **Go** — simple tooling; weaker canvas/GUI and numerics ergonomics for this problem. Rejected for impl #1.
- **Zig / Nim** — viable systems options; smaller ecosystems for GUI and linear algebra. Deferred.

### D2: Segment parameterization — endpoint form

**Choice:** Represent each segment by four scalars \((x_1,y_1,x_2,y_2)\).

**Rationale:** Makes `Endpoints` nearly identity, keeps Congruence elementary, matches Model E teaching clarity for v1.

**Alternatives considered:** Midpoint–angle–length (nicer for some rotations; worse for Congruence clarity). Can be added later as an alternate chart without changing specs.

### D3: Residual assembly API

**Choice:** Geometries own parameters; constraints implement a small residual interface (evaluate residuals, optionally analytical Jacobian later). Global assembler concatenates residuals into \(r(p)\) and builds \(J\) (finite differences acceptable for first backend).

**Rationale:** Matches Model E equation factories; keeps algorithms behind `solve(p, r, J) -> p'`.

**Alternatives considered:** Symbolic CAS layer (overkill); automatic differentiation from day one (nice-to-have after finite differences work).

### D4: First solver algorithm — damped least squares (LM-style)

**Choice:** Start with a Levenberg–Marquardt / damped Gauss–Newton style backend on CPU that minimizes structural residuals plus soft penalties for mouse follow and stay-near-previous during interaction.

**Rationale:** Standard for small dense sketch systems; easy to swap later. Soft terms encode solve-interaction without storing them in the document.

**Alternatives considered:** Hard equality-only SQP (stricter, more failure modes for underdetermined drag); pure Newton on squared residuals without damping (less robust). GPU deferred until residual API and CPU path are stable.

### D5: Interaction encoding of soft terms

**Choice:** During drag, build a temporary objective / augmented residual: hard structural \(r_s(p)=0\) (or heavily weighted) plus \(w_m\|P-m\|^2\) and \(w_s\|p-p_{\mathrm{prev}}\|^2\). Clear soft terms on pointer-up after final solve.

**Rationale:** Implements the specified hard/soft split and stay-near-previous without new persistent constraint types.

### D6: UI layer — immediate-mode canvas MVP

**Choice:** Minimal Linux desktop UI (e.g. egui + winit, or equivalent) with: place segment endpoints, join ends into logical UI points (Congruence), assign Parallel / Orthogonal / FixPoint, drag UI points.

**Rationale:** Fastest path to validate the runtime. Toolkit is an implementation detail; may change without spec changes.

**Alternatives considered:** GTK/iced retained-mode (more “app-like”, slower to spike); headless tests only (insufficient for the drag feel acceptance criterion).

### D7: Repository layout (first impl)

**Choice:** Keep shared OpenSpec at repo root; place the first implementation under something like `impls/linux-rust/` (exact name flexible) with a library crate for IR+solver and a binary for the UI.

**Rationale:** Matches the long-term “many impls, one spec” intent without coupling the IR to the GUI crate.

### D8: Undo

**Choice:** Snapshot document state at pointer-down; commit on successful pointer-up; single undo step restores the snapshot.

**Rationale:** Satisfies atomic drag with minimal machinery.

## Risks / Trade-offs

- **[Risk] Homegrown solver fragility (local minima, rank deficiency)** → Mitigation: keep sketches tiny in v1; damping + stay term; finite-difference Jacobian first; golden-path tests (right angle, rectangle drag); failure keeps last good state.
- **[Risk] Residual scaling (length vs length²)** → Mitigation: optional normalization in backend only; document that specs define zero-sets, not weights.
- **[Risk] Degenerate zero-length segments** → Mitigation: accept per catalog; avoid div-by-zero in any normalized residual variants.
- **[Risk] Over-building UI before runtime is solid** → Mitigation: tasks order IR → residuals → headless solve tests → then UI drag loop.
- **[Trade-off] Endpoint parameterization overparameterizes connected figures vs shared-point models** → Accept for Model E purity; Congruence removes DOF; UI still shows shared points.

## Migration Plan

Not applicable (greenfield). Future archive of this change will promote delta specs into `openspec/specs/`.

## Open Questions

- Exact GUI crate (egui vs iced vs gtk-rs) — pick at first UI task; does not affect specs.
- Whether headless integration tests run in CI inside the existing devcontainer — decide when the Rust toolchain is added to the container (tooling change, can be a follow-up).
