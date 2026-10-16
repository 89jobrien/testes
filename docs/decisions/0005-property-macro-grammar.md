# ADR-0005: `property!` macro grammar and expansion

- Status: Accepted
- Date: 2026-10-16
- Project: `testes`
- Scope: specifies `property!`'s grammar, expansion target, and evidence construction
  before implementation. Per ADR-0002's pattern: a declarative `macro_rules!` macro
  (no proc-macro crate), not a `TestBackend` trait (Proptest backend comes later
  as a separate adapter).

## Context

Per `docs/handoff.md` and the implementation plan, `property!` is the next dimension
after `unit!`. Unlike unit tests, property-based tests require:

- A generator/strategy (Proptest or equivalent)
- Configuration (minimum cases, seed for replay)
- Replay artifacts for failed cases
- An oracle (predicate that should hold)

ADR-0001 §5 lists property testing as an "adapter" dimension: multiple backends
are plausible (Proptest, QuickCheck-style, custom generators). However, the macro
itself is independent of the backend choice — it declares what is being tested
and records evidence of what was executed.

## Decision

### Grammar

```rust
property! {
    id: "sorted-list-property",
    subject: "Vec::sort",
    claim: "sorted lists are ordered",
    strategy: {
        vec: prop::collection::vec(0i32..100, 0..1000),
    },
    oracle: |vec: &Vec<i32>| {
        let mut sorted = vec.clone();
        sorted.sort();
        // Property: after sort, each element ≤ next
        sorted.windows(2).all(|w| w[0] <= w[1])
    },
    cases: 256,
}
```

- `id`: unique identifier for this property test (matches `TestId::new` contract)
- `subject`: what is being tested (e.g. `Vec::sort`)
- `claim`: the property being verified (e.g. "sorted lists are ordered")
- `strategy`: Proptest strategy generators (maps variable names to `prop::...` expressions)
- `oracle`: a closure `Fn(&GeneratedValue) -> bool` that checks the property
- `cases`: minimum number of test cases to execute (u64)

### Expansion target

For each property test, the macro generates:

```rust
#[test]
fn {id}() {
    use proptest::proptest;

    // Declare the test
    let declaration = TestDeclaration {
        primary_dimension: Dimension::Property,
        specification: TestSpecification::Property {
            strategy_id: "{strategy_id}",
            minimum_cases: {cases},
            replay_artifacts: vec![],  // Could be set from env or config
        },
        // ... other fields
    };
    declaration.validate().expect("TestDeclaration must be valid");

    // Run the property test via Proptest
    let mut evidence = PropertyEvidence {
        strategy_id: "{strategy_id}",
        seed: Observed::Unavailable {
            reason: "seed tracking not implemented",
        },
        completed_cases: 0,
        rejected_cases: Observed::Unavailable {
            reason: "rejection tracking not implemented",
        },
        replayed_artifacts: vec![],
        minimized_counterexample: None,
    };

    let result = proptest!(|(vec in {vec strategy})| {
        evidence.completed_cases += 1;
        let oracle_result = ({oracle})(vec);
        prop_assert!(oracle_result, "property failed for input: {:?}", vec);
    });

    // Record outcome based on proptest result
    let (status, diagnostic) = match result {
        Ok(()) => (ExecutionStatus::Passed, None),
        Err(e) => (ExecutionStatus::Failed, Some(format!("{:?}", e))),
    };

    // Build and store evidence record
    let evidence_record = EvidenceRecord {
        // ... with PropertyEvidence
    };

    // Store for introspection (thread_local like unit!)
    thread_local! {
        static LAST_PROPERTY_EVIDENCE: RefCell<...> = RefCell::new(None);
    }
    LAST_PROPERTY_EVIDENCE.with(|e| *e.borrow_mut() = Some(evidence_record));

    // Let proptest's panic/failure propagate (like unit! re-panics)
    result.expect("property test failed");
}
```

## Consequences

- **Honest evidence**: completed_cases increments as the test runs; seed and rejection
  counts stay `Unavailable` (not measured yet — ADR-0001 will decide what to instrument).
- **Strategy independence**: the macro accepts arbitrary `prop::...` expressions,
  making it agnostic to Proptest versions or future strategy libraries.
- **Backend adapter separation**: the `TestBackend` trait can implement a Proptest
  adapter later if needed (e.g., for distributed/parallel property testing),
  without changing the macro grammar or evidence model.
- **No proc-macro needed**: grammar is simple enough for `macro_rules!` (generators
  are expressions, oracle is a closure, no hygiene issues).

## Alternatives considered

1. **Generic `property!` over a `PropertyBackend` trait:** Rejected per ADR-0002
   pattern — the macro is the composition root; multiple backends are plausible
   but not yet needed (Proptest is the obvious choice for now).
2. **Procedural macro with full Proptest API introspection:** Rejected. A declarative
   macro keeps the crate dependency-free for now; Proptest integration lives in
   `testes-property` adapter.
3. **Shrinking/minimization built into the macro:** Out of scope. Proptest handles
   shrinking internally; the macro records the minimized result if it exists, but
   doesn't implement shrinking itself.

## Validation (Phase 3)

The test suite will verify:

1. **Passing property**: a property that always holds → evidence
   `ExecutionStatus::Passed`, `completed_cases > 0`.
2. **Failing property**: a property with a counterexample → evidence
   `ExecutionStatus::Failed`, diagnostic shows the minimized input.
3. **Multiple strategies**: property with multiple generators runs without panic.
4. **Seed/rejection counts**: stays `Unavailable` (not yet instrumented).

## Implementation notes

- Macro implementation goes in `testes-property/src/lib.rs`, following the
  `testes-unit` pattern.
- Proptest is a dependency of `testes-property` only, not `testes-core`.
- `PropertyEvidence` already exists in `testes-core/src/types.rs` and includes
  `seed`, `completed_cases`, `rejected_cases` (as `Observed`), `replayed_artifacts`,
  and `minimized_counterexample` — no new types needed.
- The macro stores evidence in a thread_local (like `unit!`) for test introspection.
