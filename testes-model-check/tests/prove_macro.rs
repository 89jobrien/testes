use testes_model_check::{
    BackendIdentity, ExecutionStatus, ModelCheckBackend, ModelCheckRun, ObligationId,
    OutcomeReason, ProofObligationResult, ProofStatus, prove,
};

/// In-memory backend double. It reports whatever run it was built with.
struct DoubleBackend {
    run: ModelCheckRun,
}

impl ModelCheckBackend for DoubleBackend {
    fn identity(&self) -> BackendIdentity {
        BackendIdentity {
            name: "double".to_string(),
            version: "0.0.0".to_string(),
            toolchain: None,
        }
    }

    fn check(&self, _harness_id: &str, _obligations: &[ObligationId]) -> ModelCheckRun {
        self.run.clone()
    }
}

fn established(id: &str) -> ProofObligationResult {
    ProofObligationResult {
        id: ObligationId::new(id).expect("obligation id must not be blank"),
        status: ProofStatus::Established,
        diagnostic: None,
        counterexample: None,
    }
}

#[test]
fn prove_macro_records_passing_proof() {
    let backend = DoubleBackend {
        run: ModelCheckRun::Executed(vec![established("no-panic")]),
    };
    let record = testes_model_check::prove! {
        id: "parser-total",
        subject: "parse",
        claim: "parse never panics",
        harness: "parse_harness",
        backend: backend,
        obligations: {
            "no-panic" => "parse returns without panicking",
        }
    };
    assert_eq!(record.outcome.status, ExecutionStatus::Passed);
    assert_eq!(record.validate_shape(), Ok(()));
}

#[test]
fn prove_macro_records_unavailable_backend_as_not_executed() {
    let backend = DoubleBackend {
        run: ModelCheckRun::Unavailable {
            reason: "verifier not installed".to_string(),
        },
    };
    let record = testes_model_check::prove! {
        id: "parser-total",
        subject: "parse",
        claim: "parse never panics",
        harness: "parse_harness",
        backend: backend,
        obligations: {
            "no-panic" => "parse returns without panicking",
        }
    };
    assert_eq!(record.outcome.status, ExecutionStatus::NotExecuted);
    assert_eq!(record.outcome.reason, OutcomeReason::CapabilityUnavailable);
    assert_eq!(record.validate_shape(), Ok(()));
}

#[test]
fn prove_macro_records_refuted_proof_as_failed() {
    let backend = DoubleBackend {
        run: ModelCheckRun::Executed(vec![ProofObligationResult {
            id: ObligationId::new("no-panic").expect("obligation id must not be blank"),
            status: ProofStatus::Refuted,
            diagnostic: Some("panic on empty input".to_string()),
            counterexample: None,
        }]),
    };
    let record = testes_model_check::prove! {
        id: "parser-total",
        subject: "parse",
        claim: "parse never panics",
        harness: "parse_harness",
        backend: backend,
        obligations: {
            "no-panic" => "parse returns without panicking",
        }
    };
    assert_eq!(record.outcome.status, ExecutionStatus::Failed);
    assert_eq!(record.validate_shape(), Ok(()));
}

#[test]
fn prove_function_is_reexported_for_direct_use() {
    let backend = DoubleBackend {
        run: ModelCheckRun::Executed(Vec::new()),
    };
    let spec = testes_model_check::ProveSpec {
        id: testes_model_check::TestId::new("empty").expect("id must not be blank"),
        subject: testes_model_check::SubjectId::new("parse").expect("subject must not be blank"),
        claim: "nothing declared".to_string(),
        harness_id: "parse_harness".to_string(),
        obligations: Vec::new(),
    };
    let record = prove(&spec, &backend).expect("declaration must be valid");
    assert_eq!(record.outcome.status, ExecutionStatus::NotExecuted);
}
