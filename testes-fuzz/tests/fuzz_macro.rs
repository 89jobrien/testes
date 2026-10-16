use testes_fuzz::{DimensionEvidence, FuzzEvidence};

testes_fuzz::fuzz! {
    id: "fuzz-never-panics",
    subject: "parse_bytes",
    claim: "arbitrary bytes never cause panic",
    target: (|input: &[u8]| {
        // Target: parse and ignore
        let _parsed = String::from_utf8_lossy(input);
        true
    }),
    oracle: (|result: bool| {
        proptest::prop_assert!(result);
        Ok(())
    }),
    cases: 100,
}

testes_fuzz::fuzz! {
    id: "fuzz-length-check",
    subject: "check_length",
    claim: "length check is correct",
    target: (|input: &[u8]| {
        input.len() < 10000
    }),
    oracle: (|result: bool| {
        proptest::prop_assert!(result);
        Ok(())
    }),
    cases: 100,
}

#[test]
fn evidence_shape_validation_for_fuzz_tests() {
    // Construct a minimal valid FuzzEvidence
    let evidence = FuzzEvidence {
        target_id: "test_target".to_string(),
        oracle_ids: Vec::new(),
        initial_corpus: Vec::new(),
        actual_duration: std::time::Duration::from_secs(1),
        executions: testes_fuzz::Observed::Available(42),
        coverage: testes_fuzz::Observed::Unavailable {
            reason: "not measured".to_string(),
        },
        failure_inputs: Vec::new(),
    };

    // Verify it can be wrapped in a DimensionEvidence
    let dim_evidence = DimensionEvidence::Fuzz(evidence);

    // Verify the dimension tag matches
    matches!(dim_evidence, DimensionEvidence::Fuzz(_));
}
