# Architecture and decisions

## Agreed design

The eight entry points are `unit!`, `property!`, `fuzz!`, `prove!`, `conformance!`, `integration!`, `regression!`, and `mutate!`. They are planned APIs, not implemented symbols.

Dependency direction:

```text
project tests -> public testes API
backend adapters -> core types and traits
orchestrator -> adapters + evidence evaluator + storage
project contract suites -> contract-owning project + generic conformance interfaces
```

Core must not depend on project domains, process runners, Proptest, Kani, libFuzzer, or cargo-mutants. Backend-specific integration belongs outside the core dependency boundary.

## Decision boundaries

1. `TestDeclaration` states a claim and required scope.
2. `TestBackend` executes and records actual scope/results.
3. `EvidenceEvaluator` determines supported, refuted, or insufficient evidence.
4. `GovernancePolicy` handles applicability and deferrals without converting them into verified claims.

Assertion failure belongs in evidence. Infrastructure failure that prevents trustworthy execution is not a behavioral pass or refutation.

## Ownership

Trait owners maintain semantic contract suites. Shared infrastructure handles generic case lifecycle/reporting. Projects retain domain strategies, proof assumptions, fuzz boundaries, and regression reproducers.

## Crate evolution

The workspace is split into `testes-core` plus one crate per dimension
(`testes-unit`, `testes-property`, `testes-fuzz`, `testes-model-check`,
`testes-conformance`, `testes-integration`, `testes-regression`,
`testes-mutation`), so each dimension can be pulled in independently — see
`docs/decisions/0003-eight-way-dimension-crate-split.md`. Each dimension crate
depends only on `testes-core` for now; add a procedural-macro crate or a
heavier backend dependency only when a reviewed grammar or implementation need
requires it.

## Open decisions

Record decisions with context, alternatives, consequences, and validation before implementing changes to:

- Error diagnostics dependency choice — resolved per
  `docs/decisions/0001-layered-dependencies-and-native-testing-backends.md` (`miette` + `thiserror` added to core).
- Macro grammar and hygienic dependency resolution.
- Async/object-safe port execution.
- Evidence serialization and schema evolution.
- Cryptographic digest selection and artifact trust boundary.
- Proof obligations and approved deferrals.
- Backend feature gates and toolchain/platform matrix.

Avoid duplicating dimension definitions here; docs/dimensions is their source of truth.
