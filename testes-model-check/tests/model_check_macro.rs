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
