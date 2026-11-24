//! Runtime for `integration!` scenarios: builds an integration declaration,
//! runs the scenario body in panic-safe mode, and records `IntegrationEvidence`
//! through `testes-core` types.
//!
//! Outcome mapping (a non-passing run is never reported as passing):
//! - body returns `Ok(())`                  -> `ExecutionStatus::Passed`
//! - body panics (assertion failure)        -> `ExecutionStatus::Failed`
//! - body returns `ScenarioError::Unavailable` -> `ExecutionStatus::Inconclusive`
//! - body returns `ScenarioError::NotExecuted` -> `ExecutionStatus::NotExecuted`

use std::{
    panic::{self, AssertUnwindSafe},
    time::{Duration, SystemTime},
};

use testes_core::{
    BackendConfiguration, BackendIdentity, Claim, ComponentIdentity, Dimension, DimensionEvidence,
    EvidenceRecord, ExecutionMetadata, ExecutionOutcome, ExecutionStatus, IntegrationEvidence,
    ObligationDisposition, OutcomeReason, Provenance, RepositoryRevision, ResourceLimits, RunId,
    Scope, SubjectId, TestDeclaration, TestId, TestSpecification, WorkingTreeState,
};

/// Reasons a scenario body reports that it did not verify the seam.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScenarioError {
    /// A required capability or dependency was not available in this environment.
    Unavailable { reason: String },
    /// The scenario was not executed (for example, a precondition was not met).
    NotExecuted { reason: String },
}

/// Declared identity of one integration scenario.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrationSpec {
    pub id: String,
    pub subject: String,
    pub seam: String,
    pub claim: String,
    pub components: Vec<ComponentIdentity>,
}

/// Declares the scenario, runs `body` panic-safely, and returns the recorded evidence.
pub fn run_scenario(
    spec: &IntegrationSpec,
    body: impl FnOnce() -> Result<(), ScenarioError>,
) -> EvidenceRecord {
    let test_id = TestId::new(spec.id.as_str())
        .expect("integration id must not be blank or contain control characters");
    let subject = SubjectId::new(spec.subject.as_str())
        .expect("integration subject must not be blank or contain control characters");
    let declaration = TestDeclaration {
        id: test_id,
        primary_dimension: Dimension::Integration,
        secondary_dimensions: Vec::new(),
        subject,
        claim: Claim {
            statement: spec.claim.clone(),
            obligations: Vec::new(),
        },
        required_scope: seam_scope(&spec.seam),
        disposition: ObligationDisposition::Required,
        specification: TestSpecification::Integration {
            seam_id: spec.seam.clone(),
            components: spec.components.clone(),
        },
    };
    declaration
        .validate()
        .expect("integration TestDeclaration must be valid");

    let started_at = SystemTime::now();
    let result = panic::catch_unwind(AssertUnwindSafe(body));
    let duration = started_at.elapsed().unwrap_or(Duration::ZERO);

    let (status, reason, diagnostic) = match result {
        Ok(Ok(())) => (
            ExecutionStatus::Passed,
            OutcomeReason::AssertionsSatisfied,
            None,
        ),
        Ok(Err(ScenarioError::Unavailable { reason })) => (
            ExecutionStatus::Inconclusive,
            OutcomeReason::CapabilityUnavailable,
            Some(reason),
        ),
        Ok(Err(ScenarioError::NotExecuted { reason })) => (
            ExecutionStatus::NotExecuted,
            OutcomeReason::BackendDefined("scenario not executed".to_string()),
            Some(reason),
        ),
        Err(payload) => (
            ExecutionStatus::Failed,
            OutcomeReason::AssertionViolation,
            Some(panic_message(payload.as_ref())),
        ),
    };

    EvidenceRecord {
        schema_version: 1,
        declaration,
        provenance: Provenance {
            run_id: RunId::new("integration-test-run").expect("run id must be valid"),
            revision: RepositoryRevision {
                repository: "unknown".to_string(),
                commit: "unknown".to_string(),
                working_tree: WorkingTreeState::Unknown,
            },
        },
        execution: ExecutionMetadata {
            backend: BackendIdentity {
                name: "testes-integration".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                toolchain: None,
            },
            configuration: BackendConfiguration::new(),
            started_at,
            duration,
            actual_scope: seam_scope(&spec.seam),
        },
        outcome: ExecutionOutcome {
            status,
            reason,
            diagnostic,
            obligations: Vec::new(),
        },
        artifacts: Vec::new(),
        dimension_evidence: DimensionEvidence::Integration(IntegrationEvidence {
            seam_id: spec.seam.clone(),
            components: spec.components.clone(),
            fixtures: Vec::new(),
            observations: Vec::new(),
            // No cleanup step runs in the macro path, so cleanup is recorded as not executed.
            cleanup: ExecutionOutcome {
                status: ExecutionStatus::NotExecuted,
                reason: OutcomeReason::BackendDefined("no cleanup step declared".to_string()),
                diagnostic: None,
                obligations: Vec::new(),
            },
        }),
    }
}

/// Panics unless the record's outcome is `ExecutionStatus::Passed`.
pub fn assert_scenario_passed(record: &EvidenceRecord) {
    let outcome = &record.outcome;
    if outcome.status != ExecutionStatus::Passed {
        panic!(
            "integration scenario `{}` did not pass: status {:?}, reason {:?}{}",
            record.declaration.id,
            outcome.status,
            outcome.reason,
            outcome
                .diagnostic
                .as_deref()
                .map(|d| format!(", detail: {d}"))
                .unwrap_or_default(),
        );
    }
}

fn seam_scope(seam: &str) -> Scope {
    Scope {
        input_domain: format!("integration seam: {seam}"),
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

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "scenario panicked with a non-string payload".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testes_core::BoundaryKind;

    fn spec() -> IntegrationSpec {
        IntegrationSpec {
            id: "checkout_payments_seam".to_string(),
            subject: "checkout".to_string(),
            seam: "checkout->payments".to_string(),
            claim: "checkout charges through the payments port".to_string(),
            components: vec![
                ComponentIdentity {
                    name: "checkout".to_string(),
                    version: None,
                    boundary: BoundaryKind::Real,
                },
                ComponentIdentity {
                    name: "payments".to_string(),
                    version: Some("sandbox-1".to_string()),
                    boundary: BoundaryKind::Fake,
                },
            ],
        }
    }

    #[test]
    fn passing_body_records_passed_outcome() {
        let record = run_scenario(&spec(), || Ok(()));
        assert_eq!(record.outcome.status, ExecutionStatus::Passed);
        assert_eq!(record.outcome.reason, OutcomeReason::AssertionsSatisfied);
        assert_eq!(record.outcome.diagnostic, None);
    }

    #[test]
    fn panicking_body_records_failed_outcome_with_diagnostic() {
        let record = run_scenario(&spec(), || panic!("payments returned wrong total"));
        assert_eq!(record.outcome.status, ExecutionStatus::Failed);
        assert_eq!(record.outcome.reason, OutcomeReason::AssertionViolation);
        assert!(
            record
                .outcome
                .diagnostic
                .as_deref()
                .is_some_and(|d| d.contains("payments returned wrong total")),
            "diagnostic should carry the panic message, got {:?}",
            record.outcome.diagnostic
        );
    }

    #[test]
    fn unavailable_body_records_inconclusive_not_passed() {
        let record = run_scenario(&spec(), || {
            Err(ScenarioError::Unavailable {
                reason: "payments sandbox not reachable".to_string(),
            })
        });
        assert_eq!(record.outcome.status, ExecutionStatus::Inconclusive);
        assert_eq!(record.outcome.reason, OutcomeReason::CapabilityUnavailable);
        assert_eq!(
            record.outcome.diagnostic.as_deref(),
            Some("payments sandbox not reachable")
        );
    }

    #[test]
    fn not_executed_body_records_not_executed_not_passed() {
        let record = run_scenario(&spec(), || {
            Err(ScenarioError::NotExecuted {
                reason: "precondition feature flag disabled".to_string(),
            })
        });
        assert_eq!(record.outcome.status, ExecutionStatus::NotExecuted);
        assert_eq!(
            record.outcome.reason,
            OutcomeReason::BackendDefined("scenario not executed".to_string())
        );
        assert_eq!(
            record.outcome.diagnostic.as_deref(),
            Some("precondition feature flag disabled")
        );
    }

    #[test]
    fn record_declares_integration_specification_and_components() {
        let record = run_scenario(&spec(), || Ok(()));
        assert_eq!(record.declaration.primary_dimension, Dimension::Integration);
        assert_eq!(
            record.declaration.specification.dimension(),
            Dimension::Integration
        );
        match &record.declaration.specification {
            TestSpecification::Integration {
                seam_id,
                components,
            } => {
                assert_eq!(seam_id, "checkout->payments");
                assert_eq!(components, &spec().components);
            }
            other => panic!("expected integration specification, got {other:?}"),
        }
    }

    #[test]
    fn record_carries_integration_evidence_with_no_fabricated_artifacts() {
        let record = run_scenario(&spec(), || Ok(()));
        match &record.dimension_evidence {
            DimensionEvidence::Integration(evidence) => {
                assert_eq!(evidence.seam_id, "checkout->payments");
                assert_eq!(evidence.components, spec().components);
                assert!(evidence.fixtures.is_empty());
                assert!(evidence.observations.is_empty());
            }
            other => panic!("expected integration evidence, got {other:?}"),
        }
    }

    #[test]
    fn cleanup_is_recorded_as_not_executed_because_no_cleanup_step_runs() {
        let record = run_scenario(&spec(), || Ok(()));
        match &record.dimension_evidence {
            DimensionEvidence::Integration(evidence) => {
                assert_eq!(evidence.cleanup.status, ExecutionStatus::NotExecuted);
            }
            other => panic!("expected integration evidence, got {other:?}"),
        }
    }

    #[test]
    fn every_outcome_kind_produces_a_valid_evidence_shape() {
        let records = [
            run_scenario(&spec(), || Ok(())),
            run_scenario(&spec(), || panic!("boom")),
            run_scenario(&spec(), || {
                Err(ScenarioError::Unavailable {
                    reason: "down".to_string(),
                })
            }),
            run_scenario(&spec(), || {
                Err(ScenarioError::NotExecuted {
                    reason: "skipped".to_string(),
                })
            }),
        ];
        for record in &records {
            assert_eq!(record.validate_shape(), Ok(()));
        }
    }

    #[test]
    fn assert_scenario_passed_accepts_passed_record() {
        let record = run_scenario(&spec(), || Ok(()));
        assert_scenario_passed(&record);
    }

    #[test]
    #[should_panic(expected = "did not pass")]
    fn assert_scenario_passed_rejects_unavailable_record() {
        let record = run_scenario(&spec(), || {
            Err(ScenarioError::Unavailable {
                reason: "down".to_string(),
            })
        });
        assert_scenario_passed(&record);
    }

    #[test]
    #[should_panic(expected = "did not pass")]
    fn assert_scenario_passed_rejects_not_executed_record() {
        let record = run_scenario(&spec(), || {
            Err(ScenarioError::NotExecuted {
                reason: "skipped".to_string(),
            })
        });
        assert_scenario_passed(&record);
    }

    #[test]
    #[should_panic(expected = "did not pass")]
    fn assert_scenario_passed_rejects_failed_record() {
        let record = run_scenario(&spec(), || panic!("boom"));
        assert_scenario_passed(&record);
    }
}
