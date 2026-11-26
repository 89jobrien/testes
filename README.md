# testes

Rust testing declarations, native harnesses, and execution evidence for eight test dimensions: unit, property, fuzz, model check, conformance, integration, regression, and mutation.

testes is not a test runner. It lets a project state a verification claim, runs it through the native Rust test machinery or a port, and records what was actually executed. A deferral, an unavailable capability, or an unimplemented backend is reported as not executed, never as a pass.

## Workspace

| Crate                                                | Dimension               | Macro          | Status                                                               |
| ---------------------------------------------------- | ----------------------- | -------------- | -------------------------------------------------------------------- |
| [`testes-core`](testes-core/README.md)               | Shared types and traits | none           | Declaration, evidence, and policy types; ports                       |
| [`testes-unit`](testes-unit/README.md)               | Unit                    | `unit!`        | Implemented                                                          |
| [`testes-property`](testes-property/README.md)       | Property                | `property!`    | Implemented (Proptest)                                               |
| [`testes-fuzz`](testes-fuzz/README.md)               | Fuzz                    | `fuzz!`        | Implemented as an in-process seed-corpus runner; not coverage-guided |
| [`testes-model-check`](testes-model-check/README.md) | Model check             | `prove!`       | Implemented as a `ModelCheckBackend` port; no verifier bundled       |
| [`testes-conformance`](testes-conformance/README.md) | Conformance             | `conformance!` | Not implemented                                                      |
| [`testes-integration`](testes-integration/README.md) | Integration             | `integration!` | Implemented                                                          |
| [`testes-regression`](testes-regression/README.md)   | Regression              | `regression!`  | Implemented                                                          |
| [`testes-mutation`](testes-mutation/README.md)       | Mutation                | `mutate!`      | Declaration, evidence, and assessment; no mutation campaign runner   |

Each dimension crate depends only on `testes-core`, so you can add the ones you need. See [ADR-0003](docs/decisions/0003-eight-way-dimension-crate-split.md).

## Usage

The crates are not published (`publish = false`). Depend on the ones you need by path, for example `testes-unit = { path = "../testes/testes-unit" }`, and add `paste` as well, because the `unit!` and `property!` expansions refer to it.

```rust
testes_unit::unit! {
    id: "parse-round-trip",
    subject: "Parser::parse",
    claim: "Parser::parse accepts its own output",
    examples: {
        "simple" => {
            // `parse` is the code under test in your crate.
            assert!(parse("a=1").is_ok());
        }
    }
}
```

Each dimension has a grammar. See the crate READMEs and [docs/dimensions](docs/dimensions) for the full forms.

## Evidence

Every run produces an `EvidenceRecord` that separates four concerns:

- **Declaration** states the claim and required scope (`TestDeclaration`).
- **Execution** records what actually ran, under which backend and revision (`ExecutionOutcome`).
- **Evaluation** decides whether the evidence supports, refutes, or leaves the claim unsettled.
- **Governance** handles applicability and deferrals without turning them into verified claims.

Read the [evidence contract](docs/evidence.md) before interpreting a passing result.

## Validation

Run these from the repository root after any change, and report the exact results:

```sh
cargo fmt --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo test --workspace --doc
cargo clippy --workspace --all-targets -- -D warnings
```

## Documentation

- [Testing model](docs/README.md) and [evidence contract](docs/evidence.md)
- [Architecture](docs/architecture.md) and [decision records](docs/decisions)
- [Dimension definitions](docs/dimensions)
- [Validation](docs/validation.md) and [changelog](CHANGELOG.md)
- [Agent instructions](AGENTS.md)

## Invariants

- `#![forbid(unsafe_code)]` holds in every crate.
- `testes-core` does not depend on project domains, Proptest, Kani, libFuzzer, or cargo-mutants. Its `miette` and `thiserror` dependencies are the recorded exception in [ADR-0001](docs/decisions/0001-layered-dependencies-and-native-testing-backends.md).
- Mutation analysis assesses only checks that were actually executed.
- Trait semantics and conformance cases belong to the trait owner, not to testes.
