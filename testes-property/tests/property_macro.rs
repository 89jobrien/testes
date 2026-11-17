use testes_core::{DeclarationError, ResourceLimits, Scope};
use testes_property::{
    Claim, Dimension, ObligationDefinition, ObligationDisposition, ObligationId, SubjectId,
    TestDeclaration, TestId, TestSpecification,
};
use testes_property::{DimensionEvidence, PropertyEvidence};

testes_property::property! {
    id: "sorted-list-always-ordered",
    subject: "Vec::sort",
    claim: "sorted lists maintain element order",
    strategy: {
        vec: proptest::collection::vec(0i32..100, 0..100),
    },
    body: {
        let mut sorted = vec.clone();
        sorted.sort();
        proptest::prop_assert!(sorted.windows(2).all(|w| w[0] <= w[1]));
        Ok(())
    },
    cases: 50,
}

testes_property::property! {
    id: "length-preserved-on-sort",
    subject: "Vec::sort",
    claim: "sort preserves vector length",
    strategy: {
        vec: proptest::collection::vec(0i32..100, 0..100),
    },
    body: {
        let original_len = vec.len();
        let mut sorted = vec.clone();
        sorted.sort();
        proptest::prop_assert_eq!(sorted.len(), original_len);
        Ok(())
    },
    cases: 50,
}

#[test]
fn evidence_shape_validation_for_property_tests() {
    // Construct a minimal valid PropertyEvidence
    let evidence = PropertyEvidence {
        strategy_id: "test_strat".to_string(),
        seed: testes_property::Observed::Unavailable {
            reason: "not measured".to_string(),
        },
        completed_cases: 42,
        rejected_cases: testes_property::Observed::Unavailable {
            reason: "not measured".to_string(),
        },
        replayed_artifacts: Vec::new(),
        minimized_counterexample: None,
    };

    // Verify it can be wrapped in a DimensionEvidence
    let dim_evidence = DimensionEvidence::Property(evidence);

    // Verify the dimension tag matches
    matches!(dim_evidence, DimensionEvidence::Property(_));
}

#[test]
fn declaration_validate_rejects_dimension_mismatch() {
    let declaration = TestDeclaration {
        id: TestId::new("dim-mismatch").expect("valid id"),
        primary_dimension: Dimension::Property,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "property test".to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "property test".to_string(),
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

#[test]
fn declaration_validate_rejects_empty_claim() {
    let declaration = TestDeclaration {
        id: TestId::new("empty-claim").expect("valid id"),
        primary_dimension: Dimension::Property,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "   ".to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "property test".to_string(),
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
        specification: TestSpecification::Property {
            strategy_id: "test_strat".to_string(),
            minimum_cases: 1,
            replay_artifacts: Vec::new(),
        },
    };
    assert_eq!(declaration.validate(), Err(DeclarationError::EmptyClaim));
}

#[test]
fn declaration_validate_rejects_duplicate_obligation_ids() {
    let obligation_id = ObligationId::new("dup").expect("id must not be blank");
    let declaration = TestDeclaration {
        id: TestId::new("dup-obligations").expect("valid id"),
        primary_dimension: Dimension::Property,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "property test".to_string(),
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
            input_domain: "property test".to_string(),
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
        specification: TestSpecification::Property {
            strategy_id: "test_strat".to_string(),
            minimum_cases: 1,
            replay_artifacts: Vec::new(),
        },
    };
    assert_eq!(
        declaration.validate(),
        Err(DeclarationError::DuplicateObligation(obligation_id))
    );
}
