# testes-core

Shared declarations, execution evidence, and policy interfaces for testes. Every dimension crate depends on this one, and it depends on no dimension crate.

Status: draft API. Core structs, enums, traits, and validation helpers have baseline unit coverage. No public macros or native backends live here.

## What it provides

- **Declarations** (`TestDeclaration`, `Claim`, `Scope`, `TestSpecification`): the claim a test makes, the scope it requires, and its dimension-specific specification. Identifiers such as `TestId` and `SubjectId` are validated on construction (`IdError`).
- **Execution records** (`ExecutionOutcome`, `ExecutionStatus`, `ExecutionMetadata`, `BackendIdentity`, `BackendConfiguration`, `RepositoryRevision`, `WorkingTreeState`): what ran, on which backend, against which revision. `Observed` distinguishes a measured value from `Unavailable`, which is never coerced to zero.
- **Evidence** (`EvidenceRecord`, `DimensionEvidence`, one variant per dimension): the record that a test run produces. Shape validation (`EvidenceShapeError`) checks that the dimension and its evidence agree.
- **Ports** (`traits`): `TestBackend`, `ExecutionContext`, `ArtifactStore`, `EvidenceSink`, `EvidenceEvaluator`, `GovernancePolicy`, plus the conformance interfaces `ConformanceCase`, `ConformanceSuite`, `SubjectFactory`, and `SubjectCleanup`.

The four concerns stay separate: a declaration states the claim, an execution records what ran, an evaluator decides what the evidence supports, and governance handles deferrals without turning them into verified claims.

## Dependencies

- `miette` and `thiserror` for error diagnostics. This is the recorded exception in [ADR-0001](../docs/decisions/0001-layered-dependencies-and-native-testing-backends.md). Do not add dependencies without a similar recorded decision.

`#![forbid(unsafe_code)]` is in effect.

## Related

- [Evidence contract](../docs/evidence.md)
- [Architecture](../docs/architecture.md)
- [Dimension definitions](../docs/dimensions)
