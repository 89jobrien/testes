//! `testes-unit` owns the `unit!` dimension and the `unit!` macro.
//!
//! Per ADR-0002, `unit!` generates native Rust tests with evidence recording — no
//! `TestBackend` trait needed (native execution is the only implementation). Each
//! `#[test]` function is its own composition root, constructing a `TestDeclaration`,
//! validating it, running examples with panic-safety, and recording `UnitEvidence`.
//!
//! Grammar per ADR-0004: see the `unit!` macro docs below.

#![forbid(unsafe_code)]

use std::cell::RefCell;
pub use testes_core::{
    BackendConfiguration, BackendIdentity, Claim, Dimension, DimensionEvidence, EvidenceRecord,
    ExecutionMetadata, ExecutionOutcome, ExecutionStatus, IdError, ObligationDisposition,
    ObligationId, Observed, OutcomeReason, Provenance, RepositoryRevision, ResourceLimits, RunId,
    Scope, SubjectId, TestDeclaration, TestId, TestSpecification, UnitEvidence, WorkingTreeState,
};

thread_local! {
    // Records the most recent evidence from a unit! invocation (for test introspection).
    pub static LAST_UNIT_EVIDENCE: RefCell<Option<(TestId, UnitEvidence)>> = const { RefCell::new(None) };
}

/// Declares and executes a unit test with evidence recording.
///
/// # Grammar
///
/// ```text
/// unit! {
///     id: "test-case-id",
///     subject: "TestSubject::method",
///     claim: "TestSubject::method does X",
///     examples: {
///         "example_name": {
///             // Example body: assertions and setup
///             assert!(condition);
///         }
///     }
/// }
/// ```
///
/// For each example, generates an `#[test] fn {id}_{example_name}()` that:
/// 1. Validates the `TestDeclaration`
/// 2. Runs the example body in panic-safe mode
/// 3. Records `UnitEvidence` with outcome
/// 4. Re-panics if the example failed (preserves cargo test reporting)
#[macro_export]
macro_rules! unit {
    (
        id: $id:expr,
        subject: $subject:expr,
        claim: $claim:expr,
        examples: {
            $($example_name:expr => {
                $($body:tt)*
            }),* $(,)?
        }
    ) => {
        $(
            $crate::unit_example!($id, $subject, $claim, $example_name, { $($body)* });
        )*
    };
}

/// Helper macro that generates a single `#[test]` function for one example.
#[doc(hidden)]
#[macro_export]
macro_rules! unit_example {
    ($id:tt, $subject:tt, $claim:tt, $example_name:tt, $body:block) => {
        ::pastey::paste! {
            #[test]
            fn [< $id _ $example_name >]() {
                use $crate::LAST_UNIT_EVIDENCE;

                // Parse IDs (these will panic if invalid; that's intentional for test failures).
                let test_id = $crate::TestId::new($id)
                    .expect("test id must not be blank or contain control characters");
                let subject_id = $crate::SubjectId::new($subject)
                    .expect("subject must not be blank or contain control characters");

                // Construct the declaration.
                let declaration = $crate::TestDeclaration {
                    id: test_id.clone(),
                    primary_dimension: $crate::Dimension::Unit,
                    secondary_dimensions: Vec::new(),
                    subject: subject_id,
                    claim: $crate::Claim {
                        statement: $claim.to_string(),
                        obligations: Vec::new(),
                    },
                    required_scope: $crate::Scope {
                        input_domain: "unit test".to_string(),
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
                    specification: $crate::TestSpecification::Unit {
                        example_ids: vec![$example_name.to_string()],
                    },
                };

                // Validate the declaration.
                declaration.validate().expect("TestDeclaration must be valid");

                // Run the example in panic-safe mode.
                let started_at = std::time::SystemTime::now();
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    $body
                }));
                let duration = started_at.elapsed().unwrap_or(std::time::Duration::ZERO);

                // Record the outcome.
                let (outcome_status, diagnostic) = match &result {
                    Ok(_) => ($crate::ExecutionStatus::Passed, None),
                    Err(e) => (
                        $crate::ExecutionStatus::Failed,
                        Some(format!("{:?}", e)),
                    ),
                };

                // Build the evidence.
                let unit_evidence = $crate::UnitEvidence {
                    executed_examples: vec![$example_name.to_string()],
                    assertions_evaluated: $crate::Observed::Unavailable {
                        reason: "assertion counting not implemented".to_string(),
                    },
                };

                let execution_outcome = $crate::ExecutionOutcome {
                    status: outcome_status,
                    reason: if outcome_status == $crate::ExecutionStatus::Passed {
                        $crate::OutcomeReason::AssertionsSatisfied
                    } else {
                        $crate::OutcomeReason::AssertionViolation
                    },
                    diagnostic,
                    obligations: Vec::new(),
                };

                let evidence_record = $crate::EvidenceRecord {
                    schema_version: 1,
                    declaration,
                    provenance: $crate::Provenance {
                        run_id: $crate::RunId::new("unit-test-run")
                            .expect("run id valid"),
                        revision: $crate::RepositoryRevision {
                            repository: "unknown".to_string(),
                            commit: "unknown".to_string(),
                            working_tree: $crate::WorkingTreeState::Unknown,
                        },
                    },
                    execution: $crate::ExecutionMetadata {
                        backend: $crate::BackendIdentity {
                            name: "native-rust-test".to_string(),
                            version: "1.0".to_string(),
                            toolchain: None,
                        },
                        configuration: $crate::BackendConfiguration::new(),
                        started_at,
                        duration,
                        actual_scope: $crate::Scope {
                            input_domain: "unit test".to_string(),
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
                    dimension_evidence: $crate::DimensionEvidence::Unit(unit_evidence.clone()),
                };

                // Store for introspection.
                LAST_UNIT_EVIDENCE.with(|e| {
                    *e.borrow_mut() = Some((test_id, unit_evidence));
                });

                // Re-panic if the test failed, preserving cargo test's own reporting.
                if result.is_err() {
                    panic!("Example `{}` failed", $example_name);
                }
            }
        }
    };
}
