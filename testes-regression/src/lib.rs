//! `testes-regression` owns the `regression!` macro.
#![forbid(unsafe_code)]
use std::cell::RefCell;
pub use testes_core::{
    BackendConfiguration, BackendIdentity, Claim, Dimension, DimensionEvidence, EvidenceRecord,
    ExecutionMetadata, ExecutionOutcome, ExecutionStatus, FailureId, IdError,
    ObligationDisposition, Observed, OutcomeReason, Provenance, RegressionEvidence,
    RepositoryRevision, ResourceLimits, RunId, Scope, SubjectId, TestDeclaration, TestId,
    TestSpecification, UnitEvidence, WorkingTreeState,
};

thread_local! {
    pub static LAST_REGRESSION_EVIDENCE: RefCell<Option<(TestId, RegressionEvidence)>> =
        const { RefCell::new(None) };
}

#[macro_export]
macro_rules! regression {
    (id: $id:expr, subject: $subject:expr, claim: $claim:expr, body: $body:block $(,)?) => {
        ::paste::paste! {
            #[test]
            fn [< $id >]() {
                let test_id = $crate::TestId::new($id).expect("valid id");
                let failure_id = $crate::FailureId::new($id).expect("valid failure id");
                let declaration = $crate::TestDeclaration {
                    id: test_id.clone(),
                    primary_dimension: $crate::Dimension::Regression,
                    secondary_dimensions: Vec::new(),
                    subject: $crate::SubjectId::new($subject).expect("valid subject"),
                    claim: $crate::Claim {
                        statement: $claim.to_string(),
                        obligations: Vec::new(),
                    },
                    required_scope: $crate::Scope {
                        input_domain: "regression test".to_string(),
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
                    specification: $crate::TestSpecification::Regression {
                        failure: failure_id.clone(),
                        underlying_dimension: $crate::Dimension::Unit,
                        reproducer_artifacts: vec![],
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

                let evidence = $crate::RegressionEvidence {
                    failure: failure_id,
                    reproducer_artifacts: vec![],
                    historical_failure: None,
                    execution: Box::new($crate::DimensionEvidence::Unit($crate::UnitEvidence {
                        executed_examples: vec![],
                        assertions_evaluated: $crate::Observed::Unavailable {
                            reason: "not measured".to_string(),
                        },
                    })),
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
                        run_id: $crate::RunId::new("regression-run").expect("valid"),
                        revision: $crate::RepositoryRevision {
                            repository: "unknown".to_string(),
                            commit: "unknown".to_string(),
                            working_tree: $crate::WorkingTreeState::Unknown,
                        },
                    },
                    execution: $crate::ExecutionMetadata {
                        backend: $crate::BackendIdentity {
                            name: "native-regression".to_string(),
                            version: env!("CARGO_PKG_VERSION").to_string(),
                            toolchain: None,
                        },
                        configuration: $crate::BackendConfiguration::new(),
                        started_at,
                        duration,
                        actual_scope: $crate::Scope {
                            input_domain: "regression test".to_string(),
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
                    dimension_evidence: $crate::DimensionEvidence::Regression(evidence.clone()),
                };

                $crate::LAST_REGRESSION_EVIDENCE.with(|e| {
                    *e.borrow_mut() = Some((test_id, evidence));
                });

                if result.is_err() {
                    panic!("regression test failed");
                }
            }
        }
    };
}
