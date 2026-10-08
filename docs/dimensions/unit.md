# Unit

## Definition

Verify one coherent behavior against explicit examples. A behavioral unit may call helpers; it need not be exactly one function. Prefer local tests, direct values, and lightweight fakes.

## Declaration

`Dimension::Unit` and `TestSpecification::Unit` identify the subject, claim, obligations, and example IDs. Planned public entry point: `testes::unit!` (not implemented). Intended backend: ordinary Rust tests.

## Execution evidence

`UnitEvidence` records executed example IDs and observed assertion counts. The common envelope records revision, configuration, outcome, and diagnostics. Preserve fixture identity or inputs when needed for replay.

## Acceptance

Every required example must execute and satisfy its assertions. Early return before required checks must not satisfy the claim. An unavailable assertion count must remain unavailable; do not invent instrumentation.

## Example obligation

An empty command argument is rejected with the intended error.

## Limits

Success supports the selected examples, not the entire input domain. Test scope and assertions matter more than file location.
