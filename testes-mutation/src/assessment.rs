use std::collections::BTreeSet;
use testes_core::{
    ClaimVerdict, EvidenceGap, ExecutionStatus, MutationEvidence, MutationOutcome, MutationPolicy,
    Observed, TestDeclaration, TestId, TestSpecification,
};

/// Counts of recorded outcomes. Only `caught` and `missed` enter the detection rate.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MutationCounts {
    pub caught: u64,
    pub missed: u64,
    pub unviable: u64,
    pub timeout: u64,
    pub excluded: u64,
}

/// Claim assessment of one mutation campaign against an explicit policy.
#[derive(Clone, Debug)]
pub enum MutationAssessment {
    Satisfied {
        counts: MutationCounts,
        detection_rate: f64,
    },
    Unsatisfied {
        counts: MutationCounts,
        gaps: Vec<EvidenceGap>,
    },
}

impl MutationAssessment {
    pub fn is_satisfied(&self) -> bool {
        matches!(self, Self::Satisfied { .. })
    }

    pub fn counts(&self) -> MutationCounts {
        match self {
            Self::Satisfied { counts, .. } | Self::Unsatisfied { counts, .. } => *counts,
        }
    }

    /// Satisfied maps to `Supported`; every other case maps to `Insufficient` with its gaps.
    pub fn verdict(&self) -> ClaimVerdict {
        match self {
            Self::Satisfied { .. } => ClaimVerdict::Supported,
            Self::Unsatisfied { gaps, .. } => ClaimVerdict::Insufficient { gaps: gaps.clone() },
        }
    }
}

/// Assesses a campaign's evidence against the declared scope and an explicit policy.
///
/// Only outcomes present in `evidence` are considered. See ADR-0006 for the full rules.
pub fn assess(
    declaration: &TestDeclaration,
    evidence: &MutationEvidence,
    policy: &MutationPolicy,
) -> MutationAssessment {
    let mut gaps = Vec::new();
    let counts = count(evidence);

    match &declaration.specification {
        TestSpecification::Mutation {
            source_selectors,
            operators,
            selected_checks,
            commands,
        } => {
            let scope_matches = source_selectors == &evidence.source_selectors
                && operators == &evidence.operators
                && selected_checks == &evidence.selected_checks
                && commands == &evidence.commands;
            if !scope_matches {
                gaps.push(EvidenceGap::RequiredScopeNotSatisfied {
                    detail: "recorded mutation scope differs from declared scope".to_string(),
                });
            }
            let executed: BTreeSet<&TestId> = evidence.selected_checks.iter().collect();
            for result in &evidence.mutations {
                assess_mutant(result, &executed, policy, &mut gaps);
            }
        }
        _ => gaps.push(EvidenceGap::InvalidEvidenceShape),
    }

    if evidence.baseline.status != ExecutionStatus::Passed {
        gaps.push(EvidenceGap::BaselineNotPassed);
    }
    if counts.missed > policy.maximum_missed {
        gaps.push(EvidenceGap::Other {
            detail: format!(
                "missed mutations {} exceed maximum {}",
                counts.missed, policy.maximum_missed
            ),
        });
    }

    let detection_rate = evidence.detection_rate();
    let assessed = counts.caught + counts.missed;
    if detection_rate.is_none() || assessed < policy.minimum_assessed {
        gaps.push(EvidenceGap::InsufficientMutationExperiments);
    }

    match detection_rate {
        Some(detection_rate) if gaps.is_empty() => MutationAssessment::Satisfied {
            counts,
            detection_rate,
        },
        _ => MutationAssessment::Unsatisfied { counts, gaps },
    }
}

fn count(evidence: &MutationEvidence) -> MutationCounts {
    let mut counts = MutationCounts::default();
    for result in &evidence.mutations {
        match &result.outcome {
            MutationOutcome::Caught { .. } => counts.caught += 1,
            MutationOutcome::Missed => counts.missed += 1,
            MutationOutcome::Unviable { .. } => counts.unviable += 1,
            MutationOutcome::Timeout { .. } => counts.timeout += 1,
            MutationOutcome::Excluded { .. } => counts.excluded += 1,
        }
    }
    counts
}

fn assess_mutant(
    result: &testes_core::MutationResult,
    executed: &BTreeSet<&TestId>,
    policy: &MutationPolicy,
    gaps: &mut Vec<EvidenceGap>,
) {
    let id = &result.mutation.id;
    match &result.outcome {
        MutationOutcome::Caught {
            detecting_checks: Observed::Available(checks),
            ..
        } => {
            if checks.is_empty() {
                gaps.push(EvidenceGap::Other {
                    detail: format!("caught mutation {id} has no attributed check"),
                });
            }
            for check in checks {
                if !executed.contains(check) {
                    gaps.push(EvidenceGap::Other {
                        detail: format!(
                            "caught mutation {id} is attributed to check {check}, which was not a selected check"
                        ),
                    });
                }
            }
        }
        MutationOutcome::Timeout { .. } if !policy.allow_timeouts => {
            gaps.push(EvidenceGap::UnresolvedMutation { id: id.clone() });
        }
        MutationOutcome::Excluded { rationale } if rationale.trim().is_empty() => {
            gaps.push(EvidenceGap::Other {
                detail: format!("excluded mutation {id} has no rationale"),
            });
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command;
    use std::time::Duration;
    use testes_core::{
        Claim, Dimension, ExecutionOutcome, ExecutionStatus, MutationDefinition, MutationOutcome,
        MutationResult, ObligationDisposition, Observed, OutcomeReason, ResourceLimits, Scope,
        SubjectId, TestId, TestSpecification,
    };

    fn check(name: &str) -> TestId {
        TestId::new(name).expect("check id must not be blank")
    }

    fn declared_checks() -> Vec<TestId> {
        vec![check("check-a")]
    }

    fn declared_commands() -> Vec<testes_core::CommandSpec> {
        vec![command("cargo", &["nextest", "run"])]
    }

    fn declaration() -> TestDeclaration {
        TestDeclaration {
            id: TestId::new("mutation-test").expect("id must not be blank"),
            primary_dimension: Dimension::Mutation,
            secondary_dimensions: Vec::new(),
            subject: SubjectId::new("subject").expect("subject must not be blank"),
            claim: Claim {
                statement: "checks detect mutants".to_string(),
                obligations: Vec::new(),
            },
            required_scope: Scope {
                input_domain: "src/lib.rs".to_string(),
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
            specification: TestSpecification::Mutation {
                source_selectors: vec!["src/lib.rs".to_string()],
                operators: vec!["delete-statement".to_string()],
                selected_checks: declared_checks(),
                commands: declared_commands(),
            },
        }
    }

    fn outcome(status: ExecutionStatus) -> ExecutionOutcome {
        ExecutionOutcome {
            status,
            reason: OutcomeReason::AssertionsSatisfied,
            diagnostic: None,
            obligations: Vec::new(),
        }
    }

    fn evidence(baseline: ExecutionStatus, mutations: Vec<MutationResult>) -> MutationEvidence {
        MutationEvidence {
            source_selectors: vec!["src/lib.rs".to_string()],
            operators: vec!["delete-statement".to_string()],
            selected_checks: declared_checks(),
            commands: declared_commands(),
            baseline: outcome(baseline),
            mutations,
        }
    }

    fn mutant(id: &str, outcome: MutationOutcome) -> MutationResult {
        MutationResult {
            mutation: MutationDefinition {
                id: testes_core::MutationId::new(id).expect("id must not be blank"),
                location: "src/lib.rs:1".to_string(),
                operator: "delete-statement".to_string(),
                transformation: "removed statement".to_string(),
            },
            outcome,
            duration: Duration::ZERO,
            artifacts: Vec::new(),
        }
    }

    fn caught(id: &str) -> MutationResult {
        mutant(
            id,
            MutationOutcome::Caught {
                detecting_checks: Observed::Available(vec![check("check-a")]),
                diagnostic: None,
            },
        )
    }

    fn missed(id: &str) -> MutationResult {
        mutant(id, MutationOutcome::Missed)
    }

    fn policy(minimum_assessed: u64, maximum_missed: u64, allow_timeouts: bool) -> MutationPolicy {
        MutationPolicy {
            minimum_assessed,
            maximum_missed,
            allow_timeouts,
        }
    }

    fn gap_count(assessment: &MutationAssessment) -> usize {
        match assessment {
            MutationAssessment::Unsatisfied { gaps, .. } => gaps.len(),
            MutationAssessment::Satisfied { .. } => 0,
        }
    }

    #[test]
    fn satisfied_when_baseline_passes_and_catches_are_attributed_to_selected_checks() {
        let result = assess(
            &declaration(),
            &evidence(
                ExecutionStatus::Passed,
                vec![caught("a"), caught("b"), missed("c")],
            ),
            &policy(1, 1, false),
        );
        match result {
            MutationAssessment::Satisfied {
                counts,
                detection_rate,
            } => {
                assert_eq!(counts.caught, 2);
                assert_eq!(counts.missed, 1);
                assert!((detection_rate - 2.0 / 3.0).abs() < 1e-9);
            }
            other => panic!("expected Satisfied, got {other:?}"),
        }
    }

    #[test]
    fn failing_baseline_blocks_satisfaction_even_when_every_mutant_is_caught() {
        let result = assess(
            &declaration(),
            &evidence(ExecutionStatus::Failed, vec![caught("a"), caught("b")]),
            &policy(1, 0, false),
        );
        match result {
            MutationAssessment::Unsatisfied { gaps, .. } => assert!(
                gaps.iter()
                    .any(|g| matches!(g, EvidenceGap::BaselineNotPassed)),
                "expected BaselineNotPassed gap, got {gaps:?}"
            ),
            other => panic!("expected Unsatisfied, got {other:?}"),
        }
    }

    #[test]
    fn inconclusive_baseline_is_not_passing_evidence() {
        let result = assess(
            &declaration(),
            &evidence(ExecutionStatus::Inconclusive, vec![caught("a")]),
            &policy(1, 0, false),
        );
        assert!(!result.is_satisfied());
    }

    #[test]
    fn zero_assessed_experiments_is_insufficient_even_with_zero_minimum() {
        let result = assess(
            &declaration(),
            &evidence(ExecutionStatus::Passed, vec![]),
            &policy(0, 0, false),
        );
        match result {
            MutationAssessment::Unsatisfied { gaps, .. } => assert!(
                gaps.iter()
                    .any(|g| matches!(g, EvidenceGap::InsufficientMutationExperiments)),
                "expected InsufficientMutationExperiments, got {gaps:?}"
            ),
            other => panic!("expected Unsatisfied, got {other:?}"),
        }
    }

    #[test]
    fn unviable_only_campaign_has_no_assessed_experiments() {
        let result = assess(
            &declaration(),
            &evidence(
                ExecutionStatus::Passed,
                vec![mutant(
                    "u",
                    MutationOutcome::Unviable {
                        diagnostic: "did not compile".to_string(),
                    },
                )],
            ),
            &policy(0, 0, false),
        );
        assert!(!result.is_satisfied());
        assert_eq!(result.counts().unviable, 1);
    }

    #[test]
    fn assessed_count_below_minimum_is_insufficient() {
        let result = assess(
            &declaration(),
            &evidence(ExecutionStatus::Passed, vec![caught("a"), caught("b")]),
            &policy(3, 0, false),
        );
        match result {
            MutationAssessment::Unsatisfied { gaps, .. } => assert!(
                gaps.iter()
                    .any(|g| matches!(g, EvidenceGap::InsufficientMutationExperiments)),
                "expected InsufficientMutationExperiments, got {gaps:?}"
            ),
            other => panic!("expected Unsatisfied, got {other:?}"),
        }
    }

    #[test]
    fn catch_attributed_to_unselected_check_is_not_counted() {
        let unselected = mutant(
            "a",
            MutationOutcome::Caught {
                detecting_checks: Observed::Available(vec![check("not-selected")]),
                diagnostic: None,
            },
        );
        let result = assess(
            &declaration(),
            &evidence(ExecutionStatus::Passed, vec![unselected]),
            &policy(1, 0, false),
        );
        assert!(!result.is_satisfied());
        assert!(gap_count(&result) >= 1);
    }

    #[test]
    fn caught_with_empty_attribution_is_a_gap() {
        let empty = mutant(
            "a",
            MutationOutcome::Caught {
                detecting_checks: Observed::Available(Vec::new()),
                diagnostic: None,
            },
        );
        let result = assess(
            &declaration(),
            &evidence(ExecutionStatus::Passed, vec![empty]),
            &policy(1, 0, false),
        );
        assert!(!result.is_satisfied());
    }

    #[test]
    fn caught_with_unavailable_attribution_still_counts_as_detection() {
        let unavailable = mutant(
            "a",
            MutationOutcome::Caught {
                detecting_checks: Observed::Unavailable {
                    reason: "backend does not report detecting checks".to_string(),
                },
                diagnostic: None,
            },
        );
        let result = assess(
            &declaration(),
            &evidence(ExecutionStatus::Passed, vec![unavailable]),
            &policy(1, 0, false),
        );
        assert!(result.is_satisfied());
    }

    #[test]
    fn missed_mutants_above_maximum_are_a_gap() {
        let result = assess(
            &declaration(),
            &evidence(
                ExecutionStatus::Passed,
                vec![caught("a"), missed("b"), missed("c")],
            ),
            &policy(1, 1, false),
        );
        assert!(!result.is_satisfied());
    }

    #[test]
    fn timeout_is_unresolved_unless_policy_allows_it() {
        let timeout = || {
            mutant(
                "t",
                MutationOutcome::Timeout {
                    limit: Duration::from_secs(1),
                },
            )
        };
        let disallowed = assess(
            &declaration(),
            &evidence(ExecutionStatus::Passed, vec![caught("a"), timeout()]),
            &policy(1, 0, false),
        );
        match &disallowed {
            MutationAssessment::Unsatisfied { gaps, .. } => assert!(
                gaps.iter()
                    .any(|g| matches!(g, EvidenceGap::UnresolvedMutation { .. })),
                "expected UnresolvedMutation, got {gaps:?}"
            ),
            other => panic!("expected Unsatisfied, got {other:?}"),
        }

        let allowed = assess(
            &declaration(),
            &evidence(ExecutionStatus::Passed, vec![caught("a"), timeout()]),
            &policy(1, 0, true),
        );
        assert!(allowed.is_satisfied());
    }

    #[test]
    fn excluded_without_rationale_is_a_gap() {
        let result = assess(
            &declaration(),
            &evidence(
                ExecutionStatus::Passed,
                vec![
                    caught("a"),
                    mutant(
                        "e",
                        MutationOutcome::Excluded {
                            rationale: "   ".to_string(),
                        },
                    ),
                ],
            ),
            &policy(1, 0, false),
        );
        assert!(!result.is_satisfied());
    }

    #[test]
    fn excluded_with_rationale_is_not_assessed() {
        let result = assess(
            &declaration(),
            &evidence(
                ExecutionStatus::Passed,
                vec![
                    caught("a"),
                    mutant(
                        "e",
                        MutationOutcome::Excluded {
                            rationale: "equivalent mutant".to_string(),
                        },
                    ),
                ],
            ),
            &policy(1, 0, false),
        );
        assert!(result.is_satisfied());
        assert_eq!(result.counts().excluded, 1);
    }

    #[test]
    fn evidence_commands_differing_from_declaration_is_a_scope_gap() {
        let mut ev = evidence(ExecutionStatus::Passed, vec![caught("a")]);
        ev.commands = vec![command("cargo", &["test"])];
        let result = assess(&declaration(), &ev, &policy(1, 0, false));
        match result {
            MutationAssessment::Unsatisfied { gaps, .. } => assert!(
                gaps.iter()
                    .any(|g| matches!(g, EvidenceGap::RequiredScopeNotSatisfied { .. })),
                "expected RequiredScopeNotSatisfied, got {gaps:?}"
            ),
            other => panic!("expected Unsatisfied, got {other:?}"),
        }
    }

    #[test]
    fn non_mutation_declaration_is_invalid_evidence_shape() {
        let mut decl = declaration();
        decl.primary_dimension = Dimension::Unit;
        decl.specification = TestSpecification::Unit {
            example_ids: vec!["example".to_string()],
        };
        let result = assess(
            &decl,
            &evidence(ExecutionStatus::Passed, vec![caught("a")]),
            &policy(1, 0, false),
        );
        match result {
            MutationAssessment::Unsatisfied { gaps, .. } => assert!(
                gaps.iter()
                    .any(|g| matches!(g, EvidenceGap::InvalidEvidenceShape)),
                "expected InvalidEvidenceShape, got {gaps:?}"
            ),
            other => panic!("expected Unsatisfied, got {other:?}"),
        }
    }

    #[test]
    fn verdict_maps_satisfied_to_supported_and_unsatisfied_to_insufficient() {
        let satisfied = assess(
            &declaration(),
            &evidence(ExecutionStatus::Passed, vec![caught("a")]),
            &policy(1, 0, false),
        );
        assert!(matches!(satisfied.verdict(), ClaimVerdict::Supported));

        let unsatisfied = assess(
            &declaration(),
            &evidence(ExecutionStatus::Passed, vec![]),
            &policy(0, 0, false),
        );
        assert!(matches!(
            unsatisfied.verdict(),
            ClaimVerdict::Insufficient { .. }
        ));
    }
}
