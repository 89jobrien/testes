# Regression

## Definition

Preserve a reproducer that distinguishes a previously observed defect from repaired behavior. Regression is a purpose layered over an execution dimension, not a separate backend.

## Declaration

`TestSpecification::Regression` identifies failure, underlying dimension, and reproducer artifacts. The current core accepts unit, property, fuzz, model check, conformance, or integration as the underlying dimension. Macro: `testes_regression::regression!` (native execution, no backend).

## Execution evidence

`RegressionEvidence` retains failure ID, reproducer artifacts, optional historical failure, and the boxed underlying dimension evidence. Reuse one execution; do not run it again merely to label it regression. The `regression!` macro takes the nested execution as a block returning a `NestedExecution` (outcome, dimension evidence, retained artifacts). A panic inside that block produces no record, because nested evidence cannot be fabricated.

## Identity checks

`testes_regression::check_regression_identity` requires that the recorded failure ID equals the declared failure, the recorded reproducer list equals the declared reproducers, at least one reproducer is declared, every declared reproducer and any historical failure artifact appear among the retained artifacts. Nested execution identity (for example a fuzz target or conformance contract) is not checked, because the regression layer does not carry the nested declaration. Core `validate_shape()` still enforces the underlying dimension match.

`regression!` passes only for `Passed` with `AssertionsSatisfied`. Failed, inconclusive, not-executed, and unavailable outcomes panic, so `cargo test` never reports them as passing.

## Acceptance

The retained reproducer executes and the assertion separating repaired from defective behavior succeeds. Missing replay material prevents support. Preserve regression tests and counterexamples.

## Example obligation

The input that previously overflowed budget consumption can no longer bypass exhaustion checks.

## Limits

A current pass does not establish that the original implementation failed the test. Retain historical evidence when available; absence is explicit, never fabricated.
