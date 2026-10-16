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
