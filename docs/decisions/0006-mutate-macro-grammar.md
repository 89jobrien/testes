# ADR-0006: `mutate!` macro grammar, record construction, and assessment

- Status: Accepted
- Date: 2026-11-24
- Project: `testes`
- Scope: specifies `mutate!`'s grammar, the record it produces, and how mutation
  claims are assessed, before implementation. Follows ADR-0002's pattern (a
  declarative `macro_rules!` macro, no proc-macro, no `TestBackend` trait) and
  ADR-0003's crate split (the macro lives in `testes-mutation`, which depends only
  on `testes-core`).

## Context

`docs/dimensions/mutation.md` and `docs/macro-plan.md` require that mutation
evidence is recorded only from campaigns that actually ran: a passing baseline,
isolated mutations, selected checks, and per-mutant outcomes. The planned backend
is cargo-mutants, which must stay outside `testes-core` and must never run during
macro expansion. No backend exists yet, so this ADR does not add one.

`docs/handoff.md` also records that mutation-rate arithmetic (`detection_rate()`)
does not check the baseline or the policy. `detection_rate()` is arithmetic only
and must stay that way.

## Decision

### Grammar

```rust
let record = testes_mutation::mutate! {
    id: "parser-mutations",
    subject: "parser::parse",
    claim: "the parser checks detect deleted statements",
    sources: ["src/parser.rs"],
    operators: ["delete-statement"],
    checks: ["parser-unit-tests"],
    commands: [("cargo", ["nextest", "run", "-p", "parser"])],
    policy: MutationPolicy {
        minimum_assessed: 10,
        maximum_missed: 0,
        allow_timeouts: false,
    },
    campaign: campaign_value,
};
```

- `id`, `subject`, `claim`: as for `unit!`. `id` and `subject` must satisfy the
  `TestId` / `SubjectId` rules; a blank or control-character value panics with a
  contextual message, as `unit!` does. The declaration is validated with
  `TestDeclaration::validate()`.
- `sources`, `operators`, `checks`: the declared `TestSpecification::Mutation`
  selectors, operators, and selected check identifiers. `checks` are `TestId`s.
- `commands`: each entry is `(program, [arguments...])` and becomes a `CommandSpec`
  with no working directory and no environment overrides.
- `policy`: a `MutationPolicy` value. It is a requirement, not part of the declaration.
- `campaign`: a `MutationCampaign` value supplied by the orchestrator. It carries
  `Provenance`, `ExecutionMetadata`, artifacts, and the `MutationEvidence` the
  campaign produced. The macro never invents any of these.

### Record

`record_mutation` builds:

- a `TestDeclaration` with `Dimension::Mutation` and the Mutation specification;
- an `EvidenceRecord` whose `dimension_evidence` is `DimensionEvidence::Mutation`
  taken from the campaign, and whose `outcome` is the baseline's `ExecutionOutcome`.
  The record's outcome describes execution of the baseline only. Claim satisfaction
  is never stored in the record.

`MutationRecord` pairs the `EvidenceRecord` with a `MutationAssessment`. It is
`#[must_use]`, so a caller cannot discard the assessment and read the record as a pass.

### Assessment (claim evaluation)

`assess(declaration, evidence, policy)` is separate from declaration and from
record construction. It returns `Satisfied { counts, detection_rate }` only when
all of the following hold, and otherwise `Unsatisfied { counts, gaps }` with
`EvidenceGap` values from `testes-core`:

1. The declaration's specification is a Mutation specification. Otherwise the gap is
   `InvalidEvidenceShape`.
2. The evidence's sources, operators, selected checks, and commands equal the
   declared ones. Otherwise the gap is `RequiredScopeNotSatisfied`.
3. The baseline status is `Passed`. Otherwise the gap is `BaselineNotPassed`, even
   when every mutant was caught.
4. A `Caught` outcome with `Observed::Available` detecting checks has at least one
   check, and every check is among the evidence's selected checks. A catch
   attributed to an unselected check is not counted as an assessed detection.
   `Observed::Unavailable` detecting checks are allowed; the catch still counts.
5. A `Timeout` is a gap (`UnresolvedMutation`) unless `allow_timeouts` is set.
6. Missed mutants do not exceed `maximum_missed`. Otherwise the gap is `Other`.
7. An `Excluded` outcome has a non-blank rationale. Otherwise the gap is `Other`.
8. The assessed count (caught + missed) is non-zero and at least
   `minimum_assessed`. Otherwise the gap is `InsufficientMutationExperiments`.
   A zero-experiment campaign is never full detection.

The detection rate is `MutationEvidence::detection_rate()` (caught / (caught +
missed)), reported only in the `Satisfied` case. `Unviable`, `Timeout` (when
allowed), and `Excluded` outcomes are counted separately and do not enter the rate.

`MutationAssessment::verdict()` maps `Satisfied` to `ClaimVerdict::Supported` and
`Unsatisfied` to `ClaimVerdict::Insufficient { gaps }`. No mutation assessment
produces `Refuted`, which requires a policy decision not yet made.

### Interpretation of "only executed checks"

`MutationEvidence` has no separate executed-check list. Its `selected_checks` is
the set the campaign ran. A catch counts only when its attributed checks are a
subset of that set, and the evidence's selected checks must equal the declared
checks. This is the strongest check available without a backend that reports
executed checks individually. A backend that can report them should add that to
the evidence in a later ADR.

## Consequences

- `testes-core` is unchanged. The assessment uses `MutationPolicy`,
  `EvidenceGap`, `ClaimVerdict`, and `MutationEvidence::detection_rate()` as they
  already exist.
- `testes-mutation` gains no dependencies. cargo-mutants stays out of the workspace.
- Registering a declaration cannot produce a passing record. A record needs a
  campaign, and an empty or unexecuted campaign is `Unsatisfied`.
- The campaign's `ExecutionMetadata` is supplied, not invented. A backend identity
  and duration come from the runner.

## Alternatives considered

1. **Run cargo-mutants inside `mutate!`.** Rejected: campaigns must not run during
   macro expansion, and core/crate rules keep backend integration outside the macro.
2. **Put the baseline and policy checks in `testes-core` as methods on
   `MutationEvidence`.** Rejected for this change: it would widen the core API, and
   the task asks that core change only when required. The assessment is a function
   in `testes-mutation`.
3. **Make `mutate!` panic on an unsatisfied assessment.** Rejected in favor of
   `#[must_use]` plus `assert_satisfied()`, so a caller can inspect gaps before
   failing. A caller who discards the record gets a compiler warning.

## Open follow-on

- A cargo-mutants backend adapter that produces `MutationCampaign` values.
- Whether an unsatisfied policy should ever map to `ClaimVerdict::Refuted`.
- Executed-check reporting from backends, so "executed" can be checked per check
  rather than through the selected set.
