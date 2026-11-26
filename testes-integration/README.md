# testes-integration

The `integration!` dimension. It runs one scenario across a seam between connected components and records `IntegrationEvidence`.

Status: implemented. One scenario per invocation.

## Usage

```rust
testes_integration::integration! {
    id: "seam-test-id",
    subject: "checkout",
    seam: "checkout->payments",
    claim: "checkout charges through the payments port",
    components: [("checkout", Real), ("payments", Fake)],
    scenario: charges_through_payments => {
        // Interaction and assertions. Panics fail the scenario.
        // `return Err(ScenarioError::Unavailable { .. })` reports an unavailable capability.
        // `return Err(ScenarioError::NotExecuted { .. })` reports a scenario that did not run.
    }
}
```

This generates `#[test] fn charges_through_payments()`. The test records the evidence and panics unless the scenario passed, so unavailable or not-executed scenarios show up as failures.

`components` takes `BoundaryKind` variant names: `Real`, `Fake`, or `Simulated`. Component versions are recorded as absent.

## Notes

- The macro does not clean up, so the recorded cleanup outcome is `NotExecuted`.
- Fixtures and observations are not retained yet, so none are recorded.
- Lower-level entry points are `run_scenario(&spec, body)` and `assert_scenario_passed(&record)`.

## Dependencies

- `testes-core`
