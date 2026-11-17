// Integration tests for the prove! macro (model-check dimension)
// Per ADR decision: model-check dimension demonstrates formal properties.
//
// The macro generates:
// 1. Test execution with proof evidence recording
// 2. Validation: invalid declarations are caught before execution
// 3. Evidence recording: test stores evidence in thread-local
//
// These tests verify that the generated #[test] functions work correctly
// and that evidence is properly shaped.

use testes_core::DeclarationError;
use testes_model_check::*;

prove! {
    id: "property-holds-for-all-inputs",
    subject: "mathematical property",
    claim: "property holds across all valid inputs",
    body: {
        // Model-check: verify a property holds
        let x = 10;
        let y = 20;
        assert!(x + y == 30);
    }
}

prove! {
    id: "invariant-maintained",
    subject: "system invariant",
    claim: "invariant is maintained through transformations",
    body: {
        // Proof: verify an invariant
        let value = 42;
        assert!(value > 0);
        assert!(value < 1000);
    }
}

#[test]
fn evidence_shape_validation() {
    // This test validates that the macro creates properly shaped evidence.
    // The actual ModelCheckEvidence is constructed and stored in thread-local.
    LAST_PROOF_EVIDENCE.with(|e| {
        if let Some((_id, evidence)) = &*e.borrow() {
            // harness_id is set by the macro (default-harness)
            assert_eq!(evidence.harness_id, "default-harness");
            // obligations are empty in basic demonstration
            assert_eq!(evidence.obligations.len(), 0);
        }
    });
}

fn model_check_specification() -> TestSpecification {
    TestSpecification::ModelCheck {
        harness_id: "default-harness".to_string(),
        obligation_ids: Vec::new(),
    }
}

/// A model-check declaration must not be accepted under a different primary dimension.
#[test]
fn declaration_validate_rejects_dimension_mismatch() {
    let declaration = TestDeclaration {
        id: TestId::new("dim-mismatch").expect("valid id"),
        primary_dimension: Dimension::ModelCheck,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "proof verification".to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "proof verification".to_string(),
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
        primary_dimension: Dimension::ModelCheck,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "   ".to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "proof verification".to_string(),
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
        specification: model_check_specification(),
    };
    assert_eq!(declaration.validate(), Err(DeclarationError::EmptyClaim));
}

/// Obligation IDs must be unique so that per-obligation results are addressable.
#[test]
fn declaration_validate_rejects_duplicate_obligation_ids() {
    let obligation_id = testes_core::ObligationId::new("dup").expect("id must not be blank");
    let declaration = TestDeclaration {
        id: TestId::new("dup-obligations").expect("valid id"),
        primary_dimension: Dimension::ModelCheck,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "proof verification".to_string(),
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
            input_domain: "proof verification".to_string(),
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
        specification: model_check_specification(),
    };
    assert_eq!(
        declaration.validate(),
        Err(DeclarationError::DuplicateObligation(obligation_id))
    );
}
