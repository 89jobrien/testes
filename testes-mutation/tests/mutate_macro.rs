// Integration tests for the mutate! macro (ADR-0007).
//
// The campaign values below are fixtures standing in for a runner that actually
// executed mutations. The macro must record exactly what the fixture supplies:
// it never invents mutants, checks, or backend identity.

use std::time::{Duration, SystemTime};
use testes_core::{
    ArtifactReference, BackendIdentity, ClaimVerdict, DeclarationError, Dimension,
    DimensionEvidence, EvidenceGap, ExecutionMetadata, ExecutionOutcome, ExecutionStatus, IdError,
    MutationDefinition, MutationEvidence, MutationOutcome, MutationPolicy, MutationResult,
    ObligationDisposition, Observed, OutcomeReason, Provenance, RepositoryRevision, ResourceLimits,
    RunId, Scope, TestId, TestSpecification, WorkingTreeState,
};
use testes_mutation::{
    MutationCampaign, MutationDeclarationError, MutationSpec, command, mutate, record_mutation,
};

fn scope() -> Scope {
    Scope {
        input_domain: "fixture".to_string(),
        assumptions: Vec::new(),
        bounds: Vec::new(),
        capabilities: Vec::new(),
        resources: ResourceLimits {
            wall_time: None,
            memory_bytes: None,
            max_input_bytes: None,
        },
    }
}

fn provenance() -> Provenance {
    Provenance {
        run_id: RunId::new("fixture-run").expect("run id must not be blank"),
        revision: RepositoryRevision {
            repository: "fixture".to_string(),
            commit: "0000000".to_string(),
            working_tree: WorkingTreeState::Clean,
        },
    }
}

fn execution() -> ExecutionMetadata {
    ExecutionMetadata {
        backend: BackendIdentity {
            name: "fixture-campaign-runner".to_string(),
            version: "0.0.0".to_string(),
            toolchain: None,
        },
        configuration: Default::default(),
        started_at: SystemTime::UNIX_EPOCH,
        duration: Duration::from_secs(1),
        actual_scope: scope(),
    }
}

fn baseline(status: ExecutionStatus) -> ExecutionOutcome {
    ExecutionOutcome {
        status,
        reason: OutcomeReason::AssertionsSatisfied,
        diagnostic: None,
        obligations: Vec::new(),
    }
}

fn mutant(id: &str, outcome: MutationOutcome) -> MutationResult {
    MutationResult {
        mutation: MutationDefinition {
            id: testes_core::MutationId::new(id).expect("mutation id must not be blank"),
            location: "src/parser.rs:1".to_string(),
            operator: "delete-statement".to_string(),
            transformation: "removed statement".to_string(),
        },
        outcome,
        duration: Duration::ZERO,
        artifacts: Vec::new(),
    }
}

fn caught_by(id: &str, check: &str) -> MutationResult {
    mutant(
        id,
        MutationOutcome::Caught {
            detecting_checks: Observed::Available(vec![
                TestId::new(check).expect("check id must not be blank"),
            ]),
            diagnostic: None,
        },
    )
}

fn missed(id: &str) -> MutationResult {
    mutant(id, MutationOutcome::Missed)
}

fn campaign(status: ExecutionStatus, mutations: Vec<MutationResult>) -> MutationCampaign {
    MutationCampaign {
        provenance: provenance(),
        execution: execution(),
        artifacts: Vec::<ArtifactReference>::new(),
        evidence: MutationEvidence {
            source_selectors: vec!["src/parser.rs".to_string()],
            operators: vec!["delete-statement".to_string()],
            selected_checks: vec![
                TestId::new("parser-unit-tests").expect("check id must not be blank"),
            ],
            commands: vec![command("cargo", &["nextest", "run", "-p", "parser"])],
            baseline: baseline(status),
            mutations,
        },
    }
}

#[test]
fn mutate_declares_mutation_spec_and_records_mutation_dimension_evidence() {
    let policy = MutationPolicy {
        minimum_assessed: 1,
        maximum_missed: 1,
        allow_timeouts: false,
    };
    let record = mutate! {
        id: "parser-mutations",
        subject: "parser::parse",
        claim: "the parser checks detect deleted statements",
        sources: ["src/parser.rs"],
        operators: ["delete-statement"],
        checks: ["parser-unit-tests"],
        commands: [("cargo", ["nextest", "run", "-p", "parser"])],
        policy: policy,
        campaign: campaign(
            ExecutionStatus::Passed,
            vec![
                caught_by("a", "parser-unit-tests"),
                caught_by("b", "parser-unit-tests"),
                missed("c"),
            ],
        ),
    };

    let declaration = &record.evidence.declaration;
    assert_eq!(declaration.primary_dimension, Dimension::Mutation);
    assert!(matches!(
        declaration.specification,
        TestSpecification::Mutation { .. }
    ));
    assert!(matches!(
        record.evidence.dimension_evidence,
        DimensionEvidence::Mutation(_)
    ));
    assert_eq!(record.evidence.validate_shape(), Ok(()));
    assert!(
        record.is_satisfied(),
        "expected Satisfied: {:?}",
        record.assessment
    );
    assert_eq!(record.assessment.counts().caught, 2);
    assert_eq!(record.assessment.counts().missed, 1);
    assert!(matches!(
        record.assessment.verdict(),
        ClaimVerdict::Supported
    ));
    record.assert_satisfied();
}

#[test]
fn mutate_outcome_describes_baseline_execution_not_claim_satisfaction() {
    let record = mutate! {
        id: "baseline-outcome",
        subject: "parser::parse",
        claim: "baseline execution is recorded",
        sources: ["src/parser.rs"],
        operators: ["delete-statement"],
        checks: ["parser-unit-tests"],
        commands: [("cargo", ["nextest", "run", "-p", "parser"])],
        policy: MutationPolicy {
            minimum_assessed: 5,
            maximum_missed: 0,
            allow_timeouts: false,
        },
        campaign: campaign(ExecutionStatus::Passed, vec![caught_by("a", "parser-unit-tests")]),
    };

    assert_eq!(record.evidence.outcome.status, ExecutionStatus::Passed);
    assert!(!record.is_satisfied());
}

#[test]
fn mutate_failing_baseline_is_not_satisfied_even_when_every_mutant_is_caught() {
    let record = mutate! {
        id: "failing-baseline",
        subject: "parser::parse",
        claim: "checks detect deleted statements",
        sources: ["src/parser.rs"],
        operators: ["delete-statement"],
        checks: ["parser-unit-tests"],
        commands: [("cargo", ["nextest", "run", "-p", "parser"])],
        policy: MutationPolicy {
            minimum_assessed: 1,
            maximum_missed: 0,
            allow_timeouts: false,
        },
        campaign: campaign(
            ExecutionStatus::Failed,
            vec![caught_by("a", "parser-unit-tests")],
        ),
    };

    assert!(!record.is_satisfied());
    assert!(matches!(
        record.assessment.verdict(),
        ClaimVerdict::Insufficient { .. }
    ));
}

#[test]
fn mutate_campaign_without_executed_mutants_is_insufficient_not_full_detection() {
    let record = mutate! {
        id: "no-mutants",
        subject: "parser::parse",
        claim: "checks detect deleted statements",
        sources: ["src/parser.rs"],
        operators: ["delete-statement"],
        checks: ["parser-unit-tests"],
        commands: [("cargo", ["nextest", "run", "-p", "parser"])],
        policy: MutationPolicy {
            minimum_assessed: 0,
            maximum_missed: 0,
            allow_timeouts: false,
        },
        campaign: campaign(ExecutionStatus::NotExecuted, Vec::new()),
    };

    assert!(!record.is_satisfied());
    match &record.assessment {
        testes_mutation::MutationAssessment::Unsatisfied { gaps, .. } => assert!(
            gaps.iter()
                .any(|g| matches!(g, EvidenceGap::BaselineNotPassed)),
            "expected BaselineNotPassed, got {gaps:?}"
        ),
        other => panic!("expected Unsatisfied, got {other:?}"),
    }
}

#[test]
fn mutate_catch_attributed_to_check_outside_declared_selection_is_not_counted() {
    let record = mutate! {
        id: "unselected-attribution",
        subject: "parser::parse",
        claim: "checks detect deleted statements",
        sources: ["src/parser.rs"],
        operators: ["delete-statement"],
        checks: ["parser-unit-tests"],
        commands: [("cargo", ["nextest", "run", "-p", "parser"])],
        policy: MutationPolicy {
            minimum_assessed: 1,
            maximum_missed: 0,
            allow_timeouts: false,
        },
        campaign: campaign(
            ExecutionStatus::Passed,
            vec![caught_by("a", "some-other-check")],
        ),
    };

    assert!(!record.is_satisfied());
}

#[test]
fn mutate_recorded_commands_differing_from_declared_commands_is_unsatisfied() {
    let record = mutate! {
        id: "scope-mismatch",
        subject: "parser::parse",
        claim: "checks detect deleted statements",
        sources: ["src/parser.rs"],
        operators: ["delete-statement"],
        checks: ["parser-unit-tests"],
        commands: [("cargo", ["test"])],
        policy: MutationPolicy {
            minimum_assessed: 1,
            maximum_missed: 0,
            allow_timeouts: false,
        },
        campaign: campaign(
            ExecutionStatus::Passed,
            vec![caught_by("a", "parser-unit-tests")],
        ),
    };

    assert!(!record.is_satisfied());
}

#[test]
#[should_panic(expected = "mutate! declaration must be valid")]
fn mutate_panics_with_contextual_message_on_blank_id() {
    let _ = mutate! {
        id: "   ",
        subject: "parser::parse",
        claim: "checks detect deleted statements",
        sources: ["src/parser.rs"],
        operators: ["delete-statement"],
        checks: ["parser-unit-tests"],
        commands: [("cargo", ["nextest", "run"])],
        policy: MutationPolicy {
            minimum_assessed: 1,
            maximum_missed: 0,
            allow_timeouts: false,
        },
        campaign: campaign(ExecutionStatus::Passed, vec![caught_by("a", "parser-unit-tests")]),
    };
}

#[test]
#[should_panic(expected = "mutation claim not satisfied")]
fn assert_satisfied_panics_when_claim_is_not_satisfied() {
    let record = mutate! {
        id: "assert-unsatisfied",
        subject: "parser::parse",
        claim: "checks detect deleted statements",
        sources: ["src/parser.rs"],
        operators: ["delete-statement"],
        checks: ["parser-unit-tests"],
        commands: [("cargo", ["nextest", "run", "-p", "parser"])],
        policy: MutationPolicy {
            minimum_assessed: 1,
            maximum_missed: 0,
            allow_timeouts: false,
        },
        campaign: campaign(ExecutionStatus::Failed, vec![caught_by("a", "parser-unit-tests")]),
    };
    record.assert_satisfied();
}

fn spec(id: &str, claim: &str, checks: Vec<String>) -> MutationSpec {
    MutationSpec {
        id: id.to_string(),
        subject: "parser::parse".to_string(),
        claim: claim.to_string(),
        source_selectors: vec!["src/parser.rs".to_string()],
        operators: vec!["delete-statement".to_string()],
        selected_checks: checks,
        commands: vec![command("cargo", &["nextest", "run"])],
    }
}

#[test]
fn record_mutation_reports_blank_check_identifier_as_id_error() {
    let result = record_mutation(
        spec("ok-id", "claim", vec![String::new()]),
        campaign(ExecutionStatus::Passed, Vec::new()),
        &MutationPolicy {
            minimum_assessed: 0,
            maximum_missed: 0,
            allow_timeouts: false,
        },
    );
    assert!(matches!(
        result,
        Err(MutationDeclarationError::Id(IdError::Empty))
    ));
}

#[test]
fn record_mutation_reports_blank_claim_as_declaration_error() {
    let result = record_mutation(
        spec("claim-id", "   ", vec!["parser-unit-tests".to_string()]),
        campaign(ExecutionStatus::Passed, Vec::new()),
        &MutationPolicy {
            minimum_assessed: 0,
            maximum_missed: 0,
            allow_timeouts: false,
        },
    );
    assert!(matches!(
        result,
        Err(MutationDeclarationError::Declaration(
            DeclarationError::EmptyClaim
        ))
    ));
}

#[test]
fn record_mutation_keeps_declared_obligations_empty_and_disposition_required() {
    let record = record_mutation(
        spec(
            "disposition-id",
            "claim",
            vec!["parser-unit-tests".to_string()],
        ),
        campaign(ExecutionStatus::Passed, Vec::new()),
        &MutationPolicy {
            minimum_assessed: 0,
            maximum_missed: 0,
            allow_timeouts: false,
        },
    )
    .expect("well-formed declaration must record");
    assert!(matches!(
        record.evidence.declaration.disposition,
        ObligationDisposition::Required
    ));
    assert!(record.evidence.declaration.claim.obligations.is_empty());
}
