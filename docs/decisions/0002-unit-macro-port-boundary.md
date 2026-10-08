# ADR-0002: `unit!`'s backend boundary (SOLID/hexagonal application)

- Status: Accepted
- Date: 2026-10-08
- Project: `testes`
- Scope: design-only. No macro code is implemented by this ADR; it decides where
  `unit!`'s generated harness sits relative to the existing `TestBackend` port before
  implementation starts (per `docs/handoff.md`'s requirement to specify grammar before
  implementing, and ADR-0001 §3's "specify grammar before implementation").

## Context

`src/traits.rs` already defines a generic port for dimension execution:

```rust
pub trait TestBackend {
    fn identity(&self) -> BackendIdentity;
    fn supports(&self, declaration: &TestDeclaration) -> bool;
    fn execute(&self, declaration: &TestDeclaration, request: &RunRequest,
        context: &mut dyn ExecutionContext) -> Result<EvidenceRecord, BackendError>;
}
```

This is the right port for dimensions with more than one plausible execution
strategy — e.g. `property!` could run against `proptest` or a different generator;
`fuzz!` against `cargo-fuzz`/libFuzzer or another engine (ADR-0001 §5 lists these as
"adapter" boundaries precisely because more than one backend is plausible).

`unit!` is different. ADR-0001 §5 states: _"Unit | Native Rust tests | No external
engine required."_ There is exactly one way a Rust unit test executes — the ordinary
`#[test]` harness `cargo test` already drives. There is no second implementation to
substitute, now or foreseeably.

The writing-solid-rust skill's own "When NOT to use this pattern" section applies
directly: _"Single implementation - if you'll only ever have one implementation,
trait might be premature."_ Forcing `unit!`'s generated code through `dyn TestBackend`
would be exactly that — a trait introduced for a single implementation, satisfying no
real Open/Closed need because there is nothing to open for extension yet.

## Decision

`unit!`'s generated harness does **not** implement `TestBackend`. Instead:

1. The macro expands each example into an ordinary `#[test] fn`. That function _is_
   the composition root for its own evidence: it constructs a `TestDeclaration`,
   runs the example body, and builds an `EvidenceRecord`/`UnitEvidence` directly —
   no `dyn TestBackend`, `RunRequest`, or `ExecutionContext` indirection.
2. `src/unit.rs` (the macro's implementation module, not yet written) owns exactly
   one responsibility (SRP): turning one `unit!` invocation into one `#[test] fn`
   plus its evidence-construction logic. It does not know about `ArtifactStore`,
   `EvidenceSink`, or persistence — those are separate ports this module has no
   reason to depend on (ISP: don't implement against capabilities the unit case
   doesn't need).
3. The domain boundary that _does_ apply is the existing one: the generated code
   depends only on core domain types (`TestDeclaration`, `TestSpecification::Unit`,
   `EvidenceRecord`, `DimensionEvidence::Unit`, `UnitEvidence`, `ExecutionOutcome`) —
   zero infrastructure dependency, consistent with DIP and ADR-0001 §1's core
   dependency policy. The macro is an adapter _of_ the domain, not a domain type
   itself — it's just an adapter with no alternative implementation to abstract
   over, so it isn't expressed as a trait.
4. `TestBackend` stays reserved for dimensions that genuinely have swappable
   execution strategies (`property!`/`fuzz!`/`prove!`/`mutate!` per ADR-0001 §5).
   If a second native-unit execution strategy ever materializes (e.g. an async test
   runner adapter, or a non-`cargo-test` harness), introduce a `TestBackend` impl
   for it _then_ — OCP is "extend via new implementation when a second
   implementation shows up," not "pre-build the extension point speculatively."

## Consequences

- `unit!`'s grammar and generated code stay small: no generic parameter, no trait
  object, no `Box<dyn ...>` — matching ADR-0001's "Prefer `macro_rules!` where
  sufficient" (§3) and the earlier-approved implementation plan's choice of a
  declarative macro over a procedural one.
- `TestBackend`, `ExecutionContext`, `ArtifactStore`, and `EvidenceSink` remain
  unused by `unit!` — this is intentional, not an oversight. A future reader should
  not "fix" `unit!` by routing it through those ports without a concrete second
  implementation driving that need.
- If `unit!` later needs artifact retention (e.g. capturing a panic backtrace as a
  retained artifact), that's a `CaseContext`/`ArtifactStore`-shaped need to evaluate
  on its own merits then — not a reason to adopt `TestBackend` now.

## Alternatives considered

1. **Implement `TestBackend` for a `NativeUnitBackend` struct.** Rejected: no second
   implementation exists or is anticipated; this is the "trait that mirrors a single
   implementation" anti-pattern the writing-solid-rust skill warns against, and adds
   `dyn`-dispatch and `RunRequest`/`ExecutionContext` plumbing the unit case doesn't
   use (ISP violation — `ExecutionContext::retain_artifact` etc. would be dead
   surface for the common case).
2. **Generic `unit!` over a `U: TestBackend` type parameter.** Rejected for the same
   reason, plus it reintroduces "too many generic parameters" friction for a macro
   whose whole point is to stay simple grammar.
3. **Route everything through `TestBackend` uniformly across all eight dimensions
   for consistency.** Rejected: uniformity that isn't load-bearing is premature
   abstraction, and ADR-0001 §5 already treats dimensions non-uniformly (native vs.
   adapter-wrapped) on purpose.

## Open follow-on

This ADR only settles the port boundary. `unit!`'s concrete grammar, expansion, and
tests are still specified in the earlier-approved implementation plan (baseline
tests done; grammar/macro/tests phases not yet implemented) and remain out of scope
here.
