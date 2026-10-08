# Regression

## Definition

Preserve a reproducer that distinguishes a previously observed defect from repaired behavior. Regression is a purpose layered over an execution dimension, not a separate backend.

## Declaration

`TestSpecification::Regression` identifies failure, underlying dimension, and reproducer artifacts. The current core accepts unit, property, fuzz, model check, conformance, or integration as the underlying dimension. Planned macro: `testes::regression!` (not implemented).

## Execution evidence

`RegressionEvidence` retains failure ID, reproducer artifacts, optional historical failure, and the boxed underlying dimension evidence. Reuse one execution; do not run it again merely to label it regression.

## Acceptance

The retained reproducer executes and the assertion separating repaired from defective behavior succeeds. Missing replay material prevents support. Preserve regression tests and counterexamples.

## Example obligation

The input that previously overflowed budget consumption can no longer bypass exhaustion checks.

## Limits

A current pass does not establish that the original implementation failed the test. Retain historical evidence when available; absence is explicit, never fabricated.
