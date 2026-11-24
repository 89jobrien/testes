# Current state

Read this before changing the crate layout, CI, or any doc that claims what is implemented.

**Current state**: Rust 2024 Cargo workspace of nine crates — `testes-core`
(draft types/traits in `testes-core/src/types.rs`, `testes-core/src/traits.rs`)
plus one crate per dimension (`testes-unit`, `testes-property`, `testes-fuzz`,
`testes-model-check`, `testes-conformance`, `testes-integration`,
`testes-regression`, `testes-mutation` — see
`docs/decisions/0003-eight-way-dimension-crate-split.md`), with baseline unit
coverage in `testes-core` and CI running `--workspace` (see
`.github/workflows/ci.yml`). Core error types (`IdError`, `DeclarationError`,
`EvidenceShapeError`, `BackendError`, `CheckFailure`) derive `thiserror::Error` +
`miette::Diagnostic` — `miette`/`thiserror` are `testes-core`'s only
dependencies, added as a recorded exception to the prior std-only preference
(see `docs/decisions/0001-layered-dependencies-and-native-testing-backends.md`).
The eight dimension crates depend only on `testes-core`. `testes-fuzz` implements
`fuzz!` with an in-process seed-corpus runner (no coverage guidance, no libFuzzer;
see `docs/decisions/0006-fuzz-macro-grammar.md`). `testes-model-check` implements
`prove!` with a `ModelCheckBackend` port and no Kani dependency. `testes-integration`
implements `integration!` (one scenario per invocation). `testes-regression`
implements `regression!` as a native macro with identity checks and no backend.
`conformance!` and `mutate!` are planned API, not implemented symbols, unless their
crate states otherwise. `unit!` and `property!` are planned API, not implemented
symbols, unless their crate states otherwise.
