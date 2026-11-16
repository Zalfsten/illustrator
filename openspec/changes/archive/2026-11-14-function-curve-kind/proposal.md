# Proposal: The function curve kind (smooth cases)

## Why

The product goal's central teaching figure is coordinate axes with a function curve, a constrained tangent and labels. Expressions (#78), kind attributes (#79), cubic shape pieces (#80) and curvature-based meshing (#75, #86) are in place. What is missing is the curve itself. This change is **change D** of #74 (#81), the first user-visible one. Changes E (abscissa addressing, #82), F (singularities, #83) and G (app UI, #84) build on it.

## What Changes

- **New geometry kind `function`** (name prefix `fn`): the graph of `y = f(x)` over a range `[x0, x1]`, in a frame with origin `O` and heading `θ`, scaled by `su` and `sv`:
  - **DOFs:** `O` and `θ`, as for a label. They are what the user controls relationally (join, parallel).
  - **Attributes, never solved:** the expression source, `x0`, `x1`, `su` and `sv` (default 1). They are what the user controls numerically.
  - **Chart:** `γ(t) = O + R(θ)·(su·x(t), sv·f(x(t)))`, where `t ∈ [0, 1]` is the **arc-length fraction**, so `Normalized` = `Uniform`. The local curve is precomputed whenever an attribute changes; a solve moves only `O` and `θ`.
  - **Capabilities:** curve (positions, tangent, normal, length), endpoints, anchor (`origin` = `O`) and line (`direction` = the frame's x axis).
  - **Shape:** cubic Bézier pieces within the requested tolerance, built with the Hermite builder from #80.
- **Smooth cases only:** `f`, `f′` and `f″` must be proven finite on the whole closed range (domain check of #78). Otherwise creation or the attribute change is rejected; this includes a vertical tangent at a range end. Change F widens this.
- **Commands:**
  - `create` with `kind: "function"`: `origin`, `heading`, `expression`, `range` and optional `scale`;
  - `set` for `expression`, `range` and `scale`;
  - a new error class `invalid expression` for parse errors and unproven domains, with position or sub-range.
- **Sketch file version 4:** version 3 plus function curves. Version 1–3 files load as before; a version 3 file with a function curve is rejected.
- **Spec conflict resolved:** geometry-ir's "Normalized chart is the canonical parameterization" requires `t` to be the defining formula's variable affinely mapped (here `x`). #74 decided on an arc-length chart, so that later vertical tangents (#83) keep a finite tangent. The requirement is opened for kinds whose normalized chart is the arc-length fraction.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `geometry-ir`:
  - the function curve becomes an admitted kind;
  - its chart, capabilities and smoothness rule;
  - its normalized parameter is the arc-length fraction;
  - the canonical-parameterization requirement allows arc-length charts.
- `sketch-commands`:
  - create and set for function curves, with their interchange fields;
  - the `invalid expression` error class;
  - the declarative `create` carries every function attribute.
- `sketch-file`: version 4; version 3 files load without function curves.

## Non-goals

- **Abscissa addressing** `fn.at(x=…)` (change E, #82). Addresses use `Normalized` / `Uniform` only.
- **Singularities** (vertical tangents, corners, cusps; change F, #83) and **branches across poles** (#76).
- **App UI:** a function tool and edit panel (change G, #84). Function curves are created by command or file and are drawn, dragged and exported by the existing app paths.
- **Parametric input** `x(τ)`, `y(τ)` and closed curves.
- **A provable deviation bound for the shape** (#99). The Hermite builder's sampled estimate is used.
- **Automatic Jacobians** for other kinds (#77).

The specs stay shared across implementations. The tabulation, the interpolation of the arc-length chart, the evaluation scratch and the numeric accuracies are design decisions of each implementation.

## Impact

- **impls/rust `constraint_runtime`:**
  - new kind `geometry_kinds/function.rs` with `FunctionAttrs` (attributes and the derived table);
  - `KindAttrs::Function`, the kind catalog and `with_kind!`;
  - a new dependency on the `expression` crate.
- **impls/rust `sketch_session`:**
  - `kinds/function.rs` (create, set, params), the catalog, the wire/JSON forms and the JSON Schema;
  - `CommandError::InvalidExpression`;
  - the file version 4 and its migration.
- **impls/rust `sketch_ui` / `sketch_export`:** no new code expected. Curves are drawn from the Mesh and exported from shapes generically. Tests cover the round trip.
- **Docs:** `openspec/config.yaml` (kind list, file version, dependencies).
