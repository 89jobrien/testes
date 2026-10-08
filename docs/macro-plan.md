# Macro implementation plan

All syntax remains to be specified. Do not copy conversation sketches into executable examples or describe the eight macros as available.

| Macro | Native target | Key requirements |
|---|---|---|
| `unit!` | Ordinary Rust test | Stable ID, visible assertions, honest completion |
| `property!` | Proptest harness | Strategies, shrinking, replay/persistence, campaign configuration |
| `fuzz!` | Dedicated fuzz target | Invoke boundary, explicit oracles, corpus, resource budget |
| `prove!` | Kani harness | Visible symbolic inputs, assumptions, bounds, obligations |
| `conformance!` | Trait-owned suite/cases | Fresh subjects, capability policy, setup/cleanup results |
| `integration!` | Wired seam test | Real/fake boundaries, process/fixture lifecycle, diagnostics |
| `regression!` | Underlying harness wrapper | Stable failure identity, retained reproducer, no redundant execution |
| `mutate!` | Campaign declaration/plan | Passing baseline, isolated mutation, selected checks, per-mutant outcomes |

## Per-macro acceptance

- Document exact grammar and examples before implementation.
- Generate a valid native harness or plan; do not merely label an item.
- Preserve input evaluation, ownership, and error semantics.
- Diagnose invalid declarations clearly.
- Test downstream invocation without convenient imports and with renamed dependencies where relevant.
- Test metadata-only registration cannot fabricate execution evidence.
- Keep required/optional backend behavior explicit. Missing tooling must not silently pass.
- Do not run subprocesses or mutation campaigns during expansion.

## Suggested implementation order

1. Baseline checks and core validation tests.
2. `unit!` plus one honest evidence path.
3. Conformance lifecycle and contract suite declaration.
4. Property/regression replay.
5. Integration/process helpers.
6. Fuzz and proof backend wrappers.
7. Mutation orchestration and seeded faulty subjects.

This ordering is a development plan, not a requirement to defer proof-critical behavior or postpone regression tests. Every slice includes its own tests.
