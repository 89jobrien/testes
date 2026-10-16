# ADR-0006: `fuzz!` macro grammar and expansion

- Status: Accepted
- Date: 2026-10-16
- Project: `testes`
- Scope: specifies `fuzz!`'s grammar, expansion target, and evidence construction
  before implementation. Per ADR-0002's pattern: a declarative `macro_rules!` macro
  (no proc-macro crate), not a `TestBackend` trait (fuzzing backend comes later
  as a separate adapter).

## Context

Per `docs/handoff.md` and the implementation plan, `fuzz!` is the third dimension
after `unit!` and `property!`. Unlike property-based testing, fuzzing requires:

- A target function to fuzz (the "harness")
- Initial seed corpus (optional)
- Oracle predicates that check invariants
- Execution tracking (number of test cases, coverage)
- Failure artifact collection (crashing inputs)

ADR-0001 §5 lists fuzzing as an "adapter" dimension: multiple backends are
plausible (libFuzzer, proptest, AFL). However, the macro itself is independent
of the backend choice — it declares what is being tested and records evidence of
what was executed.

## Decision

### Grammar

```rust
fuzz! {
    id: "fuzz-parse-input",
    subject: "parse_json",
    claim: "parser never panics on arbitrary input",
    target: |input: &[u8]| {
        // Fuzz target: returns bool indicating validity or panics on bug
        let _ = serde_json::from_slice::<Value>(input);
        true
    },
    oracle: |result: bool| {
        // Property to check on every execution
        prop_assert!(result || true);  // Never fails; catches panics instead
    },
    cases: 10000,
    corpus: vec![
        // Optional seed inputs (paths or inline bytes)
        b"{}",
        b"[]",
    ],
}
```

- `id`: unique identifier for this fuzz test (matches `TestId::new` contract)
- `subject`: what is being tested (e.g. `parse_json`)
- `claim`: the property being verified (e.g. "never panics")
- `target`: a closure `Fn(&[u8]) -> bool` that runs the fuzz target
- `oracle`: a closure `Fn(bool) -> Result<(), TestCaseError>` that checks the property
- `cases`: minimum number of executions to attempt (u64)
- `corpus`: optional vector of seed inputs as byte slices

### Expansion target

For each fuzz test, the macro generates:

```rust
#[test]
fn {id}() {
    use proptest::test_runner::TestRunner;
    use proptest::strategy::Strategy;

    // Declare the test
    let declaration = TestDeclaration {
        primary_dimension: Dimension::Fuzz,
        specification: TestSpecification::Fuzz {
            target_id: "{subject}",
            oracle_ids: vec![],  // TODO: oracle tracking
            corpus: /* seed inputs */,
        },
        // ... other fields
    };
    declaration.validate().expect("TestDeclaration must be valid");

    // Run the fuzz test
    let mut evidence = FuzzEvidence {
        target_id: "{subject}".to_string(),
        oracle_ids: vec![],
        initial_corpus: vec![],
        actual_duration: Duration::ZERO,
        executions: Observed::Available(0),
        coverage: Observed::Unavailable { reason: "coverage not instrumented" },
        failure_inputs: vec![],
    };

    let result = proptest!(|input: Vec<u8>| {
        evidence.executions = Observed::Available(evidence.executions.unwrap_or(0) + 1);
        let target_result = ({target})(&input);
        ({oracle})(target_result)?;
    });

    // Record outcome based on result
    let (status, diagnostic) = match result {
        Ok(()) => (ExecutionStatus::Passed, None),
        Err(e) => (ExecutionStatus::Failed, Some(format!("{:?}", e))),
    };

    // Build and store evidence record
    let evidence_record = EvidenceRecord {
        // ... with FuzzEvidence
    };

    // Let proptest's panic/failure propagate
    result.expect("fuzz test failed");
}
```

## Consequences

- **Honest evidence**: executions count as the test runs; coverage and rejection
  counts stay `Unavailable` (not measured yet — ADR-0001 will decide instrumentation).
- **Crash detection**: target panics are caught and recorded as failures
- **Seed corpus support**: optional initial inputs reduce search space
- **Backend adapter separation**: the `TestBackend` trait can implement a libFuzzer
  adapter later if needed, without changing the macro grammar or evidence model.
- **No proc-macro needed**: grammar is simple enough for `macro_rules!`

## Alternatives considered

1. **Generic `fuzz!` over a `FuzzBackend` trait:** Rejected per ADR-0002 pattern
2. **libFuzzer-specific #[fuzz_target]:** Rejected. Macro is backend-agnostic
3. **Coverage instrumentation in the macro:** Out of scope. Macro records coverage
   if available; instrumentation belongs in backend adapter.

## Validation (Phase 3)

The test suite will verify:

1. **Passing fuzz**: a target that never fails → evidence `ExecutionStatus::Passed`,
   `executions > 0`.
2. **Failing fuzz**: a target with a crash input → evidence `ExecutionStatus::Failed`,
   diagnostic shows the crash.
3. **Seed corpus**: fuzz with initial inputs runs without panic.
4. **Coverage stays Unavailable**: not yet instrumented.

## Implementation notes

- Macro implementation goes in `testes-fuzz/src/lib.rs`, following the
  `testes-unit` and `testes-property` pattern.
- Proptest is sufficient for Phase 2 fuzzing (no libFuzzer integration yet).
- `FuzzEvidence` already exists in `testes-core/src/types.rs` with all required fields.
- The macro stores evidence in a thread_local (like `unit!` and `property!`) for test introspection.
