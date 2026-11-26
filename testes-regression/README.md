# testes-regression

The `regression!` dimension. It wraps the evidence of one nested execution in a `RegressionEvidence` record, which says a previously observed defect is still prevented.

Status: implemented. Native execution only, with no backend.

## What it does

- Takes a `RegressionSpec` (id, subject, claim, the declared failure, the underlying dimension, and the declared reproducers) and a `NestedExecution` supplied by the caller.
- Reuses the nested execution. It does not run the test a second time to label it as a regression.
- Checks identity with `check_regression_identity`:
  - the recorded failure ID matches the declared one;
  - the retained reproducer list matches the declared reproducers;
  - every declared reproducer, and any historical failure artifact, is among the retained artifacts;
  - at least one reproducer is declared.

Nested execution identity, such as the fuzz target or conformance contract, is not checked here. The regression layer does not carry the nested declaration.

## API

- `record_regression(...)` builds the record.
- `check_regression_identity(&record)` returns `Result<(), RegressionError>`.
- `assert_passed(&record)` panics unless the record passed.
- `regression!` is the macro form.

Keep reproducers and failure artifacts. Never delete them to make a check pass.

## Dependencies

- `testes-core`
