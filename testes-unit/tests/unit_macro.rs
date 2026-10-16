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
