//! `testes-mutation` owns the `mutate!` macro.
#![forbid(unsafe_code)]
use std::cell::RefCell;
pub use testes_core::{
    BackendConfiguration, BackendIdentity, Claim, CommandSpec, Dimension, DimensionEvidence,
    EvidenceRecord, ExecutionMetadata, ExecutionOutcome, ExecutionStatus, IdError,
    MutationEvidence, ObligationDisposition, OutcomeReason, Provenance, RepositoryRevision,
    ResourceLimits, RunId, Scope, SubjectId, TestDeclaration, TestId, TestSpecification,
    WorkingTreeState,
};

thread_local! {
    pub static LAST_MUTATION_EVIDENCE: RefCell<Option<(TestId, MutationEvidence)>> =
        const { RefCell::new(None) };
}

#[macro_export]
macro_rules! mutate {
    (id: $id:expr, subject: $subject:expr, claim: $claim:expr, body: $body:block $(,)?) => {
        ::paste::paste! {
            #[test]
            fn [< $id >]() {
                let test_id = $crate::TestId::new($id).expect("valid id");
                let declaration = $crate::TestDeclaration {
                    id: test_id.clone(),
                    primary_dimension: $crate::Dimension::Mutation,
                    secondary_dimensions: Vec::new(),
                    subject: $crate::SubjectId::new($subject).expect("valid subject"),
                    claim: $crate::Claim {
                        statement: $claim.to_string(),
                        obligations: Vec::new(),
                    },
                    required_scope: $crate::Scope {
                        input_domain: "mutation test".to_string(),
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
                    specification: $crate::TestSpecification::Mutation {
                        source_selectors: vec![],
                        operators: vec![],
                        selected_checks: vec![],
                        commands: vec![],
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

                let baseline = $crate::ExecutionOutcome {
                    status,
                    reason: if status == $crate::ExecutionStatus::Passed {
                        $crate::OutcomeReason::AssertionsSatisfied
                    } else {
                        $crate::OutcomeReason::AssertionViolation
                    },
                    diagnostic: None,
                    obligations: Vec::new(),
                };

                let evidence = $crate::MutationEvidence {
                    source_selectors: vec![],
                    operators: vec![],
                    selected_checks: vec![],
                    commands: vec![],
                    baseline,
                    mutations: vec![],
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
                        run_id: $crate::RunId::new("mutation-run").expect("valid"),
                        revision: $crate::RepositoryRevision {
                            repository: "unknown".to_string(),
                            commit: "unknown".to_string(),
                            working_tree: $crate::WorkingTreeState::Unknown,
                        },
                    },
                    execution: $crate::ExecutionMetadata {
                        backend: $crate::BackendIdentity {
                            name: "native-mutation".to_string(),
                            version: env!("CARGO_PKG_VERSION").to_string(),
                            toolchain: None,
                        },
                        configuration: $crate::BackendConfiguration::new(),
                        started_at,
                        duration,
                        actual_scope: $crate::Scope {
                            input_domain: "mutation test".to_string(),
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
                    dimension_evidence: $crate::DimensionEvidence::Mutation(evidence.clone()),
                };

                $crate::LAST_MUTATION_EVIDENCE.with(|e| {
                    *e.borrow_mut() = Some((test_id, evidence));
                });

                if result.is_err() {
                    panic!("mutation test failed");
                }
            }
        }
    };
}
