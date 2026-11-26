# testes-fuzz

The `fuzz!` dimension. It runs inline seed inputs against a target under `catch_unwind` and records `FuzzEvidence`.

Status: implemented as an in-process seed-corpus runner. It is not a coverage-guided fuzzer.

## What it does and does not do

- Runs every seed input in the declared corpus, once, against the target.
- Records the planned inputs, the executions actually made, and the first failing input if one panics.
- Performs no mutation of inputs, records no coverage, and does not link libFuzzer.

A libFuzzer or cargo-fuzz campaign belongs in a separate adapter. That adapter returns `FuzzCampaign::Unavailable` when it cannot run, and never a pass.

## API

- `declare(id, subject, claim, target, oracles)` builds a `TestDeclaration` with a `Fuzz` specification.
- `run_seed_corpus(corpus, target)` runs the corpus and returns a `FuzzCampaign`: either `Executed(SeedCorpusRun)` or `Unavailable { reason }`.
- `record_campaign(...)` turns a campaign into an `EvidenceRecord`.
- `assert_passed(record)` panics unless the record passed.
- `fuzz!` is the declaration macro. It expands to a `#[test]` that runs the corpus and fails unless the status is `Passed`:

```rust
testes_fuzz::fuzz! {
    fn parser_survives_seeds {
        id: "parser-seeds",
        subject: "Parser::parse",
        claim: "Parser::parse does not panic on seed inputs",
        target: "parse",
        oracles: ["no-panic"],
        corpus: [b"", b"a=1", b"\xff"],
        body: |input: &[u8]| {
            let _ = parse(input);
        }
    }
}
```

Its grammar is in [ADR-0006](../docs/decisions/0006-fuzz-macro-grammar.md).

## Dependencies

- `testes-core`
