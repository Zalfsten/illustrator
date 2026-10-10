# Design

## Context

linux-rust already separates `Point2` / `Vec2` and opaque `PointExpr` / `VecExpr` / `LengthExpr`. A single `Angle` newtype exists only as a `StateAtom` (chart tests); no geometry stores it. Config still mentions a future undifferentiated `AngleExpr`. The next kind (circular arc with `start` + `sweep`) needs the absolute/free split before kind formulas land.

Specs stay shared; this design records the linux-rust approach under those contracts.

## Goals / Non-Goals

**Goals:**

- Domain types `HeadingAngle` and `TurnAngle` with Point/Vec-analogue algebra.
- Both packable as 1-DOF chart atoms.
- Opaque `HeadingAngleExpr` / `TurnAngleExpr` with the same evaluate / param_indices / param_count / accumulate_jacobian surface as `LengthExpr`.
- Docs + `openspec/config.yaml` vocabulary updated (no single `AngleExpr`).

**Non-Goals:**

- Arc / Kreissegment kind, Mesh edges for arcs, angle constraints.
- Kind factories that construct heading/turn expressions (empty keyed catalogs until a kind registers).
- `mod 2π` normalization inside the DOF types.

## Decisions

### 1. Names: `HeadingAngle` / `TurnAngle` (and matching Exprs)

**Choice:** Explicit `*Angle` suffix on both geom and expression types (`HeadingAngleExpr`, `TurnAngleExpr`).

**Why:** Makes “these are angles” obvious at call sites; parallel naming across layers. Rejected shorter `Heading`/`Turn` after product preference in exploration.

### 2. Replace `Angle`, do not keep it as an alias

**Choice:** Remove `Angle`; migrate `StateAtom` and tests to the two new types.

**Why:** An undifferentiated leftover would invite misuse. No production call sites depend on `Angle` today.

### 3. Algebra (no wrap)

```text
HeadingAngle - HeadingAngle -> TurnAngle
HeadingAngle + TurnAngle    -> HeadingAngle
HeadingAngle - TurnAngle    -> HeadingAngle
TurnAngle + TurnAngle       -> TurnAngle
TurnAngle - TurnAngle       -> TurnAngle
-TurnAngle                  -> TurnAngle
```

Heading + Heading is not implemented. Values are raw radians in State; wrapping is a later arc/constraint concern.

### 4. Expression layering matches length

**Choice:** Structs in `expr.rs`; kind-keyed enums + `impl` dispatch in `geometry/expr.rs` (extend `register_expr_kinds!` or a sibling macro path when the first kind registers formulas). Until then, keyed enums may be empty / unused — types and `StateAtom` packing are still required so the arc chart can compile next.

**Why:** Same Model B opacity as `LengthExpr`; constraints never match formula kinds.

### 5. Docs / config before arc

**Choice:** Update glossary and config angle vocabulary in this change so the arc proposal does not re-litigate Point/Vec-for-angles.

## Risks / Trade-offs

- **Empty expression catalogs:** Heading/turn Exprs exist without a constructing factory until the arc change — acceptable for foundation; tests cover geom algebra + `StateAtom` pack/unpack, not full eval dispatch.
- **PackAtoms arity:** Arc init `(Point2, Length, HeadingAngle, TurnAngle)` already fits the 4-tuple `PackAtoms` impl; no change required in this slice beyond atom impls.
- **Spec wording:** “Scalar expressions” now means typed families (length | heading | turn), not one erased type — implementations must keep separate APIs.

## Migration

linux-rust only: rename/replace `Angle` in `geom`, `chart`, tests, `lib` re-exports. No document migration (no stored angles yet).
