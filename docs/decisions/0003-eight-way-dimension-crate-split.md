# ADR-0003: Split into a core crate plus one crate per dimension

- Status: Accepted
- Date: 2026-10-08
- Project: `testes`
- Amends: ADR-0001 §9 ("Package/workspace split" was listed as an open decision,
  not yet settled) and `docs/architecture.md`'s "Crate evolution" section (_"No
  package layout beyond the current library is committed"_) — this ADR settles
  both, ahead of any single dimension actually needing a heavier dependency.

## Context

Seven of the eight planned dimensions (`property!`, `fuzz!`, `prove!`,
`conformance!`, `integration!`, `regression!`, `mutate!`) will eventually carry a
dimension-specific dependency: Proptest, a fuzz engine (`libfuzzer-sys`/cargo-fuzz),
Kani, cargo-mutants, or process/IO helpers. `unit!` carries none (ADR-0001 §5, ADR-
0002). ADR-0001 §9 left "package/workspace split" explicitly open and
`docs/architecture.md` committed to no layout beyond the single crate "until an
actual implementation need" arose.

The owner has decided not to wait for that need to materialize dimension-by-
dimension: the split happens now, ahead of any backend implementation, rather than
incrementally as each backend lands.

## Decision

Convert `testes` into a Cargo workspace with nine members:

```text
testes/                     (workspace root, no root package)
├── Cargo.toml               [workspace] members only
├── testes-core/              the current crate's content: types, traits,
│                             evidence model, ports (TestBackend, ArtifactStore,
│                             EvidenceSink, EvidenceEvaluator, GovernancePolicy,
│                             ConformanceCase, SubjectFactory, etc.)
├── testes-unit/               unit! — depends only on testes-core (ADR-0002: no
│                             external engine, no TestBackend impl needed)
├── testes-property/           property! — will depend on testes-core + proptest
├── testes-fuzz/                fuzz! — will depend on testes-core + libfuzzer-sys/cargo-fuzz tooling
├── testes-model-check/         prove! — will depend on testes-core + Kani integration
├── testes-conformance/         conformance! — depends on testes-core only (generic
│                             runner; concrete cases stay project-owned per
│                             docs/README.md's ownership model)
├── testes-integration/         integration! — depends on testes-core + process/IO helpers
├── testes-regression/          regression! — depends on testes-core (wraps another
│                             dimension's evidence; may depend on that dimension's
│                             crate to re-run/reference its backend)
└── testes-mutation/            mutate! — depends on testes-core + cargo-mutants orchestration
```

Each dimension crate:

- Depends on `testes-core` for all shared types (`TestDeclaration`, `EvidenceRecord`,
  `Dimension`, the relevant `DimensionEvidence` variant, etc.). `testes-core` depends
  on nothing workspace-internal — it's the dependency sink, same role the single
  crate played before this split.
- Owns its own macro (`unit!`, `property!`, etc.) and, where applicable, its own
  `TestBackend` implementation and backend-specific dependency.
- Starts as a real crate with a real `Cargo.toml`, not a stub — per the owner's
  explicit choice of "full 8-way split now" over a stub/re-export-only layout.
  Crates for dimensions with no implementation yet (all except `testes-unit`, which
  still has no macro implemented either — only ADR-0002's design) carry no
  dependency beyond `testes-core` until that dimension's backend work begins; they
  are not required to contain working code yet, only to exist as the place that
  code will go.

`testes-core` keeps `#![forbid(unsafe_code)]` and the `miette`/`thiserror`
dependency from ADR-0001. Dimension crates inherit no obligation to stay
dependency-light beyond their own documented backend dependency (ADR-0001 §1's
core-only constraint applies to `testes-core`, not to the dimension crates, which
exist specifically to carry the dependencies core must not).

## Consequences

- A consumer who only wants `unit!` depends on `testes-core` + `testes-unit` and
  pulls in zero Proptest/fuzz/Kani/mutation tooling — this was the motivating
  benefit named in ADR-0001's "Benefits" list ("lightweight consumers avoid
  unrelated engines and toolchains") and is now achieved by crate boundary rather
  than by deferral.
- `Cargo.lock` becomes workspace-wide; CI (`.github/workflows/ci.yml`) must run
  `--workspace` rather than assuming a single package, and per-crate `publish`
  status needs its own decision later (still gated by the existing "no registry
  publish without owner approval" rule).
- Cross-crate re-exports: nothing in `testes-core` needs to re-export dimension
  crates (wrong direction); whether a facade crate re-exporting all eight for
  single-dependency convenience is wanted is **not** decided here — out of scope,
  raise separately if wanted.
- `docs/architecture.md`'s "Crate evolution" section is now stale (still says "no
  package layout beyond the current library is committed") and needs updating to
  point at this ADR.
- Existing single-crate paths in docs (`src/types.rs`, `src/traits.rs`) move to
  `testes-core/src/types.rs`, `testes-core/src/traits.rs` — every doc that names
  those paths needs updating. `AGENTS.md`'s "Read first" list (`src/types.rs` and
  `src/traits.rs`) is one of them.

## Alternatives considered

1. **Stay single-crate, split incrementally per ADR-0001 §9's original plan.**
   Rejected by explicit owner decision — not waiting for each need to materialize.
2. **Stub-only workspace** (crates exist as thin re-export shells, real dependencies
   added later). Considered and declined in favor of real crates now, since the
   crate boundary itself (not the backend code inside it) is what's being locked in
   today.
3. **Core + single `testes-macros` crate for grammar, backends added incrementally.**
   This was the recommended middle option; declined — owner chose the full split.

## Open follow-on (not settled here)

- Exact per-crate `Cargo.toml` contents beyond `testes-core` and `testes-unit`
  (which dependency, which version) — settled when each backend is implemented.
- Whether a facade/meta crate re-exporting all eight exists.
- Workspace-wide `publish` policy per crate.
- CI matrix shape once dimension crates have divergent toolchain requirements
  (e.g. Kani's own toolchain for `testes-model-check`).
