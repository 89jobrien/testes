# ADR-0004: `unit!` macro grammar and expansion

- Status: Accepted
- Date: 2026-10-16
- Project: `testes`
- Scope: specifies `unit!`'s grammar, expansion target, and evidence construction
  before implementation. See ADR-0002 for the port-boundary decision (no
  `TestBackend`); this ADR covers the macro syntax and what code is generated.

## Context

Per `docs/handoff.md:27-29` ("Specify `unit!` grammar, implement native test generation,
and connect one execution to explicit evidence") and ADR-0001 §3 ("record decisions with
context, alternatives, consequences, and validation before implementing"), the macro
grammar must be settled before code is written.

ADR-0002 established that `unit!` does not implement `TestBackend` — each generated
`#[test] fn` is its own composition root that constructs a `TestDeclaration` and
`EvidenceRecord` directly. This ADR specifies exactly what that expansion looks like.

## Decision

### Grammar

```rust
testes::unit! {
    id: "test-case-id",
    subject: "TestSubject::method_name",
    claim: "method_name behaves as expected",
    examples: {
        "example_name": {
            // Example body: ordinary assertions
            assert!(condition);
        }
    }
}
```

- `id`: a unique identifier for this test declaration (matches `TestId::new` contract:
  no blank/control-character strings, enforced at macro expansion time if possible, or
  validated at runtime by `TestDeclaration::validate()`)
- `subject`: a human-readable name for the thing being tested (stored as `scope.subject`)
- `claim`: a statement of what is being verified (stored as `specification.claim`)
- `examples`: one or more named example blocks; each name becomes an example ID

### Expansion target

For each example, the macro generates:

```rust
#[test]
fn {id}_{example_name}() {
    // Validate the declaration first — if this fails, the test fails immediately
    // with the validation diagnostic (proves malformed declarations cannot pass).
    let declaration = TestDeclaration {
        primary_dimension: Dimension::Unit,
        scope: Scope {
            subject: "{subject}",
            revision: "{revision or placeholder}",
        },
        specification: TestSpecification::Unit {
            claim: "{claim}",
            example_ids: vec!["{example_name}"],
        },
        obligations: vec![],
    };
    declaration.validate().expect("TestDeclaration must be valid");

    // Construct the evidence record that will be produced by this test.
    let mut evidence = EvidenceRecord {
        test_id: TestId::new("{id}").expect("TestId valid"),
        declaration: declaration.clone(),
        execution: DimensionEvidence::Unit(UnitEvidence {
            executed_examples: vec!["{example_name}"],
            assertions_evaluated: Observed::Unavailable {
                reason: "assertion counting not implemented",
            },
        }),
        backend_used: Some(BackendIdentity {
            name: "native-rust-test",
            version: "1.0",
        }),
        timestamp: /* current time */,
    };

    // Run the example body in a panic-safe context.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // User's example code injected here
        {example_body}
    }));

    // Record the outcome.
    let outcome = match result {
        Ok(_) => ExecutionOutcome {
            status: OutcomeStatus::Passed,
            assertion_evaluated: 1, // No real count; this is a placeholder
        },
        Err(payload) => ExecutionOutcome {
            status: OutcomeStatus::Failed,
            diagnostic: format!("{:?}", payload), // panic payload as diagnostic
        },
    };
    evidence.execution = DimensionEvidence::Unit(UnitEvidence {
        executed_examples: vec!["{example_name}"],
        assertions_evaluated: Observed::Unavailable {
            reason: "assertion counting not implemented",
        },
        outcomes: vec![(TestId::new("{example_name}").unwrap(), outcome)],
    });

    // Store the evidence in a thread-local for tests to inspect (Phase 3 validation).
    thread_local! {
        static LAST_UNIT_EVIDENCE: RefCell<Option<EvidenceRecord>> = RefCell::new(None);
    }
    LAST_UNIT_EVIDENCE.with(|e| *e.borrow_mut() = Some(evidence.clone()));

    // Re-panic if the example failed, so cargo test's own pass/fail reporting is unaffected.
    // The macro adds evidence, it does not replace Rust's test harness.
    if let ExecutionOutcome {
        status: OutcomeStatus::Failed,
        ..
    } = outcome
    {
        panic!("Example failed: {:?}", outcome.diagnostic);
    }
}
```

## Consequences

- **Honest evidence**: every generated `#[test]` builds a real `TestDeclaration`,
  validates it, and records `UnitEvidence` with `Observed::Unavailable` for assertion
  counts (because no instrumentation exists to measure them yet — never invented).
- **Proof that registration ≠ success**: the validation step (line 12 above) ensures
  a malformed declaration (duplicate obligation IDs, dimension mismatch, etc.) causes
  the test to fail before any example body runs.
- **Not-executed detection**: a test decorated with `unit!` but never run will have
  an `ExecutionOutcome::NotExecuted` entry in evidence (Phase 3 will test this).
- **Namespace safety**: generated function names are `{id}_{example_name}`, deterministic
  from the input, no hygiene issues because the macro doesn't need to capture bindings
  from the surrounding scope.

## Alternatives considered

1. **Procedural macro using `syn`/`quote`:** Rejected. A declarative `macro_rules!`
   suffices for this grammar (no custom diagnostics, no name binding capture needed).
   ADR-0002's choice to avoid a proc-macro crate stands.
2. **Generic over a `TestBackend` or `CaseRunner` trait:** Rejected per ADR-0002
   (single implementation anti-pattern). The generated code is the composition root.
3. **Async/await support in the macro:** Out of scope for this phase. Native `#[test]`
   doesn't support async directly; a future `async_unit!` variant would be a separate
   macro. This grammar targets synchronous examples only.
4. **Assertion counting via compile-time macro introspection:** Not feasible with
   `macro_rules!` and not a requirement for Phase 2 (Phase 3 tests will verify that
   counts stay `Unavailable`, not invented).

## Validation (Phase 3)

The plan's Phase 3 test suite will verify:

1. **Passing example**: an example with a true assertion → evidence
   `ExecutionOutcome::Passed`, `executed_examples` contains the name.
2. **Failing example**: an example with `assert!(false)` → evidence
   `ExecutionOutcome::Failed`, generated `#[test]` itself fails (expected).
3. **Invalid declaration**: macro invoked with duplicate obligation IDs or other
   validation violation → `TestDeclaration::validate()` rejects it, evidence never
   produced as `Passed`.
4. **Not-executed case**: a declaration without a matching generated test → default
   state is `NotExecuted`, not `Passed`.

Use the `thread_local!` evidence slot from the macro to inspect `EvidenceRecord` in
each case, not just the `#[test]` pass/fail bit.

## Implementation notes

- Macro implementation goes in `testes-unit/src/lib.rs` or `testes-unit/src/unit.rs`
  (re-exported from `lib.rs`), per the workspace split (ADR-0003). No separate
  proc-macro crate needed.
- The `Observed::Unavailable` reason string stays consistent across all `unit!`-generated
  evidence (enables searching/filtering on that reason in aggregate evidence stores).
- `BackendIdentity::name` for native unit tests is `"native-rust-test"` to distinguish
  from any future adapter-based backends.
