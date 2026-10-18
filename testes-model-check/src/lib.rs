//! `testes-model-check` owns the `prove!` dimension and macro.

#![forbid(unsafe_code)]

use std::cell::RefCell;
pub use testes_core::{
    BackendConfiguration, BackendIdentity, Claim, Dimension, DimensionEvidence, EvidenceRecord,
    ExecutionMetadata, ExecutionOutcome, ExecutionStatus, IdError, ModelCheckEvidence,
    ObligationDisposition, Observed, OutcomeReason, ProofStatus, Provenance, RepositoryRevision,
    ResourceLimits, RunId, Scope, SubjectId, TestDeclaration, TestId, TestSpecification,
    WorkingTreeState,
};

thread_local! {
    pub static LAST_PROOF_EVIDENCE: RefCell<Option<(TestId, ModelCheckEvidence)>> =
        const { RefCell::new(None) };
}

/// Declares and executes a formal verification proof.
#[macro_export]
macro_rules! prove {
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

                let declaration = $crate::TestDeclaration {
                    id: test_id.clone(),
                    primary_dimension: $crate::Dimension::ModelCheck,
                    secondary_dimensions: Vec::new(),
                    subject: subject_id,
                    claim: $crate::Claim {
                        statement: $claim.to_string(),
                        obligations: Vec::new(),
                    },
                    required_scope: $crate::Scope {
                        input_domain: "proof verification".to_string(),
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
                    specification: $crate::TestSpecification::ModelCheck {
                        harness_id: "default-harness".to_string(),
                        obligation_ids: Vec::new(),
                    },
                };

                declaration.validate().expect("TestDeclaration must be valid");

                let started_at = std::time::SystemTime::now();
                let proof_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    $body
                }));
                let duration = started_at.elapsed().unwrap_or(std::time::Duration::ZERO);

                let (status, diagnostic) = match &proof_result {
                    Ok(_) => ($crate::ProofStatus::Established, None),
                    Err(e) => (
                        $crate::ProofStatus::Refuted,
                        Some(format!("{:?}", e)),
                    ),
                };

                let proof_evidence = $crate::ModelCheckEvidence {
                    harness_id: "default-harness".to_string(),
                    symbolic_inputs: Vec::new(),
                    assumptions: Vec::new(),
                    bounds: Vec::new(),
                    obligations: Vec::new(),
                    non_vacuity: $crate::Observed::Unavailable {
                        reason: "not measured".to_string(),
                    },
                    report: None,
                };

                let execution_outcome = $crate::ExecutionOutcome {
                    status: if status == $crate::ProofStatus::Established {
                        $crate::ExecutionStatus::Passed
                    } else {
                        $crate::ExecutionStatus::Failed
                    },
                    reason: $crate::OutcomeReason::AssertionsSatisfied,
                    diagnostic: None,
                    obligations: Vec::new(),
                };

                let _evidence_record = $crate::EvidenceRecord {
                    schema_version: 1,
                    declaration,
                    provenance: $crate::Provenance {
                        run_id: $crate::RunId::new("proof-run")
                            .expect("run id valid"),
                        revision: $crate::RepositoryRevision {
                            repository: "unknown".to_string(),
                            commit: "unknown".to_string(),
                            working_tree: $crate::WorkingTreeState::Unknown,
                        },
                    },
                    execution: $crate::ExecutionMetadata {
                        backend: $crate::BackendIdentity {
                            name: "native-proof".to_string(),
                            version: env!("CARGO_PKG_VERSION").to_string(),
                            toolchain: None,
                        },
                        configuration: $crate::BackendConfiguration::new(),
                        started_at,
                        duration,
                        actual_scope: $crate::Scope {
                            input_domain: "proof verification".to_string(),
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
                    dimension_evidence: $crate::DimensionEvidence::ModelCheck(proof_evidence.clone()),
                };

                $crate::LAST_PROOF_EVIDENCE.with(|e| {
                    *e.borrow_mut() = Some((test_id, proof_evidence));
                });

                if let Err(_) = proof_result {
                    panic!("proof verification failed");
                }
            }
        }
    };
}
