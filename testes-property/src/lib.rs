//! `testes-property` owns the `property!` dimension and the `property!` macro.
//!
//! Per ADR-0005, `property!` generates property-based tests using Proptest,
//! with evidence recording. Unlike `unit!`, property tests use strategies to
//! generate inputs and record property evidence.

#![forbid(unsafe_code)]

use std::cell::RefCell;
pub use testes_core::{
    BackendConfiguration, BackendIdentity, Claim, Dimension, DimensionEvidence, EvidenceRecord,
    ExecutionMetadata, ExecutionOutcome, ExecutionStatus, IdError, ObligationDisposition,
    ObligationId, Observed, OutcomeReason, PropertyEvidence, Provenance, RepositoryRevision,
    ResourceLimits, RunId, Scope, SubjectId, TestDeclaration, TestId, TestSpecification,
    WorkingTreeState,
};

thread_local! {
    // Records the most recent evidence from a property! invocation (for test introspection).
    pub static LAST_PROPERTY_EVIDENCE: RefCell<Option<(TestId, PropertyEvidence)>> = const { RefCell::new(None) };
}

/// Declares and executes a property-based test with evidence recording.
///
/// # Grammar
///
/// ```text
/// property! {
///     id: "property-id",
///     subject: "function_name",
///     claim: "property statement",
///     strategy: {
///         var1: prop::...,
///         var2: prop::...,
///     },
///     body: {
///         // Property check using var1, var2
///         prop_assert!(var1.is_some());
///     },
///     cases: 256,
/// }
/// ```
///
/// For each property test, generates a `#[test]` function that:
/// 1. Validates the `TestDeclaration`
/// 2. Runs Proptest with the given strategies and body
/// 3. Records `PropertyEvidence` with case counts and any counterexample
/// 4. Propagates proptest failures (like unit! re-panics)
#[macro_export]
macro_rules! property {
    (
        id: $id:expr,
        subject: $subject:expr,
        claim: $claim:expr,
        strategy: {
            $($var:ident: $strat:expr),* $(,)?
        },
        body: $body:block,
        cases: $cases:expr
        $(,)?
    ) => {
        ::pastey::paste! {
            #[test]
            fn [< $id >]() {
                use proptest::test_runner::TestRunner;
                use proptest::strategy::Strategy;

                let test_id = $crate::TestId::new($id)
                    .expect("test id must not be blank or contain control characters");
                let subject_id = $crate::SubjectId::new($subject)
                    .expect("subject must not be blank or contain control characters");

                let strategy_id = stringify!($($var),*);

                // Construct the declaration.
                let declaration = $crate::TestDeclaration {
                    id: test_id.clone(),
                    primary_dimension: $crate::Dimension::Property,
                    secondary_dimensions: Vec::new(),
                    subject: subject_id,
                    claim: $crate::Claim {
                        statement: $claim.to_string(),
                        obligations: Vec::new(),
                    },
                    required_scope: $crate::Scope {
                        input_domain: "property test".to_string(),
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
                    specification: $crate::TestSpecification::Property {
                        strategy_id: strategy_id.to_string(),
                        minimum_cases: $cases,
                        replay_artifacts: Vec::new(),
                    },
                };

                // Validate the declaration.
                declaration.validate().expect("TestDeclaration must be valid");

                // Track case count with atomic
                let case_count = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
                let started_at = std::time::SystemTime::now();

                // Build combined strategy and run
                let test_result = {
                    let case_count_clone = case_count.clone();
                    let mut runner = TestRunner::default();
                    runner.run(&($($strat),*), |($($var),*)| {
                        case_count_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        $body
                    })
                };
                let duration = started_at.elapsed().unwrap_or(std::time::Duration::ZERO);
                let case_count_value = case_count.load(std::sync::atomic::Ordering::Relaxed);

                // Record the outcome
                let (outcome_status, diagnostic) = match &test_result {
                    Ok(()) => ($crate::ExecutionStatus::Passed, None),
                    Err(e) => (
                        $crate::ExecutionStatus::Failed,
                        Some(format!("{:?}", e)),
                    ),
                };

                // Build the evidence
                let property_evidence = $crate::PropertyEvidence {
                    strategy_id: strategy_id.to_string(),
                    seed: $crate::Observed::Unavailable {
                        reason: "seed tracking not implemented".to_string(),
                    },
                    completed_cases: case_count_value,
                    rejected_cases: $crate::Observed::Unavailable {
                        reason: "rejection tracking not implemented".to_string(),
                    },
                    replayed_artifacts: Vec::new(),
                    minimized_counterexample: None,
                };

                let execution_outcome = $crate::ExecutionOutcome {
                    status: outcome_status,
                    reason: if outcome_status == $crate::ExecutionStatus::Passed {
                        $crate::OutcomeReason::AssertionsSatisfied
                    } else {
                        $crate::OutcomeReason::CounterexampleFound
                    },
                    diagnostic,
                    obligations: Vec::new(),
                };

                let evidence_record = $crate::EvidenceRecord {
                    schema_version: 1,
                    declaration,
                    provenance: $crate::Provenance {
                        run_id: $crate::RunId::new("property-test-run")
                            .expect("run id valid"),
                        revision: $crate::RepositoryRevision {
                            repository: "unknown".to_string(),
                            commit: "unknown".to_string(),
                            working_tree: $crate::WorkingTreeState::Unknown,
                        },
                    },
                    execution: $crate::ExecutionMetadata {
                        backend: $crate::BackendIdentity {
                            name: "proptest".to_string(),
                            version: env!("CARGO_PKG_VERSION").to_string(),
                            toolchain: None,
                        },
                        configuration: $crate::BackendConfiguration::new(),
                        started_at,
                        duration,
                        actual_scope: $crate::Scope {
                            input_domain: "property test".to_string(),
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
                    dimension_evidence: $crate::DimensionEvidence::Property(property_evidence.clone()),
                };

                // Store for introspection.
                $crate::LAST_PROPERTY_EVIDENCE.with(|e| {
                    *e.borrow_mut() = Some((test_id, property_evidence));
                });

                // Propagate proptest result (panic if it failed)
                test_result.expect("property test failed");
            }
        }
    };
}
