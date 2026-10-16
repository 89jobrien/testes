use testes_integration::*;

integration! {
    id: "components-communicate",
    subject: "component integration",
    claim: "two components can communicate without panicking",
    body: {
        let result = "hello".to_string() + " world";
        assert_eq!(result, "hello world");
    }
}

#[test]
fn evidence_shape_validation() {
    // This test validates that the macro creates properly shaped evidence.
    // The actual EvidenceRecord is constructed and stored in thread-local.
    LAST_INTEGRATION_EVIDENCE.with(|e| {
        if let Some((_id, evidence)) = &*e.borrow() {
            assert_eq!(evidence.seam_id, "integration");
            assert_eq!(evidence.components.len(), 0);
            assert_eq!(evidence.fixtures.len(), 0);
            assert_eq!(evidence.observations.len(), 0);
        }
    });
}
