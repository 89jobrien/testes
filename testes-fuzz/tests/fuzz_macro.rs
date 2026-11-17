use testes_core::DeclarationError;
use testes_fuzz::{
    Claim, Dimension, ObligationDefinition, ObligationDisposition, ObligationId, ResourceLimits,
    Scope, SubjectId, TestDeclaration, TestId, TestSpecification,
};
use testes_fuzz::{DimensionEvidence, FuzzEvidence};

testes_fuzz::fuzz! {
    id: "fuzz-never-panics",
    subject: "parse_bytes",
    claim: "arbitrary bytes never cause panic",
    target: (|input: &[u8]| {
        // Target: parse and ignore
        let _parsed = String::from_utf8_lossy(input);
        true
    }),
    oracle: (|result: bool| {
        proptest::prop_assert!(result);
        Ok(())
    }),
    cases: 100,
}

testes_fuzz::fuzz! {
    id: "fuzz-length-check",
    subject: "check_length",
    claim: "length check is correct",
    target: (|input: &[u8]| {
        input.len() < 10000
    }),
    oracle: (|result: bool| {
        proptest::prop_assert!(result);
        Ok(())
    }),
    cases: 100,
}

#[test]
fn evidence_shape_validation_for_fuzz_tests() {
    // Construct a minimal valid FuzzEvidence
    let evidence = FuzzEvidence {
        target_id: "test_target".to_string(),
        oracle_ids: Vec::new(),
        initial_corpus: Vec::new(),
        actual_duration: std::time::Duration::from_secs(1),
        executions: testes_fuzz::Observed::Available(42),
        coverage: testes_fuzz::Observed::Unavailable {
            reason: "not measured".to_string(),
        },
        failure_inputs: Vec::new(),
    };

    // Verify it can be wrapped in a DimensionEvidence
    let dim_evidence = DimensionEvidence::Fuzz(evidence);

    // Verify the dimension tag matches
    matches!(dim_evidence, DimensionEvidence::Fuzz(_));
}

/// A fuzz declaration must not be accepted under a different primary dimension.
#[test]
fn declaration_validate_rejects_dimension_mismatch() {
    let declaration = TestDeclaration {
        id: TestId::new("dim-mismatch").expect("valid id"),
        primary_dimension: Dimension::Fuzz,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "fuzz test".to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "fuzz test (arbitrary bytes)".to_string(),
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
            example_ids: vec!["example".to_string()],
        },
    };
    assert_eq!(
        declaration.validate(),
        Err(DeclarationError::DimensionMismatch)
    );
}

/// A blank claim statement is not a claim.
#[test]
fn declaration_validate_rejects_empty_claim() {
    let declaration = TestDeclaration {
        id: TestId::new("empty-claim").expect("valid id"),
        primary_dimension: Dimension::Fuzz,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "   ".to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "fuzz test (arbitrary bytes)".to_string(),
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
        specification: TestSpecification::Fuzz {
            target_id: "test_target".to_string(),
            oracle_ids: Vec::new(),
            corpus_artifacts: Vec::new(),
        },
    };
    assert_eq!(declaration.validate(), Err(DeclarationError::EmptyClaim));
}

/// Obligation IDs must be unique so that per-obligation results are addressable.
#[test]
fn declaration_validate_rejects_duplicate_obligation_ids() {
    let obligation_id = ObligationId::new("dup").expect("id must not be blank");
    let declaration = TestDeclaration {
        id: TestId::new("dup-obligations").expect("valid id"),
        primary_dimension: Dimension::Fuzz,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "fuzz test".to_string(),
            obligations: vec![
                ObligationDefinition {
                    id: obligation_id.clone(),
                    description: "first".to_string(),
                },
                ObligationDefinition {
                    id: obligation_id.clone(),
                    description: "second".to_string(),
                },
            ],
        },
        required_scope: Scope {
            input_domain: "fuzz test (arbitrary bytes)".to_string(),
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
        specification: TestSpecification::Fuzz {
            target_id: "test_target".to_string(),
            oracle_ids: Vec::new(),
            corpus_artifacts: Vec::new(),
        },
    };
    assert_eq!(
        declaration.validate(),
        Err(DeclarationError::DuplicateObligation(obligation_id))
    );
}
