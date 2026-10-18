# ADR-0011: `mutate!` macro grammar and expansion

- Status: Accepted
- Date: 2026-10-17
- Project: `testes`
- Scope: decides `mutate!`'s grammar and expansion target, per `docs/macro-plan.md` acceptance
  item 1. No grammar ADR existed before this one — only `docs/dimensions/mutation.md` and one row of
  `docs/macro-plan.md`. A first-pass `mutate!` already exists in `testes-mutation/src/lib.rs`; this
  ADR is the grammar of record.

## Context

`mutate!` is the only dimension where the macro's most important property is a **negative**: what
it must not do.

`docs/dimensions/mutation.md` Declaration: "**Launch campaigns through orchestration, never
during macro expansion.**"

ADR-0001 §6: "`mutate!` declares a campaign scope and selected checks. **Orchestration executes
the campaign.**" ADR-0001 §3 lists it among the macro requirements: "**Never execute campaigns
or mutate source during expansion.**"

`AGENTS.md` makes it a project invariant: "Source mutation is isolated from the user's working
tree. **Never execute campaigns during macro expansion.**"

Three documents, one rule. Everything below follows from it.

### This is the one dimension that need not produce a `#[test]`

`docs/macro-plan.md` per-macro acceptance item 2: "Generate a valid native harness **or plan**;
do not merely label an item."

For seven dimensions a native harness is the natural expansion. For `mutate!` the campaign _is_
the harness, and the campaign must not run at expansion or test time. So `mutate!` generates a
**plan** that orchestration consumes, and nothing else.

The alternative is worse than awkward: a generated `#[test]` that validates a declaration and then
passes without executing anything reports a green test bit for a campaign that never ran. That is
`missing-tooling-must-not-pass` in its most direct form.

### The types already carry most of the declaration

```rust
// types.rs:154-159
TestSpecification::Mutation {
    source_selectors: Vec<String>,
    operators: Vec<String>,
    selected_checks: Vec<TestId>,
    commands: Vec<CommandSpec>,
}

// types.rs:499-507
pub struct MutationEvidence {
    pub source_selectors: Vec<String>,
    pub operators: Vec<String>,
    pub selected_checks: Vec<TestId>,
    pub commands: Vec<CommandSpec>,
    pub baseline: ExecutionOutcome,
    pub mutations: Vec<MutationResult>,
}
```

Three things this determines:

1. **`selected_checks: Vec<TestId>` and `commands: Vec<CommandSpec>` are separate fields.** Checks
   are identities; commands are process invocations. So the grammar needs both — it is not one
   `checks:` field of command strings, as a naive design would have it.
2. **`baseline` is a single `ExecutionOutcome`, not a list and not `Observed`.** So the baseline is
   one verification command's result. ADR-0001 §6's "Establish a passing unmutated baseline"
   therefore maps to one `CommandSpec` designated as the baseline.
3. **`CommandSpec` is reused from integration** (`types.rs:351-356`) — `program`, `arguments`,
   `working_directory`, `environment`. So a mutation command is a typed, per-command environment,
   same as an integration command. Good: one command type for the whole project.

### `MutationOutcome::Excluded` requires a rationale — the type already forbids silent exclusion

```rust
// types.rs:475-491
pub enum MutationOutcome {
    Caught { detecting_checks: Observed<Vec<TestId>>, diagnostic: Option<String> },
    Missed,
    Unviable { diagnostic: String },
    Timeout { limit: Duration },
    Excluded { rationale: String },
}
```

`Excluded { rationale: String }` has **no variant without a rationale**. So
`docs/dimensions/mutation.md`'s "exclusions need rationale" is already enforced by the type, and
the grammar needs no `skip:` field to avoid inventing a bypass around it.

Similarly `Caught { detecting_checks: Observed<Vec<TestId>> }` — which checks detected a given
mutant is itself possibly unavailable. So `detection_rate()` can be computed while the attribution
is unknown, and the two must be recorded independently.

## Decision

### Grammar

```rust
mutate! {
    id: "budget-exhaustion-suite",
    subject: "budget::exhaustion",
    claim: "the selected checks detect deliberate defects in budget exhaustion",

    // Declared source scope. Never conflated with what the backend actually assessed.
    selectors: [ "src/budget.rs" ],

    operators: [ "negate_condition", "replace_literal" ],

    // The selected checks — identities, resolved by orchestration to commands.
    checks: [ "budget::exhaustion::wraparound", "budget::exhaustion::external_input" ],

    // The verification commands. One is designated the baseline.
    baseline: CommandSpec {
        program: "cargo",
        arguments: vec!["test", "--workspace", "budget"],
        working_directory: None,
        environment: BTreeMap::new(),
    },
    commands: [
        CommandSpec { program: "cargo", arguments: vec!["test", "--workspace", "budget",
                       "--", "wraparound"], ..baseline.clone() },
    ],

    policy: mutation::MutationPolicy::strict(),
}
```

Eight fields:

- `id`, `subject`, `claim` — as for every macro; `TestId::new` contract on `id`
- `selectors` — declared source scope, `Vec<String>`
- `operators` — the mutations to apply, `Vec<String>`
- `checks` — the selected checks, `Vec<TestId>`. **Required.** "Mutation assesses only checks
  actually executed" is only enforceable if the selected set is explicit.
- `baseline` — the passing unmutated verification command, a `CommandSpec`. **Required:** "Baseline
  must pass."
- `commands` — the per-mutant verification commands, `Vec<CommandSpec>`. Distinct from `checks`,
  per the type.
- `policy` — the `MutationPolicy` the campaign is evaluated against

**No `skip:`, `timeout:`, `retries:`, or `path:` field.** Rationale:

| Omitted    | Why the type already handles it                                                                                                           |
| ---------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `skip:`    | `MutationOutcome::Excluded { rationale }` requires rationale; a grammar-level skip would create an exclusion with no contract requirement |
| `timeout:` | `MutationOutcome::Timeout { limit }` records the limit the backend used; letting the caller set it per-call would duplicate it            |
| `retries:` | a retry is an orchestration policy, not a declaration concern                                                                             |
| `path:`    | `CommandSpec.working_directory` already carries it, per-command                                                                           |

### Expansion target: a plan, not a test

```rust
/// Campaign plan for {id}. Consumed by orchestration; never executed by the test harness.
///
/// Generated by `mutate!`. Do not invoke directly — orchestration owns execution,
/// and ADR-0011 §Decision forbids execution during expansion.
pub fn {id}_campaign() -> mutation::MutationCampaign {
    mutation::MutationCampaign {
        test_id: TestId::new("{id}").expect("TestId valid"),
        subject: "{subject}".into(),
        claim: "{claim}".into(),
        source_selectors: vec![ /* ... */ ],
        operators: vec![ /* ... */ ],
        selected_checks: vec![ /* TestId values */ ],
        baseline: { /* baseline CommandSpec */ },
        commands: vec![ /* ... */ ],
        policy: {policy},
        /// DECLARED scope only. The backend's REPORTED scope is recorded in
        /// `MutationEvidence.source_selectors` as actually assessed — these two are
        /// never conflated, per `docs/dimensions/mutation.md`'s "Record actual backend
        /// mutation scope."
        declared_scope: vec![ /* ... */ ],
    }
}
```

Three decisions:

1. **No `#[test]` is generated.** `docs/macro-plan.md` item 2 permits "a valid native harness **or
   plan**", and for `mutate!` the campaign is the harness. A generated `#[test]` that passes without
   executing a campaign would manufacture a green bit for work never done.
2. **A doc comment states the negative**, so a reader encountering the function in a `use` statement
   learns the boundary without opening the ADR.
3. **`declared_scope` is separate from reported scope.** `docs/dimensions/mutation.md`: "Record
   **actual** backend mutation scope." A campaign that mutates 40 of 60 declared sites must record
   40 assessed, not 60 declared — and the 20 unaddressed sites stay visible rather than being
   implied away.

A function rather than a `const` because `TestId::new(...)` is fallible (`Result<TestId,
IdError>`), so a `const` could not hold a validated id. See open questions.

### What a passing campaign looks like when it _does_ run

Included because `docs/dimensions/mutation.md` says `detection_rate()` "is arithmetic only and does
not establish baseline validity or policy satisfaction," and the reader needs to see where those
two live. This is **orchestration's** shape, not the macro's:

```rust
// Orchestration, NOT macro expansion. Runs in an isolated workspace.
let campaign = budget_exhaustion_suite_campaign();

// 1. Baseline first. A failing baseline invalidates every mutant result.
let baseline = orchestration::verify(&campaign.baseline)?;
if !matches!(baseline.status, ExecutionStatus::Passed) {
    return Err(OrchestrationError::BaselineFailed);   // no per-mutant report at all
}

// 2. The campaign, in an isolated workspace.
let report = orchestration::campaign(&campaign, /* isolated workspace */)?;

// 3. Build the evidence with ACTUAL scope, not declared scope.
let evidence = MutationEvidence {
    source_selectors: report.actual_scope,   // what the backend assessed
    operators: campaign.operators.clone(),
    selected_checks: campaign.selected_checks.clone(),
    commands: campaign.commands.clone(),
    baseline,
    mutations: report.results,
};

// 4. Policy decides satisfaction. detection_rate() is arithmetic only.
let verdict = campaign.policy.evaluate(&evidence);
```

### Honest measurement before execution

While a campaign has not run, the corresponding measurements are unavailable, not zero:

| Field              | Before execution                                                                                                                                                                                                                                                     |
| ------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `baseline`         | `Observed::Unavailable { reason: "campaign not executed; orchestration owns execution" }` at the _campaign_ level — note `MutationEvidence.baseline` is a plain `ExecutionOutcome`, so the unavailability is recorded on the campaign plan, not on the evidence type |
| `mutations`        | empty — **not** "all missed"                                                                                                                                                                                                                                         |
| `source_selectors` | declared, but `actual_scope` is `Unavailable` until the backend reports                                                                                                                                                                                              |
| `detection_rate()` | `None`, not `0.0` and not `1.0`                                                                                                                                                                                                                                      |

This is the plan-level application of `Observed<T>`. `docs/dimensions/mutation.md`: "**Zero
denominator means unavailable, not 100%.**" And `types.rs:509` on `detection_rate()`: "Excludes
unviable, timeout, and excluded outcomes. None means no assessed experiments."

### The expanded-macro coverage gap must be recorded, not assumed

`docs/dimensions/mutation.md`: "**Do not assume source mutation covers expanded declarative macro
code.** Record actual backend support."

All eight macros are declarative (`macro_rules!`) per ADR-0004 through this ADR. A source-level
mutator operating on `src/budget.rs` sees `unit!`/`property!` **invocation sites**, not the
generated `#[test]` bodies with their `catch_unwind`, evidence construction, and outcome mapping.

So a mutant inside generated code may be unaddressable, and a campaign's score would overstate the
suite's sensitivity. The plan therefore **does not claim** coverage of generated code; only
`report.actual_scope` establishes what was assessed. ADR-0001's risk row — "Mutation results
overstate generated-code coverage" → "Record actual assessed scope" — is implemented by this
separation.

### ADR-0002 relationship

`mutate!` is not a composition root, because it composes no execution. It produces a value.

ADR-0002 §4 does list `mutate!` among dimensions reserved for `TestBackend`, so if a second
campaign implementation ever appears (a mutator other than cargo-mutants, or a seeded-faulty
mode), a backend becomes appropriate **then**. Per ADR-0002: "OCP is 'extend via new implementation
when a second implementation shows up.'"

Note the deliberate contrast with ADR-0009: `mutate!` needs no cross-macro evidence channel,
because it consumes no execution.

## Consequences

- **`mutate!` is the only macro with no `#[test]` and no `thread_local!` slot.** Nothing about it
  participates in the cargo test pass/fail bit. CI cannot detect a broken campaign declaration by
  running tests — it must consume the plan, which means a **new CI lane**, not an extension of the
  existing one. `docs/validation.md`: "Report separate lanes ... **Do not count an omitted lane as
  passed.**"
- **`MutationPolicy` must exist before this ADR can be accepted.** See open questions — it is
  named in three documents and defined in none.
- **A malformed campaign declaration is caught when the plan is built**, at the accessor, rather
  than when tests run. That is weaker than a compile-time check.
- **`declared_scope` vs reported scope is a permanent two-field distinction**, and the plan is the
  reason it stays visible rather than collapsing into one "coverage" number.
- **The expanded-macro gap is stated but not solved.** Honest reporting is available; accurate
  mutation of macro-generated code is not. This ADR makes the gap visible and obliges the record to
  reflect it, which is all the current design permits.
- **`selectors` path resolution is a silent-failure risk.** A selector that resolves to nothing
  produces a vacuous campaign — the mutation analogue of `empty-scaffold-target`. `MutationPolicy`
  is the natural place for an assessed-count threshold, which is one more reason it must exist.
- **CI needs a seventh lane**, and the plan/observation mechanism is entangled with the still
  unspecified evidence serialization boundary (ADR-0001 §7).

## Alternatives considered

1. **Generate a `#[test]` that validates the declaration and passes.** Rejected: a green test bit
   for a campaign that never ran — the most direct instance of
   `missing-tooling-must-not-pass` in the project. `docs/dimensions/fuzz.md:25`'s "Empty scaffold
   targets provide no fuzz evidence" is the same principle.
2. **Generate a `#[test]` that panics with "campaign requires orchestration".** Rejected: a
   permanently failing test breaks `cargo test` for every consumer that declares a campaign, and its
   failure carries no information about whether the campaign worked.
3. **Execute the campaign during expansion.** Rejected by three documents
   (`docs/dimensions/mutation.md`, ADR-0001 §3, `AGENTS.md`).
4. **Add a `skip:` field for mutants that do not compile.** Rejected:
   `MutationOutcome::Unviable { diagnostic }` already exists, and `Excluded { rationale }` already
   requires justification — a grammar-level skip would bypass both.
5. **Add a `timeout:` field.** Rejected: `MutationOutcome::Timeout { limit }` records the limit the
   backend used; a per-call field would duplicate it and let the caller pre-empt the backend's
   report.
6. **One `verify:` field instead of `checks:` + `commands:` + `baseline:`.** Rejected: the types
   keep all three distinct, because conflating the baseline (which must pass on _unmutated_ code,
   and whose failure invalidates everything) with per-mutant checks would destroy the attribution
   rule.
7. **Put `MutationCampaign` in `testes-core`.** Rejected: it would add a campaign type to the
   dependency sink, which ADR-0001 §1 keeps free of orchestration and campaign concerns.
8. **A `const` campaign plan.** Rejected: `TestId::new` is fallible, so a `const` cannot hold a
   validated id without an `unwrap`. See open questions.

## Validation (to add when implemented)

1. **Expansion generates no `#[test]`** — `cargo test` collects nothing from a `mutate!` invocation
   (compile fixture asserting no test function name is produced).
2. **Expansion executes nothing** — the plan is a plain function over literals; a fixture asserts
   no subprocess or filesystem write occurs during compilation of a crate containing `mutate!`.
3. **`baseline` is required** — omitting it is a compile error, not a defaulted value.
4. **`checks` is required** — a campaign with no selected checks is a compile error, since
   "Mutation assesses only checks actually executed" is otherwise unenforceable.
5. **No `skip:` / `timeout:` / `retries:` field** — compile-fail fixtures attempting each.
6. **Pre-execution measurements are unavailable, not zero** — a record built from the plan alone
   has an unavailable baseline, empty `mutations`, unavailable actual scope, and
   `detection_rate() == None`.
7. **`declared_scope` ≠ actual scope** — a campaign reporting 40 of 60 declared sites records both;
   a fixture asserts the record shows 40 assessed, not 60 declared.
8. **Malformed declaration rejected at plan construction** — empty `selectors`, empty `operators`,
   or a `baseline` that is empty is rejected.
9. **Orchestration rejects a failing baseline** — a campaign whose baseline fails produces
   `BaselineFailed` and **no** per-mutant report (fixtures using a real mutator, gated on toolchain
   availability per `docs/validation.md`).
10. **`MutationPolicy` is evaluated separately from `detection_rate()`** — a fixture asserting a
    policy-failing campaign is reported unsatisfied even when the arithmetic rate looks acceptable.
11. **Exclusion requires rationale** — an `Excluded` outcome with an empty rationale is rejected
    (the type should already make this unrepresentable; the fixture pins it).

Items 1–8 need no external toolchain. Items 9–11 require cargo-mutants installed and are the
"native mutation checks" `docs/validation.md` says have "no project wrappers" yet.

## Open questions for the owner

1. **`MutationPolicy` does not exist.** It is named in `docs/dimensions/mutation.md` Acceptance
   ("Assessed count, missed results, and unresolved timeouts must satisfy explicit
   `MutationPolicy`") and Reporting, and `docs/handoff.md` flags "Mutation-rate arithmetic does
   not check baseline validity or policy" as an open review question — but there is **no
   definition, no type, and no threshold anywhere in the repository**. This ADR's `policy:` field
   needs the type. **This is blocking**: designing `MutationPolicy` should precede implementation.
   This ADR is Accepted as a grammar decision; accepting it does not resolve this question.
2. **Function or `const` plan?** A `const` would give compile-time evaluation but cannot hold
   `TestId::new(...)` without an `unwrap`, and `AGENTS.md` prefers contextual `expect` over
   unexplained `unwrap`. A function loses const checking. Is a plain function acceptable?
3. **Who resolves `selectors` paths?** Workspace root, the invoking crate, or the orchestration
   working directory? A wrong answer silently mutates nothing and produces a vacuous report.
4. **How does CI observe a campaign without a `#[test]`?** A new lane is required; its shape —
   consuming a binary, a JSON artifact, or an orchestration run — is unsettled and interacts with
   the still-unspecified evidence serialization boundary (ADR-0001 §7).
5. **Is `Observed` needed on the campaign plan at all?** `MutationEvidence.baseline` is a plain
   `ExecutionOutcome`, so there is no `Observed` slot in the evidence type for "not yet executed."
   If the plan is a separate type, should it carry `Observed` where the evidence type does not?
6. **Does `mutate!` need ADR-0009's evidence channel?** It consumes no execution, so no. But if
   campaign orchestration ever publishes `MutationEvidence` through `testes-core`, the two ADRs
   interact.

## Implementation notes

- Lives in `testes-mutation/src/lib.rs`, with `campaign::MutationCampaign` alongside it.
- `testes-mutation` depends on `testes-core` + cargo-mutants orchestration (ADR-0003). Per
  ADR-0001 §5 the campaign runs as an **external campaign adapter**, invoked by orchestration —
  never linked into core, never run during expansion.
- The plan references core types (`TestId`, `Dimension::Mutation`, `MutationEvidence`,
  `MutationResult`, `MutationDefinition`, `MutationOutcome`, `CommandSpec`, `ExecutionOutcome`,
  `ArtifactId`, `Observed`) plus `testes-mutation`'s `MutationCampaign` and `MutationPolicy`. No
  campaign type enters `testes-core`.
- No `testes-core` **type** change is required — only the `MutationPolicy` _type_ must be authored,
  and it belongs in `testes-mutation`.
- `docs/macro-plan.md` order item 7 places mutation orchestration and seeded faulty subjects last,
  paired deliberately.
- `AGENTS.md` non-negotiables this ADR implements: "Source mutation is isolated from the user's
  working tree. **Never execute campaigns during macro expansion.**" and "Record actual scope,
  revision, backend configuration, and available measurements truthfully."
- `docs/macro-plan.md` item 7 also names "seeded faulty subjects" — a separate mode from source
  mutation (ADR-0001 §6), and **not** something `mutate!` should express. If it needs a declaration
  form, that is a separate ADR.

Related: [[mutation]], [[mutation-macro]], [[mutation-evidence]], [[mutation-policy]],
[[mutation-campaign-isolation]], [[mutation-detection-rate]], [[unmutated-baseline]],
[[mutation-outcome]], [[zero-denominator-is-unavailable]],
[[expanded-macro-code-coverage-gap]], [[missing-tooling-must-not-pass]],
[[registration-cannot-fabricate-evidence]], [[the-grammar-adr-series]], [[testes-mutation]],
[[testes-core]], [[ci-pipeline]].
