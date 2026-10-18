# ADR-0012: Grammar coverage for the remaining four dimensions

- Status: Accepted
- Date: 2026-10-17
- Project: `testes`
- Scope: records the sequencing and cross-cutting consequences of ADR-0008 through ADR-0011, and
  the amendments they require to earlier ADRs. Accepting the grammars does **not** close the open
  questions those ADRs raise; those stay open and are enumerated in §4.

## Context

`docs/macro-plan.md` per-macro acceptance item 1: "Document exact grammar and examples **before**
implementation."

Until 2026-10-17, four of eight macros had grammar ADRs:

| Macro          | Grammar ADR  | Lines | Completeness                                                                           |
| -------------- | ------------ | ----- | -------------------------------------------------------------------------------------- |
| `unit!`        | ADR-0004     | 182   | complete                                                                               |
| `property!`    | ADR-0005     | 161   | complete                                                                               |
| `fuzz!`        | ADR-0006     | 151   | complete                                                                               |
| `prove!`       | ADR-0007     | 45    | stub — no expansion code, no thread-local slot, no Phase 3 plan, no ADR-0002 reference |
| `conformance!` | **ADR-0008** | —     | accepted                                                                               |
| `regression!`  | **ADR-0009** | —     | accepted                                                                               |
| `integration!` | **ADR-0010** | —     | accepted                                                                               |
| `mutate!`      | **ADR-0011** | —     | accepted                                                                               |

All eight now have an ADR. But "has an ADR" is a weaker claim than it looks, and this one records
what the new four actually change.

### Grounding note: the four were drafted against the real type surface

Each proposal was written against `testes-core/src/types.rs` and `traits.rs` as they stand, not
against the summaries in `docs/`. That changed three drafts substantially:

| Real type                                                                                                                      | Effect on the proposal                                                                                                      |
| ------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------- |
| `ConformanceSuite::contract() -> ContractIdentity`, `SubjectFactory::implementation() -> ImplementationIdentity`               | ADR-0008's grammar drops `contract:` and `subject:` fields — they are already typed and supplied                            |
| `ContractCaseOutcome::{Executed, NotApplicable, NotExecuted}` and `SubjectFactory::capabilities()`                             | ADR-0008's applicability model is **derived**, not declared — no `skip:` field is even representable                        |
| `ComponentIdentity { boundary: BoundaryKind }`, `CommandSpec { environment }`, `IntegrationEvidence.cleanup: ExecutionOutcome` | ADR-0010 needs **no** `testes-core` type change; the three-way classification and the per-command environment already exist |
| `MutationOutcome::Excluded { rationale }`, `MutationOutcome::Timeout { limit }`                                                | ADR-0011 needs no `skip:` or `timeout:` field — the types already enforce rationale and record the limit                    |
| `RegressionEvidence` has **no field** naming the underlying declaration                                                        | ADR-0009's core-channel option would require a `testes-core` type change — see §2                                           |
| `CheckFailure::{Violation, Infrastructure, Inconclusive}`                                                                      | ADR-0008 maps all three without flattening, closing one ADR-0001 §2 prohibition                                             |

Any of these types changing would invalidate part of the corresponding ADR. This table was
re-checked against the source on 2026-10-18, the acceptance date; **re-check it again on any
change to these types.**

## Decision

### 1. Status is uniform; completeness is not

All twelve are **Accepted**. Acceptance covers the grammar, the expansion target, and the evidence
construction — it does not close the open questions each grammar ADR raises:

- ADR-0001–0007 were accepted before this batch.
- ADR-0008–0012 are `Status: Accepted` as of 2026-10-18, on the owner's review.
- The unresolved decisions in §4 below and in each ADR's own open-questions section **remain open**.
  Accepting a grammar does not settle them; where one determines a field's type (ADR-0009's
  acquisition strategy) the grammar records the gap explicitly instead of guessing.

`docs/architecture.md`'s rule applied: the grammars were recorded with context, alternatives,
consequences, and validation before the expansions were reworked.

**Implementation is gated on the open decisions, not on ADR status.** The shipped first-pass
expansions in `testes-*/src/lib.rs` predate this batch and do not yet match these grammars.

### 2. ADR-0009 is the only one of the four requiring a `testes-core` change — and it is conditional on an unsettled choice

`RegressionEvidence` (`types.rs:460-467`) has four fields: `failure: FailureId`,
`reproducer_artifacts: Vec<ArtifactId>`, `historical_failure: Option<ArtifactId>`, and
`execution: Box<DimensionEvidence>`.

**None of them names the declaration whose live execution is being framed.** `failure` identifies
the defect, `reproducer_artifacts` identifies retained material, `underlying_dimension` identifies
the kind. So there is no field that could key an evidence lookup.

ADR-0009 presents three acquisition options and does not choose:

| Option                                                                                   | Needs a `testes-core` change?                                                                                                                                                                |
| ---------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **A. Same-expansion wrapping** — the reproducer runs inside the same generated `#[test]` | **no**                                                                                                                                                                                       |
| **B. Shared core channel** — thread-local, `TestId`-keyed, take-on-read                  | **yes** — mutable state (the project's first) _and_ a new field for the key                                                                                                                  |
| **C. Artifact-mediated replay** — load the reproducer's evidence from an `ArtifactStore` | **no**, and most consistent with `reproducer_artifacts: Vec<ArtifactId>` — but blocked on `ArtifactStore` having no implementation, and it weakens "one execution" to "a previous execution" |

**A is the only option that satisfies "reuse one execution" unconditionally.** So the choice is
between the type-consistent option (C) and the requirement-satisfying option (A), with B available
at the cost of two dependency-sink changes.

If B is chosen it would additionally amend ADR-0004, ADR-0005, ADR-0006, and ADR-0007, whose
per-macro single-value `thread_local!` slots it supersedes. Their grammar, expansion sequence,
backend identity, and honest-unavailability decisions would be unaffected — only the storage
mechanism changes.

ADR-0008, ADR-0010, and ADR-0011 require **no `testes-core` type change** on the evidence side.
ADR-0011 still needs `MutationPolicy` _authored_, but it belongs in `testes-mutation`.

### 3. ADR-0011 is the only macro that produces no `#[test]`

`docs/macro-plan.md` item 2 permits "a valid native harness **or plan**."

| Macro          | Expansion product           | Produces `#[test]`? | `thread_local!` slot? |
| -------------- | --------------------------- | ------------------- | --------------------- |
| `unit!`        | `#[test]` per example       | yes                 | yes                   |
| `property!`    | one `#[test]`               | yes                 | yes                   |
| `fuzz!`        | one `#[test]`               | yes                 | yes                   |
| `prove!`       | one `#[test]`               | yes                 | (unspecified)         |
| `conformance!` | one `#[test]`               | yes                 | yes                   |
| `regression!`  | one `#[test]`               | yes                 | yes                   |
| `integration!` | one `#[test]`               | yes                 | yes                   |
| **`mutate!`**  | **`MutationCampaign` plan** | **no**              | **no**                |

Consequence: `cargo test` cannot observe a campaign declaration at all. Detecting a broken
campaign requires a **new CI lane**, not an extension of the existing one. `docs/validation.md`:
"Report separate lanes ... **Do not count an omitted lane as passed.**"

### 4. The owner decisions these four ADRs raise

Seventeen, listed once here for triage. Each ADR repeats its own.

| #   | Decision                                                                                                                         | ADR  | Blocking?                                          |
| --- | -------------------------------------------------------------------------------------------------------------------------------- | ---- | -------------------------------------------------- |
| 1   | `regression!`'s acquisition strategy: A same-expansion, B core channel, or C artifact-mediated                                   | 0009 | **yes** — determines the `reproducer` field's type |
| 2   | Whether `RegressionEvidence` gains a field naming the underlying declaration                                                     | 0009 | **yes, if B is chosen**                            |
| 3   | Whether `historical_failure: Option<ArtifactId>` is sufficient (it records presence, not sufficiency)                            | 0009 | no                                                 |
| 4   | Whether ADR-0007's private slot is fixed in the same pass                                                                        | 0009 | no                                                 |
| 5   | Whether `SubjectFactory::create` is its own lifecycle phase, rather than folded into the case outcome                            | 0008 | no                                                 |
| 6   | Where conformance's retained artifacts go — `CaseContext::retain_artifact` needs an `ArtifactStore`, which has no implementation | 0008 | no                                                 |
| 7   | Whether partial capability coverage executes the case or marks it `NotApplicable`                                                | 0008 | no                                                 |
| 8   | What produces `ContractCaseOutcome::NotExecuted` as distinct from `NotApplicable`                                                | 0008 | no                                                 |
| 9   | Whether `IntegrationContext`'s `fixtures`/`observations` can be retained before `ArtifactStore` exists                           | 0010 | no                                                 |
| 10  | Whether `seam_id: String` should become a typed `SeamId` so `validate_shape()` checks identity rather than string equality       | 0010 | no                                                 |
| 11  | `sanitize`'s redaction mechanism — allow-list, deny-list, or hook                                                                | 0010 | no                                                 |
| 12  | Where integration's `default_budget()` comes from (it cannot live in `CommandSpec`, or a caller could pre-empt the bound)        | 0010 | no                                                 |
| 13  | Whether substitution belongs in evidence rather than a `tracing::warn!` — `IntegrationEvidence` has no field for it              | 0010 | no                                                 |
| 14  | **`MutationPolicy` does not exist** — no definition, no type, no threshold anywhere in the repository                            | 0011 | **yes** — ADR-0011's `policy:` field needs it      |
| 15  | Whether the campaign plan is a `const` or a function (`TestId::new` is fallible)                                                 | 0011 | no                                                 |
| 16  | Who resolves `selectors` paths — a wrong answer yields a vacuous campaign                                                        | 0011 | no                                                 |
| 17  | How CI observes a campaign with no `#[test]`                                                                                     | 0011 | no                                                 |

Three are genuinely blocking: **#1** (ADR-0009), **#2** (conditional on #1), and **#14**
(ADR-0011).

### 5. `MutationPolicy` is a pre-existing gap, exposed not created

`MutationPolicy` is named in:

- `docs/dimensions/mutation.md` Acceptance: "Assessed count, missed results, and unresolved
  timeouts must satisfy **explicit `MutationPolicy`**."
- `docs/dimensions/mutation.md` Reporting: "`detection_rate()` is arithmetic only and does not
  establish baseline validity or **policy satisfaction**."
- `docs/handoff.md`: "Mutation-rate arithmetic does not check baseline validity or policy."

And defined nowhere. `docs/handoff.md` files this under "Known review questions", described as
"design review items, not claimed compiler failures."

So ADR-0011 does not introduce the gap — it surfaces it, because a grammar with a `policy:` field
needs the type to exist. Designing `MutationPolicy` is a separate task and should precede
ADR-0011's acceptance.

### 6. ADR-0007 remains a stub

`prove!` still has ADR-0007's 45-line ADR while the other seven have 151–182 lines plus these four
proposals. Its missing pieces relative to ADR-0004/0005/0006:

- no expansion code
- no `thread_local!` slot (and per §2, it would need the ADR-0009 channel)
- no panic-propagation behavior
- no implementation location
- no rejected alternatives
- no Phase 3 test plan beyond three outcome rows
- **no restatement of ADR-0002's boundary**, which the other seven all cite explicitly
- no Kani toolchain integration, though Kani is not a library and needs its own binaries

`prove!` has the most demanding _spec_ (`docs/dimensions/model-check.md`: bounded scope, justified
assumptions, non-vacuity checks, the anti-wait-for-a-bug rule) and the least specified macro.
**This ADR does not fix that** — rewriting ADR-0007 is a separate piece of work, and doing it in
the same batch as four new ADRs would make review harder, not easier.

### 7. Implementation order is unchanged

`docs/macro-plan.md` order:

| Item | Slice                                                | Grammar status after this batch                                                    |
| ---- | ---------------------------------------------------- | ---------------------------------------------------------------------------------- |
| 1    | Baseline checks and core validation tests            | **done**                                                                           |
| 2    | `unit!` plus one honest evidence path                | ADR-0004 accepted; ADR-0009 accepted (open decision #1 still gates implementation) |
| 3    | Conformance lifecycle and contract suite declaration | ADR-0008 accepted                                                                  |
| 4    | Property/regression replay                           | ADR-0005 accepted; ADR-0009 accepted                                               |
| 5    | Integration/process helpers                          | ADR-0010 accepted                                                                  |
| 6    | Fuzz and proof backend wrappers                      | ADR-0006 accepted; ADR-0007 stub                                                   |
| 7    | Mutation orchestration and seeded faulty subjects    | ADR-0011 accepted; **`MutationPolicy` undefined**                                  |

Adding four ADRs does not change the order, and per `docs/macro-plan.md` "This ordering is a
development plan, **not a requirement to defer proof-critical behavior or postpone regression
tests.**"

### 8. Documentation drift this batch does not fix

Recorded, not addressed — each is a separate edit:

| Document                                 | Drift                                                                              |
| ---------------------------------------- | ---------------------------------------------------------------------------------- |
| `docs/handoff.md`                        | pre-workspace paths (`src/types.rs`), "no dependencies", zero tests                |
| `docs/architecture.md` "Crate evolution" | predates ADR-0003's split                                                          |
| `docs/README.md` "Implementation status" | says macros are unimplemented; still true, but four more grammars are now Accepted |
| ADR-0001 §8                              | CI runs plain `cargo test`, not `--workspace`                                      |
| `docs/validation.md`                     | records no results and omits `--workspace`                                         |

## Consequences

- **Eight of eight macros now have an ADR**, seven of them substantial. The
  `docs/macro-plan.md` precondition for `conformance!`, `regression!`, `integration!`, and
  `mutate!` is met.
- **The grammars are decided; the expansions are not.** Each of the four has unresolved owner
  decisions, three of them blocking, and the shipped first-pass expansions do not yet match them.
- **Only ADR-0009 potentially changes `testes-core`.** ADR-0008, ADR-0010, and ADR-0011 need no
  core type change — the types already carry the classification, command, capability, and outcome
  models those dimensions require. That is a materially smaller footprint than drafting from
  `docs/` alone would have produced.
- **ADR-0009's acquisition choice is unresolved**, so `regression!`'s grammar has five settled
  fields and one whose type is not. It is deliberately incomplete.
- **`MutationPolicy` must be authored before `mutate!` can be implemented** — a gap
  `docs/handoff.md` already flagged, now on the critical path and load-bearing for
  `docs/dimensions/mutation.md`'s Acceptance clause.
- **`ArtifactStore` blocks conformance and integration artifact retention.** `CaseContext::retain_artifact`
  and `IntegrationEvidence`'s `fixtures`/`observations` all need it, and it has no implementation.
  This was not on any list before this batch; it surfaces from reading the type signatures.
- **One new CI lane is implied** by ADR-0011, and the plan/observation mechanism is entangled with
  the still-unspecified evidence serialization boundary (ADR-0001 §7).
- **ADR-0007 is still the weakest ADR in the repo** and remains a known hole.

## Alternatives considered

1. **Leave the four new ADRs at Proposed indefinitely.** Rejected: the grammars are settled enough
   to record, and the open decisions are enumerated in §4 where they stay trackable. Holding status
   at Proposed forever conflated "grammar not yet decided" with "grammar decided, follow-ups
   still open". The alternative actually chosen accepts the grammars and carries the open
   decisions explicitly.
2. **Fix ADR-0007 in the same batch, completing the series.** Rejected: four new ADRs plus a
   rewrite of an existing accepted one is too much for one review. ADR-0007 is a separate task.
3. **One combined ADR for all four dimensions instead of four.** Rejected: the four differ
   substantially — `mutate!` produces no test at all, `regression!` requires a core change,
   `conformance!` forbids case definitions, `integration!` forbids a timeout field. A single
   document would obscure exactly the contrasts that matter.
4. **Number them 0008–0011 by alphabet rather than implementation order.** Rejected:
   `docs/macro-plan.md` order (conformance 3, regression 4, integration 5, mutation 7) is the
   project's own sequence, so numbering in it makes the ADR series readable alongside the plan.
5. **Defer all four until ADR-0007 is fixed.** Rejected: `docs/macro-plan.md` order item 3 is
   conformance, which precedes everything ADR-0007 serves. Fixing `prove!` first would invert the
   documented sequence for no gain.

## Validation

This ADR records no code, so there is nothing to test. Its validation obligation is procedural:

1. Each of ADR-0008–0011 reaches `Status: Accepted` or is revised.
2. The blocking decisions (#1, #2, #14 above) are resolved before the corresponding macro is
   implemented.
3. The `testes-core` type surface is re-checked against the four proposals — the grounding note in
   §Context lists which types each proposal depends on, and any change invalidates part of it.
4. If option B is chosen for ADR-0009, ADR-0004, ADR-0005, and ADR-0007 gain amendment notes
   pointing to it.
5. `docs/macro-plan.md`'s implementation order is re-checked against accepted grammar coverage.
6. `docs/README.md`'s implementation status is updated to reflect eight accepted grammars.

Items 4–6 are documentation edits deferred until the ADRs are accepted, so that no document claims
more than has been decided.

## Related

[[the-grammar-adr-series]], [[where-the-project-actually-stands]], [[mutation-policy]],
[[composition-root]], [[declarative-macro]], [[macro-implementation-plan]],
[[adr-0002-unit-macro-port-boundary]], [[adr-0003-nine-crate-workspace-split]],
[[adr-0004-unit-macro-grammar-and-expansion]], [[adr-0007-prove-macro-grammar-and-expansion]],
[[conformance]], [[regression]], [[integration]], [[mutation]], [[open-decisions]],
[[known-review-questions]], [[next-vertical-slice]].
