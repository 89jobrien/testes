# Design gating and scope

Read this before adding a public API, a new crate or dependency, or any change that goes beyond the task you were given.

- Small, design-gated changes. Record material API decisions (with context,
  alternatives, consequences) before implementing — `docs/architecture.md` has an
  "Open decisions" list of things not yet settled (macro grammar, async/object-safe
  ports, evidence serialization, digest selection, proof policy, backend feature
  gates). Don't silently resolve one of these inside an unrelated change.
- No unrelated refactors. The workspace split is done (see
  `docs/decisions/0003-eight-way-dimension-crate-split.md`); don't add further
  crates or heavier per-dimension dependencies without an actual implementation
  need (see `docs/architecture.md`'s "Crate evolution").
