//! `testes-integration` owns the `integration!` macro.
#![forbid(unsafe_code)]
use std::cell::RefCell;
pub use testes_core::{
    BackendConfiguration, BackendIdentity, Claim, ComponentIdentity, Dimension, DimensionEvidence,
    EvidenceRecord, ExecutionMetadata, ExecutionOutcome, ExecutionStatus, IdError,
    IntegrationEvidence, ObligationDisposition, ObligationId, Observed, OutcomeReason, Provenance,
    RepositoryRevision, ResourceLimits, RunId, Scope, SubjectId, TestDeclaration, TestId,
    TestSpecification, WorkingTreeState,
};

thread_local! {
    pub static LAST_INTEGRATION_EVIDENCE: RefCell<Option<(TestId, IntegrationEvidence)>> =
        const { RefCell::new(None) };
}

#[macro_export]
macro_rules! integration {
    (id: $id:expr, subject: $subject:expr, claim: $claim:expr, body: $body:block $(,)?) => {
        ::pastey::paste! {
            #[test]
            fn [< $id >]() {
                let test_id = $crate::TestId::new($id).expect("valid id");
                let declaration = $crate::TestDeclaration {
                    id: test_id.clone(),
                    primary_dimension: $crate::Dimension::Integration,
                    secondary_dimensions: Vec::new(),
                    subject: $crate::SubjectId::new($subject).expect("valid subject"),
                    claim: $crate::Claim {
                        statement: $claim.to_string(),
                        obligations: Vec::new(),
                    },
                    required_scope: $crate::Scope {
                        input_domain: "integration test".to_string(),
                        assumptions: Vec::new(),
                        bounds: Vec::new(),
                        capabilities: Vec::new(),
                        resources: $crate::ResourceLimits {
                            wall_time: None,
                            memory_bytes: None,
                            max_input_bytes: None,
                        },
                    },
                    disposition: $crate::ObligationDisposition::Required,
                    specification: $crate::TestSpecification::Integration {
                        seam_id: "integration".to_string(),
                        components: vec![],
                    },
                };
                declaration.validate().expect("valid declaration");

                let started_at = std::time::SystemTime::now();
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    $body
                }));
                let duration = started_at.elapsed().unwrap_or(std::time::Duration::ZERO);

                let status = if result.is_ok() {
                    $crate::ExecutionStatus::Passed
                } else {
                    $crate::ExecutionStatus::Failed
                };

                let evidence = $crate::IntegrationEvidence {
                    seam_id: "integration".to_string(),
                    components: vec![],
                    fixtures: vec![],
                    observations: vec![],
                    cleanup: $crate::ExecutionOutcome {
                        status: $crate::ExecutionStatus::Passed,
                        reason: $crate::OutcomeReason::AssertionsSatisfied,
                        diagnostic: None,
                        obligations: Vec::new(),
                    },
                };

                let outcome = $crate::ExecutionOutcome {
                    status,
                    reason: if status == $crate::ExecutionStatus::Passed {
                        $crate::OutcomeReason::AssertionsSatisfied
                    } else {
                        $crate::OutcomeReason::AssertionViolation
                    },
                    diagnostic: None,
                    obligations: Vec::new(),
                };

                let _record = $crate::EvidenceRecord {
                    schema_version: 1,
                    declaration,
                    provenance: $crate::Provenance {
                        run_id: $crate::RunId::new("integration-run").expect("valid"),
                        revision: $crate::RepositoryRevision {
                            repository: "unknown".to_string(),
                            commit: "unknown".to_string(),
                            working_tree: $crate::WorkingTreeState::Unknown,
                        },
                    },
                    execution: $crate::ExecutionMetadata {
                        backend: $crate::BackendIdentity {
                            name: "native-integration".to_string(),
                            version: env!("CARGO_PKG_VERSION").to_string(),
                            toolchain: None,
                        },
                        configuration: $crate::BackendConfiguration::new(),
                        started_at,
                        duration,
                        actual_scope: $crate::Scope {
                            input_domain: "integration test".to_string(),
                            assumptions: Vec::new(),
                            bounds: Vec::new(),
                            capabilities: Vec::new(),
                            resources: $crate::ResourceLimits {
                                wall_time: None,
                                memory_bytes: None,
                                max_input_bytes: None,
                            },
                        },
                    },
                    outcome,
                    artifacts: Vec::new(),
                    dimension_evidence: $crate::DimensionEvidence::Integration(evidence.clone()),
                };

                $crate::LAST_INTEGRATION_EVIDENCE.with(|e| {
                    *e.borrow_mut() = Some((test_id, evidence));
                });

                if result.is_err() {
                    panic!("integration test failed");
                }
            }
        }
    };
}
