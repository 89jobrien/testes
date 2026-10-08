# Conformance

## Definition

Verify an implementation against a reusable suite owned by its trait or protocol contract. Compilation proves neither semantic correctness nor contract coverage.

## Declaration

`TestSpecification::Conformance` identifies contract/version, implementation/configuration, and cases. Planned macro: `testes::conformance!` (not implemented). Existing interfaces are `ConformanceCase`, `ConformanceSuite`, `SubjectFactory`, and `SubjectCleanup`; their orchestration is not implemented.

## Execution evidence

`ConformanceEvidence` records contract, implementation, capability observations, case IDs, obligations, durations, and outcomes. Each case should receive an isolated subject. Setup and cleanup failures must remain visible.

## Acceptance

Every applicable required obligation executes and passes. Contract-owned capability rules determine applicability; an implementation cannot opt out of inconvenient cases. Missing required capability does not satisfy the claim.

## Example obligation

Every observation-store adapter rejects conflicting identities while preserving idempotent replay.

## Limits

Keep semantic cases with the trait owner. Real I/O or generated inputs may be used without changing the contract-owned nature of the suite.
