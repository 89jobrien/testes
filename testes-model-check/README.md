# testes-model-check

The `prove!` dimension. It declares a model-check claim, runs it through a `ModelCheckBackend` port, and records `ModelCheckEvidence`.

Status: implemented as a port. The crate has no dependency on any verifier. Backend adapters, such as a future Kani adapter, live outside this crate.

## Port

```rust
pub trait ModelCheckBackend {
    fn identity(&self) -> BackendIdentity;
    fn check(&self, harness_id: &str, obligations: &[ObligationId]) -> ModelCheckRun;
}
```

`ModelCheckRun` is either `Executed(Vec<ProofObligationResult>)` or `Unavailable { reason }`. Adapters must return `Unavailable` rather than invent results.

## Evidence rules

- Only a declared obligation with an `Established` result counts as proved.
- An unavailable backend yields `NotExecuted`, never `Passed`.
- A declared obligation with no backend result yields `NotExecuted`.
- A declaration with no obligations cannot pass, because nothing was proved.

## API

- `ProveSpec` holds the declaration inputs: id, subject, claim, harness id, and obligations.
- `prove(&spec, &backend)` returns `Err(DeclarationError)` only when the declaration is invalid. Every run outcome, including unavailability, comes back as an `EvidenceRecord`.
- `prove!` is the macro form.

## Dependencies

- `testes-core`
