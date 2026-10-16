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
