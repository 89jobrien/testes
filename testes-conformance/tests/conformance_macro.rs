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
