//! `testes-regression` owns the `regression!` dimension and the `regression!` macro.
//!
//! A regression check wraps the evidence of one nested execution in a
//! `DimensionEvidence::Regression` record. The nested execution is supplied by the
//! caller and is reused, never run a second time to label it as a regression.
//! Native execution only; no `TestBackend` is involved.
//!
//! Identity decision (see [`check_regression_identity`]): the record's failure ID must
//! match the declared failure, the retained reproducer list must match the declared
//! reproducers, every declared reproducer and any historical failure artifact must be
//! among the retained artifacts, and at least one reproducer must be declared.
//! Nested execution identity (for example a fuzz target or conformance contract) is
//! not checked here because the regression layer does not carry the nested declaration.

#![forbid(unsafe_code)]

use std::{
    fmt,
    time::{Duration, SystemTime},
};

pub use testes_core::{
    ArtifactDisclosure, ArtifactId, ArtifactKind, ArtifactReference, BackendConfiguration,
    BackendIdentity, Claim, ContentDigest, DeclarationError, Dimension, DimensionEvidence,
    EvidenceRecord, EvidenceShapeError, ExecutionMetadata, ExecutionOutcome, ExecutionStatus,
    FailureId, IdError, ObligationDisposition, Observed, OutcomeReason, Provenance,
    RegressionEvidence, RepositoryRevision, ResourceLimits, RunId, Scope, SubjectId,
    TestDeclaration, TestId, TestSpecification, WorkingTreeState,
};

/// Declared inputs for one regression check.
#[derive(Clone, Debug)]
pub struct RegressionSpec {
    pub id: TestId,
    pub subject: SubjectId,
    pub claim: String,
    pub failure: FailureId,
    pub underlying: Dimension,
    pub reproducers: Vec<ArtifactId>,
}

/// The result of one nested execution, reused as the regression's underlying evidence.
#[derive(Clone, Debug)]
pub struct NestedExecution {
    pub outcome: ExecutionOutcome,
    pub evidence: DimensionEvidence,
    pub artifacts: Vec<ArtifactReference>,
}

/// Structural or identity failure while building or checking regression evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegressionError {
    Declaration(DeclarationError),
    Shape(EvidenceShapeError),
    NotRegression,
    FailureMismatch {
        declared: FailureId,
        recorded: FailureId,
    },
    NoReproducer,
    ReproducerMismatch {
        declared: Vec<ArtifactId>,
        recorded: Vec<ArtifactId>,
    },
    ReproducerArtifactMissing {
        id: ArtifactId,
    },
    HistoricalArtifactMissing {
        id: ArtifactId,
    },
}

impl fmt::Display for RegressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Declaration(error) => write!(f, "regression declaration is invalid: {error}"),
            Self::Shape(error) => write!(f, "regression evidence has an invalid shape: {error}"),
            Self::NotRegression => {
                f.write_str("record does not carry a regression declaration and evidence")
            }
            Self::FailureMismatch { declared, recorded } => write!(
                f,
                "recorded failure `{recorded}` does not match declared failure `{declared}`"
            ),
            Self::NoReproducer => {
                f.write_str("regression must declare at least one reproducer artifact")
            }
            Self::ReproducerMismatch { .. } => {
                f.write_str("recorded reproducer artifacts do not match declared reproducers")
            }
            Self::ReproducerArtifactMissing { id } => {
                write!(
                    f,
                    "reproducer artifact `{id}` is not among the retained artifacts"
                )
            }
            Self::HistoricalArtifactMissing { id } => write!(
                f,
                "historical failure artifact `{id}` is not among the retained artifacts"
            ),
        }
    }
}

impl std::error::Error for RegressionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Declaration(error) => Some(error),
            Self::Shape(error) => Some(error),
            _ => None,
        }
    }
}

/// Builds a regression record around one nested execution.
///
/// `execute` runs exactly once. A panic inside it propagates and no record is
/// produced, because nested evidence cannot be fabricated for a panic.
pub fn record_regression<F>(
    spec: RegressionSpec,
    execute: F,
) -> Result<EvidenceRecord, RegressionError>
where
    F: FnOnce() -> NestedExecution,
{
    let RegressionSpec {
        id,
        subject,
        claim,
        failure,
        underlying,
        reproducers,
    } = spec;
    let declaration = TestDeclaration {
        id: id.clone(),
        primary_dimension: Dimension::Regression,
        secondary_dimensions: Vec::new(),
        subject,
        claim: Claim {
            statement: claim,
            obligations: Vec::new(),
        },
        required_scope: regression_scope(),
        disposition: ObligationDisposition::Required,
        specification: TestSpecification::Regression {
            failure: failure.clone(),
            underlying_dimension: underlying,
            reproducer_artifacts: reproducers.clone(),
        },
    };
    declaration
        .validate()
        .map_err(RegressionError::Declaration)?;

    let started_at = SystemTime::now();
    let nested = execute();
    let duration = started_at.elapsed().unwrap_or(Duration::ZERO);

    let record = EvidenceRecord {
        schema_version: 1,
        declaration,
        provenance: Provenance {
            run_id: RunId::new("regression-test-run").expect("run id must be valid"),
            revision: RepositoryRevision {
                repository: "unknown".to_string(),
                commit: "unknown".to_string(),
                working_tree: WorkingTreeState::Unknown,
            },
        },
        execution: ExecutionMetadata {
            backend: BackendIdentity {
                name: "native-rust-test".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                toolchain: None,
            },
            configuration: BackendConfiguration::new(),
            started_at,
            duration,
            actual_scope: regression_scope(),
        },
        outcome: nested.outcome,
        artifacts: nested.artifacts,
        dimension_evidence: DimensionEvidence::Regression(RegressionEvidence {
            failure,
            reproducer_artifacts: reproducers,
            historical_failure: None,
            execution: Box::new(nested.evidence),
        }),
    };
    record.validate_shape().map_err(RegressionError::Shape)?;
    check_regression_identity(&record)?;
    Ok(record)
}

fn regression_scope() -> Scope {
    Scope {
        input_domain: "regression test".to_string(),
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

/// Checks that a regression record's declaration and evidence agree on identity.
pub fn check_regression_identity(record: &EvidenceRecord) -> Result<(), RegressionError> {
    let (
        TestSpecification::Regression {
            failure: declared_failure,
            reproducer_artifacts: declared_reproducers,
            ..
        },
        DimensionEvidence::Regression(evidence),
    ) = (
        &record.declaration.specification,
        &record.dimension_evidence,
    )
    else {
        return Err(RegressionError::NotRegression);
    };

    if &evidence.failure != declared_failure {
        return Err(RegressionError::FailureMismatch {
            declared: declared_failure.clone(),
            recorded: evidence.failure.clone(),
        });
    }
    if declared_reproducers.is_empty() {
        return Err(RegressionError::NoReproducer);
    }
    if &evidence.reproducer_artifacts != declared_reproducers {
        return Err(RegressionError::ReproducerMismatch {
            declared: declared_reproducers.clone(),
            recorded: evidence.reproducer_artifacts.clone(),
        });
    }
    let retained = |id: &ArtifactId| record.artifacts.iter().any(|artifact| &artifact.id == id);
    for id in declared_reproducers {
        if !retained(id) {
            return Err(RegressionError::ReproducerArtifactMissing { id: id.clone() });
        }
    }
    if let Some(id) = &evidence.historical_failure
        && !retained(id)
    {
        return Err(RegressionError::HistoricalArtifactMissing { id: id.clone() });
    }
    Ok(())
}

/// Panics unless the record's outcome is a genuine pass.
///
/// Only `Passed` with `AssertionsSatisfied` counts. Failed, inconclusive, not-executed,
/// and unavailable outcomes all panic, so `cargo test` never reports them as passing.
pub fn assert_passed(record: &EvidenceRecord) {
    let outcome = &record.outcome;
    let genuine_pass = outcome.status == ExecutionStatus::Passed
        && outcome.reason == OutcomeReason::AssertionsSatisfied;
    if !genuine_pass {
        let diagnostic = outcome
            .diagnostic
            .as_deref()
            .map(|text| format!(": {text}"))
            .unwrap_or_default();
        panic!(
            "regression `{}` did not pass: status {:?}, reason {:?}{diagnostic}",
            record.declaration.id, outcome.status, outcome.reason
        );
    }
}

/// Declares a regression check, runs its nested execution once, and asserts it passed.
///
/// ```text
/// regression! {
///     name: test_fn_name,
///     id: "regression-id",
///     subject: "Subject::method",
///     claim: "the defect stays repaired",
///     failure: "failure-id",
///     underlying: Unit,
///     reproducers: ["reproducer-artifact-id"],
///     execution: {
///         NestedExecution { outcome, evidence, artifacts }
///     }
/// }
/// ```
///
/// Generates a `#[test] fn test_fn_name()`. Attributes such as `#[should_panic]` may
/// be placed before `name:`. The `underlying` identifier is a `Dimension` variant and
/// must match the dimension of the evidence returned by `execution`.
#[macro_export]
macro_rules! regression {
    (
        $(#[$meta:meta])*
        name: $name:ident,
        id: $id:expr,
        subject: $subject:expr,
        claim: $claim:expr,
        failure: $failure:expr,
        underlying: $underlying:ident,
        reproducers: [$($reproducer:expr),* $(,)?],
        execution: $execution:block $(,)?
    ) => {
        $(#[$meta])*
        #[test]
        fn $name() {
            let spec = $crate::RegressionSpec {
                id: $crate::TestId::new($id)
                    .expect("regression id must not be blank or contain control characters"),
                subject: $crate::SubjectId::new($subject)
                    .expect("subject must not be blank or contain control characters"),
                claim: ($claim).to_string(),
                failure: $crate::FailureId::new($failure)
                    .expect("failure id must not be blank or contain control characters"),
                underlying: $crate::Dimension::$underlying,
                reproducers: vec![
                    $($crate::ArtifactId::new($reproducer)
                        .expect("reproducer id must not be blank or contain control characters"),)*
                ],
            };
            let record = $crate::record_regression(spec, || $execution)
                .expect("regression evidence must have a valid shape and identity");
            $crate::assert_passed(&record);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use testes_core::UnitEvidence;

    fn artifact(value: &str) -> ArtifactId {
        ArtifactId::new(value).expect("artifact id must be valid")
    }

    fn failure(value: &str) -> FailureId {
        FailureId::new(value).expect("failure id must be valid")
    }

    fn reference(id: &str) -> ArtifactReference {
        ArtifactReference {
            id: artifact(id),
            kind: ArtifactKind::Fixture,
            location: format!("fixtures/{id}"),
            digest: ContentDigest {
                algorithm: "test".to_string(),
                value: "fixture-digest".to_string(),
            },
            disclosure: ArtifactDisclosure::Public,
        }
    }

    fn spec(underlying: Dimension) -> RegressionSpec {
        RegressionSpec {
            id: TestId::new("budget-overflow-regression").expect("test id must be valid"),
            subject: SubjectId::new("BudgetLedger::charge").expect("subject must be valid"),
            claim: "overflowing charge is rejected".to_string(),
            failure: failure("budget-overflow"),
            underlying,
            reproducers: vec![artifact("overflow-input")],
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

    fn outcome(status: ExecutionStatus, reason: OutcomeReason) -> ExecutionOutcome {
        ExecutionOutcome {
            status,
            reason,
            diagnostic: None,
            obligations: Vec::new(),
        }
    }

    fn nested_with(outcome: ExecutionOutcome) -> NestedExecution {
        NestedExecution {
            outcome,
            evidence: unit_evidence(),
            artifacts: vec![reference("overflow-input")],
        }
    }

    fn passing_nested() -> NestedExecution {
        nested_with(outcome(
            ExecutionStatus::Passed,
            OutcomeReason::AssertionsSatisfied,
        ))
    }

    fn passing_record() -> EvidenceRecord {
        record_regression(spec(Dimension::Unit), passing_nested)
            .expect("well-formed regression must record")
    }

    fn regression_evidence_mut(record: &mut EvidenceRecord) -> &mut RegressionEvidence {
        match &mut record.dimension_evidence {
            DimensionEvidence::Regression(evidence) => evidence,
            other => panic!("expected regression evidence, got {other:?}"),
        }
    }

    #[test]
    fn record_regression_success_records_passed_regression() {
        let record = passing_record();
        assert_eq!(record.declaration.primary_dimension, Dimension::Regression);
        assert_eq!(record.outcome.status, ExecutionStatus::Passed);
        assert_eq!(record.dimension_evidence.dimension(), Dimension::Regression);
        match &record.dimension_evidence {
            DimensionEvidence::Regression(evidence) => {
                assert_eq!(evidence.failure, failure("budget-overflow"));
                assert_eq!(
                    evidence.reproducer_artifacts,
                    vec![artifact("overflow-input")]
                );
                assert_eq!(evidence.execution.dimension(), Dimension::Unit);
                assert!(evidence.historical_failure.is_none());
            }
            other => panic!("expected regression evidence, got {other:?}"),
        }
        assert_eq!(record.validate_shape(), Ok(()));
    }

    #[test]
    fn record_regression_failure_records_failed_outcome() {
        let record = record_regression(spec(Dimension::Unit), || {
            nested_with(outcome(
                ExecutionStatus::Failed,
                OutcomeReason::AssertionViolation,
            ))
        })
        .expect("failed nested execution must still record");
        assert_eq!(record.outcome.status, ExecutionStatus::Failed);
        assert_eq!(record.outcome.reason, OutcomeReason::AssertionViolation);
    }

    #[test]
    fn record_regression_unavailable_records_inconclusive_outcome() {
        let record = record_regression(spec(Dimension::Unit), || {
            nested_with(outcome(
                ExecutionStatus::Inconclusive,
                OutcomeReason::CapabilityUnavailable,
            ))
        })
        .expect("unavailable nested execution must still record");
        assert_eq!(record.outcome.status, ExecutionStatus::Inconclusive);
        assert_eq!(record.outcome.reason, OutcomeReason::CapabilityUnavailable);
    }

    #[test]
    fn record_regression_not_executed_records_not_executed_outcome() {
        let record = record_regression(spec(Dimension::Unit), || {
            nested_with(outcome(
                ExecutionStatus::NotExecuted,
                OutcomeReason::Cancelled,
            ))
        })
        .expect("not-executed nested execution must still record");
        assert_eq!(record.outcome.status, ExecutionStatus::NotExecuted);
    }

    #[test]
    fn record_regression_runs_nested_execution_exactly_once() {
        let calls = Cell::new(0_u32);
        let _ = record_regression(spec(Dimension::Unit), || {
            calls.set(calls.get() + 1);
            passing_nested()
        })
        .expect("well-formed regression must record");
        assert_eq!(calls.get(), 1);
    }

    #[test]
    #[should_panic(expected = "nested boom")]
    fn record_regression_propagates_nested_panic_without_fabricating_evidence() {
        let _ = record_regression(spec(Dimension::Unit), || -> NestedExecution {
            panic!("nested boom")
        });
    }

    #[test]
    fn record_regression_rejects_mutation_as_underlying_dimension() {
        let error = record_regression(spec(Dimension::Mutation), passing_nested)
            .expect_err("mutation cannot be wrapped by regression");
        assert_eq!(
            error,
            RegressionError::Declaration(DeclarationError::InvalidRegressionDimension)
        );
    }

    #[test]
    fn record_regression_rejects_nested_evidence_with_wrong_dimension() {
        let error = record_regression(spec(Dimension::Fuzz), passing_nested)
            .expect_err("declared fuzz underlying must not accept unit evidence");
        assert_eq!(
            error,
            RegressionError::Shape(EvidenceShapeError::InvalidRegressionPayload)
        );
    }

    #[test]
    fn record_regression_rejects_declaration_without_reproducer() {
        let mut declared = spec(Dimension::Unit);
        declared.reproducers = Vec::new();
        let error = record_regression(declared, passing_nested)
            .expect_err("regression without a reproducer must not record");
        assert_eq!(error, RegressionError::NoReproducer);
    }

    #[test]
    fn check_identity_accepts_well_formed_record() {
        assert_eq!(check_regression_identity(&passing_record()), Ok(()));
    }

    #[test]
    fn check_identity_rejects_non_regression_record() {
        let mut record = passing_record();
        record.dimension_evidence = unit_evidence();
        assert_eq!(
            check_regression_identity(&record),
            Err(RegressionError::NotRegression)
        );
    }

    #[test]
    fn check_identity_rejects_failure_mismatch() {
        let mut record = passing_record();
        regression_evidence_mut(&mut record).failure = failure("other-failure");
        assert_eq!(
            check_regression_identity(&record),
            Err(RegressionError::FailureMismatch {
                declared: failure("budget-overflow"),
                recorded: failure("other-failure"),
            })
        );
    }

    #[test]
    fn check_identity_rejects_reproducer_list_mismatch() {
        let mut record = passing_record();
        regression_evidence_mut(&mut record).reproducer_artifacts = vec![artifact("other-input")];
        assert_eq!(
            check_regression_identity(&record),
            Err(RegressionError::ReproducerMismatch {
                declared: vec![artifact("overflow-input")],
                recorded: vec![artifact("other-input")],
            })
        );
    }

    #[test]
    fn check_identity_rejects_declared_reproducer_missing_from_artifacts() {
        let mut record = passing_record();
        record.artifacts.clear();
        assert_eq!(
            check_regression_identity(&record),
            Err(RegressionError::ReproducerArtifactMissing {
                id: artifact("overflow-input"),
            })
        );
    }

    #[test]
    fn check_identity_rejects_historical_failure_missing_from_artifacts() {
        let mut record = passing_record();
        regression_evidence_mut(&mut record).historical_failure = Some(artifact("old-failure"));
        assert_eq!(
            check_regression_identity(&record),
            Err(RegressionError::HistoricalArtifactMissing {
                id: artifact("old-failure"),
            })
        );
    }

    #[test]
    fn check_identity_accepts_historical_failure_present_in_artifacts() {
        let mut record = passing_record();
        record.artifacts.push(reference("old-failure"));
        regression_evidence_mut(&mut record).historical_failure = Some(artifact("old-failure"));
        assert_eq!(check_regression_identity(&record), Ok(()));
    }

    #[test]
    fn assert_passed_accepts_passed_with_assertions_satisfied() {
        assert_passed(&passing_record());
    }

    #[test]
    #[should_panic(expected = "did not pass")]
    fn assert_passed_rejects_failed_outcome() {
        let record = record_regression(spec(Dimension::Unit), || {
            nested_with(outcome(
                ExecutionStatus::Failed,
                OutcomeReason::AssertionViolation,
            ))
        })
        .expect("failed nested execution must still record");
        assert_passed(&record);
    }

    #[test]
    #[should_panic(expected = "did not pass")]
    fn assert_passed_rejects_unavailable_outcome() {
        let record = record_regression(spec(Dimension::Unit), || {
            nested_with(outcome(
                ExecutionStatus::Inconclusive,
                OutcomeReason::CapabilityUnavailable,
            ))
        })
        .expect("unavailable nested execution must still record");
        assert_passed(&record);
    }

    #[test]
    #[should_panic(expected = "did not pass")]
    fn assert_passed_rejects_not_executed_outcome() {
        let record = record_regression(spec(Dimension::Unit), || {
            nested_with(outcome(
                ExecutionStatus::NotExecuted,
                OutcomeReason::Cancelled,
            ))
        })
        .expect("not-executed nested execution must still record");
        assert_passed(&record);
    }

    #[test]
    #[should_panic(expected = "did not pass")]
    fn assert_passed_rejects_pass_status_with_unavailable_reason() {
        let record = record_regression(spec(Dimension::Unit), || {
            nested_with(outcome(
                ExecutionStatus::Passed,
                OutcomeReason::CapabilityUnavailable,
            ))
        })
        .expect("nested execution must record");
        assert_passed(&record);
    }
}
