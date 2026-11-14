# Proposal: Split long spec requirements

## Why

`openspec validate --specs --strict` fails for 12 of 19 specs. The only cause is the warning "Requirement text is very long (>500 characters)": 53 requirements state several behaviours in one description. This change implements #62. The issue lists them; later changes added a few more.

Long requirements are hard to read and to test one behaviour at a time, and every spec change has to step around the warnings.

## What Changes

The split rule from #62 is applied to every requirement whose description exceeds 500 characters:

1. **MODIFIED:** the long requirement keeps its heading and **every** scenario. Its description is cut to one behaviour, without changing its meaning.
2. **ADDED:** each behaviour removed from the description becomes its own requirement with its own scenarios. Where an existing requirement already covers that topic, that requirement takes it over instead (MODIFIED).
3. **No behaviour change:** every SHALL statement survives, in the original requirement or in a new one. Implementations and tests stay as they are.

Implementation details that do not belong in a shared spec, such as module names of one implementation, are **not** removed here. That needs a change that says so. Where a sentence like that exists, it moves along with its behaviour.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

Requirement texts only; the behaviour is unchanged.

- `geometry-ir`: 15 requirements
- `geometry-eval`: 7
- `system-architecture`: 5
- `display-mesh`: 4
- `state-vector`: 4
- `system-assembly`: 4
- `constraint-catalog`: 3
- `sketch-commands`: 3
- `solve-interaction`: 3
- `sketch-file`: 2
- `solver-port`: 2
- `drawing-export`: 1

## Non-goals

- **Changing behaviour**, or removing or weakening any requirement.
- **Removing implementation details from specs** (see above).
- **Splitting requirements of 500 characters or fewer** for style.
- **Product progress:** this change advances no feature of the product goal. It makes the shared specs, which every implementation follows, easier to read and validate.

## Impact

- `openspec/specs/*/spec.md` for the 12 capabilities above, after sync.
- No code, test or file-format change.
- `openspec validate --specs --strict` passes afterwards.
