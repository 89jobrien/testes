# testes-mutation

The `mutate!` dimension. It declares a mutation campaign, records the result of a campaign that a runner supplied, and assesses that result against an explicit policy.

Status: declaration, evidence recording, and assessment are implemented. The crate runs no mutation campaign itself. cargo-mutants integration is not implemented and must stay outside `testes-core`.

## Usage

```rust
testes_mutation::mutate! {
    id: "mutation-id",
    subject: "module::item",
    claim: "claim statement",
    sources: ["src/lib.rs"],
    operators: ["delete-statement"],
    checks: ["check-id"],
    commands: [("cargo", ["nextest", "run"])],
    policy: MutationPolicy {
        minimum_assessed: 1,
        maximum_missed: 0,
        allow_timeouts: false,
    },
    campaign: campaign_value,
}
```

The macro evaluates to a `MutationRecord`. The record is `#[must_use]`. Call `assert_satisfied()` or inspect `assessment` before you report the claim as verified. The macro panics with a contextual message if the id, subject, claim, or check identifiers are invalid.

## Assessment rules

- The baseline run must pass.
- Caught mutants must be attributed to a selected check.
- Timeouts and exclusions follow the policy.
- A zero assessed count is never reported as full detection.

Mutation analysis assesses only the checks that actually executed, not a hypothetical set.

## API

- `MutationSpec`, `MutationCampaign`, and `record_mutation(...)` build the record.
- `command(program, arguments)` builds a `CommandSpec`.
- `assess(...)` returns a `MutationAssessment` with `MutationCounts`.
- The grammar is in [ADR-0007](../docs/decisions/0007-mutate-macro-grammar.md).

## Dependencies

- `testes-core`
