# Property

## Definition

Search for counterexamples to an invariant over a strategy-defined domain. This is generated-input evidence, not universal proof.

## Declaration

`TestSpecification::Property` identifies the strategy, minimum cases, and regression artifacts. Required scope states domain constraints. Planned macro: `testes::property!` (not implemented). Intended backend: Proptest. Strategies remain project-owned.

## Execution evidence

`PropertyEvidence` records strategy ID, seed availability, completed/rejected cases, replayed artifacts, and any minimized counterexample. Record backend version and configuration in the envelope. Retain discovered counterexamples permanently.

## Acceptance

The required campaign completes, required regressions replay, and no invariant violation occurs. Generation abort or incomplete budget is inconclusive. Missing required replay material prevents claim support.

## Example obligation

Serialization followed by deserialization preserves every generated valid identifier.

## Limits

A restrictive strategy can hide defects. Use recorded seeds for reproduction without assuming seed behavior is permanently stable across backend versions. Maintain exploratory campaigns as well as deterministic replay.
