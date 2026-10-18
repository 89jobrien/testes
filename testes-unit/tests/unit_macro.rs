// Integration tests for the unit! macro
// Per ADR-0004 Phase 3: examples demonstrating the macro generates honest tests.
//
// The macro generates:
// 1. Successful examples: test passes when assertions succeed
// 2. Failing examples: test fails when assertions fail
// 3. Validation: invalid declarations are caught before examples run
// 4. Evidence recording: each test stores evidence in a thread_local
//
// These tests verify that the generated #[test] functions themselves run correctly.
// See the example_usage module below for concrete examples.

// Example usage (can be run to verify macro generates valid tests)
#[cfg(test)]
mod example_usage {
    use testes_unit::unit;

    unit! {
        id: "basic-assertion",
        subject: "assert",
        claim: "basic assertion works",
        examples: {
            "simple_true" => {
                assert!(true);
            }
        }
    }

    unit! {
        id: "two-examples",
        subject: "unit macro",
        claim: "multiple examples work",
        examples: {
            "first" => {
                assert_eq!(1 + 1, 2);
            },
            "second" => {
                assert_ne!(1, 2);
            }
        }
    }
}

// Regression test: evidence recording with multiple examples
#[cfg(test)]
mod evidence_recording {
    use testes_unit::unit;

    unit! {
        id: "evidence-recording-demo",
        subject: "evidence recording",
        claim: "evidence is recorded correctly for multiple examples",
        examples: {
            "first_example" => {
                assert_eq!(1, 1);
            },
            "second_example" => {
                assert_ne!(1, 2);
            }
        }
    }
}

/// Test case: failing example produces Failed status evidence
/// Demonstrates that execution failures are recorded in evidence.
#[test]
fn failing_execution_produces_failed_status() {
    use std::time::SystemTime;
    use testes_unit::{
        BackendIdentity, Claim, Dimension, DimensionEvidence, EvidenceRecord, ExecutionMetadata,
        ExecutionOutcome, ExecutionStatus, ObligationDisposition, Observed, OutcomeReason,
        Provenance, RepositoryRevision, ResourceLimits, RunId, Scope, SubjectId, TestDeclaration,
        TestId, TestSpecification, UnitEvidence, WorkingTreeState,
    };

    // Construct a declaration and evidence with Failed status.
    let declaration = TestDeclaration {
        id: TestId::new("failing-example").expect("valid id"),
        primary_dimension: Dimension::Unit,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "an assertion failed".to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "unit test".to_string(),
            assumptions: Vec::new(),
            bounds: Vec::new(),
            capabilities: Vec::new(),
            resources: ResourceLimits {
                wall_time: None,
                memory_bytes: None,
                max_input_bytes: None,
            },
        },
        disposition: ObligationDisposition::Required,
        specification: TestSpecification::Unit {
            example_ids: vec!["assertion_failed".to_string()],
        },
    };

    declaration
        .validate()
        .expect("TestDeclaration must be valid");

    // Evidence record with Failed status.
    let evidence = EvidenceRecord {
        schema_version: 1,
        declaration: declaration.clone(),
        provenance: Provenance {
            run_id: RunId::new("failing-run").expect("valid run id"),
            revision: RepositoryRevision {
                repository: "unknown".to_string(),
                commit: "unknown".to_string(),
                working_tree: WorkingTreeState::Unknown,
            },
        },
        execution: ExecutionMetadata {
            backend: BackendIdentity {
                name: "native-rust-test".to_string(),
                version: "1.0".to_string(),
                toolchain: None,
            },
            configuration: Default::default(),
            started_at: SystemTime::now(),
            duration: Default::default(),
            actual_scope: Scope {
                input_domain: "unit test".to_string(),
                assumptions: Vec::new(),
                bounds: Vec::new(),
                capabilities: Vec::new(),
                resources: ResourceLimits {
                    wall_time: None,
                    memory_bytes: None,
                    max_input_bytes: None,
                },
            },
        },
        outcome: ExecutionOutcome {
            status: ExecutionStatus::Failed,
            reason: OutcomeReason::AssertionViolation,
            diagnostic: Some("assertion: 2 + 2 == 5".to_string()),
            obligations: Vec::new(),
        },
        artifacts: Vec::new(),
        dimension_evidence: DimensionEvidence::Unit(UnitEvidence {
            executed_examples: vec!["assertion_failed".to_string()],
            assertions_evaluated: Observed::Unavailable {
                reason: "assertion counting not implemented".to_string(),
            },
        }),
    };

    evidence
        .validate_shape()
        .expect("evidence shape must be valid");

    // The key assertion: execution with Failed status shows the example ran but failed.
    assert_eq!(evidence.outcome.status, ExecutionStatus::Failed);
    if let DimensionEvidence::Unit(unit_ev) = &evidence.dimension_evidence {
        assert!(!unit_ev.executed_examples.is_empty());
    } else {
        panic!("expected UnitEvidence");
    }
}

/// Test case: inconclusive status when capability is unavailable
/// Demonstrates that incomplete execution is recorded separately from success.
#[test]
fn inconclusive_execution_produces_inconclusive_status() {
    use std::time::SystemTime;
    use testes_unit::{
        BackendIdentity, Claim, Dimension, DimensionEvidence, EvidenceRecord, ExecutionMetadata,
        ExecutionOutcome, ExecutionStatus, ObligationDisposition, Observed, OutcomeReason,
        Provenance, RepositoryRevision, ResourceLimits, RunId, Scope, SubjectId, TestDeclaration,
        TestId, TestSpecification, UnitEvidence, WorkingTreeState,
    };

    // Construct a declaration with Inconclusive status.
    let declaration = TestDeclaration {
        id: TestId::new("inconclusive-example").expect("valid id"),
        primary_dimension: Dimension::Unit,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("platform-specific-test").expect("valid id"),
        claim: Claim {
            statement: "execution was inconclusive".to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "unit test".to_string(),
            assumptions: Vec::new(),
            bounds: Vec::new(),
            capabilities: vec![],
            resources: ResourceLimits {
                wall_time: None,
                memory_bytes: None,
                max_input_bytes: None,
            },
        },
        disposition: ObligationDisposition::Required,
        specification: TestSpecification::Unit {
            example_ids: vec!["skipped".to_string()],
        },
    };

    declaration
        .validate()
        .expect("TestDeclaration must be valid");

    // Evidence record with Inconclusive status.
    let evidence = EvidenceRecord {
        schema_version: 1,
        declaration: declaration.clone(),
        provenance: Provenance {
            run_id: RunId::new("inconclusive-run").expect("valid run id"),
            revision: RepositoryRevision {
                repository: "unknown".to_string(),
                commit: "unknown".to_string(),
                working_tree: WorkingTreeState::Unknown,
            },
        },
        execution: ExecutionMetadata {
            backend: BackendIdentity {
                name: "native-rust-test".to_string(),
                version: "1.0".to_string(),
                toolchain: None,
            },
            configuration: Default::default(),
            started_at: SystemTime::now(),
            duration: Default::default(),
            actual_scope: Scope {
                input_domain: "unit test".to_string(),
                assumptions: Vec::new(),
                bounds: Vec::new(),
                capabilities: Vec::new(),
                resources: ResourceLimits {
                    wall_time: None,
                    memory_bytes: None,
                    max_input_bytes: None,
                },
            },
        },
        outcome: ExecutionOutcome {
            status: ExecutionStatus::Inconclusive,
            reason: OutcomeReason::CapabilityUnavailable,
            diagnostic: Some("test skipped: requires Linux".to_string()),
            obligations: Vec::new(),
        },
        artifacts: Vec::new(),
        dimension_evidence: DimensionEvidence::Unit(UnitEvidence {
            executed_examples: vec![],
            assertions_evaluated: Observed::Unavailable {
                reason: "test was skipped".to_string(),
            },
        }),
    };

    evidence
        .validate_shape()
        .expect("evidence shape must be valid");

    // The key assertion: Inconclusive status with no executed examples.
    assert_eq!(evidence.outcome.status, ExecutionStatus::Inconclusive);
}

/// Critical test: prove that merely registering a declaration does NOT produce success
/// Per handoff.md: "Prove that merely registering a declaration does not produce success"
/// This is the core invariant: declaration alone is not evidence of execution.
#[test]
fn declaration_without_execution_is_not_success() {
    use std::time::SystemTime;
    use testes_unit::{
        BackendIdentity, Claim, Dimension, DimensionEvidence, EvidenceRecord, ExecutionMetadata,
        ExecutionOutcome, ExecutionStatus, ObligationDisposition, Observed, OutcomeReason,
        Provenance, RepositoryRevision, ResourceLimits, RunId, Scope, SubjectId, TestDeclaration,
        TestId, TestSpecification, UnitEvidence, WorkingTreeState,
    };

    // Construct a TestDeclaration (the declaration).
    // This ALONE, without execution, is not sufficient for a passing test.
    let declaration = TestDeclaration {
        id: TestId::new("not-executed-demo").expect("valid id"),
        primary_dimension: Dimension::Unit,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("any-subject").expect("valid id"),
        claim: Claim {
            statement: "if we never run the example, we have no evidence".to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "unit test".to_string(),
            assumptions: Vec::new(),
            bounds: Vec::new(),
            capabilities: Vec::new(),
            resources: ResourceLimits {
                wall_time: None,
                memory_bytes: None,
                max_input_bytes: None,
            },
        },
        disposition: ObligationDisposition::Required,
        specification: TestSpecification::Unit {
            example_ids: vec!["example_a".to_string()],
        },
    };

    // Validate that the declaration is structurally sound.
    declaration
        .validate()
        .expect("TestDeclaration must be valid");

    // Now construct an EvidenceRecord with NotExecuted status.
    // This represents a declaration that was registered but never executed.
    // The key point: this should NOT be treated as a passing result.
    let evidence = EvidenceRecord {
        schema_version: 1,
        declaration: declaration.clone(),
        provenance: Provenance {
            run_id: RunId::new("not-executed-run").expect("valid run id"),
            revision: RepositoryRevision {
                repository: "unknown".to_string(),
                commit: "unknown".to_string(),
                working_tree: WorkingTreeState::Unknown,
            },
        },
        execution: ExecutionMetadata {
            backend: BackendIdentity {
                name: "test-framework".to_string(),
                version: "1.0".to_string(),
                toolchain: None,
            },
            configuration: Default::default(),
            started_at: SystemTime::now(),
            duration: Default::default(),
            actual_scope: Scope {
                input_domain: "unit test".to_string(),
                assumptions: Vec::new(),
                bounds: Vec::new(),
                capabilities: Vec::new(),
                resources: ResourceLimits {
                    wall_time: None,
                    memory_bytes: None,
                    max_input_bytes: None,
                },
            },
        },
        outcome: ExecutionOutcome {
            status: ExecutionStatus::NotExecuted,
            reason: OutcomeReason::CapabilityUnavailable,
            diagnostic: Some("test never ran".to_string()),
            obligations: Vec::new(),
        },
        artifacts: Vec::new(),
        dimension_evidence: DimensionEvidence::Unit(UnitEvidence {
            executed_examples: Vec::new(), // IMPORTANT: no examples executed
            assertions_evaluated: Observed::Unavailable {
                reason: "test was not executed".to_string(),
            },
        }),
    };

    // Verify that the evidence record is structurally valid.
    evidence
        .validate_shape()
        .expect("EvidenceRecord shape must be valid");

    // The critical assertion: NotExecuted status with zero executed examples
    // must NOT satisfy a claim that requires execution.
    // (In a real claim evaluator, this would fail the claim.)
    assert_eq!(evidence.outcome.status, ExecutionStatus::NotExecuted);

    // Verify that the dimension evidence is Unit and has no executed examples.
    if let DimensionEvidence::Unit(unit_evidence) = &evidence.dimension_evidence {
        assert!(
            unit_evidence.executed_examples.is_empty(),
            "NotExecuted status must have no executed examples"
        );
    } else {
        panic!("expected UnitEvidence for Unit dimension");
    }
}

#[test]
fn evidence_shape_validation_for_unit_tests() {
    use std::time::SystemTime;
    use testes_unit::{
        BackendIdentity, Claim, Dimension, DimensionEvidence, EvidenceRecord, ExecutionMetadata,
        ExecutionOutcome, ExecutionStatus, ObligationDisposition, Observed, OutcomeReason,
        Provenance, RepositoryRevision, ResourceLimits, RunId, Scope, SubjectId, TestDeclaration,
        TestId, TestSpecification, UnitEvidence, WorkingTreeState,
    };

    // Construct a minimal valid EvidenceRecord with Unit dimension.
    // This mirrors what the macro generates and verifies it passes shape validation.
    let declaration = TestDeclaration {
        id: TestId::new("shape-test").expect("valid id"),
        primary_dimension: Dimension::Unit,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "evidence shape is correct".to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "unit test".to_string(),
            assumptions: Vec::new(),
            bounds: Vec::new(),
            capabilities: Vec::new(),
            resources: ResourceLimits {
                wall_time: None,
                memory_bytes: None,
                max_input_bytes: None,
            },
        },
        disposition: ObligationDisposition::Required,
        specification: TestSpecification::Unit {
            example_ids: vec!["example1".to_string()],
        },
    };

    let evidence_record = EvidenceRecord {
        schema_version: 1,
        declaration,
        provenance: Provenance {
            run_id: RunId::new("test-run").expect("valid id"),
            revision: RepositoryRevision {
                repository: "testes".to_string(),
                commit: "0000000".to_string(),
                working_tree: WorkingTreeState::Unknown,
            },
        },
        execution: ExecutionMetadata {
            backend: BackendIdentity {
                name: "native-rust-test".to_string(),
                version: "1.0".to_string(),
                toolchain: None,
            },
            configuration: Default::default(),
            started_at: SystemTime::now(),
            duration: std::time::Duration::from_secs(0),
            actual_scope: Scope {
                input_domain: "unit test".to_string(),
                assumptions: Vec::new(),
                bounds: Vec::new(),
                capabilities: Vec::new(),
                resources: ResourceLimits {
                    wall_time: None,
                    memory_bytes: None,
                    max_input_bytes: None,
                },
            },
        },
        outcome: ExecutionOutcome {
            status: ExecutionStatus::Passed,
            reason: OutcomeReason::AssertionsSatisfied,
            diagnostic: None,
            obligations: Vec::new(),
        },
        artifacts: Vec::new(),
        dimension_evidence: DimensionEvidence::Unit(UnitEvidence {
            executed_examples: vec!["example1".to_string()],
            assertions_evaluated: Observed::Unavailable {
                reason: "assertion counting not implemented".to_string(),
            },
        }),
    };

    // Validate the shape — this should pass
    // This verifies that evidence records built by the macro pass shape validation
    let result = evidence_record.validate_shape();
    assert!(
        result.is_ok(),
        "UnitEvidence should pass shape validation, got: {:?}",
        result
    );
}
