# ADR-0006: fuzz! Macro Grammar and In-Process Seed Runner

- Status: Accepted
- Date: 2026-11-24
- Project: `testes`
- Issue: #2

## Context

`testes-fuzz` was a stub. The `fuzz!` dimension needs a declaration macro that
records `DimensionEvidence::Fuzz` through the existing `testes-core` types, and it
must not add a libFuzzer or cargo-fuzz dependency to core or to the crate without a
recorded decision (ADR-0001 §5 keeps libFuzzer behind a dedicated backend).

ADR-0001 lists the macro grammar as an open decision. This ADR settles the grammar
for `fuzz!` only.

## Decision

1. **Grammar.** `fuzz!` takes a named function and expands to a `#[test]` function:

   ```text
   fuzz! {
       #[optional attributes, e.g. should_panic]
       fn test_name {
           id: "test-id",
           subject: "subject-id",
           claim: "claim statement",
           target: "target-id",
           oracles: ["oracle-id", ...],
           corpus: [b"seed", ...],
           body: |input: &[u8]| { ... },
       }
   }
   ```

   The test name is an ident supplied by the caller. The macro does not use `paste`,
   so no new dependency is needed.

2. **Execution.** The generated test builds a `TestDeclaration` with a
   `TestSpecification::Fuzz` specification, runs the inline seed corpus through
   `run_seed_corpus`, records the result with `record_campaign`, and calls
   `assert_passed`. The test fails unless the status is `Passed`. A failing body,
   an empty corpus (`Inconclusive`), and an unavailable campaign (`NotExecuted`)
   all fail the test. None of them reads as a pass.

3. **Runner scope.** The bundled runner is a std-only, in-process seed-corpus runner.
   It runs each inline input under `catch_unwind` and stops at the first panic. It
   does not mutate inputs, does not guide by coverage, and does not link libFuzzer.
   Executions are recorded as `Observed::Available(n)`, where `n` is the number of
   target invocations actually made. Coverage is `Observed::Unavailable`.

4. **Unavailable campaigns.** `FuzzCampaign::Unavailable { reason }` is the port
   result for a backend that could not run, such as a missing cargo-fuzz toolchain.
   It maps to `ExecutionStatus::NotExecuted` with `OutcomeReason::CapabilityUnavailable`
   and `executions: Observed::Unavailable`. It is never coerced to zero. The macro does
   not yet expose this path. A future libFuzzer adapter will return it.

5. **Failure inputs.** A failing seed is reported by index and panic message. Its bytes
   are not retained, because no artifact store exists yet. `failure_inputs` stays
   empty, and the diagnostic says so. No artifact ID is invented.

6. **Oracles.** `oracle_ids` are recorded labels. The `body` closure encodes the oracle
   checks. The runner does not evaluate oracles separately.

7. **Declaration, execution, and evaluation stay separate.** `record_campaign` returns
   an `EvidenceRecord`. It does not decide whether the claim is supported. Claim
   evaluation and governance remain in `testes-core` traits.

## Consequences

- Core is unchanged. `testes-fuzz` depends only on `testes-core`. No new dependency.
- Passing the in-process seed runner is weaker than a coverage-guided fuzz campaign.
  Evidence records the runner identity (`testes-fuzz-seed-runner`) so this stays visible.
- A libFuzzer backend can later return `Executed` or `Unavailable` results through the
  same `FuzzCampaign` shape, without changing the macro grammar.
- The `should_panic` attribute can be used on declared fuzz tests, which lets the
  integration tests check the failure paths.
