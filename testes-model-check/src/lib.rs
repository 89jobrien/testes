//! `testes-model-check` owns the `prove!` dimension (model checking).
//!
//! The crate declares a model-check claim, runs it through a [`ModelCheckBackend`]
//! port, and records a [`DimensionEvidence::ModelCheck`] result. The port is
//! implemented by backend adapters that live outside this crate (for example a
//! future Kani adapter); this crate has no dependency on any verifier.
//!
//! Evidence rules enforced here:
//! - Only a declared obligation with an `Established` result counts as proved.
//! - An unavailable backend yields `NotExecuted`, never `Passed`.
//! - A declared obligation with no backend result yields `NotExecuted`.
//! - A declaration with no obligations cannot pass (nothing was proved).

#![forbid(unsafe_code)]

use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime},
};

pub use testes_core::{
    BackendIdentity, Claim, DeclarationError, Dimension, DimensionEvidence, EvidenceRecord,
    ExecutionMetadata, ExecutionOutcome, ExecutionStatus, ModelCheckEvidence, ObligationDefinition,
    ObligationDisposition, ObligationId, ObligationResult, Observed, OutcomeReason,
    ProofObligationResult, ProofStatus, Provenance, RepositoryRevision, ResourceLimits, RunId,
    Scope, SubjectId, TestDeclaration, TestId, TestSpecification, WorkingTreeState,
};

/// What a backend reports for one model-check run.
#[derive(Clone, Debug)]
pub enum ModelCheckRun {
    /// The backend executed the harness and reported these obligation results.
    Executed(Vec<ProofObligationResult>),
    /// The backend could not run the harness (missing tool, unsupported target, ...).
    Unavailable { reason: String },
}

/// Port implemented by model-check backend adapters.
///
/// Implementations must report `Unavailable` rather than fabricate results.
pub trait ModelCheckBackend {
    fn identity(&self) -> BackendIdentity;
    fn check(&self, harness_id: &str, obligations: &[ObligationId]) -> ModelCheckRun;
}

/// Declaration inputs for one `prove!` invocation.
#[derive(Clone, Debug)]
pub struct ProveSpec {
    pub id: TestId,
    pub subject: SubjectId,
    pub claim: String,
    pub harness_id: String,
    pub obligations: Vec<ObligationDefinition>,
}

/// Declares, runs, and records one model-check claim.
///
/// Returns `Err` only when the declaration itself is invalid. Every run outcome,
/// including unavailability, is returned as an `EvidenceRecord`.
pub fn prove(
    spec: &ProveSpec,
    backend: &dyn ModelCheckBackend,
) -> Result<EvidenceRecord, DeclarationError> {
    let obligation_ids: Vec<ObligationId> = spec
        .obligations
        .iter()
        .map(|definition| definition.id.clone())
        .collect();
    let declaration = TestDeclaration {
        id: spec.id.clone(),
        primary_dimension: Dimension::ModelCheck,
        secondary_dimensions: Vec::new(),
        subject: spec.subject.clone(),
        claim: Claim {
            statement: spec.claim.clone(),
            obligations: spec.obligations.clone(),
        },
        required_scope: model_check_scope(),
        disposition: ObligationDisposition::Required,
        specification: TestSpecification::ModelCheck {
            harness_id: spec.harness_id.clone(),
            obligation_ids: obligation_ids.clone(),
        },
    };
    declaration.validate()?;

    let started_at = SystemTime::now();
    let run = backend.check(&spec.harness_id, &obligation_ids);
    let duration = started_at.elapsed().unwrap_or(Duration::ZERO);

    let (status, reason, diagnostic, obligations, non_vacuity) = match run {
        ModelCheckRun::Unavailable { reason } => {
            let obligations = spec
                .obligations
                .iter()
                .map(|definition| ProofObligationResult {
                    id: definition.id.clone(),
                    status: ProofStatus::NotExecuted,
                    diagnostic: Some(reason.clone()),
                    counterexample: None,
                })
                .collect();
            (
                ExecutionStatus::NotExecuted,
                OutcomeReason::CapabilityUnavailable,
                Some(reason),
                obligations,
                Observed::Unavailable {
                    reason: "model check did not execute".to_string(),
                },
            )
        }
        ModelCheckRun::Executed(reported) => {
            let obligations = merge_results(&spec.obligations, reported);
            let (status, reason, diagnostic) = overall_status(&spec.obligations, &obligations);
            (
                status,
                reason,
                diagnostic,
                obligations,
                Observed::Unavailable {
                    reason: "non-vacuity not reported by backend".to_string(),
                },
            )
        }
    };

    let outcome_obligations = obligations
        .iter()
        .take(spec.obligations.len())
        .map(|result| ObligationResult {
            id: result.id.clone(),
            status: execution_status(result.status),
            diagnostic: result.diagnostic.clone(),
        })
        .collect();

    let identity = backend.identity();
    Ok(EvidenceRecord {
        schema_version: 1,
        declaration,
        provenance: Provenance {
            run_id: RunId::new("model-check-run").expect("run id valid"),
            revision: RepositoryRevision {
                repository: "unknown".to_string(),
                commit: "unknown".to_string(),
                working_tree: WorkingTreeState::Unknown,
            },
        },
        execution: ExecutionMetadata {
            backend: identity,
            configuration: BTreeMap::new(),
            started_at,
            duration,
            actual_scope: model_check_scope(),
        },
        outcome: ExecutionOutcome {
            status,
            reason,
            diagnostic,
            obligations: outcome_obligations,
        },
        artifacts: Vec::new(),
        dimension_evidence: DimensionEvidence::ModelCheck(ModelCheckEvidence {
            harness_id: spec.harness_id.clone(),
            symbolic_inputs: Vec::new(),
            assumptions: Vec::new(),
            bounds: Vec::new(),
            obligations,
            non_vacuity,
            report: None,
        }),
    })
}

fn model_check_scope() -> Scope {
    Scope {
        input_domain: "model check".to_string(),
        assumptions: Vec::new(),
        bounds: Vec::new(),
        capabilities: Vec::new(),
        resources: ResourceLimits {
            wall_time: None,
            memory_bytes: None,
            max_input_bytes: None,
        },
    }
}

/// Returns one result per declared obligation (in declaration order), followed by
/// any backend results for undeclared obligations. A declared obligation without a
/// backend result is recorded as `NotExecuted`.
fn merge_results(
    declared: &[ObligationDefinition],
    reported: Vec<ProofObligationResult>,
) -> Vec<ProofObligationResult> {
    let mut merged: Vec<ProofObligationResult> = declared
        .iter()
        .map(|definition| {
            reported
                .iter()
                .find(|result| result.id == definition.id)
                .cloned()
                .unwrap_or_else(|| ProofObligationResult {
                    id: definition.id.clone(),
                    status: ProofStatus::NotExecuted,
                    diagnostic: Some("backend reported no result for this obligation".to_string()),
                    counterexample: None,
                })
        })
        .collect();
    merged.extend(
        reported
            .into_iter()
            .filter(|result| !declared.iter().any(|definition| definition.id == result.id)),
    );
    merged
}

/// A refuted obligation anywhere fails the run. Otherwise only a non-empty set of
/// declared obligations that were all established passes.
fn overall_status(
    declared: &[ObligationDefinition],
    results: &[ProofObligationResult],
) -> (ExecutionStatus, OutcomeReason, Option<String>) {
    if results
        .iter()
        .any(|result| result.status == ProofStatus::Refuted)
    {
        return (
            ExecutionStatus::Failed,
            OutcomeReason::CounterexampleFound,
            Some("one or more obligations refuted".to_string()),
        );
    }
    let declared_results = &results[..declared.len()];
    if declared.is_empty() {
        return (
            ExecutionStatus::NotExecuted,
            OutcomeReason::BackendDefined("no obligations declared".to_string()),
            Some("no obligations declared; nothing was proved".to_string()),
        );
    }
    if declared_results
        .iter()
        .any(|result| result.status == ProofStatus::NotExecuted)
    {
        return (
            ExecutionStatus::NotExecuted,
            OutcomeReason::BackendDefined("obligation not executed".to_string()),
            Some("one or more declared obligations were not executed".to_string()),
        );
    }
    if declared_results
        .iter()
        .any(|result| result.status == ProofStatus::Inconclusive)
    {
        return (
            ExecutionStatus::Inconclusive,
            OutcomeReason::BackendDefined("obligation inconclusive".to_string()),
            Some("one or more declared obligations were inconclusive".to_string()),
        );
    }
    (
        ExecutionStatus::Passed,
        OutcomeReason::AssertionsSatisfied,
        None,
    )
}

fn execution_status(status: ProofStatus) -> ExecutionStatus {
    match status {
        ProofStatus::Established => ExecutionStatus::Passed,
        ProofStatus::Refuted => ExecutionStatus::Failed,
        ProofStatus::Inconclusive => ExecutionStatus::Inconclusive,
        ProofStatus::NotExecuted => ExecutionStatus::NotExecuted,
    }
}

/// Declares and runs a model-check claim, returning its evidence record.
///
/// ```text
/// prove! {
///     id: "parser-total",
///     subject: "parse",
///     claim: "parse never panics",
///     harness: "parse_harness",
///     backend: my_backend,
///     obligations: {
///         "no-panic" => "parse returns without panicking",
///     }
/// }
/// ```
///
/// The macro expands to an expression that evaluates to an `EvidenceRecord`. It
/// panics only on an invalid declaration (for example a duplicate obligation id);
/// a failed or unavailable proof is recorded in the returned evidence, and callers
/// must inspect `outcome.status`.
#[macro_export]
macro_rules! prove {
    (
        id: $id:expr,
        subject: $subject:expr,
        claim: $claim:expr,
        harness: $harness:expr,
        backend: $backend:expr,
        obligations: {
            $($oid:expr => $odesc:expr),* $(,)?
        } $(,)?
    ) => {{
        let spec = $crate::ProveSpec {
            id: $crate::TestId::new($id)
                .expect("test id must not be blank or contain control characters"),
            subject: $crate::SubjectId::new($subject)
                .expect("subject must not be blank or contain control characters"),
            claim: ($claim).to_string(),
            harness_id: ($harness).to_string(),
            obligations: vec![
                $($crate::ObligationDefinition {
                    id: $crate::ObligationId::new($oid)
                        .expect("obligation id must not be blank or contain control characters"),
                    description: ($odesc).to_string(),
                }),*
            ],
        };
        $crate::prove(&spec, &$backend).expect("prove! declaration must be valid")
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeBackend {
        run: ModelCheckRun,
    }

    impl ModelCheckBackend for FakeBackend {
        fn identity(&self) -> BackendIdentity {
            BackendIdentity {
                name: "fake-model-checker".to_string(),
                version: "0.0.0".to_string(),
                toolchain: None,
            }
        }

        fn check(&self, _harness_id: &str, _obligations: &[ObligationId]) -> ModelCheckRun {
            self.run.clone()
        }
    }

    fn oid(value: &str) -> ObligationId {
        ObligationId::new(value).expect("obligation id must not be blank")
    }

    fn definition(value: &str) -> ObligationDefinition {
        ObligationDefinition {
            id: oid(value),
            description: format!("{value} holds"),
        }
    }

    fn result(value: &str, status: ProofStatus) -> ProofObligationResult {
        ProofObligationResult {
            id: oid(value),
            status,
            diagnostic: None,
            counterexample: None,
        }
    }

    fn spec(obligations: Vec<ObligationDefinition>) -> ProveSpec {
        ProveSpec {
            id: TestId::new("parser-total").expect("id must not be blank"),
            subject: SubjectId::new("parse").expect("subject must not be blank"),
            claim: "parse never panics".to_string(),
            harness_id: "parse_harness".to_string(),
            obligations,
        }
    }

    fn two_obligations() -> Vec<ObligationDefinition> {
        vec![definition("no-panic"), definition("total")]
    }

    fn run_with(run: ModelCheckRun, obligations: Vec<ObligationDefinition>) -> EvidenceRecord {
        let backend = FakeBackend { run };
        prove(&spec(obligations), &backend).expect("declaration must be valid")
    }

    fn model_check_obligations(record: &EvidenceRecord) -> &[ProofObligationResult] {
        match &record.dimension_evidence {
            DimensionEvidence::ModelCheck(evidence) => &evidence.obligations,
            other => panic!("expected model-check evidence, got {:?}", other.dimension()),
        }
    }

    #[test]
    fn all_established_obligations_pass_and_shape_is_valid() {
        let record = run_with(
            ModelCheckRun::Executed(vec![
                result("no-panic", ProofStatus::Established),
                result("total", ProofStatus::Established),
            ]),
            two_obligations(),
        );
        assert_eq!(record.outcome.status, ExecutionStatus::Passed);
        assert_eq!(record.outcome.reason, OutcomeReason::AssertionsSatisfied);
        assert_eq!(record.execution.backend.name, "fake-model-checker");
        assert_eq!(record.declaration.primary_dimension, Dimension::ModelCheck);
        assert_eq!(record.validate_shape(), Ok(()));
    }

    #[test]
    fn refuted_obligation_fails_with_counterexample_reason() {
        let record = run_with(
            ModelCheckRun::Executed(vec![
                result("no-panic", ProofStatus::Established),
                ProofObligationResult {
                    id: oid("total"),
                    status: ProofStatus::Refuted,
                    diagnostic: Some("panics on empty input".to_string()),
                    counterexample: None,
                },
            ]),
            two_obligations(),
        );
        assert_eq!(record.outcome.status, ExecutionStatus::Failed);
        assert_eq!(record.outcome.reason, OutcomeReason::CounterexampleFound);
        assert_eq!(record.validate_shape(), Ok(()));
        let refuted = model_check_obligations(&record)
            .iter()
            .find(|result| result.id == oid("total"))
            .expect("refuted obligation must be recorded");
        assert_eq!(refuted.status, ProofStatus::Refuted);
    }

    #[test]
    fn inconclusive_obligation_is_not_a_pass() {
        let record = run_with(
            ModelCheckRun::Executed(vec![
                result("no-panic", ProofStatus::Established),
                result("total", ProofStatus::Inconclusive),
            ]),
            two_obligations(),
        );
        assert_eq!(record.outcome.status, ExecutionStatus::Inconclusive);
        assert_ne!(record.outcome.status, ExecutionStatus::Passed);
        assert_eq!(record.validate_shape(), Ok(()));
    }

    #[test]
    fn unavailable_backend_is_not_executed_and_never_passes() {
        let record = run_with(
            ModelCheckRun::Unavailable {
                reason: "kani toolchain not installed".to_string(),
            },
            two_obligations(),
        );
        assert_eq!(record.outcome.status, ExecutionStatus::NotExecuted);
        assert_eq!(record.outcome.reason, OutcomeReason::CapabilityUnavailable);
        assert_eq!(
            record.outcome.diagnostic.as_deref(),
            Some("kani toolchain not installed")
        );
        assert_eq!(record.validate_shape(), Ok(()));
        for obligation in model_check_obligations(&record) {
            assert_eq!(obligation.status, ProofStatus::NotExecuted);
        }
        match &record.dimension_evidence {
            DimensionEvidence::ModelCheck(evidence) => {
                assert!(matches!(evidence.non_vacuity, Observed::Unavailable { .. }));
                assert!(evidence.report.is_none());
            }
            other => panic!("expected model-check evidence, got {:?}", other.dimension()),
        }
    }

    #[test]
    fn declared_obligation_without_backend_result_is_not_executed() {
        let record = run_with(
            ModelCheckRun::Executed(vec![result("no-panic", ProofStatus::Established)]),
            two_obligations(),
        );
        assert_eq!(record.outcome.status, ExecutionStatus::NotExecuted);
        assert_ne!(record.outcome.status, ExecutionStatus::Passed);
        let missing = model_check_obligations(&record)
            .iter()
            .find(|result| result.id == oid("total"))
            .expect("missing obligation must still be recorded");
        assert_eq!(missing.status, ProofStatus::NotExecuted);
        assert_eq!(record.validate_shape(), Ok(()));
    }

    #[test]
    fn declaration_without_obligations_cannot_pass() {
        let record = run_with(ModelCheckRun::Executed(Vec::new()), Vec::new());
        assert_eq!(record.outcome.status, ExecutionStatus::NotExecuted);
        assert_eq!(record.validate_shape(), Ok(()));
    }

    #[test]
    fn undeclared_refuted_result_still_fails_and_is_recorded() {
        let record = run_with(
            ModelCheckRun::Executed(vec![
                result("no-panic", ProofStatus::Established),
                result("extra", ProofStatus::Refuted),
            ]),
            vec![definition("no-panic")],
        );
        assert_eq!(record.outcome.status, ExecutionStatus::Failed);
        assert!(
            model_check_obligations(&record)
                .iter()
                .any(|result| result.id == oid("extra") && result.status == ProofStatus::Refuted),
            "undeclared refuted result must appear in evidence"
        );
    }

    #[test]
    fn duplicate_declared_obligation_is_rejected_before_running() {
        let backend = FakeBackend {
            run: ModelCheckRun::Executed(Vec::new()),
        };
        let result = prove(
            &spec(vec![definition("no-panic"), definition("no-panic")]),
            &backend,
        );
        assert_eq!(
            result.map(|_| ()),
            Err(DeclarationError::DuplicateObligation(oid("no-panic")))
        );
    }

    #[test]
    fn blank_claim_is_rejected() {
        let backend = FakeBackend {
            run: ModelCheckRun::Executed(Vec::new()),
        };
        let mut blank = spec(two_obligations());
        blank.claim = "   ".to_string();
        assert_eq!(
            prove(&blank, &backend).map(|_| ()),
            Err(DeclarationError::EmptyClaim)
        );
    }
}
