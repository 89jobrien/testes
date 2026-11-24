//! Downstream checks for the `regression!` macro. Each invocation generates a `#[test]`
//! function, so these run through the normal test harness.

use testes_core::UnitEvidence;
use testes_regression::{
    ArtifactDisclosure, ArtifactId, ArtifactKind, ArtifactReference, ContentDigest,
    DimensionEvidence, ExecutionOutcome, ExecutionStatus, NestedExecution, Observed, OutcomeReason,
    regression,
};

fn reference(id: &str) -> ArtifactReference {
    ArtifactReference {
        id: ArtifactId::new(id).expect("artifact id must be valid"),
        kind: ArtifactKind::Fixture,
        location: format!("fixtures/{id}"),
        digest: ContentDigest {
            algorithm: "test".to_string(),
            value: "fixture-digest".to_string(),
        },
        disclosure: ArtifactDisclosure::Public,
    }
}

fn unit_evidence() -> DimensionEvidence {
    DimensionEvidence::Unit(UnitEvidence {
        executed_examples: vec!["overflow".to_string()],
        assertions_evaluated: Observed::Unavailable {
            reason: "assertion counting not implemented".to_string(),
        },
    })
}

fn nested(status: ExecutionStatus, reason: OutcomeReason) -> NestedExecution {
    NestedExecution {
        outcome: ExecutionOutcome {
            status,
            reason,
            diagnostic: None,
            obligations: Vec::new(),
        },
        evidence: unit_evidence(),
        artifacts: vec![reference("overflow-input")],
    }
}

regression! {
    name: regression_repaired_defect_passes,
    id: "budget-overflow-regression",
    subject: "BudgetLedger::charge",
    claim: "overflowing charge is rejected",
    failure: "budget-overflow",
    underlying: Unit,
    reproducers: ["overflow-input"],
    execution: {
        nested(ExecutionStatus::Passed, OutcomeReason::AssertionsSatisfied)
    }
}

regression! {
    #[should_panic(expected = "did not pass")]
    name: regression_defect_still_present_fails,
    id: "budget-overflow-regression",
    subject: "BudgetLedger::charge",
    claim: "overflowing charge is rejected",
    failure: "budget-overflow",
    underlying: Unit,
    reproducers: ["overflow-input"],
    execution: {
        nested(ExecutionStatus::Failed, OutcomeReason::AssertionViolation)
    }
}

regression! {
    #[should_panic(expected = "did not pass")]
    name: regression_unavailable_capability_is_not_a_pass,
    id: "budget-overflow-regression",
    subject: "BudgetLedger::charge",
    claim: "overflowing charge is rejected",
    failure: "budget-overflow",
    underlying: Unit,
    reproducers: ["overflow-input"],
    execution: {
        nested(ExecutionStatus::Inconclusive, OutcomeReason::CapabilityUnavailable)
    }
}

regression! {
    #[should_panic(expected = "did not pass")]
    name: regression_not_executed_is_not_a_pass,
    id: "budget-overflow-regression",
    subject: "BudgetLedger::charge",
    claim: "overflowing charge is rejected",
    failure: "budget-overflow",
    underlying: Unit,
    reproducers: ["overflow-input"],
    execution: {
        nested(ExecutionStatus::NotExecuted, OutcomeReason::Cancelled)
    }
}

regression! {
    #[should_panic(expected = "regression evidence must have a valid shape and identity")]
    name: regression_nested_dimension_must_match_underlying,
    id: "budget-overflow-regression",
    subject: "BudgetLedger::charge",
    claim: "overflowing charge is rejected",
    failure: "budget-overflow",
    underlying: Fuzz,
    reproducers: ["overflow-input"],
    execution: {
        nested(ExecutionStatus::Passed, OutcomeReason::AssertionsSatisfied)
    }
}
