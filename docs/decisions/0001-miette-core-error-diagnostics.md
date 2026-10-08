# 0001: miette + thiserror for core error diagnostics

## Context

`testes`'s core error enums — `IdError`, `DeclarationError`, `EvidenceShapeError`
(`src/types.rs`) and `BackendError`, `CheckFailure` (`src/traits.rs`) — carry meaning that
benefits from richer diagnostics than a bare `Display` string: source-span and help-text
style reporting makes evidence reporting more honest, especially for `DeclarationError` and
`EvidenceShapeError`, which previously had no `Display`/`Error` impl at all. Owner-approved
exception to the "prefer std-only core logic" convention (`AGENTS.md`), recorded here rather
than left as an undocumented drift.

## Alternatives considered

1. **Stay std-only + hand-rolled `Display`** (status quo). Keeps the zero-dependency
   property but means every error enum needs a hand-written `impl fmt::Display` and
   `impl std::error::Error`, and provides no path to diagnostic-grade output (spans, help
   text, error codes) without a larger rewrite later.
2. **`thiserror` alone, without `miette`.** Removes the Display/Error boilerplate but gives
   up on `Diagnostic`-level reporting entirely — doesn't address the stated motivation.
3. **`miette` + `thiserror` together (chosen).** `thiserror::Error` handles the
   `Display`/`std::error::Error` boilerplate; `miette::Diagnostic` derives alongside it on
   the same enum without conflict, and is purpose-built to pair with `thiserror` (this is
   `miette`'s documented primary use case). Gets both the boilerplate reduction and a path
   to diagnostic-grade reporting (source spans, help text, error codes) without a second
   migration later.

## Decision

Add `miette` and `thiserror` as dependencies of the core `testes` crate. Convert the five
error enums above to derive `thiserror::Error` + `miette::Diagnostic`, in addition to their
existing `Clone`/`Debug` derives:

- `IdError`, `BackendError`: preserve their existing `Display` text exactly
  (`#[error("...")]` per variant, reproducing today's wording verbatim — including
  `BackendError`'s bare `{reason}` with no variant-name prefix, which
  `backend_error_display_shows_reason_for_each_variant` pins).
- `DeclarationError`, `EvidenceShapeError`, `CheckFailure`: had no prior `Display`/`Error`
  impl. New, concise `#[error("...")]` text was authored for each variant (see
  `src/types.rs`/`src/traits.rs`), pinned by new regression tests
  (`id_error_display_shows_message_for_each_variant`,
  `declaration_error_display_shows_message_for_each_variant`,
  `evidence_shape_error_display_shows_message_for_each_variant`).
- `BackendError` and `CheckFailure` additionally gained `Eq`/`PartialEq` derives as part of
  this change (net-new capability, not a preservation — none of their fields block
  derivation).
- Manual `impl std::fmt::Display`/`impl std::error::Error` blocks for `IdError` and
  `BackendError` were removed; the derives now provide both.

### `fancy` feature

`miette`'s `fancy` feature (pretty-printed terminal diagnostic rendering, graphical
source-span underlines) is **not** enabled here. This is a core library crate consumed by
backends/orchestrators, not a terminal binary — pulling in `fancy`'s rendering/terminal
dependencies at the core-crate level would push presentation concerns into a layer that
should stay presentation-agnostic (see `docs/architecture.md`'s dependency-direction rules).
A consumer that wants fancy terminal rendering can enable `miette`'s `fancy` feature itself
at the binary/orchestrator layer, where it belongs.

## Consequences

`testes` is no longer a zero-dependency crate. This is a deliberate, recorded exception to
`AGENTS.md`'s "prefer std-only core logic" convention — the convention's own text requires
justifying dependencies rather than forbidding them outright, and that justification is this
document. See `AGENTS.md` and `docs/architecture.md` for pointers back to this record.

## Validation

`cargo check`, `cargo test`, and `cargo clippy --all-targets` all run clean after the
conversion, with no new warnings and the pre-existing `backend_error_display_shows_reason_for_each_variant`
test passing unchanged.
