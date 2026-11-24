# Architecture: decision layers

Read this before changing how declarations, backends, evidence, verdicts, or governance interact, or before adding a dependency between crates.

Four-layer decision boundary (see `docs/architecture.md`):

1. `TestDeclaration` (testes-core/src/types.rs) — states a claim and required `Scope`. Validated
   structurally via `TestDeclaration::validate()` (dimension/specification match, no
   duplicate obligation IDs, regression can't wrap itself or mutation).
2. `TestBackend` (testes-core/src/traits.rs) — executes a declaration against a `RunRequest` and
   `ExecutionContext`, returns an `EvidenceRecord` or a `BackendError`. Assertion
   failures are evidence (`ExecutionOutcome`), not `BackendError` — infrastructure
   failure that prevents trustworthy execution is a different thing from a failing
   assertion.
3. `EvidenceEvaluator` — turns declaration + evidence + `EvidenceRequirements` into a
   `ClaimVerdict` (`Supported` / `Refuted` / `Insufficient { gaps }`).
4. `GovernancePolicy` — decides applicability/deferral policy on top of a
   `ClaimVerdict`, but an accepted deferral must never flip `Insufficient`/`Refuted`
   into `Supported`.

Dependency direction is one-way: project tests → public testes API; backend adapters →
core types/traits; orchestrator → adapters + evaluator + storage; project contract
suites → the project that owns the port + generic conformance interfaces. Core never
depends upward into any of these.
