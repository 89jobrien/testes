# ADR-0001: Layered Dependencies, Native Testing Backends, and Miette Diagnostics

- Status: Accepted
- Date: 2026-10-08
- Project: `testes`
- Supersedes: an earlier narrower draft of this decision scoped only to the
  miette/thiserror conversion of five core error enums (now folded into §1–2 below).
  That conversion is implemented; §3–9 are not yet implemented, only decided.

## Context

`testes` provides a declaration and execution-evidence model for eight testing dimensions:

| Dimension   | Public macro   |
| ----------- | -------------- |
| Unit        | `unit!`        |
| Property    | `property!`    |
| Fuzz        | `fuzz!`        |
| Model check | `prove!`       |
| Conformance | `conformance!` |
| Integration | `integration!` |
| Regression  | `regression!`  |
| Mutation    | `mutate!`      |

The architecture separates four responsibilities:

1. Declaring a verification claim and required scope.
2. Executing that claim through a native backend.
3. Evaluating whether execution evidence supports the claim.
4. Applying governance policy, including applicability and deferrals.

The initial core was designed as a dependency-free library. The project owner subsequently directed the implementation agent to use miette for error handling. This ADR incorporates that choice while preserving typed errors and dependency boundaries. It does not establish the current implementation or validation status.

Dependency selection must support macro development, reproducible testing, actionable diagnostics, and truthful execution evidence without coupling every consumer to every testing engine.

A dependency-heavy facade would impose unnecessary tools and runtimes on lightweight consumers. Reimplementing established engines would increase maintenance and risk losing capabilities such as shrinking, failure persistence, and backend-specific verification behavior.

## Decision

Adopt a **layered** dependency architecture:

- Keep core dependencies minimal and purpose-specific.
- Approve miette as the diagnostic protocol for typed library errors.
- Preserve concrete errors for programmatic handling.
- Isolate terminal rendering and application-level reports from core semantics.
- Use established crates for macro implementation and framework validation.
- Integrate native testing engines through explicit backend boundaries.
- Invoke heavyweight campaign tools through orchestration rather than linking them into core.
- Introduce a versioned evidence serialization boundary independently of core types.
- Add dependencies incrementally when an implemented capability requires them.

This ADR defines dependency boundaries and preferred components. It does not require immediate adoption of every listed crate or prescribe a final workspace layout.

## 1. Core dependency policy

The core owns:

- Stable identities.
- Test declarations and specifications.
- Required and actual scope.
- Execution outcomes.
- Dimension-specific evidence.
- Artifact references.
- Backend and storage ports.
- Evidence-evaluation and governance interfaces.
- Generic conformance interfaces.
- Typed errors and diagnostic metadata.

Miette is an approved core/library diagnostic dependency. `thiserror` may be used to derive standard error implementations when it reduces mechanical code.

The core shall not directly depend on:

- Proptest.
- libFuzzer.
- Kani.
- cargo-mutants.
- Process-execution frameworks.
- Project-specific domain crates.
- Serialization formats.
- Async runtimes.

Core behavior must remain independently testable without installing campaign tools.

Dev-dependencies used to test the core do not automatically become public runtime dependencies.

**Current implementation status**: `miette` and `thiserror` are the crate's only
dependencies (`Cargo.toml`). `IdError`, `DeclarationError`, `EvidenceShapeError`
(`src/types.rs`) and `BackendError`, `CheckFailure` (`src/traits.rs`) derive
`thiserror::Error` + `miette::Diagnostic`. None of Proptest, libFuzzer, Kani,
cargo-mutants, process-execution frameworks, serialization formats, or async runtimes
have been added. This satisfies §1 as currently implemented.

## 2. Typed errors and miette diagnostics

### Library interfaces

Public library interfaces shall retain concrete error types:

```rust
Result<EvidenceRecord, BackendError>
```

They shall not be replaced indiscriminately with application-level report types:

```rust
miette::Result<EvidenceRecord>
```

Concrete errors shall implement `std::error::Error` and may implement or derive `miette::Diagnostic`.

Use diagnostics to provide:

- Stable diagnostic codes.
- Actionable help.
- Relevant source locations and labels.
- Underlying causes.
- Context identifying the declaration, backend, or artifact involved.

Miette supports concrete library diagnostics and recommends typed library errors rather than pervasive use of its application-oriented `Report` type. [docs](https://docs.rs/miette)

### Application boundaries

CLI and orchestration entry points may convert typed errors into `miette::Report` or use `miette::Result` for presentation.

Enable miette's `fancy` rendering feature only in the top-level application, not in the shared library's default dependency configuration. Miette recommends this separation to avoid imposing rendering dependencies on library consumers. [docs](https://docs.rs/miette)

| Layer                     | Error-handling policy                                   |
| ------------------------- | ------------------------------------------------------- |
| Core/library              | Concrete typed errors with diagnostic metadata          |
| Backend adapters          | Typed failure categories and preserved causes           |
| CLI/orchestrator boundary | Application-level reports permitted                     |
| Terminal renderer         | Top-level `fancy` feature                               |
| Evidence and policy       | Structured results, independent of rendered diagnostics |

**Current implementation status**: all five converted error enums remain concrete,
typed, derive `Eq`/`PartialEq` where feasible, and are returned directly from library
functions (`TestDeclaration::validate() -> Result<(), DeclarationError>`, etc.) — no
function in this crate returns `miette::Report` or `miette::Result`. `miette`'s
`fancy` feature is not enabled in `Cargo.toml`; no CLI/orchestrator boundary exists
yet to enable it at.

### Semantic boundaries

Diagnostics must preserve these distinctions:

| Condition                                | Representation                  |
| ---------------------------------------- | ------------------------------- |
| Assertion or contract violation          | Execution outcome               |
| Failure preventing trustworthy execution | Backend or infrastructure error |
| Evidence insufficient to support a claim | Evidence-evaluation result      |
| Applicability or approved deferral       | Governance decision             |

A diagnostic report explains a result; it does not determine the result's semantics.

Do not:

- Parse rendered terminal output to reconstruct evidence.
- Convert every failing assertion into an infrastructure error.
- Convert a deferral into verified success.
- Replace structured outcome fields with an error string.
- Flatten actionable backend causes into an undifferentiated message.

### Disclosure and redaction

Source excerpts, labels, causes, and help text may contain sensitive inputs or configuration.

Diagnostic construction and rendering must respect artifact-disclosure and redaction policy. Missing or restricted source context must remain explicit rather than being fabricated.

Rendered diagnostics are supplementary artifacts, not the authoritative evidence schema.

## 3. Macro-authoring dependencies

Use the following toolchain when procedural expansion is justified:

| Crate              | Responsibility                                                |
| ------------------ | ------------------------------------------------------------- |
| `syn`              | Parse declarations and Rust items                             |
| `quote`            | Generate native Rust harnesses                                |
| `proc-macro2`      | Represent tokens in testable expansion logic                  |
| `proc-macro-crate` | Resolve consumer dependencies, including renamed dependencies |

Declarative macros remain acceptable where `macro_rules!` can express the reviewed grammar with clear diagnostics.

The eight macros need not share one expansion mechanism.

Macro implementations must:

- Preserve documented input evaluation and ownership semantics.
- Generate a native harness or execution plan.
- Keep strategies, proof assumptions, bounds, and fuzz oracles visible.
- Support documented dependency naming and facade behavior.
- Produce actionable compiler diagnostics for invalid declarations.
- Never execute campaigns or mutate source during expansion.

Compile-time macro errors remain compiler diagnostics. Miette's runtime reporting must not replace proper expansion errors.

`darling` may be introduced if an attribute-based API develops enough configuration complexity to justify declarative attribute parsing.

**Current implementation status**: not yet started. No macros are implemented; no
macro-authoring dependency has been added.

## 4. Framework-validation dependencies

Adopt the following components incrementally:

| Crate        | Intended use                                                                 |
| ------------ | ---------------------------------------------------------------------------- |
| `proptest`   | Generate declarations, evidence, and outcome combinations to test invariants |
| `trybuild`   | Verify downstream compilation and invalid-declaration diagnostics            |
| `tempfile`   | Isolate filesystem fixtures, artifact-store tests, and mutation workspaces   |
| `insta`      | Review normalized diagnostics and evidence-format snapshots                  |
| `assert_cmd` | Test a CLI or orchestration executable                                       |

These are framework-testing dependencies unless a public capability requires them at runtime.

Snapshot tests supplement semantic assertions. Matching a snapshot does not establish claim correctness.

Temporary-resource tests must verify cleanup explicitly when cleanup success is part of the obligation.

Initial tests should cover:

- Declaration/specification dimension consistency.
- Evidence/declaration identity consistency.
- Required obligation completeness.
- Revision and artifact mismatches.
- Required-versus-actual scope.
- Deferral handling.
- Mutation outcome accounting.
- Unavailable-versus-zero measurements.
- Regression execution identity.
- Typed error categories and diagnostic codes.
- Redaction of diagnostic source context.

Diagnostic snapshots should normalize incidental rendering differences. Tests should assert stable semantic fields separately from presentation.

**Current implementation status**: baseline inline `#[cfg(test)]` coverage exists for
dimension/declaration consistency, duplicate-obligation rejection, regression-nesting
rejection, evidence-shape mismatch, and mutation detection-rate accounting (see
`src/types.rs`, `src/traits.rs`). None of `proptest`, `trybuild`, `tempfile`, `insta`,
or `assert_cmd` has been added yet — none of this crate's current tests need them.

## 5. Native dimension backends

Use the following execution model:

| Dimension   | Preferred backend                                | Dependency boundary                      |
| ----------- | ------------------------------------------------ | ---------------------------------------- |
| Unit        | Native Rust tests                                | No external engine required              |
| Property    | Proptest                                         | Property adapter                         |
| Fuzz        | libFuzzer through `libfuzzer-sys` and cargo-fuzz | Dedicated fuzz package and orchestration |
| Model check | Kani                                             | Explicit verifier/toolchain integration  |
| Conformance | Trait-owned suites using `testes` interfaces     | Generic runner plus project-owned cases  |
| Integration | Native tests with explicit process/I/O helpers   | Integration adapter or dev-dependencies  |
| Regression  | Underlying dimension's backend                   | Evidence wrapper or classification       |
| Mutation    | cargo-mutants initially                          | External campaign adapter                |

Adapters must preserve native capabilities rather than replacing them with weaker generic abstractions.

These include:

- Property shrinking and persisted counterexamples.
- Fuzz corpus handling and reproduction.
- Proof assumptions, bounds, and obligation results.
- Conformance capability and lifecycle reporting.
- Mutation baselines and per-mutation outcomes.

Regression shall reuse underlying execution evidence rather than cause a second execution solely to obtain a regression label.

**Current implementation status**: no backend for any dimension is implemented yet.

## 6. Mutation-testing boundary

`mutate!` declares a campaign scope and selected checks. Orchestration executes the campaign.

Mutation campaigns must:

- Establish a passing unmutated baseline.
- Execute in an isolated workspace.
- Record exact verification commands.
- Preserve caught, missed, unviable, timeout, and excluded outcomes.
- Avoid attributing unrelated infrastructure failures to defect detection.
- Report actual backend mutation scope.

The project shall not assume source mutation covers expanded declarative macro code.

Seeded faulty implementations are a separate planned mechanism for assessing contract-suite sensitivity. They must not be represented as a capability supplied by cargo-mutants.

Miette diagnostics may explain campaign setup failures or summarize findings, but the mutation evidence remains structured and independently evaluable.

**Current implementation status**: `MutationEvidence`/`MutationOutcome` types and
`detection_rate()` exist and are tested (`src/types.rs`); no campaign orchestration or
`cargo-mutants` integration exists yet.

## 7. Evidence serialization

Use `serde` and `serde_json` for the initial JSON evidence adapter.

Prefer explicit wire types and validated conversions:

```text
Core evidence
    ↕ validated conversion
Versioned wire representation
    ↕ serialization
JSON artifact or stream
```

Do not automatically serialize every core type merely because serialization derives are available.

Before stabilizing the format, define:

- Schema-version behavior.
- Stable dimension and outcome names.
- Timestamp and duration representation.
- Unknown-field handling.
- Validated identifier decoding.
- Diagnostic identifiers and structured context.
- Redacted configuration representation.
- Replay limitations caused by omitted or restricted material.

Deserialization is not evidence validation.

Rendered miette reports shall not become the wire representation of typed errors or execution outcomes.

JSON serialization shall not be assumed to provide canonical bytes. Digest semantics must identify retained bytes or a separately defined canonical representation.

**Current implementation status**: no serialization dependency or wire format exists
yet. `EvidenceRecord.schema_version: u32` exists as a field but no serialization
boundary consumes it.

## 8. External tooling

Use orchestration and CI to invoke:

| Tool          | Purpose                      |
| ------------- | ---------------------------- |
| cargo-nextest | Ordinary test execution      |
| cargo-fuzz    | Fuzz campaigns               |
| Kani tooling  | Proof harness execution      |
| cargo-mutants | Mutation campaigns           |
| cargo-expand  | Inspection of generated Rust |

Tool installation and versions must be explicit.

Missing required tools produce not-executed or inconclusive evidence—not successful verification.

Lightweight consumers must not inherit heavyweight campaign tooling unnecessarily.

Doctests require a separate validation lane where the selected runner does not execute them.

**Current implementation status**: `.github/workflows/ci.yml` currently runs `cargo
fmt --check`, `cargo check --all-targets`, `cargo test --all-targets`, `cargo test
--doc`, and `cargo clippy --all-targets -- -D warnings` only — plain `cargo test`,
not `cargo-nextest`. None of cargo-fuzz, Kani tooling, cargo-mutants, or cargo-expand
are wired into CI yet, consistent with no backend for those dimensions existing yet.

## 9. Deferred dependencies

Do not introduce the following without a demonstrated implementation need:

| Dependency or category               | Required justification                                            |
| ------------------------------------ | ----------------------------------------------------------------- |
| `tokio` or `async-trait`             | Reviewed async execution and conformance strategy                 |
| `inventory` or `linkme`              | Need for decentralized discovery rather than an explicit registry |
| `rstest`                             | Fixture or parameterization needs not met by the declaration API  |
| `schemars`                           | Evidence wire schema stable enough to justify generated schemas   |
| Database                             | Concrete persistence and query requirements                       |
| General workflow engine              | Orchestration needs beyond explicit backend adapters              |
| Additional error-reporting framework | A specific unmet requirement beyond typed errors and miette       |

Deferral is not rejection. Dependencies may be adopted through focused decisions when their value is established.

## Implementation sequence

### Increment 1: Establish the current baseline

- Inspect the agent's current miette implementation and dependency configuration.
- Compile, format, lint, and test the current code.
- Preserve concrete error categories.
- Verify rendering features are isolated from library defaults.
- Add unit tests for validation helpers and diagnostic semantics.
- Add framework dev-dependencies as needed.

**Status: done.** `cargo fmt --check`, `cargo check --all-targets`, `cargo test
--all-targets`, and `cargo clippy --all-targets -- -D warnings` all pass clean as of
this ADR's acceptance. Five error enums carry concrete, typed `thiserror`/`miette`
diagnostics; `fancy` is not enabled; no framework dev-dependency has been needed yet.

### Increment 2: Implement the first declaration macro

- Specify grammar before implementation.
- Prefer `macro_rules!` where sufficient.
- Introduce the procedural toolchain only if required.
- Add downstream and compile-fail fixtures.
- Verify registration cannot fabricate passing evidence.

**Status: not started.**

### Increment 3: Produce a complete evidence path

- Connect native execution to recorded evidence.
- Implement identity, scope, obligation, and artifact checks.
- Separate evidence evaluation from governance.
- Add the property backend without discarding native replay behavior.
- Introduce versioned JSON evidence conversion.

**Status: not started.**

### Increment 4: Expand backend coverage

- Add conformance and integration execution.
- Integrate fuzz and proof tooling explicitly.
- Add isolated mutation orchestration.
- Validate against real downstream consumers.

**Status: not started.**

## Consequences

### Benefits

- Lightweight consumers avoid unrelated engines and toolchains.
- Core semantics remain independently testable.
- Errors remain matchable and actionable.
- Diagnostics provide consistent codes and contextual help.
- Backend-specific strengths remain available.
- Evidence formats evolve independently of terminal rendering.
- Development proceeds through small, verifiable increments.

### Costs

- Miette becomes an intentional library dependency.
- Diagnostic metadata requires maintenance and compatibility decisions.
- Backend adapters require explicit mapping and tests.
- Multiple tooling environments must be managed.
- Wire conversion adds implementation work.
- Dependency renaming and downstream expansion require fixtures.

### Risks and mitigations

| Risk                                               | Mitigation                                                  |
| -------------------------------------------------- | ----------------------------------------------------------- |
| Rendering dependencies leak into consumers         | Enable `fancy` only at the application boundary             |
| Typed failures become generic reports              | Retain concrete library error types                         |
| Diagnostics leak sensitive input                   | Apply disclosure and redaction policy before presentation   |
| Wrappers suppress native behavior                  | Test backend semantics and replay explicitly                |
| Optional features omit required checks silently    | Record missing execution and reject insufficient evidence   |
| Serialized evidence is treated as validated        | Require validated conversion and evaluation                 |
| Mutation results overstate generated-code coverage | Record actual assessed scope                                |
| Rendering changes break tests unnecessarily        | Assert semantic fields separately from normalized snapshots |

## Alternatives considered

### Strict std-only core with application-only diagnostics

Not selected because miette is an intentional project choice and diagnostic metadata is useful on typed library errors.

Rendering remains application-owned, so this does not require embedding terminal presentation throughout core logic.

### `miette::Report` for every public error

Rejected because it would weaken programmatic classification and encourage conflation of domain outcomes with operational failures.

### One dependency-heavy facade

Rejected because lightweight consumers should not inherit unrelated engines, runtimes, and campaign tools.

### Reimplement all testing engines

Rejected because it would create substantial maintenance work and risk losing mature generation, shrinking, replay, and verification behavior.

### One generic runner for every dimension

Rejected because native harnesses and execution constraints differ. Uniform declarations standardize identity and evidence, not backend semantics.

### Keep all testing infrastructure project-local

Not selected as the long-term direction because repeated execution and evidence mechanics justify sharing. Domain contracts and harness semantics nevertheless remain project-owned.

## Acceptance criteria

This decision is implemented when:

- Core dependencies are minimal and explicitly justified.
- Library errors remain concrete and preserve semantic categories.
- Miette diagnostics provide appropriate codes and context.
- Terminal rendering features remain application-owned.
- Diagnostics respect redaction policy.
- Backend dependencies remain isolated.
- Macro expansion performs no campaign execution.
- Downstream fixtures validate dependency naming.
- Property replay and failure persistence remain functional.
- Evidence decoding is followed by validation.
- Required missing backends cannot produce passing evidence.
- Mutation evidence preserves baseline validity and outcome categories.
- Documentation distinguishes available capabilities from planned integrations.
- Validation results are recorded rather than inferred.

As of acceptance, only the first five criteria are satisfied (§1–2, Increment 1). The
remainder apply to work not yet started (§3–9, Increments 2–4) and gate those future
increments rather than this ADR's acceptance.

## Open decisions

This ADR does not settle:

- Macro grammar.
- Diagnostic-code naming and compatibility policy.
- Async/object-safe execution strategy.
- MSRV and supported platform matrix.
- Artifact digest algorithm.
- Proof-critical obligation policy.
- Persistence storage.
- Package/workspace split.
- License and registry publication.
- Backend version-pinning and update policy.
