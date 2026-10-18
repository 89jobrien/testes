# ADR-0008: `conformance!` macro grammar and expansion

- Status: Accepted
- Date: 2026-10-17
- Project: `testes`
- Scope: decides `conformance!`'s grammar, expansion target, and evidence construction, per
  `docs/macro-plan.md` acceptance item 1. A first-pass `conformance!` already exists in
  `testes-conformance/src/lib.rs` with a different grammar (`id, subject, claim, body`); this ADR
  is the grammar of record and that expansion is to be brought into line with it.

## Context

`conformance!` is the only dimension whose mechanics already exist in `testes-core` while its
orchestration does not.

`testes-core/src/traits.rs:129-151` defines four traits, all generic over `?Sized Port`, with
`ConformanceCase` doc-commented "Concrete cases belong to the project that owns the port
contract":

```rust
pub trait ConformanceCase<Port: ?Sized> {
    fn declaration(&self) -> &ContractCaseDeclaration;
    fn check(&self, subject: &mut Port, context: &mut dyn CaseContext)
        -> Result<(), CheckFailure>;
}
pub trait SubjectFactory<Port: ?Sized> {
    fn implementation(&self) -> ImplementationIdentity;
    fn capabilities(&self) -> Vec<CapabilityObservation>;
    fn create(&self, context: &mut dyn CaseContext) -> Result<Box<Port>, CheckFailure>;
}
pub trait ConformanceSuite<Port: ?Sized> {
    fn contract(&self) -> ContractIdentity;
    fn cases(&self) -> &[Box<dyn ConformanceCase<Port>>];
}
pub trait SubjectCleanup<Port: ?Sized> {
    fn cleanup(&self, subject: Box<Port>, context: &mut dyn CaseContext)
        -> Result<(), CheckFailure>;
}
```

`docs/dimensions/conformance.md` records the state: "Existing interfaces are `ConformanceCase`,
`ConformanceSuite`, `SubjectFactory`, and `SubjectCleanup`; **their orchestration is not
implemented.**"

So this ADR's subject is **orchestration over existing ports**, not new types. Note what the
traits already supply, because it constrains the grammar considerably:

| Supplied by                         | Field / method                                                                   | Consequence for the grammar                                        |
| ----------------------------------- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| `ConformanceSuite::contract()`      | `ContractIdentity { id, version }`                                               | **no `contract:` field** — it is typed and already declared        |
| `SubjectFactory::implementation()`  | `ImplementationIdentity { id, configuration_id }`                                | **no `subject:` field** for the implementation — already declared  |
| `ConformanceCase::declaration()`    | `ContractCaseDeclaration { id, obligation, description, required_capabilities }` | case identity, obligation, and requirements are all contract-owned |
| `SubjectFactory::capabilities()`    | `Vec<CapabilityObservation>`                                                     | applicability is **derivable**, not declarable                     |
| `CaseContext` (`traits.rs:115-122`) | `is_cancelled()`, `retain_artifact()`                                            | a per-case context exists — no `ExecutionContext` plumbing needed  |

### The constraint that shapes the grammar

`docs/dimensions/conformance.md` Limits: "**Keep semantic cases with the trait owner.**" Real I/O
or generated inputs may be used "without changing the contract-owned nature of the suite."

Therefore `conformance!` **cannot define cases**. A grammar that inlined case bodies would
relocate contract semantics into whichever crate invoked the macro — inverting
`docs/architecture.md`'s "Core must not depend on project domains."

The macro must reference a suite and supply only the execution envelope. The grammar is
therefore _smaller_ than the other seven.

### Applicability is derived, which is why the grammar has no opt-out

`docs/dimensions/conformance.md` Acceptance: "**Contract-owned capability rules determine
applicability; an implementation cannot opt out of inconvenient cases.** Missing required
capability does not satisfy the claim."

The types already express this. `ContractCaseDeclaration.required_capabilities: Vec<CapabilityRequirement>`
declares what a case needs; `SubjectFactory.capabilities()` returns
`Vec<CapabilityObservation { name, state }>` where `CapabilityState` is
`Available | Unavailable { reason }` (`types.rs:423-426` for the enum,
`types.rs:428-431` for the observation). Comparing the two is a set operation
the _contract_ authored on both sides.

So there is nothing for a grammar to declare — and, importantly, **nothing for an implementer to
declare**. Adding a `skip:` field would reintroduce exactly the opt-out the spec forbids.

### `CheckFailure` already encodes the three-way split

`traits.rs:108-113`:

```rust
Violation { message: String }        // contract violated  -> outcome Failed
Infrastructure { message: String }  // could not be trusted -> outcome Inconclusive
Inconclusive { message: String }     // no verdict          -> outcome Inconclusive
```

The expansion must map these rather than flattening them into a string — ADR-0001 §2 forbids
"replace structured outcome fields with an error string" and "flatten actionable backend causes
into an undifferentiated message."

## Decision

### Grammar

```rust
conformance! {
    id: "observation-store-contract",
    claim: "adapters reject conflicting identities while preserving idempotent replay",

    // The subject factory. Its type binds `Port` for all four traits.
    factory: store::PgObservationStoreFactory::new(&config),

    // The trait-owned case set. `conformance!` never defines cases.
    suite: observation_store::ContractSuite,
}
```

Four fields — the smallest of the eight macros:

- `id` — unique identifier, `TestId::new` contract
- `claim` — the contract claim being verified. **The only human-authored descriptive field.**
- `factory` — an expression producing a `SubjectFactory`. Supplies `implementation()`,
  `capabilities()`, and `create()`.
- `suite` — an expression producing a `ConformanceSuite`. Supplies `contract()` and `cases()`.

Absent by design: `contract:` (typed, from `suite`), `subject:`/`implementation:` (typed, from
`factory`), `cases:` / case bodies (contract-owned), applicability rules (derived), expected
outcomes (per obligation), and `skip:` (forbidden).

### Expansion target

One generated `#[test] fn {id}`. Conformance runs in-process, so the ordinary cargo test harness
applies, as with `unit!`.

```rust
#[test]
fn {id}() {
    let factory = {factory};
    let suite = {suite};

    let contract = suite.contract();
    let implementation = factory.implementation();
    let observed = factory.capabilities();

    // Schematic: the field *names* are the real `testes-core` fields (`TestDeclaration`,
    // types.rs:109-119); initializers are elided. Note `TestDeclaration` has no `scope` and no
    // `obligations` field — it carries `required_scope`, and obligations travel with the
    // contract cases in `ContractCaseDeclaration`.
    let declaration = TestDeclaration {
        id: /* TestId::new(id) */,
        primary_dimension: Dimension::Conformance,
        secondary_dimensions: vec![],
        subject: /* SubjectId from the implementation identity */,
        claim: /* the authored claim */,
        required_scope: Scope {
            // Capability requirements are *derived* from the contract's cases, never declared.
            capabilities: /* flattened from suite.cases() */,
            /* ..remaining Scope fields */
        },
        disposition: ObligationDisposition::Required,
        specification: TestSpecification::Conformance {
            contract: contract.clone(),
            implementation: implementation.clone(),
            case_ids: suite.cases().iter()
                .map(|c| c.declaration().id.clone())
                .collect(),
        },
    };
    // Validate BEFORE executing: a malformed declaration fails before any case runs.
    declaration.validate().expect("TestDeclaration must be valid");

    let mut case_evidence: Vec<ContractCaseEvidence> = Vec::new();

    for case in suite.cases() {
        let case_decl = case.declaration();

        // Applicability: a set comparison between two contract-owned declarations.
        let unsatisfied: Vec<&CapabilityRequirement> = case_decl.required_capabilities.iter()
            .filter(|req| !observed.iter().any(|o| o.name == req.name && o.state == Available))
            .collect();

        let outcome = if !unsatisfied.is_empty() && case_decl.required_capabilities.len() == unsatisfied.len() {
            // No required capability is present: the contract's rule makes this case
            // inapplicable. Recorded WITH the contract's rationale, never silently dropped.
            ContractCaseOutcome::NotApplicable {
                rationale: format!(
                    "contract {contract:?} excludes this case: unsatisfied required capabilities {:?}",
                    unsatisfied.iter().map(|r| &r.name).collect::<Vec<_>>()
                ),
            }
        } else if !unsatisfied.is_empty() {
            // Some capabilities present: run it, but record what is absent rather than
            // assuming the case ran to its full requirement set.
            run_case(&mut case_evidence, &factory, case.as_ref(), &observed);
            continue;
        } else {
            run_case(&mut case_evidence, &factory, case.as_ref(), &observed);
            continue;
        };

        case_evidence.push(ContractCaseEvidence {
            case_id: case_decl.id.clone(),
            obligation: case_decl.obligation.clone(),
            duration: Duration::ZERO,
            outcome,
        });
    }

    // Schematic: `EvidenceRecord` (types.rs:553-562) has no `test_id`, `backend_used`, or
    // `timestamp` field. The dimension payload lives in `dimension_evidence`; backend identity and
    // the run's timing live in `execution: ExecutionMetadata`.
    let evidence = EvidenceRecord {
        schema_version: /* ADR-0001 §7 version */,
        declaration,
        provenance: /* Provenance */,
        execution: /* ExecutionMetadata, carrying the BackendIdentity */,
        outcome: /* ExecutionOutcome */,
        artifacts: /* retained artifacts */,
        dimension_evidence: DimensionEvidence::Conformance(ConformanceEvidence {
            contract,
            implementation,
            capabilities: observed,
            cases: case_evidence,
        }),
    };

    thread_local! {
        static LAST_CONFORMANCE_EVIDENCE: RefCell<Option<EvidenceRecord>> = RefCell::new(None);
    }
    LAST_CONFORMANCE_EVIDENCE.with(|e| *e.borrow_mut() = Some(evidence.clone()));

    // A suite that executes nothing is NotExecuted, not Passed. This is the conformance
    // instance of `empty-scaffold-target`: a contract suite that checks nothing has
    // established nothing, and "compilation proves neither semantic correctness nor
    // contract coverage" still holds.
    let DimensionEvidence::Conformance(conformance) = &evidence.dimension_evidence else {
        unreachable!("declaration and evidence dimensions are built together")
    };
    let executed = conformance
        .cases
        .iter()
        .filter(|c| matches!(c.outcome, ContractCaseOutcome::Executed(_)))
        .count();
    assert!(executed > 0,
        "conformance suite {id} executed no cases; a vacuous suite is NotExecuted, not Passed");

    // Any non-Passed execution fails the test. The macro adds evidence; it does not replace
    // Rust's test harness.
    let failed: Vec<_> = conformance
        .cases
        .iter()
        .filter(|c| {
            !matches!(
                &c.outcome,
                ContractCaseOutcome::Executed(o) if o.status == ExecutionStatus::Passed
            )
        })
        .collect();
    assert!(failed.is_empty(), "conformance cases did not pass: {failed:?}");
}
```

`run_case` — the per-case body, with fresh subject and guaranteed cleanup:

```rust
fn run_case(
    out: &mut Vec<ContractCaseEvidence>,
    factory: &dyn SubjectFactory<Port>,
    case: &dyn ConformanceCase<Port>,
    observed: &[CapabilityObservation],
) {
    let case_decl = case.declaration();
    let mut context = CaseContextImpl::new(/* artifact sink */);

    let started = Instant::now();
    let outcome = match factory.create(&mut context) {
        // Subject creation failed: infrastructure, not a contract violation.
        Err(CheckFailure::Infrastructure { message }) => ContractCaseOutcome::Executed(
            ExecutionOutcome { status: Inconclusive, reason: "subject creation failed",
                                diagnostic: Some(message), ..Default::default() }),
        Err(CheckFailure::Inconclusive { message }) => ContractCaseOutcome::Executed(
            ExecutionOutcome { status: Inconclusive, reason: "subject creation inconclusive",
                                diagnostic: Some(message), ..Default::default() }),
        Err(CheckFailure::Violation { message }) => ContractCaseOutcome::Executed(
            ExecutionOutcome { status: Failed, diagnostic: Some(message), ..Default::default() }),

        Ok(mut subject) => {
            // Fresh subject per case: no case may observe another's state.
            let result = case.check(subject.as_mut(), &mut context);

            // Cleanup runs even when the check failed or the context was cancelled, and a
            // cleanup failure is retained rather than discarded.
            let cleanup = SubjectCleanup::cleanup(factory, subject, &mut context);

            match (result, cleanup) {
                (Ok(()), Ok(())) => ContractCaseOutcome::Executed(
                    ExecutionOutcome { status: Passed, ..Default::default() }),
                (Err(failure), Ok(())) => ContractCaseOutcome::Executed(map_failure(failure)),
                // Execution passed but cleanup failed. The obligation is NOT satisfied:
                // a cleanup failure is a resource leak, and "Setup and cleanup failures
                // must remain visible" means visible as a failure.
                (Ok(()), Err(failure)) => ContractCaseOutcome::Executed(
                    ExecutionOutcome { status: Failed,
                        reason: "check passed but cleanup failed",
                        diagnostic: Some(format!("{:?}", failure)), ..Default::default() }),
                (Err(failure), Err(cleanup_failure)) => ContractCaseOutcome::Executed(
                    ExecutionOutcome { status: Failed,
                        reason: "check failed and cleanup also failed",
                        diagnostic: Some(format!("{failure:?}; cleanup: {cleanup_failure:?}")),
                        ..Default::default() }),
            }
        }
    };

    out.push(ContractCaseEvidence {
        case_id: case_decl.id.clone(),
        obligation: case_decl.obligation.clone(),
        duration: started.elapsed(),
        outcome,
    });
}

/// Map CheckFailure's three variants without flattening them (ADR-0001 §2).
fn map_failure(failure: CheckFailure) -> ExecutionOutcome {
    match failure {
        CheckFailure::Violation { message } =>
            ExecutionOutcome { status: Failed, diagnostic: Some(message), ..Default::default() },
        CheckFailure::Infrastructure { message } =>
            ExecutionOutcome { status: Inconclusive, reason: "infrastructure failure",
                               diagnostic: Some(message), ..Default::default() },
        CheckFailure::Inconclusive { message } =>
            ExecutionOutcome { status: Inconclusive, reason: "check inconclusive",
                               diagnostic: Some(message), ..Default::default() },
    }
}
```

Six decisions worth stating explicitly:

1. **Cleanup failure fails the case.** `(Ok(()), Err(cleanup))` maps to `Failed`, not `Passed`.
2. **`CheckFailure::Infrastructure` → `Inconclusive`, not `Failed`.** An infrastructure failure is
   "a failure preventing trustworthy execution," which `docs/evidence.md` classifies separately
   from an assertion violation. Flattening it to `Failed` would put it in `BackendError`'s
   category by accident.
3. **Zero executed cases panics.** The conformance instance of `empty-scaffold-target`.
4. **`ContractCaseOutcome::NotApplicable` retains the contract's rationale**, so a case excluded
   from execution shows _which contract rule_ excluded it and _who_ decided — preserving
   "absence is explicit, never fabricated."
5. **Cleanup runs on every path**, including a failed check and a cancelled context
   (`CaseContext::is_cancelled()`), so a cancellation cannot leak a subject.
6. **`NotApplicable` is decided by the contract's declared requirements vs. the factory's
   observations.** The implementation never participates in the decision.

### Capability coverage is recorded, not assumed

`ConformanceEvidence.capabilities: Vec<CapabilityObservation>` is recorded once from
`factory.capabilities()`. `CapabilityState::Unavailable { reason }` is where a probe that could
not determine capability state is expressed — the `Observed`-style distinction again, so "we
couldn't tell" is not recorded as "absent" or "present".

### Backend identity

`BackendIdentity { name: "conformance-runner", version: "1.0" }` — distinct from ADR-0004's
`"native-rust-test"`, because conformance runs a contract-owned suite rather than an inline
example body. Any future alternative runner takes a different name.

### ADR-0002 relationship

`conformance!` is a composition root, exactly like `unit!`: the generated `#[test]` constructs the
declaration, runs the suite, and builds the evidence directly — no `dyn TestBackend`,
`RunRequest`, or `ExecutionContext` indirection.

ADR-0002 §4 reserves `TestBackend` for dimensions with genuinely swappable execution strategies.
Conformance is _nearly_ that case — the cases differ per contract — but the _execution mechanism_
is always the same in-process loop. The variety is in the data, not the algorithm. So the trait
adds indirection without buying substitution.

Note that conformance already has a context type of its own: `CaseContext` (`traits.rs:115`), not
`ExecutionContext`. That per-case context is the right granularity here and should be used rather
than the run-level `ExecutionContext`.

## Consequences

- **Ownership is structurally enforced, not merely documented.** There is no grammar position from
  which an implementer can define a case, declare a requirement, or skip one. The rule that cases
  and applicability belong to the contract owner is enforced by the macro's surface.
- **The grammar is four fields** — the smallest of the eight — because the traits already declare
  contract, implementation, case identity, obligations, and capabilities. Anyone expecting
  `contract:` and `subject:` fields should read `traits.rs` first.
- **`conformance!` cannot express a case.** A project wanting an ad-hoc conformance check outside
  a trait-owned suite has no macro for it — deliberately, since that is the `unit!`/`property!` use
  case.
- **`docs/handoff.md`'s lifecycle gap is partly addressed.** `ContractCaseEvidence` already carries
  per-case `duration` and a three-variant `ContractCaseOutcome`, so lifecycle _outcomes_ are
  separately encoded. What is **not** encoded is a separate setup phase: `SubjectFactory::create`
  failures are folded into the case's execution outcome. Whether `create` should be its own
  recorded phase is left open below.
- **The async adapter question is untouched.** The four traits remain synchronous.
  `docs/handoff.md` records "Synchronous conformance interfaces may need an explicit async adapter
  strategy," and ADR-0001 §9 defers `tokio`/`async-trait` until one exists. This ADR does not
  resolve it and must not be read as doing so.
- **`CheckFailure` mapping is now specified**, which closes one instance of ADR-0001 §2's
  "flatten actionable backend causes" prohibition.
- **`CaseContextImpl` is new work in `testes-core`.** `CaseContext` is a trait with no
  implementation. `retain_artifact()` needs an `ArtifactStore` to call — and `ArtifactStore` is
  itself an unimplemented port. So the conformance runner cannot record retained artifacts until
  that port has an implementation.

## Alternatives considered

1. **Let `conformance!` define cases inline.** Rejected: it relocates contract semantics into the
   invoking crate, inverting `trait-ownership`. It also duplicates `unit!` with a worse name.
2. **Add a `skip:` or `applies_to:` clause.** Rejected: applicability is already derivable from two
   contract-owned declarations, so a clause would be redundant _and_ would create the opt-out the
   spec forbids.
3. **Add `contract:` / `subject:` fields for explicitness.** Rejected: both are already supplied
   by `ConformanceSuite::contract()` and `SubjectFactory::implementation()` as typed values.
   Duplicating them as strings would create a second source of truth that could disagree with the
   typed identity.
4. **Implement `ConformanceBackend` as a `TestBackend`.** Rejected per ADR-0002 §4: the execution
   algorithm is fixed; only the data varies.
5. **Require the caller to pre-build `ConformanceEvidence`.** Rejected: it would make evidence
   construction the caller's responsibility, which is the mechanic the framework exists to own.
6. **Run `cleanup` only when `check` succeeds.** Rejected: that is precisely how a resource leak
   survives, and `docs/dimensions/conformance.md` requires setup and cleanup failures to remain
   visible.

## Validation (to add when implemented)

Using the `thread_local!` slot rather than the pass/fail bit:

1. **All cases pass** → every `ContractCaseOutcome::Executed(Passed)`, `cases` length equals
   `suite.cases().len()`, no `NotApplicable`.
2. **A case fails** → `CheckFailure::Violation` maps to `Executed(Failed)` with the message as
   `diagnostic`; generated `#[test]` fails.
3. **Infrastructure failure** → `CheckFailure::Infrastructure` maps to `Executed(Inconclusive)` —
   **not** `Failed`.
4. **Inconclusive check** → maps to `Executed(Inconclusive)`, distinct from failure.
5. **Zero applicable cases** → expansion panics with the vacuous-suite message; the
   `NotApplicable` entries and their rationales remain in the evidence.
6. **Partial capability coverage** → the case still executes, and the absent requirements appear
   in `ConformanceEvidence.capabilities` as `Unavailable { reason }`.
7. **Subject isolation** — a case that mutates its subject does not change a later case's result.
8. **Cleanup failure after a passing check** → `Executed(Failed)` with reason "check passed but
   cleanup failed".
9. **Cleanup runs after a failing check** — subject released; the recorded diagnostic is the
   check's failure, not the cleanup's.
10. **Cleanup runs when `CaseContext::is_cancelled()`** — no subject leaked.
11. **Malformed declaration** — duplicate obligation IDs, dimension mismatch — `validate()` rejects
    before any case runs.
12. **Ownership** — no grammar position accepts a case body or a `skip:` field (compile-fail
    fixtures attempting each).
13. **Typed identity, not string** — `TestSpecification::Conformance.contract` and
    `.implementation` equal `suite.contract()` and `factory.implementation()` respectively
    (asserted by equality, not string comparison).

## Open questions for the owner

1. **Is `SubjectFactory::create` a separate lifecycle phase?** `docs/handoff.md` says evidence "does
   not yet encode every lifecycle phase separately." This ADR folds subject creation into the
   case's execution outcome. Should creation be its own `ContractCaseOutcome` variant, so a subject
   failure is distinguishable from a check failure in the record rather than only in the reason
   string?
2. **`CaseContextImpl` has nowhere to retain artifacts.** `CaseContext::retain_artifact()` requires
   an `ArtifactStore`, which has no implementation. Should conformance ship a minimal in-memory
   implementation, or does the runner defer retention until the port exists? This blocks items 5
   and 6 above from recording retained evidence.
3. **Partial capability coverage.** This ADR executes a case when _some_ required capabilities are
   present, recording the absent ones. Alternatively, any absent required capability could make the
   case `NotApplicable`. The spec says "Missing required capability does not satisfy the claim,"
   which reads as the _former_, but the intended rule is not explicit.
4. **`NotApplicable` vs `NotExecuted`.** `ContractCaseOutcome` has both variants. This ADR uses
   `NotApplicable` for capability-driven exclusion. What produces `NotExecuted` — a contract rule,
   or a runtime condition such as cancellation? Both may be needed.
5. **`ConformanceSuite::cases()` returns a slice, not an iterator.** So the expansion iterates a
   `&[Box<dyn ConformanceCase<Port>>]`, which is fine. Is there a reason it was not an iterator —
   for instance, that conformance is expected to run cases in declaration order or in parallel?
   This ADR preserves slice order sequentially.

## Implementation notes

- Lives in `testes-conformance/src/lib.rs`, per ADR-0003. `testes-conformance` depends only on
  `testes-core`, which the composition-root approach preserves — the generated code needs no
  backend dependency.
- The expansion references only core types: `TestDeclaration`, `TestSpecification::Conformance`,
  `EvidenceRecord`, `DimensionEvidence::Conformance`, `ConformanceEvidence`,
  `ContractCaseEvidence`, `ContractCaseOutcome`, `ContractIdentity`, `ImplementationIdentity`,
  `CapabilityObservation`, `CapabilityState`, `ContractCaseDeclaration`, `ExecutionOutcome`,
  `CheckFailure`, `CaseContext`, `TestId`, `ObligationId`, `Scope`, `BackendIdentity`. Zero
  infrastructure dependency, per ADR-0001 §1.
- All four traits are generic over `?Sized Port`, so `Box<Port>` is the subject handle — the
  expansion must keep the `Box` alive across `check` and hand it to `cleanup`.
- The four traits' bounds are sufficient as written: `SubjectFactory::create` and
  `SubjectCleanup::cleanup` both take `&mut dyn CaseContext`, so the runner needs a
  `CaseContext` implementation and nothing else. **No trait changes are required** — which is
  worth stating explicitly, since it was the main open question while drafting.
- `docs/macro-plan.md` order item 3 places conformance third — after `unit!`, before
  property/regression replay. `docs/handoff.md`: "Then implement reusable conformance mechanics
  **before** broad campaign orchestration."

Related: [[conformance]], [[trait-owned-suite]], [[capability-policy]], [[conformance-case]],
[[conformance-suite]], [[subject-factory]], [[subject-cleanup]],
[[isolated-subject-per-case]], [[conformance-lifecycle-encoding]], [[conformance-async-adapter]],
[[the-grammar-adr-series]], [[adr-0004-unit-macro-grammar-and-expansion]],
[[adr-0002-unit-macro-port-boundary]], [[testes-conformance]], [[testes-core]].
