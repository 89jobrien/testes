//! `testes-conformance` owns the `conformance!` macro.

#![forbid(unsafe_code)]

use std::cell::RefCell;
pub use testes_core::{
    BackendConfiguration, BackendIdentity, Claim, ConformanceEvidence, ContractId,
    ContractIdentity, Dimension, DimensionEvidence, EvidenceRecord, ExecutionMetadata,
    ExecutionOutcome, ExecutionStatus, IdError, ImplementationId, ImplementationIdentity,
    ObligationDisposition, ObligationId, Observed, OutcomeReason, Provenance, RepositoryRevision,
    ResourceLimits, RunId, Scope, SubjectId, TestDeclaration, TestId, TestSpecification,
    WorkingTreeState,
};

thread_local! {
    pub static LAST_CONFORMANCE_EVIDENCE: RefCell<Option<(TestId, ConformanceEvidence)>> =
        const { RefCell::new(None) };
}

#[macro_export]
macro_rules! conformance {
    (
        id: $id:expr,
        subject: $subject:expr,
        claim: $claim:expr,
        body: $body:block
        $(,)?
    ) => {
        ::pastey::paste! {
            #[test]
            fn [< $id >]() {
                let test_id = $crate::TestId::new($id)
                    .expect("test id must not be blank or contain control characters");
                let subject_id = $crate::SubjectId::new($subject)
                    .expect("subject must not be blank or contain control characters");

                let contract_id = $crate::ContractId::new($id)
                    .expect("contract id must be valid");
                let implementation_id = $crate::ImplementationId::new($id)
                    .expect("implementation id must be valid");

                let declaration = $crate::TestDeclaration {
                    id: test_id.clone(),
                    primary_dimension: $crate::Dimension::Conformance,
                    secondary_dimensions: Vec::new(),
                    subject: subject_id,
                    claim: $crate::Claim {
                        statement: $claim.to_string(),
                        obligations: Vec::new(),
                    },
                    required_scope: $crate::Scope {
                        input_domain: "conformance test".to_string(),
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
                    specification: $crate::TestSpecification::Conformance {
                        contract: $crate::ContractIdentity {
                            id: contract_id.clone(),
                            version: "0.1.0".to_string(),
                        },
                        implementation: $crate::ImplementationIdentity {
                            id: implementation_id.clone(),
                            configuration_id: "default".to_string(),
                        },
                        case_ids: vec![],
                    },
                };

                declaration.validate().expect("TestDeclaration must be valid");

                let started_at = std::time::SystemTime::now();
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    $body
                }));
                let duration = started_at.elapsed().unwrap_or(std::time::Duration::ZERO);

                let (status, diagnostic) = match &result {
                    Ok(_) => ($crate::ExecutionStatus::Passed, None),
                    Err(e) => ($crate::ExecutionStatus::Failed, Some(format!("{:?}", e))),
                };

                let conformance_evidence = $crate::ConformanceEvidence {
                    contract: $crate::ContractIdentity {
                        id: contract_id,
                        version: "0.1.0".to_string(),
                    },
                    implementation: $crate::ImplementationIdentity {
                        id: implementation_id,
                        configuration_id: "default".to_string(),
                    },
                    capabilities: vec![],
                    cases: vec![],
                };

                let execution_outcome = $crate::ExecutionOutcome {
                    status,
                    reason: if status == $crate::ExecutionStatus::Passed {
                        $crate::OutcomeReason::AssertionsSatisfied
                    } else {
                        $crate::OutcomeReason::AssertionViolation
                    },
                    diagnostic,
                    obligations: Vec::new(),
                };

                let _evidence = $crate::EvidenceRecord {
                    schema_version: 1,
                    declaration,
                    provenance: $crate::Provenance {
                        run_id: $crate::RunId::new("conformance-run")
                            .expect("run id valid"),
                        revision: $crate::RepositoryRevision {
                            repository: "unknown".to_string(),
                            commit: "unknown".to_string(),
                            working_tree: $crate::WorkingTreeState::Unknown,
                        },
                    },
                    execution: $crate::ExecutionMetadata {
                        backend: $crate::BackendIdentity {
                            name: "native-conformance".to_string(),
                            version: env!("CARGO_PKG_VERSION").to_string(),
                            toolchain: None,
                        },
                        configuration: $crate::BackendConfiguration::new(),
                        started_at,
                        duration,
                        actual_scope: $crate::Scope {
                            input_domain: "conformance test".to_string(),
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
                    outcome: execution_outcome,
                    artifacts: Vec::new(),
                    dimension_evidence: $crate::DimensionEvidence::Conformance(
                        conformance_evidence.clone(),
                    ),
                };

                $crate::LAST_CONFORMANCE_EVIDENCE.with(|e| {
                    *e.borrow_mut() = Some((test_id, conformance_evidence));
                });

                if result.is_err() {
                    panic!("conformance test failed");
                }
            }
        }
    };
}
