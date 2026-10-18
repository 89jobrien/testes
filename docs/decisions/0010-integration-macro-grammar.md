# ADR-0010: `integration!` macro grammar and expansion

- Status: Accepted
- Date: 2026-10-17
- Project: `testes`
- Scope: decides `integration!`'s grammar, expansion target, and evidence construction, per
  `docs/macro-plan.md` acceptance item 1. No grammar ADR existed before this one — only
  `docs/dimensions/integration.md` and one row of `docs/macro-plan.md`. A first-pass `integration!`
  already exists in `testes-integration/src/lib.rs`; this ADR is the grammar of record.

## Context

`integration!` must cross a real boundary, which is what makes its grammar the most constrained of
the four missing ADRs.

ADR-0003 assigns it the only dimension dependency beyond `testes-core` and the plain engines:

> "`testes-integration/ integration!` - depends on testes-core + **process/IO helpers**"

ADR-0001 §5 names the boundary: "Integration | Native tests with explicit process/I/O helpers |
Integration adapter or dev-dependencies." And ADR-0001 §1 bars `testes-core` from depending on
"process-execution frameworks" — so those helpers live in `testes-integration`, not core.

### The types already solve most of the hard parts

This ADR is short on invention because `testes-core/src/types.rs` already provides the three
things a naive design would have to invent:

```rust
// types.rs:339-349 — the three-way classification is ALREADY a core type
pub struct ComponentIdentity {
    pub name: String,
    pub version: Option<String>,
    pub boundary: BoundaryKind,      // Real | Fake | Simulated
}
pub enum BoundaryKind { Real, Fake, Simulated }

// types.rs:351-356 — a child-process command with its OWN environment
pub struct CommandSpec {
    pub program: String,
    pub arguments: Vec<String>,
    pub working_directory: Option<String>,
    pub environment: BTreeMap<String, ConfigurationValue>,
}

// types.rs:453-459 — cleanup is a first-class ExecutionOutcome, not a wrapper
pub struct IntegrationEvidence {
    pub seam_id: String,
    pub components: Vec<ComponentIdentity>,
    pub fixtures: Vec<ArtifactId>,
    pub observations: Vec<ArtifactId>,
    pub cleanup: ExecutionOutcome,
}
```

Two consequences reshape the grammar:

1. **`BoundaryKind` has three variants, not two.** `Fake` (a working stand-in under test control)
   and `Simulated` (a model, not executed) are distinct in core, so the grammar must preserve the
   distinction. `docs/dimensions/conformance.md` states the same rule for suites; here it is
   already typed.
2. **`CommandSpec.environment` implements the child-process rule structurally.** A `CommandSpec`
   carries its own `environment` map, so an integration test can supply configuration to a child
   process without touching the test process's own environment. This is not a convention this ADR
   imposes — it is the type's shape. `docs/dimensions/integration.md`'s "Prefer child-process
   environment configuration to routine mutation of the test process environment" is already
   _expressible_, and `AGENTS.md`'s "do not add parent-process environment mutation helpers" is
   satisfiable by simply not adding one.

### Five requirements that constrain the surface

| Requirement                                                                                          | Source                                           | Grammar consequence                                                            |
| ---------------------------------------------------------------------------------------------------- | ------------------------------------------------ | ------------------------------------------------------------------------------ |
| The seam must be **named**                                                                           | "Verify observable behavior across a named seam" | `seam:` required; `IntegrationEvidence.seam_id`                                |
| Each boundary classified                                                                             | "Mark each boundary as real, fake, or simulated" | `components:` producing `Vec<ComponentIdentity>`; `BoundaryKind` is exhaustive |
| "Do not claim a real transport was verified when substituted"                                        | Limits                                           | classification is recorded evidence; substitution is surfaced                  |
| "Failed setup, missing capabilities, and execution timeout must not masquerade as seam verification" | Acceptance                                       | setup, capability, and budget each map to a non-`Passed` status                |
| "Prefer child-process environment configuration"                                                     | Limits                                           | `command:` is a `CommandSpec`, whose `environment` is per-command              |

### The dimension is also forbidden from duplicating `unit!`

`docs/dimensions/integration.md`: "Test **wiring** and boundary behavior rather than duplicating
unit assertions." A grammar that accepted arbitrary assertion bodies would invite exactly that.

## Decision

### Grammar

```rust
integration! {
    id: "cli-to-application-request",
    seam: "cli -> application operation",
    claim: "a CLI request reaches the application operation and yields the expected structured response",

    // Every component is classified. The classification IS the evidence.
    components: [
        ComponentIdentity { name: "cliruntime::CliRuntime", version: None, boundary: BoundaryKind::Real },
        ComponentIdentity { name: "appdb::InMemoryObservationStore", version: None, boundary: BoundaryKind::Fake },
    ],

    // The boundary under test, as a child-process command with its own environment.
    // Nothing here mutates the test process's environment.
    command: CommandSpec {
        program: "target/debug/testes-integration-cli",
        arguments: vec!["submit".into(), "--id".into(), "OBS-1".into()],
        working_directory: None,
        environment: fixtures::cli_env(),
    },

    // Observable assertions across the seam only.
    observe: {
        assert_eq!(response.status, 200);
        assert!(response.body.contains("OBS-1"));
    },
}
```

Six fields:

- `id` — `TestId::new` contract
- `seam` — **required** name; recorded in `IntegrationEvidence.seam_id` and
  `TestSpecification::Integration.seam_id`
- `claim` — the behavior asserted across that seam
- `components` — expressions evaluating to `Vec<ComponentIdentity>`. Classification is exhaustive
  by construction because `BoundaryKind` has exactly three variants and every component names one.
- `command` — a `CommandSpec`. Its `environment` is the configuration applied **at the boundary**;
  the test process's environment is untouched.
- `observe` — the observable assertions. Deliberately named `observe` rather than `body` so the
  grammar does not invite general-purpose assertions.

Absent by design: `timeout:` (see below), `skip:` (forbidden), `env_mut` (forbidden by `AGENTS.md`),
and any assertion about a component _behind_ the seam.

### Why there is no `timeout:` field

`docs/dimensions/integration.md` requires that "execution timeout must not masquerade as seam
verification." A `timeout:` field would let the caller pick a bound and then have budget exhaustion
become the recorded outcome — the exact masquerade the spec forbids. So the expansion owns the
bound and maps exhaustion to `Inconclusive`.

Note `MutationOutcome::Timeout { limit: Duration }` exists in core (`types.rs:485-487`) — but for
mutation, where the _backend_ reports the limit it used. Integration deliberately inverts that: the
runner owns the bound and reports `Inconclusive` without letting the caller choose it.

### Expansion target

One generated `#[test] fn {id}`.

```rust
#[test]
fn {id}() {
    let command = {command};
    let components: Vec<ComponentIdentity> = {components};

    // Schematic: field *names* are the real `testes-core` fields (`TestDeclaration`,
    // types.rs:109-119); initializers are elided. There is no `scope` or `obligations` field —
    // it carries `required_scope`.
    let declaration = TestDeclaration {
        id: /* TestId::new(id) */,
        primary_dimension: Dimension::Integration,
        secondary_dimensions: vec![],
        subject: /* SubjectId::new("{seam}") */,
        claim: /* the authored claim */,
        required_scope: Scope {
            /* input_domain, assumptions, bounds, capabilities, resources */
        },
        disposition: ObligationDisposition::Required,
        specification: TestSpecification::Integration {
            seam_id: "{seam}".into(),
            components: components.clone(),
        },
    };
    declaration.validate().expect("TestDeclaration must be valid");

    let mut context = IntegrationContext::new();

    // Setup is separately represented so it cannot be read as verification.
    let prepared = context.prepare(&components, &command);
    let outcome = match prepared {
        // Setup failure means the seam was never exercised: no evidence either way.
        Err(failure) => ExecutionOutcome {
            status: ExecutionStatus::Inconclusive,
            reason: /* OutcomeReason::… — not a String (types.rs:247) */,
            diagnostic: Some(context.sanitize(&failure.to_string())),
            /* ExecutionOutcome derives no Default (types.rs:244) */
        },
        Ok(boundary) => match boundary.invoke(default_budget()) {
            // Budget exhaustion is inconclusive, never a pass.
            Err(IntegrationError::BudgetExceeded { .. }) => ExecutionOutcome {
                status: ExecutionStatus::Inconclusive,
                reason: /* OutcomeReason::… — not a String (types.rs:247) */,
                /* ExecutionOutcome derives no Default (types.rs:244) */
            },
            // Required capability missing: not verification.
            Err(IntegrationError::CapabilityUnavailable { name, reason }) => ExecutionOutcome {
                status: ExecutionStatus::Inconclusive,
                reason: /* OutcomeReason::… — not a String (types.rs:247) */,
                diagnostic: Some(context.sanitize(&format!("{name}: {reason}"))),
                /* ExecutionOutcome derives no Default (types.rs:244) */
            },
            Err(IntegrationError::Transport { diagnostic }) => ExecutionOutcome {
                status: ExecutionStatus::Failed,
                diagnostic: Some(context.sanitize(&diagnostic)),
                /* ExecutionOutcome derives no Default (types.rs:244) */
            },
            Ok(response) => {
                // Assertions run against OBSERVABLE behavior only.
                match context.check_observations(&response, |r| {
                    let response = r;
                    {observe}
                }) {
                    Ok(()) => ExecutionOutcome { status: ExecutionStatus::Passed,
                                                /* ExecutionOutcome derives no Default (types.rs:244) */ },
                    Err(failure) => ExecutionOutcome {
                        status: ExecutionStatus::Failed,
                        diagnostic: Some(context.sanitize(&failure.to_string())),
                        /* ExecutionOutcome derives no Default (types.rs:244) */
                    },
                }
            }
        },
    };

    // Cleanup is a first-class ExecutionOutcome (types.rs:458) and runs on every path.
    let cleanup = context.teardown(&components);

    // Schematic: `EvidenceRecord` (types.rs:553-562) has no `test_id`, `backend_used`, or
    // `timestamp` field. The dimension payload lives in `dimension_evidence`.
    let evidence = EvidenceRecord {
        schema_version: /* ADR-0001 §7 version */,
        declaration,
        provenance: /* Provenance */,
        execution: /* ExecutionMetadata, carrying the BackendIdentity */,
        outcome: /* ExecutionOutcome */,
        artifacts: /* retained artifacts */,
        dimension_evidence: DimensionEvidence::Integration(IntegrationEvidence {
            seam_id: "{seam}".into(),
            components,
            fixtures: context.fixture_artifact_ids(),
            observations: context.observation_artifact_ids(),
            cleanup,
        }),
    };

    // Substitution is surfaced, not hidden. A fake transport does not support a real-transport
    // claim, and the record must say which it was.
    let substituted: Vec<&str> = substituted_components(&evidence).collect();
    if !substituted.is_empty() {
        // Recorded on the case note so the claim is read at the classification's strength.
        tracing::warn!(
            "integration {id} substituted {substituted:?}; \
             the claim applies to these components, not to real transports"
        );
    }

    thread_local! {
        static LAST_INTEGRATION_EVIDENCE: RefCell<Option<EvidenceRecord>> = RefCell::new(None);
    }
    LAST_INTEGRATION_EVIDENCE.with(|e| *e.borrow_mut() = Some(evidence.clone()));

    // A required cleanup failure fails the test — recording it alone would let a leak pass.
    // `cleanup` is a field on `IntegrationEvidence`, not a helper on `EvidenceRecord`.
    let DimensionEvidence::Integration(integration) = &evidence.dimension_evidence else {
        unreachable!("declaration and evidence dimensions are built together")
    };
    if integration.cleanup.status == ExecutionStatus::Failed {
        panic!("integration {id} required cleanup failed");
    }
    if outcome.status != ExecutionStatus::Passed {
        panic!("integration {id} did not verify the seam: {outcome:?}");
    }
}
```

Five decisions to state explicitly:

1. **Setup failure → `Inconclusive`, not `Failed`.** A setup failure means the seam was never
   exercised, so there is no evidence either way. This matches `docs/evidence.md`'s separation of
   "failures preventing trustworthy execution" from assertion violations.
2. **Budget exhaustion → `Inconclusive`.** Never `Passed`.
3. **Cleanup runs on every path** and lands in `IntegrationEvidence.cleanup: ExecutionOutcome` —
   a plain `ExecutionOutcome`, so no new wrapper type is invented. A required cleanup failure both
   records `Failed` **and** fails the test: recording alone would let a resource leak pass, and
   failing alone would lose the diagnostic.
4. **Diagnostics are sanitized at construction.** ADR-0001 §2's disclosure and redaction policy
   applies here with the most force, because integration is the layer where real request bodies and
   configuration flow.
5. **Substitution is recorded and warned.** `ComponentIdentity.boundary` is in the evidence
   already; the expansion surfaces which components were not `Real`, making "Do not claim a real
   transport was verified when substituted" enforceable rather than advisory.

### Environment handling is structural, not conventional

`CommandSpec.environment: BTreeMap<String, ConfigurationValue>` is per-command. The test process's
own environment is never written.

This satisfies three documents at once:

- `docs/dimensions/integration.md`: "Prefer child-process environment configuration to routine
  mutation of the test process environment."
- `AGENTS.md`: "do not add parent-process environment mutation helpers."
- `AGENTS.md`: "Keep `#![forbid(unsafe_code)]`" — `std::env::set_var` would need `unsafe` in newer
  editions.

So there is **no helper to add and nothing to forbid**: the grammar simply has no position for it.
`working_directory: Option<String>` likewise scopes process state without touching ambient state.

### Backend identity

`"integration-adapter"` / `"1.0"` — distinct from ADR-0004's `"native-rust-test"` and ADR-0008's
`"conformance-runner"`, because integration invokes a real process boundary.

### ADR-0002 relationship

A composition root, like `unit!` and `conformance!`: the generated `#[test]` constructs the
declaration, assembles the boundary, invokes the command, and builds the evidence directly.

`testes-integration` holds `IntegrationContext`, `boundary::prepare`, `default_budget`,
`IntegrationError`, and the sanitizer — the "process/IO helpers" ADR-0003 anticipated. Core stays
free of them, per ADR-0001 §1's ban on process-execution frameworks in the dependency sink.

## Consequences

- **The classification is exhaustive by construction.** `BoundaryKind` has exactly three variants
  and every `ComponentIdentity` names one, so "unclassified" is unrepresentable.
- **A substituted boundary is visible in evidence**, which makes the "don't claim a real transport"
  rule enforceable — but also means a passing integration test does **not** imply a verified real
  transport. Reviewers must read `ComponentIdentity.boundary`.
- **`IntegrationContext` is new work in `testes-integration`**, and its `fixtures` and
  `observations` fields produce `ArtifactId`s — so the context needs an `ArtifactStore` to retain
  them, and `ArtifactStore` is an unimplemented port (`traits.rs:37`). Until it exists, retained
  inputs and observations have nowhere to go.
- **`sanitize` is a new obligation on `testes-integration`.** ADR-0001 §2's redaction policy
  requires it, and ADR-0001 §2's checklist item "Redaction of diagnostic source context"
  (`0001-layered-dependencies-and-native-testing-backends.md:232`) records it as a test to add.
- **`observe` is deliberately narrower than `body`.** Naming it `observe` is a naming-level
  constraint on how the field reads; it does not technically prevent a unit-style assertion inside
  it. The spec rule against duplicating unit assertions is thus encouraged, not enforced.
- **The `timeout` omission is restrictive by design.** Users wanting a different budget change
  `default_budget()` in `testes-integration` or their own configuration — slightly more indirect
  than a per-call field, and auditable.

## Alternatives considered

1. **Add a `timeout:` field.** Rejected: it lets the caller choose a bound and then have budget
   exhaustion become the recorded outcome — the exact masquerade the spec forbids.
2. **Add a `skip:` field for known-unavailable boundaries.** Rejected: the same opt-out problem
   ADR-0008 rejects. A skipped integration test is indistinguishable from one that never existed,
   and `AGENTS.md` requires that unavailable capability never become passing evidence.
3. **Add an `env:` closure for configuration.** Rejected: `CommandSpec.environment` already does
   this, per-command and scoped. A closure would be a second, weaker mechanism.
4. **A `real:`/`fake:` pair instead of `Vec<ComponentIdentity>`.** Rejected: `BoundaryKind` has
   three variants in core. A two-field grammar would silently drop `Simulated`.
5. **Accept arbitrary assertions in `body:` with no seam discipline.** Rejected: invites
   duplicating `unit!`, which the spec forbids.
6. **Route integration through `TestBackend`.** Rejected per ADR-0002 §4: there is one expansion
   shape. The substitutability is in _which components are wired_, not in how the harness executes.
7. **A `set_env!` helper using `std::env::set_var`.** Rejected outright: violates `AGENTS.md` twice
   over (parent-process mutation helpers; `#![forbid(unsafe_code)]`) and is not parallel-safe.

## Validation (to add when implemented)

Using the `thread_local!` slot rather than the pass/fail bit:

1. **Happy path, all `Real`** → `Passed`, `cleanup.status == Passed`, no substitution warning.
2. **`Fake` boundary** → `Passed`, the substituted component named in the warning, and
   `IntegrationEvidence.components[i].boundary == Fake`. The record does not claim a real transport.
3. **Setup failure** → `Inconclusive`, reason "boundary setup failed"; `command` never invoked.
4. **Assertion failure in `observe`** → `Failed`, sanitized `diagnostic`; generated `#[test]` fails.
5. **Budget exhaustion** → `Inconclusive`, reason "execution budget exhausted" — **not** `Passed`.
6. **Capability unavailable** → `Inconclusive`, reason "required capability unavailable".
7. **Cleanup failure after a passing body** → `IntegrationEvidence.cleanup.status == Failed` **and**
   the generated `#[test]` fails.
8. **Cleanup runs after a failed body** — child process released; the recorded outcome is the
   body's failure.
9. **Sanitization** — a body failing with a diagnostic containing a sensitive-looking value has
   that value absent from `IntegrationEvidence`, from `cleanup`'s diagnostic, and from the emitted
   warning.
10. **Exhaustive classification** — a `ComponentIdentity` is required; there is no syntax for an
    unclassified component.
11. **Environment isolation** — two integration tests with different `CommandSpec.environment` maps
    running concurrently observe only their own configuration, and the test process's environment
    is unchanged by either (asserted by comparing a captured copy).
12. **Malformed declaration** — `validate()` rejects before `command` is invoked.

## Open questions for the owner

1. **`IntegrationContext` has nowhere to retain artifacts.** `fixtures: Vec<ArtifactId>` and
   `observations: Vec<ArtifactId>` require an `ArtifactStore`, which has no implementation. Until it
   does, items 7–8 can record cleanup but retained inputs cannot be retained at all. Should
   `testes-integration` ship a minimal artifact store, or is this blocked on the store?
2. **`seam_id` is a `String` in both `TestSpecification::Integration` and `IntegrationEvidence`.**
   Should it become a typed `SeamId` in core so `validate_shape()` can check that the declared seam
   matches the recorded one by identity rather than by string equality? The failure mode today is a
   silent mismatch.
3. **`sanitize`'s redaction mechanism.** The policy is specified (ADR-0001 §2) but the mechanism —
   allow-list, deny-list, or caller-supplied redaction hook — is not. `testes-integration` must
   implement it.
4. **Where `default_budget()` comes from.** A crate-level constant, an env-driven value, or a
   per-`CommandSpec` field. Note that adding it to `CommandSpec` would let a caller choose the
   bound, which is what ADR's decision 1 rejects — so it must live outside the spec type.
5. **Should `observe` be able to retain values for replay?** `docs/dimensions/integration.md`: "Retain
   interaction inputs where replay requires them." That is what `observations: Vec<ArtifactId>` is
   for, but how an assertion's value gets retained is unspecified — a hook, or automatic capture of
   every observed field.
6. **Is `tracing::warn!` the right surface for the substitution note?** It reaches the log rather
   than the evidence. If the substitution must be part of the record, it belongs in a field rather
   than a log line — but `IntegrationEvidence` has no such field, which may be a gap in the type.

## Implementation notes

- Lives in `testes-integration/src/lib.rs`, with `IntegrationContext`, `boundary`, `IntegrationError`,
  `default_budget`, and `sanitize` as sibling modules re-exported from `lib.rs`.
- `testes-integration` is the only dimension crate that may hold process-execution helpers, per
  ADR-0003. Core stays free of them per ADR-0001 §1.
- The expansion references only core types: `TestDeclaration`, `TestSpecification::Integration`,
  `EvidenceRecord`, `DimensionEvidence::Integration`, `IntegrationEvidence`, `ComponentIdentity`,
  `BoundaryKind`, `CommandSpec`, `ConfigurationValue`, `ExecutionOutcome`, `TestId`, `Scope`,
  `BackendIdentity`. Everything else is `testes-integration` or user code.
- All of the classification, command, and cleanup machinery is already typed in core — this ADR
  requires **no `testes-core` type changes**. That is worth stating explicitly: integration's
  grammar can be designed without touching the dependency sink.
- `docs/macro-plan.md` order item 5 places integration after conformance and property/regression
  replay.
- Per `AGENTS.md`, prefer child-process environment configuration over routine mutation of the test
  process environment — and do not add a helper that does otherwise.

Related: [[integration]], [[integration-macro]], [[integration-evidence]], [[named-seam]],
[[boundary-classification]], [[substituted-boundary-claim]], [[missing-tooling-must-not-pass]],
[[child-process-environment-configuration]], [[cleanup-failure-visibility]],
[[the-grammar-adr-series]], [[adr-0002-unit-macro-port-boundary]], [[testes-integration]],
[[testes-core]], [[artifact-store]].
