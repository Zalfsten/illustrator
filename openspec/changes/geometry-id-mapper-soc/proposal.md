# Proposal

## Why

Document geometry storage already uses a shared shell and object-safe instances, but identity allocation, State↔chart mapping, and geometry lookup are still tangled: `DocumentStructure` owns `next_id`, unpack helpers take a `GeometryId` and look up again, and callers pass id+State separately. Clearer module boundaries are needed without ECS side-maps or nested type towers — so chart math stays free of registry concerns while evaluate/solve stays flat and readable.

## What Changes

- Introduce a dedicated **geometry identity allocator** (allocate / observe) used by Document structural add; geometries do not self-assign ids and do not call a global generator.
- Introduce a **chart mapper** concern: given a resolved geometry instance and a State, unpack that geometry's chart init (and expose its parameter window) without a second id lookup inside unpack.
- Keep **integer stable geometry ids** and a single **GeometryShell** companion (`id`, role, param offset) on instances — no UUID default, no split companions, no offset-only side tables in this change.
- Structural add remains through the Document composite (atomic Structure + State); Document assigns id and offset when registering.
- Expressions continue to store geometry ids; evaluate resolves once, then unpacks via geometry reference.

## Non-goals

- UUID-based ids; ambient/global id generation from inside geometry constructors.
- ECS / side-map layout (id or offset only in external tables).
- Splitting GeometryShell into multiple companion structs.
- New curve kinds, solver, Mesh, or UI features.
- Mandating a particular language module layout in shared specs (linux-rust design records IdGenerator / Mapper names).

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `geometry-ir`: Clarify that stable ids are allocated by the document/registry authority on structural add (and observably updated on explicit import), not by chart formulas; resolution remains by identity without full scans.
- `state-vector`: Clarify that chart unpack / parameter-window reads for evaluation take a **resolved geometry record** (plus State), not a fresh identity lookup inside the unpack step; window still comes from the geometry record in structure.

## Impact

- linux-rust: extract id allocation from `DocumentStructure.next_id`; add a thin mapper/view (or equivalent store helpers) taking `&dyn Geometry`; update `expr` evaluate path to resolve then unpack; docs/`openspec/config.yaml` context alignment.
- Parallel to (does not replace) change `geometry-object-safe-shell`, which syncs object-safe shell language into main specs — apply/archive order: prefer finishing shell-spec sync first or merge carefully if both touch `geometry-ir`.
- Specs remain implementation-agnostic; concrete `IdGenerator` / `Mapper` types live in design.md.
