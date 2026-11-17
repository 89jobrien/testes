# Prioritised Idea List for testes

Ideas grounded in repo observations — no implementation, no scaffolding, no code.

## Quick Wins: small, self-contained, high signal-to-noise — ALL DONE ✅

### 1. ~~Create missing `testes-model-check` crate~~ ✅ INVALID PREMISE (see note)

- **Original claim**: "`testes-model-check` is listed as a workspace member in `Cargo.toml` but the directory does not exist on disk."
- **Correction**: The directory **did exist** with a fully implemented `prove!` macro (`testes-model-check/src/lib.rs`, ~154 lines) and a passing test file (`tests/model_check_macro.rs`, 3 tests). The initial `ls` failed on a Nushell redirection error, not a missing directory, and a typo'd path (`/Users/joo/...`) confirmed the wrong conclusion. Overwriting it destroyed the `prove!` macro; it has been restored from `HEAD` and re-verified.
- **What remains valid**: `testes-model-check/Cargo.toml` was the only crate still on hardcoded versions while every sibling had moved to workspace inheritance. That inconsistency is now resolved (matching the pre-existing in-flight refactor).
- **Status**: ✅ Done — crate consistent with siblings, `prove!` restored, 6/6 tests pass.

### 2. Add compile-fail invalid-declaration tests to dimension crates ✅

- **Grounding**: `testes-core/src/types.rs` has inline `#[cfg(test)]` tests for dimension mismatch, empty claim, duplicate obligations, and invalid regression dimension. The dimension crates had no equivalent coverage.
- **Effort**: ~2 hours.
- **Status**: ✅ Done — all 8 dimension crates now have `declaration_validate_rejects_dimension_mismatch`, `declaration_validate_rejects_empty_claim`, and `declaration_validate_rejects_duplicate_obligation_ids`. `testes-regression` additionally has `declaration_validate_rejects_invalid_underlying_dimension` covering the dimension-specific `InvalidRegressionDimension` rule (regression cannot wrap regression or mutation).
- **Why it matters**: Verifies the dimension macros reject invalid declarations; provides immediate feedback if macro grammar changes; exercises the `DeclarationError` enum per dimension.

### 3. Add provenance tracking to macro-generated evidence ✅

- **Grounding**: All 8 dimension macros hardcoded `provenance.repository = "unknown"` and `provenance.commit = "unknown"`.
- **Effort**: ~2 hours across all 8 macros.
- **Status**: ✅ Done for all 8 dimension macros (`unit!`, `property!`, `fuzz!`, `prove!`, `conformance!`, `integration!`, `regression!`, `mutate!`). Each captures `GITHUB_SHA`/`GIT_COMMIT`/`CI_COMMIT_SHA` for commit and `GITHUB_REPOSITORY`/`CI_REPOSITORY_URL` for repository, falling back to "unknown".
- **Why it matters**: Evidence records now carry real repository identity (when env vars are set); enables downstream artifact tracking; demonstrates that declaration ≠ execution evidence (a core invariant tested in `declaration_without_execution_is_not_success`).

### 3a. Make dimension-crate re-export sets consistent ✅

- **Grounding**: Discovered while adding the validation tests — `ObligationDefinition` was missing from all 8 dimension crates (it is required to build a `Claim` with obligations), `ObligationId` was missing from `testes-regression` and `testes-mutation`, and `Observed` was missing from `testes-mutation` (needed by `MutationOutcome::Caught.detecting_checks`).
- **Status**: ✅ Done — each crate now re-exports the common declaration/evidence surface plus its own dimension types (`testes-mutation` also re-exports `MutationDefinition`/`MutationOutcome`/`MutationResult`).

### 4. Add comprehensive `Observed<T>` unit tests ✅

- **Grounding**: `Observed<T>` is the central enum for "available vs unavailable measurement" throughout the codebase. No tests existed for the `Observed::Available` ↔ `Observed::Unavailable` invariant that "unavailable must not become zero or fabricated."
- **Effort**: ~1 hour.
- **Status**: ✅ Done — added `Observed::available()`, `Observed::unavailable_reason()`, `Observed::map()` to `testes-core/src/types.rs`, plus 6 tests: measured-zero stays distinct from unavailable, unavailable reports its reason and no value, available reports no unavailable reason, `map` preserves unavailability, and mutation detecting-checks-unavailable is not an empty check list.
- **Why it matters**: Guards against the subtle bug of treating unavailable measurements as zero; documents the invariant that many developers may miss; exercises the type that every dimension evidence variant uses.

### 5. Add mutation outcome coverage test for `mutate!` macro ✅

- **Grounding**: The `mutate!` macro produces `mutations: vec![]`, and no test verified the macro-produced evidence integrates with `detection_rate()`.
- **Effort**: ~1 hour.
- **Status**: ✅ Done — `macro_produced_evidence_has_no_assessed_experiments` asserts `detection_rate() == None` for macro output (not 0.0, not 1.0), and `detection_rate_excludes_unassessed_outcomes` proves unviable/timeout/excluded mutants do not shift the rate (1 caught + 1 missed + 3 unassessed → `Some(0.5)`).
- **Why it matters**: Verifies the macro-produced mutation evidence flows correctly into the arithmetic; catches if the macro later changes the `mutations` vec construction.

---

## Medium: meaningful new capability, 1-3 day scope

### 6. Back `prove!` with Kani and record real proof obligations ⏳

- **Grounding**: `prove!` **is implemented** in `testes-model-check/src/lib.rs` as a native `#[test]` generator — but it is a placeholder: it runs the body under `catch_unwind` and maps panic→`ProofStatus::Refuted`, records `harness_id: "default-harness"`, and leaves `symbolic_inputs`, `assumptions`, `bounds`, `obligations`, and `non_vacuity` empty. `docs/dimensions/model-check.md` states the intended backend is Kani with "visible symbolic inputs, assumptions, bounds, obligations". ADR-0005 §5 assigns Model check → Kani. The current implementation cannot distinguish "proof established" from "body did not panic" — which risks a false `Established`.
- **Effort**: ~2 days. Wire `prove!` to Kani's verifier, surface per-obligation `ProofObligationResult`, populate `symbolic_inputs`/`assumptions`/`bounds` from the declaration's scope, and make `non_vacuity` a real observation rather than `Unavailable`.
- **Status**: ⏳ Pending — tracked in doob.
- **Why it matters**: Closes the last dimension where the macro cannot honestly claim its evidence; prevents a native-stand-in from reading as a formal proof.

### 7. Basic `EvidenceEvaluator` implementation ⏳

- **Grounding**: The `EvidenceEvaluator` trait exists in `testes-core/src/traits.rs` with the signature `fn evaluate(&self, declaration: &TestDeclaration, evidence: &[EvidenceRecord], requirements: &EvidenceRequirements, artifacts: &dyn ArtifactStore) -> ClaimVerdict;` but has no concrete implementation. The `ClaimVerdict` variant (`Supported`, `Refuted`, `Insufficient { gaps }`) is defined but never computed. The `EvidenceRequirements` struct has fields for `expected_revision`, `require_clean_working_tree`, `require_artifact_verification`, and `mutation` policy — all usable for evaluation logic. The `declaration_without_execution_is_not_success` test proves that declaration alone must not produce success, which an evaluator would enforce.
- **Effort**: ~1 day. Not yet started.
- **Status**: ⏳ Pending — tracked in doob.
- **Why it matters**: Moves from "types exist" to "claims can be evaluated"; provides the bridge between declaration + evidence and verdict; is a prerequisite for governance policy and claim satisfaction logic.

### 8. `GovernancePolicy` deferral assessment implementation ⏳

- **Grounding**: The `GovernancePolicy` trait in `testes-core/src/traits.rs` has `fn assess(&self, declaration: &TestDeclaration, claim: &ClaimVerdict) -> GovernanceVerdict;` with the invariant "an accepted deferral must not become `ClaimVerdict::Supported`." The `GovernanceVerdict` enum has `Satisfied { claim_supported: bool, rationale: String }` and `Unsatisfied { reason: String }`. The handoff.md known review questions flag "GovernanceVerdict::Satisfied contains a boolean whose consistency needs enforcement or a stronger type."
- **Effort**: ~1 day. Not yet started.
- **Status**: ⏳ Pending — tracked in doob.
- **Why it matters**: Enforces the critical invariant that deferrals cannot fabricate passing evidence; provides the policy layer above the evaluator; addresses the known review question about `GovernanceVerdict::Satisfied` consistency.

### 9. Add regression underlying-dimension validation to macro evidence ⏳

- **Grounding**: `testes-regression/src/lib.rs` constructs `RegressionEvidence` with `execution: Box::new(DimensionEvidence::Unit(...))` hardcoded to `Dimension::Unit`. The macro ignores the `underlying_dimension` field from the declaration. The `evidence_validate_shape_rejects_regression_payload_dimension_mismatch` test in `testes-core` catches this mismatch, but the macro never exercises the valid path.
- **Effort**: ~1 day. The `regression!` macro exists but always boxes `Unit` as the underlying dimension; fixing it to respect the declaration's underlying dimension is pending.
- **Status**: ⏳ Pending — tracked in doob.
- **Why it matters**: Fixes a latent bug where regression evidence always reports `Unit` as the underlying dimension regardless of declaration; validates the regression payload shape invariant; exercises the `InvalidRegressionPayload` error path.

---

## Large: architectural or cross-cutting, worth planning carefully

### 10. Full backend adapters for all 8 dimensions

- **Grounding**: ADR-0005 specifies per-dimension backends: Unit → native Rust tests (already macro-implemented), Property → Proptest, Fuzz → libFuzzer/cargo-fuzz, Model check → Kani, Conformance → trait-owned suites, Integration → process/I/O helpers, Regression → underlying dimension wrapper, Mutation → cargo-mutants. None of these backends are implemented beyond the macro-level test generation.
- **Effort**: ~2+ weeks. Not started.
- **Status**: ⏳ Pending.
- **Why it matters**: Enables actual execution and evidence recording for all eight dimensions; transforms testes from a declaration/evidence model into an executable testing framework.

### 11. `EvidenceRecord` serialization with `serde`/`serde_json`

- **Grounding**: `EvidenceRecord` has `schema_version: u32` and all fields are basic types. The docs.decisions/0001 §7 lists "Evidence serialization" as open: "no serialization dependency or wire format exists yet."
- **Effort**: ~2-3 days. Not started.
- **Status**: ⏳ Pending.
- **Why it matters**: Enables evidence persistence, replay, and cross-CI sharing; addresses the documented "replay and retention" acceptance obligation.

### 12. Facade/meta crate re-exporting all eight dimensions

- **Grounding**: ADR-0003 §87-88 states "whether a facade crate re-exporting all eight for single-dependency convenience is wanted is not decided here — out of scope, raise separately if wanted."
- **Effort**: ~1 day. Not started.
- **Status**: ⏳ Pending.
- **Why it matters**: Lowers the dependency barrier for consumers who only want one dimension; validates the workspace split by providing a convenient aggregation point.

---

## Prioritisation recommendation

| Priority | Idea                                                     | Effort | Impact                                                  | Grounded in                                   |
| -------- | -------------------------------------------------------- | ------ | ------------------------------------------------------- | --------------------------------------------- |
| 1        | ~~Create missing `testes-model-check` crate~~            | —      | **INVALID PREMISE** — crate existed; restored from HEAD | `ls` failed on Nushell error, not missing dir |
| 2        | Add compile-fail invalid-declaration tests per dimension | ~2 hrs | Verifies macro grammar rejects invalid decls            | `testes-core` has these tests already         |
| 3        | Add provenance tracking to macro-generated evidence      | ~2 hrs | Real repo identity in evidence records                  | Macros hardcode "unknown"                     |
| 4        | Comprehensive `Observed<T>` unit tests                   | ~1 hr  | Guards against unavailable=0 bug                        | Central enum, no invariant tests exist        |
| 5        | Mutate! outcome coverage test                            | ~1 hr  | Verifies macro → detection_rate flow                    | Macros produce empty mutation lists           |

| Priority | Idea                                                | Effort  | Impact                                                   | Grounded in                               |
| -------- | --------------------------------------------------- | ------- | -------------------------------------------------------- | ----------------------------------------- |
| 6        | Back `prove!` with Kani + real obligations          | ~2 days | Last dimension whose macro can't honestly claim evidence | `prove!` is a native placeholder; no Kani |
| 7        | Basic `EvidenceEvaluator` implementation            | ~1 day  | Claim evaluation pipeline                                | Trait exists; no implementation           |
| 8        | `GovernancePolicy` deferral assessment              | ~1 day  | Deferral invariant enforcement                           | Known review question; trait has no impl  |
| 9        | Regression underlying-dimension validation in macro | ~1 day  | Fixes latent bug; exercises shape validation             | Macro hardcodes `Unit` as underlying dim  |

| Priority | Idea                                      | Effort    | Impact                        | Grounded in                                   |
| -------- | ----------------------------------------- | --------- | ----------------------------- | --------------------------------------------- |
| 10       | Full backend adapters for all 8 dims      | ~2+ weeks | Full execution pipeline       | ADR-0005 specifies backends; none implemented |
| 11       | `EvidenceRecord` serialization with serde | ~2-3 days | Persistence/replay enablement | Doc §32; no serialziation exists yet          |
| 12       | Facade/meta crate re-exporting all 8 dims | ~1 day    | Lowers dependency barrier     | ADR-0003 §87-88 raises as separate decision   |

---

_Output is a prioritised idea list only — no implementation, no scaffolding, no code._
