# Developer handoff

## Baseline

Handoff baseline: `9923548431dbc5794f9d12e1ced80f4f40977488` on main. The initial core commit is `0b257213b68363f7de2f0e89ffbefa6ab2abb0ca`. This document adds guidance; it does not claim checks have run.

## Existing assets

- `Cargo.toml`: a single Rust 2024 package, no dependencies, `publish = false`.
- `src/lib.rs`: public re-exports and unsafe-code prohibition.
- `src/types.rs`: identities, declarations, scope, provenance, artifacts, all eight evidence variants, and limited shape validation.
- `src/traits.rs`: backend/context/storage/sink ports, evaluators, governance, and conformance interfaces.
- `docs/dimensions/`: one definition and evidence contract per dimension.

No public dimension macros, concrete backends, tests, CI, workspace split, persistence implementation, or serialization are present in this baseline. Compilation, formatting, and lint status are unverified.

## First bounded task

1. Check toolchain availability and run the validation commands in validation.md.
2. Record failures before changing code; fix compilation/formatting issues in a focused change.
3. Add inline unit tests for ID construction, declaration validation, evidence shape checks, and mutation-rate calculation. Use contextual setup failures.
4. Add downstream compilation fixtures for public traits, including `dyn` conformance subjects.
5. Produce a baseline report distinguishing executed results from unavailable checks.

Do not begin with all backend integrations at once.

## Next vertical slice

Specify `unit!` grammar, implement native test generation, and connect one execution to explicit evidence. Include successful, failing, inconclusive, and not-executed examples. Prove that merely registering a declaration does not produce success. Then implement reusable conformance mechanics before broad campaign orchestration.

## Known review questions

- Public fields allow malformed evidence; shape validation is intentionally incomplete.
- Subject identity, required/actual scope, obligation IDs, and referenced artifacts need semantic matching.
- Regression nesting currently checks dimension only, not all failure identity or artifact consistency.
- Mutation-rate arithmetic does not check baseline validity or policy.
- `GovernanceVerdict::Satisfied` contains a boolean whose consistency needs enforcement or a stronger type.
- Synchronous conformance interfaces may need an explicit async adapter strategy.
- Setup/cleanup timing exists, but conformance evidence does not yet encode every lifecycle phase separately.

These are design review items, not claimed compiler failures.

## Owner decisions needed

Macro grammar; proof-critical policy; MSRV and platform support; async strategy; artifact storage/retention; serialization/schema compatibility; license and publication plan. The crate name on a public registry has not been checked.
