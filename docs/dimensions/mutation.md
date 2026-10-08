# Mutation

## Definition

Assess whether selected verification detects deliberately introduced behavioral defects. Mutation is cross-cutting and evaluates only checks actually executed.

## Declaration

`TestSpecification::Mutation` identifies source selectors, operators, selected checks, and commands. Planned macro: `testes::mutate!` (not implemented). Intended initial backend: cargo-mutants; seeded faulty implementations are a separate planned mode. Launch campaigns through orchestration, never during macro expansion.

## Execution evidence

`MutationEvidence` records scope, checks/commands, unmutated baseline, and per-mutation transformation, outcome, duration, and artifacts. Outcomes are caught, missed, unviable, timeout, and excluded. Preserve detecting-check identities when available.

## Acceptance

Baseline must pass. Assessed count, missed results, and unresolved timeouts must satisfy explicit `MutationPolicy`. A catch must be attributable to the mutation rather than unrelated infrastructure failure.

## Reporting

Detection rate is caught / (caught + missed). Report other counts separately. Zero denominator means unavailable, not 100%. `detection_rate()` is arithmetic only and does not establish baseline validity or policy satisfaction.

## Limits

Do not assume source mutation covers expanded declarative macro code. Record actual backend support. A surviving mutant requires adequacy or equivalence review; exclusions need rationale.
