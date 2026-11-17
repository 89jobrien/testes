use testes_core::DeclarationError;
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

/// The macro cannot run a campaign, so it produces no assessed experiments.
/// `detection_rate()` must therefore report "unavailable" (`None`), never 0.0
/// and never 1.0. A zero denominator means unavailable, not a perfect score.
#[test]
fn macro_produced_evidence_has_no_assessed_experiments() {
    LAST_MUTATION_EVIDENCE.with(|e| {
        if let Some((_id, evidence)) = &*e.borrow() {
            assert!(evidence.mutations.is_empty());
            assert_eq!(evidence.detection_rate(), None);
        }
    });
}

/// Macro-produced evidence must flow into the same arithmetic as any other
/// `MutationEvidence`: unviable, timeout, and excluded outcomes are not
/// assessed, so they must not shift the detection rate.
#[test]
fn detection_rate_excludes_unassessed_outcomes() {
    let baseline = ExecutionOutcome {
        status: ExecutionStatus::Passed,
        reason: OutcomeReason::AssertionsSatisfied,
        diagnostic: None,
        obligations: Vec::new(),
    };

    let mutation = |id: &str, outcome: MutationOutcome| MutationResult {
        mutation: MutationDefinition {
            id: testes_core::MutationId::new(id).expect("id must not be blank"),
            location: "src/lib.rs:1".to_string(),
            operator: "delete-statement".to_string(),
            transformation: "removed statement".to_string(),
        },
        outcome,
        duration: std::time::Duration::ZERO,
        artifacts: Vec::new(),
    };

    let caught = MutationOutcome::Caught {
        detecting_checks: Observed::Available(Vec::new()),
        diagnostic: None,
    };
    let missed = MutationOutcome::Missed;

    let evidence = MutationEvidence {
        source_selectors: Vec::new(),
        operators: Vec::new(),
        selected_checks: Vec::new(),
        commands: Vec::new(),
        baseline,
        mutations: vec![
            mutation("caught", caught),
            mutation("missed", missed),
            mutation(
                "unviable",
                MutationOutcome::Unviable {
                    diagnostic: "did not compile".to_string(),
                },
            ),
            mutation(
                "timeout",
                MutationOutcome::Timeout {
                    limit: std::time::Duration::from_secs(1),
                },
            ),
            mutation(
                "excluded",
                MutationOutcome::Excluded {
                    rationale: "not covered by selected checks".to_string(),
                },
            ),
        ],
    };

    // Only the caught and missed mutants are assessed: 1 / 2.
    assert_eq!(evidence.detection_rate(), Some(0.5));
}

fn mutation_specification() -> TestSpecification {
    TestSpecification::Mutation {
        source_selectors: Vec::new(),
        operators: Vec::new(),
        selected_checks: Vec::new(),
        commands: Vec::new(),
    }
}

/// A mutation declaration must not be accepted under a different primary dimension.
#[test]
fn declaration_validate_rejects_dimension_mismatch() {
    let declaration = TestDeclaration {
        id: TestId::new("dim-mismatch").expect("valid id"),
        primary_dimension: Dimension::Mutation,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "mutation test".to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "mutation test".to_string(),
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
        primary_dimension: Dimension::Mutation,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "   ".to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "mutation test".to_string(),
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
        specification: mutation_specification(),
    };
    assert_eq!(declaration.validate(), Err(DeclarationError::EmptyClaim));
}

/// Obligation IDs must be unique so that per-obligation results are addressable.
#[test]
fn declaration_validate_rejects_duplicate_obligation_ids() {
    let obligation_id = ObligationId::new("dup").expect("id must not be blank");
    let declaration = TestDeclaration {
        id: TestId::new("dup-obligations").expect("valid id"),
        primary_dimension: Dimension::Mutation,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new("test-subject").expect("valid id"),
        claim: Claim {
            statement: "mutation test".to_string(),
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
            input_domain: "mutation test".to_string(),
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
        specification: mutation_specification(),
    };
    assert_eq!(
        declaration.validate(),
        Err(DeclarationError::DuplicateObligation(obligation_id))
    );
}
