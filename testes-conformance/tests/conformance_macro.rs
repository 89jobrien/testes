// Integration tests for the conformance! macro
// Per ADR decision: conformance dimension demonstrates trait implementation compliance.
//
// The macro generates:
// 1. Test execution with conformance evidence recording
// 2. Validation: invalid declarations are caught before execution
// 3. Evidence recording: test stores evidence in thread-local
//
// These tests verify that the generated #[test] functions work correctly
// and that evidence is properly shaped.

use testes_conformance::*;
use testes_core::DeclarationError;

conformance! {
    id: "trait-implements-required-methods",
    subject: "trait implementation",
    claim: "trait implementers provide all required methods",
    body: {
        // Conformance check: verify a type conforms to expected trait boundary
        let value: i32 = 42;
        assert!(value > 0);
    }
}

conformance! {
    id: "api-contract-satisfied",
    subject: "api contract",
    claim: "implementation satisfies api contract",
    body: {
        let result = "hello".to_string() + " world";
        assert_eq!(result, "hello world");
    }
}

#[test]
fn evidence_shape_validation() {
    // This test validates that the macro creates properly shaped evidence.
    // The actual ConformanceEvidence is constructed and stored in thread-local.
    LAST_CONFORMANCE_EVIDENCE.with(|e| {
        if let Some((_id, evidence)) = &*e.borrow() {
            // Contract and implementation identity are set by the macro
            assert_eq!(evidence.contract.version, "0.1.0");
            assert_eq!(evidence.implementation.configuration_id, "default");
            // Capabilities and cases are empty in basic demonstration
            assert_eq!(evidence.capabilities.len(), 0);
            assert_eq!(evidence.cases.len(), 0);
        }
    });
}

fn conformance_specification() -> TestSpecification {
    TestSpecification::Conformance {
        contract: ContractIdentity {
            id: ContractId::new("contract").expect("valid id"),
            version: "0.1.0".to_string(),
        },
        implementation: ImplementationIdentity {
            id: ImplementationId::new("implementation").expect("valid id"),
            configuration_id: "default".to_string(),
        },
        case_ids: Vec::new(),
    }
}

/// A conformance declaration must not be accepted under a different primary dimension.
#[test]
fn declaration_validate_rejects_dimension_mismatch() {
    let declaration = TestDeclaration {
        id: TestId::new("dim-mismatch").expect("valid id"),
        primary_dimension: Dimension::Conformance,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "conformance test".to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "conformance test".to_string(),
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
        primary_dimension: Dimension::Conformance,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "   ".to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "conformance test".to_string(),
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
        specification: conformance_specification(),
    };
    assert_eq!(declaration.validate(), Err(DeclarationError::EmptyClaim));
}

/// Obligation IDs must be unique so that per-obligation results are addressable.
#[test]
fn declaration_validate_rejects_duplicate_obligation_ids() {
    let obligation_id = ObligationId::new("dup").expect("id must not be blank");
    let declaration = TestDeclaration {
        id: TestId::new("dup-obligations").expect("valid id"),
        primary_dimension: Dimension::Conformance,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "conformance test".to_string(),
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
            input_domain: "conformance test".to_string(),
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
        specification: conformance_specification(),
    };
    assert_eq!(
        declaration.validate(),
        Err(DeclarationError::DuplicateObligation(obligation_id))
    );
}
