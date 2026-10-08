# Validation and acceptance checklist

## Current baseline commands

Run from the repository root with a toolchain supporting the manifest edition:

```sh
cargo fmt --check
cargo check --all-targets
cargo test --all-targets
cargo test --doc
cargo clippy --all-targets -- -D warnings
```

These are verification instructions, not recorded successes. The initial publication did not execute them. Compilation or a zero-test run is not behavioral coverage. Formatting may require correction before checks pass.

## Core behavior tests to add

- IDs reject blank/control-character input and preserve valid values.
- Declaration/specification dimension mismatch fails.
- Duplicate obligation IDs fail.
- Regression cannot wrap unsupported dimensions.
- Evidence/specification dimension mismatch fails.
- Regression payload matches the declared underlying dimension.
- Mutation rate handles mixed outcomes and zero assessed experiments.
- Evaluators cannot accept missing obligations, wrong revisions, insufficient scope, or missing/corrupt artifacts.
- Required capabilities and deferrals cannot become verified claims.

Use inline unit tests for pure logic. Add Proptest strategies for non-trivial domains only after defining the invariant. Preserve regressions. Model-check designated safety-critical arithmetic under justified assumptions once proof policy is agreed.

## Macro and backend checks to add

Use downstream compilation and compile-fail fixtures, including dependency renaming and invocation without imports. Test side-effecting inputs are evaluated as documented. Native fuzz/proof/mutation checks require explicit toolchain and backend installation; there are no project wrappers for these commands yet.

Execute all required conformance cases and report setup, capabilities, execution, and cleanup. Integration checks must name the seam and distinguish real from substituted boundaries. Mutation checks need a passing baseline and separate caught/missed/unviable/timeout/excluded outcomes.

## CI and release

Add CI after establishing a local baseline. Report separate lanes and exact backend versions/configuration. Do not count an omitted lane as passed. Future release gating must include downstream consumers, docs/examples, and retained replay evidence. Keep `publish = false` until licensing, registry naming, and release approval are resolved.

## Completion report

Record revision, files changed, exact commands/results, executed dimensions, unavailable checks, artifact references, unresolved design questions, and next task. Do not claim complete eight-dimensional verification from symbol presence or source grep.
