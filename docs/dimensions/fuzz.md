# Fuzz

## Definition

Exercise untrusted-input boundaries with adversarial input and explicit safety or semantic oracles. Distinguish arbitrary-byte parsers from string parsers with an explicit decoding gate.

## Declaration

`TestSpecification::Fuzz` identifies target, oracles, and corpus artifacts. Scope includes resource limits and input interpretation. Planned macro: `testes::fuzz!` (not implemented). Intended backend: cargo-fuzz/libFuzzer in a dedicated fuzz package.

## Execution evidence

`FuzzEvidence` records target/oracles, initial corpus, actual duration, available execution/coverage measurements, and failure inputs. Record instrumentation/configuration in the envelope. Preserve representative corpus and minimized failures.

## Acceptance

The required campaign executes without detected violations. Initialization failure, incomplete budget, missing required instrumentation, or a target that never exercises its subject cannot support success.

## Example obligation

Arbitrary protocol input cannot panic; accepted frames satisfy declared framing invariants.

## Limits

Crash-freedom is not all semantic correctness. Empty scaffold targets provide no fuzz evidence. Keep unavailable coverage measurements explicit.
