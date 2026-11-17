use testes_core::DeclarationError;
use testes_regression::*;

regression! {
    id: "bug-12-recheck",
    subject: "regression test",
    claim: "the previously fixed bug does not reoccur",
    body: {
        let value = 42;
        assert!(value > 0);
    }
}

#[test]
fn evidence_shape_validation() {
    LAST_REGRESSION_EVIDENCE.with(|e| {
        if let Some((_id, evidence)) = &*e.borrow() {
            assert_eq!(evidence.reproducer_artifacts.len(), 0);
            assert!(evidence.historical_failure.is_none());
        }
    });
}

fn regression_specification(underlying_dimension: Dimension) -> TestSpecification {
    TestSpecification::Regression {
        failure: FailureId::new("failure").expect("valid id"),
        underlying_dimension,
        reproducer_artifacts: Vec::new(),
    }
}

/// A regression declaration must not be accepted under a different primary dimension.
#[test]
fn declaration_validate_rejects_dimension_mismatch() {
    let declaration = TestDeclaration {
        id: TestId::new("dim-mismatch").expect("valid id"),
        primary_dimension: Dimension::Regression,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "regression test".to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "regression test".to_string(),
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
        primary_dimension: Dimension::Regression,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "   ".to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "regression test".to_string(),
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
        specification: regression_specification(Dimension::Unit),
    };
    assert_eq!(declaration.validate(), Err(DeclarationError::EmptyClaim));
}

/// Obligation IDs must be unique so that per-obligation results are addressable.
#[test]
fn declaration_validate_rejects_duplicate_obligation_ids() {
    let obligation_id = ObligationId::new("dup").expect("id must not be blank");
    let declaration = TestDeclaration {
        id: TestId::new("dup-obligations").expect("valid id"),
        primary_dimension: Dimension::Regression,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "regression test".to_string(),
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
            input_domain: "regression test".to_string(),
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
        specification: regression_specification(Dimension::Unit),
    };
    assert_eq!(
        declaration.validate(),
        Err(DeclarationError::DuplicateObligation(obligation_id))
    );
}

/// Regression is a purpose layered over another dimension. It cannot wrap
/// itself, and it cannot wrap mutation (mutation is cross-cutting and
/// already evaluates the checks a regression would re-run).
#[test]
fn declaration_validate_rejects_invalid_underlying_dimension() {
    for underlying in [Dimension::Regression, Dimension::Mutation] {
        let declaration = TestDeclaration {
            id: TestId::new("invalid-underlying").expect("valid id"),
            primary_dimension: Dimension::Regression,
            secondary_dimensions: Vec::new(),
            subject: SubjectId::new("test-subject").expect("valid id"),
            claim: Claim {
                statement: "regression test".to_string(),
                obligations: Vec::new(),
            },
            required_scope: Scope {
                input_domain: "regression test".to_string(),
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
            specification: regression_specification(underlying),
        };
        assert_eq!(
            declaration.validate(),
            Err(DeclarationError::InvalidRegressionDimension),
            "underlying dimension {underlying:?} must be rejected"
        );
    }
}
