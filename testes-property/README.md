# testes-property

The `property!` dimension. It generates a `#[test]` that runs a Proptest strategy against a property body and records `PropertyEvidence` with case counts and any counterexample.

Status: implemented. Execution uses Proptest's `TestRunner`.

## Usage

```rust
testes_property::property! {
    id: "reverse-involutive",
    subject: "reverse",
    claim: "reversing a vector twice returns the original",
    strategy: {
        values: proptest::collection::vec(proptest::num::i32::ANY, 0..64),
    },
    body: {
        let mut twice = values.clone();
        twice.reverse();
        twice.reverse();
        proptest::prop_assert_eq!(twice, values);
    },
    cases: 256,
}
```

The generated test:

1. Validates the `TestDeclaration`.
2. Runs Proptest with the given strategies and body.
3. Records `PropertyEvidence`, including the counterexample if one was found.
4. Propagates the failure, as `unit!` does.

The grammar is in [ADR-0005](../docs/decisions/0005-property-macro-grammar.md). Some evidence fields are still `Unavailable` placeholders where a measurement was not taken. Those are reported as unavailable, not as zero.

## Notes

- The expansion refers to `::paste` and uses `proptest` by path, so a crate that uses `property!` must depend on both directly.
- `LAST_PROPERTY_EVIDENCE` is a thread-local holding the most recent evidence, for test introspection.

## Dependencies

- `testes-core`
- `proptest`
- `paste`
