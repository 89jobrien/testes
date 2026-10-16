# ADR-0007: `prove!` macro grammar and expansion

- Status: Accepted
- Date: 2026-10-16
- Project: `testes`
- Scope: specifies `prove!`'s grammar for model-checking/formal verification

## Context

`prove!` is the model-checking dimension. Unlike fuzzing or property tests,
prove! uses formal verification backends (Kani, Prusti) to exhaustively verify
properties. Evidence records whether the proof succeeded, failed, or was inconclusive.

## Decision

### Grammar

```rust
prove! {
    id: "prove-sort-works",
    subject: "Vec::sort",
    claim: "sort produces ordered output",
    body: {
        let mut vec = vec![3, 1, 4, 1, 5];
        vec.sort();
        assert!(vec.windows(2).all(|w| w[0] <= w[1]));
    },
}
```

- `id`: unique identifier
- `subject`: what is being verified
- `claim`: the property
- `body`: verification code (asserts must hold)

### Expansion target

Generates a `#[test]` function that validates the declaration and records
`ProofEvidence` with `ProofStatus` (Established/Refuted/Inconclusive).

## Validation

- Passing proof: property holds → `ProofStatus::Established`
- Failing proof: counterexample found → `ProofStatus::Refuted`
- Inconclusive: timeout or unknown state → `ProofStatus::Inconclusive`
