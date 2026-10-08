# Execution evidence contract

A declaration is a claim, not execution evidence. A passing run supports only its recorded subject and scope.

## Common record

`EvidenceRecord` combines:

- `TestDeclaration`: stable ID, dimension, subject, claim, obligations, required scope, disposition, and specification.
- `Provenance`: repository revision, working-tree state, and run identity.
- `ExecutionMetadata`: backend/version, configuration, start time, duration, and actual scope.
- `ExecutionOutcome`: status, reason, diagnostic, and obligation results.
- `ArtifactReference`: retained material, location, digest, and disclosure policy.
- `DimensionEvidence`: dimension-specific observations.

Requested scope and actual scope must remain distinct. `Observed::Unavailable` must not become zero or fabricated measurement data. Redact sensitive evidence before storage; record restrictions that prevent reproduction.

## Outcomes versus policy

Execution statuses are `Passed`, `Failed`, `Inconclusive`, and `NotExecuted`. Assertion violations are evidence; failures preventing trustworthy execution are infrastructure errors. Cancellation and incomplete budgets do not support success.

`ObligationDisposition` expresses required, not applicable, or deferred policy intent. Applicability and deferrals require rationale; deferrals also identify an owner and review condition. A governance decision accepting a deferral does not establish the claim.

`EvidenceEvaluator` returns `Supported`, `Refuted`, or `Insufficient`. `GovernancePolicy` separately decides policy satisfaction. Neither interface has an implementation yet.

## Acceptance obligations

An evaluator must check declaration identity, subject, revision, required scope, required obligation results, backend completion, and required artifact availability/integrity. Dirty trees need reproducible patch evidence. Missing required capability means the obligation was not satisfied.

`validate()` and `validate_shape()` perform limited structural checks only. They do not establish scope coverage, artifact integrity, truthful measurements, baseline success, or policy compliance.

## Replay and retention

Retain minimized counterexamples, representative fuzz inputs, proof reports/traces, mutation reports, and historical reproductions where available. Do not fabricate historical failure evidence. Track declared, executed, satisfied, deferred, and unresolved obligations separately.
