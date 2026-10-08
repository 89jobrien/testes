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

Start from the existing normal library. Add a procedural-macro crate only when a reviewed grammar requires it. Add optional backend packages or an orchestration binary only with an actual implementation need. No package layout beyond the current library is committed.

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
