# ADR-0009: `regression!` macro grammar and expansion

- Status: Accepted
- Date: 2026-10-17
- Project: `testes`
- Scope: decides `regression!`'s grammar, expansion target, and evidence construction, per
  `docs/macro-plan.md` acceptance item 1. This dimension had **no grammar ADR** before this one —
  only `docs/dimensions/regression.md` and one row of `docs/macro-plan.md`. A first-pass
  `regression!` already exists in `testes-regression/src/lib.rs`; this ADR is the grammar of record.

## Context

`regression!` is the only dimension whose defining property is a _negative_ — what it must not do.

`docs/dimensions/regression.md` Definition: "**Regression is a purpose layered over an execution
dimension, not a separate backend.**"

That decides the crate layout (ADR-0003 gives `testes-regression` a dependency on `testes-core`
only, because it executes nothing itself) and it decides the macro's hardest constraint:

> "**Reuse one execution; do not run it again merely to label it regression.**"

ADR-0001 §5 repeats it: "Regression shall reuse underlying execution evidence rather than cause
a second execution solely to obtain a regression label."

### The types

```rust
// types.rs:149-153
TestSpecification::Regression {
    failure: FailureId,
    underlying_dimension: Dimension,
    reproducer_artifacts: Vec<ArtifactId>,
}

// types.rs:460-467
pub struct RegressionEvidence {
    pub failure: FailureId,
    pub reproducer_artifacts: Vec<ArtifactId>,
    pub historical_failure: Option<ArtifactId>,
    /// Reuses the underlying execution rather than running it twice.
    pub execution: Box<DimensionEvidence>,
}
```

Three observations that shape everything below.

**1. `failure` is a typed `FailureId`, not a string.** So failure identity is already
identity-bearing — which partly answers the gap `docs/handoff.md` flags ("regression nesting
currently checks dimension only, not all failure identity or artifact consistency"). The
_remaining_ gap is that artifact consistency is unchecked, not that identity is untyped.

**2. `execution: Box<DimensionEvidence>` is an inline value, not a lookup.** The field's doc
comment states the intent: "Reuses the underlying execution rather than running it twice." So the
evidence type already anticipates that a boxed underlying record is _supplied_.

**3. There is no field for "which declaration produced this execution."** `failure` identifies the
_defect_; `reproducer_artifacts` identifies retained _material_; `underlying_dimension` identifies
the _kind_. None of them names the declaration whose live execution is being framed.

That third observation is the crux of this ADR, and it is why this ADR is mostly a question rather
than a design.

### The acquisition problem

A generated `#[test]` must do something, but `regression!` must not run a second campaign. So it
must obtain a `DimensionEvidence` that some other declaration's generated test already produced.

ADR-0004/0005/0006 each solve the analogous problem with a **per-crate private slot**:

```rust
thread_local! {
    static LAST_UNIT_EVIDENCE: RefCell<Option<EvidenceRecord>> = RefCell::new(None);
}
```

That does not generalise to regression:

| Problem      | Why a private slot is insufficient                                                                                                      |
| ------------ | --------------------------------------------------------------------------------------------------------------------------------------- |
| Cross-crate  | `testes-regression` cannot read `testes-unit`'s private static without depending on `testes-unit`, inverting ADR-0003's dependency sink |
| Single-value | each slot holds one `Option<EvidenceRecord>`; tests run concurrently in one binary, so slots collide                                    |
| No identity  | nothing keys the evidence to _which_ declaration produced it — and, per observation 3, no field exists to record that key               |

ADR-0004's instruction — "Use the `thread_local!` evidence slot from the macro to inspect
`EvidenceRecord` in each case" — assumes one macro per binary. `regression!` is the first
dimension to consume a sibling's evidence, so that assumption now fails.

## Decision

### The grammar's shape depends on an unresolved acquisition strategy

This ADR proposes the grammar's descriptive fields, and identifies the field that cannot be typed
until the owner settles acquisition.

```rust
regression! {
    id: "budget-exhaustion-overflow",
    subject: "budget::exhaustion",
    claim: "external input that previously overflowed can no longer bypass exhaustion checks",

    underlying: Dimension::Unit,

    failure: FailureId::new("ISSUE-2026-0417-budget-overflow"),

    reproducer: /* DEPENDS ON THE ACQUISITION STRATEGY — see below */,
}
```

Five fields:

- `id` — this regression's identifier, `TestId::new` contract
- `subject` — the component whose behavior is protected
- `claim` — what the regression asserts
- `underlying` — the `Dimension` whose evidence is wrapped. One of unit, property, fuzz, model
  check, conformance, integration. **Mutation is not accepted**, and the grammar rejects it at
  expansion time rather than at runtime.
- `failure` — a typed `FailureId`. Not a string.
- `reproducer` — **type unresolved.** Three candidates below.

Note what is absent: **no body.** `regression!` declares no inline assertion. A body would either
duplicate an existing test or constitute a second execution — both forbidden. This is the only one
of the eight macros with no place to put an assertion.

### The acquisition question: three options, one owner decision

| Option                          | Shape of `reproducer:`                                  | How the boxed `execution` is obtained                                                                                                                             | Cost                                                                                                                                                                                                        |
| ------------------------------- | ------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **A. Same-expansion wrapping**  | `reproducer { /* the reproducer's own declaration */ }` | The reproducer body runs **inside the same generated `#[test]`**, publishes its evidence, then the framing code boxes it. One execution, one test, deterministic. | The reproducer must be syntactically nested or referenced; `macro_rules!` can express nesting, not cross-invocation references                                                                              |
| **B. Shared core channel**      | `reproducer: TestId`                                    | `testes-core` gains a thread-local, `TestId`-keyed, take-on-read channel (`record_execution` / `take_execution`).                                                 | Adds the project's **first mutable state** to the dependency sink, **and requires a new field** (observation 3: no field currently records the key)                                                         |
| **C. Artifact-mediated replay** | `reproducer: ArtifactId`                                | The reproducer's retained evidence is loaded from an `ArtifactStore` lookup.                                                                                      | Already type-consistent with `reproducer_artifacts: Vec<ArtifactId>` — but `ArtifactStore` has no implementation, and it changes "one execution" into "a previous execution", weakening the reuse guarantee |

**This ADR does not choose.** The choice changes the grammar's `reproducer` field type, so it must
be settled before the grammar is fixed.

What can be said now:

- **A is the only option that satisfies "reuse one execution" unconditionally.** The failure mode
  of B and C is that the regression and the reproducer are separate runs, so the framed evidence
  and the retained reproducer describe different executions and replay becomes ambiguous —
  precisely what `docs/dimensions/regression.md` means by "Reuse one execution."
- **C is the most consistent with the existing type surface**, since `reproducer_artifacts` is
  already `Vec<ArtifactId>`. That is a real argument in its favour, and it is why this ADR
  presents it rather than dismissing it.
- **B requires a `testes-core` change that observation 3 shows the types do not currently support.**
  If B is chosen, a field (or a convention) recording the underlying declaration's `TestId` must be
  added to `RegressionEvidence` or `TestSpecification::Regression`. That is a dependency-sink type
  change proposed for one macro's benefit — which is the strongest argument against B.
- **B also adds mutable state where there is none.** `testes-core` has zero implementations, zero
  mutable state, and eight trait-only ports. ADR-0002's composition-root design otherwise avoids
  state entirely.

### Evidence construction (common to all three options)

```rust
// Pseudo-expansion. The acquisition step differs by option; the rest does not.
// Field *names* are the real `testes-core` fields (`TestDeclaration`, types.rs:109-119);
// initializers are elided. There is no `scope` or `obligations` field — it carries
// `required_scope`.
let declaration = TestDeclaration {
    id: /* TestId::new(id) */,
    primary_dimension: Dimension::Regression,
    secondary_dimensions: vec![],
    subject: /* SubjectId::new("{subject}") */,
    claim: /* the authored claim */,
    required_scope: Scope {
        /* input_domain, assumptions, bounds, capabilities, resources */
    },
    disposition: ObligationDisposition::Required,
    specification: TestSpecification::Regression {
        failure: failure.clone(),
        underlying_dimension: {underlying},
        reproducer_artifacts: vec![reproducer_artifact],
    },
};
declaration.validate().expect("TestDeclaration must be valid");

// Acquisition — OPTION-DEPENDENT. Option A: the reproducer ran inline just above.
// Option B: take_execution(&reproducer_test_id). Option C: store.load(&reproducer_artifact).
let underlying_record: EvidenceRecord = /* per option */;

// The underlying record must actually be the dimension this regression claims.
// `EvidenceRecord.dimension_evidence` is the `DimensionEvidence`; `.execution` is
// `ExecutionMetadata` and has no `dimension()`.
assert_eq!(
    underlying_record.dimension_evidence.dimension(),
    {underlying},
    "regression {id} frames a {:?} record but declares {:?}",
    underlying_record.dimension_evidence.dimension(),
    {underlying},
);

// Schematic: `EvidenceRecord` (types.rs:553-562) has no `test_id`, `backend_used`, or `timestamp`
// field. The dimension payload lives in `dimension_evidence`.
let evidence = EvidenceRecord {
    schema_version: /* ADR-0001 §7 version */,
    declaration,
    provenance: /* Provenance */,
    execution: /* ExecutionMetadata, carrying the BackendIdentity */,
    outcome: /* ExecutionOutcome */,
    artifacts: /* retained artifacts */,
    dimension_evidence: DimensionEvidence::Regression(RegressionEvidence {
        failure: failure.clone(),
        reproducer_artifacts: vec![reproducer_artifact],
        // OPTIONAL and explicitly absent. See below.
        historical_failure: None,
        // The underlying evidence is MOVED, not re-executed and not cloned-then-rerun.
        execution: Box::new(underlying_record.dimension_evidence),
    }),
};

thread_local! {
    static LAST_REGRESSION_EVIDENCE: RefCell<Option<EvidenceRecord>> = RefCell::new(None);
}
LAST_REGRESSION_EVIDENCE.with(|e| *e.borrow_mut() = Some(evidence.clone()));
```

**Moving, not cloning.** `underlying_record.execution` is moved into the `Box`. Cloning would
leave the underlying record available, so a second regression framing the same reproducer could
find it too — and under option B's channel, `take_execution` would have to enforce single
consumption anyway. Moving is the honest representation of "one execution, one framing."

### Historical failure evidence

`RegressionEvidence::historical_failure: Option<ArtifactId>` is `None` here — **explicitly, not by
accident**.

`docs/dimensions/regression.md` Limits: "**A current pass does not establish that the original
implementation failed the test.** Retain historical evidence when available; absence is explicit,
never fabricated."

So `None` means "no retained historical evidence is available for this `FailureId`." The record
therefore documents a currently-passing reproducer, **not** a demonstrated regression — and the gap
is visible in the record rather than papered over. Nothing in this ADR fabricates one.

Note the type is `Option<ArtifactId>`, not a richer struct: absence is the only distinction it can
represent. Whether a retained artifact is _sufficient_ to establish historical failure is
unrecorded.

### Backend identity

`"regression-wrapper"` / `"1.0"` — distinct from ADR-0004's `"native-rust-test"`, ADR-0008's
`"conformance-runner"`, and the property/fuzz harness names, because regression executes nothing
of its own.

### ADR-0002 relationship

A composition root, like `unit!`, `conformance!`, and `integration!`: the generated `#[test]`
constructs the declaration, obtains the underlying evidence, and builds the evidence directly — no
`dyn TestBackend`, `RunRequest`, or `ExecutionContext` indirection.

ADR-0002 §4 does **not** list regression among dimensions reserved for `TestBackend` (it lists
`property!`/`fuzz!`/`prove!`/`mutate!`), which is consistent: regression executes nothing, so
there is no execution strategy to abstract over.

## Consequences

- **The grammar cannot be fixed until acquisition is settled.** Three of five fields are settled;
  `reproducer:` is not. This ADR is therefore deliberately incomplete.
- **If option B is chosen, `testes-core` needs two changes**: a mutable thread-local channel, and
  a new field recording the underlying declaration's `TestId` (per observation 3). Both are
  dependency-sink changes proposed for one macro's benefit.
- **Option B would also amend ADR-0004, ADR-0005, and ADR-0006**, whose per-macro single-value
  `thread_local!` slots it supersedes. Their grammar, expansion sequence, backend identity, and
  honest-unavailability decisions would be unaffected — only the storage mechanism changes.
- **Option C is blocked on `ArtifactStore`**, which has no implementation (`traits.rs:37`), and it
  weakens "one execution" to "a previous execution."
- **`regression!` cannot express a body**, so it has no place to assert. If acquisition lands as
  option A, the assertion lives inside the nested reproducer — which must then be an _existing_
  declaration, or regression becomes a second way to write tests.
- **The failure-identity gap is partly pre-solved.** `FailureId` is typed, so identity is
  identity-bearing. `docs/handoff.md`'s remaining complaint — "not all failure identity or
  artifact consistency" — is about the _checks_, not the types.
- **A same-thread channel (option B) would need thread-locality to be correct**, since `cargo test`
  runs tests on separate threads and a process-global map would let a regression frame another
  test's evidence — a silent wrong-evidence bug.

## Alternatives considered

1. **Generate a second execution.** Rejected outright — forbidden by
   `docs/dimensions/regression.md` and ADR-0001 §5. It also produces evidence from a different run
   than the retained reproducer, so replay would be ambiguous.
2. **Clone the underlying `DimensionEvidence` rather than move it.** Rejected: cloning leaves the
   underlying record reachable, so "one execution, one framing" is not enforced.
3. **A process-global `Mutex<HashMap>`.** Rejected: parallel test threads would interleave, and
   `Mutex` poisoning plus whole-binary contention is the wrong shape for a same-thread handoff.
4. **Put the channel in a new shared crate between core and the dimension crates.** Rejected: adds a
   workspace member with no independent content, contrary to ADR-0003's reasoning for nine
   members rather than more.
5. **Let `testes-regression` depend on `testes-unit` and friends.** Rejected: inverts the dependency
   sink and would create cycles as dimensions accumulate.
6. **Option A via a runtime registry instead of nesting.** Rejected as a variant of option B with
   more machinery and the same cross-crate problem.

## Validation (to add when implemented, once acquisition is fixed)

Using the `thread_local!` slot rather than the pass/fail bit:

1. **Happy path** — a passing reproducer is framed: `RegressionEvidence.execution` carries the
   reproducer's `UnitEvidence` with `executed_examples` intact, `failure` matches, and
   `historical_failure` is `None`.
2. **Underlying dimension mismatch** — a regression declaring `Dimension::Fuzz` over a `Unit`
   record fails with the dimension-mismatch message.
3. **Mutation is rejected** — `underlying: Dimension::Mutation` is a compile error, not a runtime
   failure.
4. **Missing reproducer** — a regression whose reproducer did not produce evidence fails with a
   clear message and does **not** fabricate an underlying record.
5. **No redundant execution** — the framed record's `executed_examples` count equals the
   reproducer's own count; no additional example ID appears.
6. **Single consumption** — two regressions framing the same reproducer: the first succeeds, the
   second reports the missing record (option B only).
7. **Cross-thread isolation** — a reproducer executed on thread A is not visible to a regression on
   thread B (option B only).
8. **Absent history is explicit** — `historical_failure` is `None`, and the record does not claim a
   previously observed defect.
9. **Movement, not clone** — after framing, the underlying record is no longer independently
   retrievable (option B only).

Items 6, 7, and 9 apply only if option B is chosen; option A needs its own determinism fixture
(the framed evidence is the same run's, not a later run's).

## Open questions for the owner

1. **Acquisition: A, B, or C.** The blocking question. It determines the `reproducer` field's type,
   and it decides whether `testes-core` changes.
2. **If B: is the core channel acceptable?** It adds the project's first mutable state to a
   dependency-sink crate, **and** requires a new field per observation 3. If B is rejected, C
   avoids the core change at the cost of weakening "one execution" to "a previous execution."
3. **Does `RegressionEvidence` need a field naming the underlying declaration?** Under B it does.
   Under A and C it does not. Its absence is the reason B currently requires a type change.
4. **`record_execution`'s signature** — `Result<_, DeclarationError>` for forward compatibility,
   or plain? Only relevant under B.
5. **Is `Option<ArtifactId>` sufficient for `historical_failure`?** It records _presence_, not
   _sufficiency_. Should a retained artifact carry enough to establish that the original
   implementation failed — for example, the revision and the failing assertion? If so this is a
   core type change.
6. **Should ADR-0007 be fixed in the same pass?** `prove!` is the last dimension whose slot is a
   private single-value one, so option B would change it too.

## Implementation notes

- `regression!` lives in `testes-regression/src/lib.rs`. Per ADR-0003, `testes-regression`
  depends only on `testes-core`.
- The expansion references only core types: `TestDeclaration`, `TestSpecification::Regression`,
  `EvidenceRecord`, `DimensionEvidence::Regression`, `RegressionEvidence`, `Dimension`,
  `FailureId`, `ArtifactId`, `TestId`, `Scope`, `BackendIdentity`. Zero infrastructure dependency
  per ADR-0001 §1.
- `Dimension` must be checked against the six accepted underlying dimensions at expansion time, so
  `Dimension::Mutation` and `Dimension::Regression` are rejected. `TestDeclaration::validate()`
  checks this for `Regression` (types.rs:200, with fixtures at 584, 720, 775), but a compile-time
  rejection is preferable and is what this grammar specifies.
- `docs/macro-plan.md` order item 4 pairs property and regression replay. ADR-0005 (`property!`) is
  complete; this ADR's acquisition question should be settled in the same slice.
- `AGENTS.md` non-negotiable: "Preserve property regressions, representative fuzz corpus, failure
  reproducers, and evidence needed for replay. **Never delete them merely to make checks pass.**"
  `regression!` is where that obligation becomes executable — and, under option A, where the
  retained reproducer must live.

Related: [[regression]], [[regression-macro]], [[no-redundant-execution]], [[failure-identity]],
[[stable-failure-identity]], [[historical-failure-evidence]], [[evidence-thread-local]],
[[regression-evidence]], [[replay-artifacts]], [[the-grammar-adr-series]],
[[adr-0004-unit-macro-grammar-and-expansion]], [[adr-0002-unit-macro-port-boundary]],
[[testes-regression]], [[testes-core]].
