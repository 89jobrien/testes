# testes-unit

The `unit!` dimension. It generates native Rust `#[test]` functions that declare a unit claim, run each example with panic safety, and record `UnitEvidence`.

Status: implemented.

## Usage

```rust
testes_unit::unit! {
    id: "parse-round-trip",
    subject: "Parser::parse",
    claim: "Parser::parse accepts its own output",
    examples: {
        "simple" => {
            assert!(parse("a=1").is_ok());
        }
    }
}
```

For each example, the macro generates `#[test] fn {id}_{example}()`, which:

1. Validates the `TestDeclaration`.
2. Runs the example body under `catch_unwind`.
3. Records `UnitEvidence` with the outcome.
4. Re-panics on failure, so `cargo test` reports it normally.

The grammar is in [ADR-0004](../docs/decisions/0004-unit-macro-grammar.md). Native execution is the only implementation, so no `TestBackend` is involved ([ADR-0002](../docs/decisions/0002-unit-macro-port-boundary.md)).

## Notes

- The expansion refers to `::paste`, so a crate that uses `unit!` must depend on `paste` directly.
- `LAST_UNIT_EVIDENCE` is a thread-local holding the most recent evidence, for test introspection.
- `unit_example!` is an internal helper (`#[doc(hidden)]`). Call `unit!`.

## Dependencies

- `testes-core`
- `paste`
