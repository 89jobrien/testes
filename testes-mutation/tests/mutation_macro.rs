use testes_mutation::*;

mutate! {
    id: "check-detects-mutations",
    subject: "mutation score",
    claim: "the unit check detects introduced mutations",
    body: {
        let value = 42;
        assert!(value > 0);
    }
}

#[test]
fn evidence_shape_validation() {
    LAST_MUTATION_EVIDENCE.with(|e| {
        if let Some((_id, evidence)) = &*e.borrow() {
            assert_eq!(evidence.source_selectors.len(), 0);
            assert_eq!(evidence.operators.len(), 0);
            assert_eq!(evidence.selected_checks.len(), 0);
            assert_eq!(evidence.mutations.len(), 0);
        }
    });
}
