---
name: local-coder
description: Delegate small, mechanical, test-verifiable Rust coding tasks (tests from OpenSpec scenarios, boilerplate impls, single functions with a fixed signature, compiler-error fixes, rustdoc) to the local Ollama model via aider, then review the result. Use before writing such code yourself.
---

# Delegating to the local model

`tools/local-coder/delegate.sh` runs aider with a local Ollama model inside the
dev container. aider edits, runs `cargo test`, and retries on failure, all
locally. You only see a compact summary. The point is to save your output
tokens, so only delegate when the task spec is much shorter than the code.

## When to delegate

Delegate:
- `#[test]` functions derived from OpenSpec `WHEN/THEN` scenarios
- trait impls / `From` / `Display` / builders following an existing pattern
- one function whose signature, location and expected behavior you can state exactly
- rustdoc for existing items

Do it yourself:
- anything touching solver math, geometry IR design, or several crates
- tasks where you cannot name a concrete test that proves success
- renames/refactors (rust-analyzer, `sed`)
- a task the local model already failed once after a refined prompt

## Writing the task

The local model sees only the listed files plus a repo map. Be explicit:
- the exact file and the exact item (fn signature, impl header, test names)
- the types/functions it must use, with their module paths
- what not to touch ("do not change other functions in this file")
- the test that must pass

Keep each delegation to one file where possible; small models rewrite whole files.

## Running

```bash
tools/local-coder/delegate.sh -p "<task>" [-r context.rs ...] [-t "<narrow test cmd>"] <file>...
```

- Paths are relative to the repo root.
- Narrow the test command to the crate/test you care about, e.g.
  `-t "cargo test --manifest-path impls/rust/Cargo.toml -p geometry_eval scenario_"`;
  the workspace-wide default is slow.
- Runs take minutes: call Bash with `timeout: 600000`, or `run_in_background`.
- Model: default `qwen3-coder:30b` (~17 tok/s on this machine); `-m qwen2.5-coder:7b`
  is faster but only for trivial single-file edits.
- Works from the host (docker exec into the dev container) and inside the container.
- Exit 2 means setup problem (container not running, Ollama down): tell the user, don't retry.

## Reviewing

- `result=green`: read `git diff -- <files>` and review it like any other code
  (spec conformance, project idiom, no weakened assertions, no unrelated edits).
  Fix small issues directly.
- `result=red|timeout`: look at the excerpt only. Either retry once with a
  sharper task, or run the printed rollback command and do it yourself.
  Don't read `aider.log` unless diagnosing the setup.

Run history (for judging whether delegation pays off): `.local-coder/runs.tsv`.
