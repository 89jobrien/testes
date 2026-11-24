//! `testes-fuzz` owns the `fuzz!` dimension.
//!
//! This crate ships a std-only seed-corpus runner. It executes the inline seed
//! inputs declared with `fuzz!` under `catch_unwind` and records
//! `DimensionEvidence::Fuzz`. It is not a coverage-guided fuzzer: it performs no
//! mutation, records no coverage, and does not link libFuzzer. A libFuzzer or
//! cargo-fuzz backend belongs in a separate adapter, which would return
//! [`FuzzCampaign::Unavailable`] or an executed run as appropriate.
//!
//! See `docs/decisions/0006-fuzz-macro-grammar.md` for the macro grammar.

#![forbid(unsafe_code)]

pub use testes_core::{
    BackendConfiguration, BackendIdentity, Claim, DeclarationError, Dimension, DimensionEvidence,
    EvidenceRecord, ExecutionMetadata, ExecutionOutcome, ExecutionStatus, FuzzEvidence, IdError,
    ObligationDisposition, Observed, OutcomeReason, Provenance, RepositoryRevision, ResourceLimits,
    RunId, Scope, SubjectId, TestDeclaration, TestId, TestSpecification, WorkingTreeState,
};

/// A seed input that made the target panic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeedFailure {
    /// Zero-based position of the failing input within the seed corpus.
    pub input_index: u64,
    /// Panic message, or a placeholder for non-string payloads.
    pub message: String,
}

/// Measurements from one in-process seed-corpus run.
#[derive(Clone, Debug)]
pub struct SeedCorpusRun {
    pub started_at: std::time::SystemTime,
    pub duration: std::time::Duration,
    /// Number of inline seed inputs supplied to the run.
    pub planned_inputs: u64,
    /// Number of target invocations actually made, including a failing one.
    pub executions: u64,
    pub failure: Option<SeedFailure>,
}

/// The result of attempting a fuzz campaign.
#[derive(Clone, Debug)]
pub enum FuzzCampaign {
    Executed(SeedCorpusRun),
    /// The campaign could not run. This is reported as not executed, never as a pass.
    Unavailable {
        reason: String,
    },
}

/// Builds a fuzz declaration with a `Fuzz` specification.
pub fn declare(
    id: &str,
    subject: &str,
    claim: &str,
    target: &str,
    oracles: Vec<String>,
) -> Result<TestDeclaration, IdError> {
    Ok(TestDeclaration {
        id: TestId::new(id)?,
        primary_dimension: Dimension::Fuzz,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new(subject)?,
        claim: Claim {
            statement: claim.to_string(),
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: "fuzz seed corpus".to_string(),
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
        specification: TestSpecification::Fuzz {
            target_id: target.to_string(),
            oracle_ids: oracles,
            corpus_artifacts: Vec::new(),
        },
    })
}

/// Runs `target` against each seed input, stopping at the first panic.
pub fn run_seed_corpus<F>(corpus: &[&[u8]], mut target: F) -> FuzzCampaign
where
    F: FnMut(&[u8]),
{
    let started_at = std::time::SystemTime::now();
    let started = std::time::Instant::now();
    let mut executions = 0_u64;
    let mut failure = None;
    for (index, input) in corpus.iter().enumerate() {
        executions += 1;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| target(input)));
        if let Err(payload) = result {
            failure = Some(SeedFailure {
                input_index: index as u64,
                message: panic_message(payload.as_ref()),
            });
            break;
        }
    }
    FuzzCampaign::Executed(SeedCorpusRun {
        started_at,
        duration: started.elapsed(),
        planned_inputs: corpus.len() as u64,
        executions,
        failure,
    })
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

/// Converts a campaign result into an evidence record for `declaration`.
///
/// Returns `DimensionMismatch` when the declaration's specification is not `Fuzz`.
/// Unavailable campaigns map to `NotExecuted` with unavailable (not zero) executions.
pub fn record_campaign(
    declaration: TestDeclaration,
    campaign: FuzzCampaign,
) -> Result<EvidenceRecord, DeclarationError> {
    declaration.validate()?;
    let TestSpecification::Fuzz {
        target_id,
        oracle_ids,
        ..
    } = &declaration.specification
    else {
        return Err(DeclarationError::DimensionMismatch);
    };
    let target_id = target_id.clone();
    let oracle_ids = oracle_ids.clone();

    let (status, reason, diagnostic, executions, duration, started_at, input_domain) =
        match campaign {
            FuzzCampaign::Executed(run) => {
                let input_domain =
                    format!("fuzz seed corpus: {} inline input(s)", run.planned_inputs);
                let executions = Observed::Available(run.executions);
                if let Some(failure) = run.failure {
                    (
                        ExecutionStatus::Failed,
                        OutcomeReason::CrashDetected,
                        Some(format!(
                            "seed input {} panicked: {}; the failing input is not retained as an artifact",
                            failure.input_index, failure.message
                        )),
                        executions,
                        run.duration,
                        run.started_at,
                        input_domain,
                    )
                } else if run.executions == 0 {
                    (
                        ExecutionStatus::Inconclusive,
                        OutcomeReason::InsufficientBounds,
                        Some("seed corpus is empty; the target was never exercised".to_string()),
                        executions,
                        run.duration,
                        run.started_at,
                        input_domain,
                    )
                } else {
                    (
                        ExecutionStatus::Passed,
                        OutcomeReason::AssertionsSatisfied,
                        None,
                        executions,
                        run.duration,
                        run.started_at,
                        input_domain,
                    )
                }
            }
            FuzzCampaign::Unavailable { reason } => (
                ExecutionStatus::NotExecuted,
                OutcomeReason::CapabilityUnavailable,
                Some(reason.clone()),
                Observed::Unavailable {
                    reason: reason.clone(),
                },
                std::time::Duration::ZERO,
                std::time::SystemTime::now(),
                "fuzz campaign not executed".to_string(),
            ),
        };

    Ok(EvidenceRecord {
        schema_version: 1,
        declaration,
        provenance: Provenance {
            run_id: RunId::new("fuzz-seed-run").expect("run id literal is valid"),
            revision: RepositoryRevision {
                repository: "unknown".to_string(),
                commit: "unknown".to_string(),
                working_tree: WorkingTreeState::Unknown,
            },
        },
        execution: ExecutionMetadata {
            backend: BackendIdentity {
                name: "testes-fuzz-seed-runner".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                toolchain: None,
            },
            configuration: BackendConfiguration::new(),
            started_at,
            duration,
            actual_scope: Scope {
                input_domain,
                assumptions: Vec::new(),
                bounds: Vec::new(),
                capabilities: Vec::new(),
                resources: ResourceLimits {
                    wall_time: None,
                    memory_bytes: None,
                    max_input_bytes: None,
                },
            },
        },
        outcome: ExecutionOutcome {
            status,
            reason,
            diagnostic,
            obligations: Vec::new(),
        },
        artifacts: Vec::new(),
        dimension_evidence: DimensionEvidence::Fuzz(FuzzEvidence {
            target_id,
            oracle_ids,
            initial_corpus: Vec::new(),
            actual_duration: duration,
            executions,
            coverage: Observed::Unavailable {
                reason: "coverage instrumentation is not available to the in-process seed runner"
                    .to_string(),
            },
            failure_inputs: Vec::new(),
        }),
    })
}

/// Panics unless the record's status is `Passed`.
pub fn assert_passed(record: &EvidenceRecord) {
    if record.outcome.status != ExecutionStatus::Passed {
        panic!(
            "fuzz test `{}` did not pass: status {:?}, reason {:?}{}",
            record.declaration.id,
            record.outcome.status,
            record.outcome.reason,
            record
                .outcome
                .diagnostic
                .as_deref()
                .map(|d| format!(", diagnostic: {d}"))
                .unwrap_or_default(),
        );
    }
}

/// Declares a fuzz test. Expands to a `#[test]` function that runs the seed
/// corpus, records `DimensionEvidence::Fuzz`, and fails unless the status is `Passed`.
#[macro_export]
macro_rules! fuzz {
    (
        $(#[$meta:meta])*
        fn $name:ident {
            id: $id:expr,
            subject: $subject:expr,
            claim: $claim:expr,
            target: $target:expr,
            oracles: [$($oracle:expr),* $(,)?],
            corpus: [$($seed:expr),* $(,)?],
            body: $body:expr $(,)?
        }
    ) => {
        $(#[$meta])*
        #[test]
        fn $name() {
            let mut oracles: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();
            $( oracles.push(::std::string::ToString::to_string(&$oracle)); )*
            let mut corpus: ::std::vec::Vec<&[u8]> = ::std::vec::Vec::new();
            $( corpus.push($seed); )*
            let declaration = $crate::declare($id, $subject, $claim, $target, oracles)
                .expect("fuzz! identifiers must not be blank or contain control characters");
            let campaign = $crate::run_seed_corpus(&corpus, $body);
            let record = $crate::record_campaign(declaration, campaign)
                .expect("fuzz! declaration must be valid");
            $crate::assert_passed(&record);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decl() -> TestDeclaration {
        declare(
            "parse-frames",
            "frame parser",
            "arbitrary bytes cannot panic the parser",
            "frame_parser",
            vec!["no-panic".to_string()],
        )
        .expect("declaration identifiers must be valid")
    }

    fn executed(campaign: FuzzCampaign) -> SeedCorpusRun {
        match campaign {
            FuzzCampaign::Executed(run) => run,
            other => panic!("expected an executed campaign, got {other:?}"),
        }
    }

    fn fuzz_evidence(record: &EvidenceRecord) -> &FuzzEvidence {
        match &record.dimension_evidence {
            DimensionEvidence::Fuzz(evidence) => evidence,
            other => panic!("expected fuzz evidence, got {other:?}"),
        }
    }

    #[test]
    fn declare_builds_fuzz_specification_with_target_and_oracles() {
        let declaration = decl();
        assert_eq!(declaration.primary_dimension, Dimension::Fuzz);
        match &declaration.specification {
            TestSpecification::Fuzz {
                target_id,
                oracle_ids,
                corpus_artifacts,
            } => {
                assert_eq!(target_id, "frame_parser");
                assert_eq!(oracle_ids, &vec!["no-panic".to_string()]);
                assert!(corpus_artifacts.is_empty());
            }
            other => panic!("expected a Fuzz specification, got {other:?}"),
        }
    }

    #[test]
    fn declare_rejects_blank_identifier() {
        let result = declare("   ", "frame parser", "claim", "frame_parser", Vec::new());
        assert!(matches!(result, Err(IdError::Empty)));
    }

    #[test]
    fn run_seed_corpus_executes_every_input_when_target_passes() {
        let corpus: [&[u8]; 3] = [b"a", b"bb", b"ccc"];
        let run = executed(run_seed_corpus(&corpus, |_| {}));
        assert_eq!(run.planned_inputs, 3);
        assert_eq!(run.executions, 3);
        assert_eq!(run.failure, None);
    }

    #[test]
    fn run_seed_corpus_stops_at_first_panicking_input() {
        let corpus: [&[u8]; 3] = [b"ok", b"boom", b"never"];
        let run = executed(run_seed_corpus(&corpus, |input| {
            if input == &b"boom"[..] {
                panic!("boom input");
            }
        }));
        assert_eq!(run.planned_inputs, 3);
        assert_eq!(run.executions, 2);
        assert_eq!(
            run.failure,
            Some(SeedFailure {
                input_index: 1,
                message: "boom input".to_string(),
            })
        );
    }

    #[test]
    fn run_seed_corpus_with_empty_corpus_executes_nothing() {
        let corpus: [&[u8]; 0] = [];
        let run = executed(run_seed_corpus(&corpus, |_| {}));
        assert_eq!(run.planned_inputs, 0);
        assert_eq!(run.executions, 0);
        assert_eq!(run.failure, None);
    }

    #[test]
    fn passing_run_records_passed_fuzz_evidence_with_observed_executions() {
        let corpus: [&[u8]; 3] = [b"a", b"bb", b"ccc"];
        let record = record_campaign(decl(), run_seed_corpus(&corpus, |_| {}))
            .expect("fuzz declaration must be valid");
        assert_eq!(record.outcome.status, ExecutionStatus::Passed);
        assert_eq!(record.outcome.reason, OutcomeReason::AssertionsSatisfied);
        assert_eq!(record.outcome.diagnostic, None);
        assert!(record.validate_shape().is_ok());

        let evidence = fuzz_evidence(&record);
        assert_eq!(evidence.target_id, "frame_parser");
        assert_eq!(evidence.oracle_ids, vec!["no-panic".to_string()]);
        assert!(matches!(evidence.executions, Observed::Available(3)));
        assert!(matches!(evidence.coverage, Observed::Unavailable { .. }));
        assert!(evidence.failure_inputs.is_empty());
    }

    #[test]
    fn failing_run_records_failed_crash_with_panic_message() {
        let corpus: [&[u8]; 2] = [b"a", b"boom"];
        let record = record_campaign(
            decl(),
            run_seed_corpus(&corpus, |input| {
                if input == &b"boom"[..] {
                    panic!("boom input");
                }
            }),
        )
        .expect("fuzz declaration must be valid");
        assert_eq!(record.outcome.status, ExecutionStatus::Failed);
        assert_eq!(record.outcome.reason, OutcomeReason::CrashDetected);
        let diagnostic = record
            .outcome
            .diagnostic
            .as_deref()
            .expect("failed run must carry a diagnostic");
        assert!(diagnostic.contains("boom input"));
        assert!(record.validate_shape().is_ok());
        assert!(matches!(
            fuzz_evidence(&record).executions,
            Observed::Available(2)
        ));
    }

    #[test]
    fn empty_corpus_is_inconclusive_not_passed() {
        let corpus: [&[u8]; 0] = [];
        let record = record_campaign(decl(), run_seed_corpus(&corpus, |_| {}))
            .expect("fuzz declaration must be valid");
        assert_eq!(record.outcome.status, ExecutionStatus::Inconclusive);
        assert_eq!(record.outcome.reason, OutcomeReason::InsufficientBounds);
        assert!(record.validate_shape().is_ok());
        assert!(matches!(
            fuzz_evidence(&record).executions,
            Observed::Available(0)
        ));
    }

    #[test]
    fn unavailable_campaign_is_not_executed_and_executions_are_not_zero() {
        let record = record_campaign(
            decl(),
            FuzzCampaign::Unavailable {
                reason: "cargo-fuzz not installed".to_string(),
            },
        )
        .expect("fuzz declaration must be valid");
        assert_eq!(record.outcome.status, ExecutionStatus::NotExecuted);
        assert_eq!(record.outcome.reason, OutcomeReason::CapabilityUnavailable);
        assert_eq!(
            record.outcome.diagnostic.as_deref(),
            Some("cargo-fuzz not installed")
        );
        assert!(record.validate_shape().is_ok());
        let evidence = fuzz_evidence(&record);
        assert!(matches!(evidence.executions, Observed::Unavailable { .. }));
        assert!(matches!(evidence.coverage, Observed::Unavailable { .. }));
    }

    #[test]
    fn record_rejects_declaration_whose_specification_is_not_fuzz() {
        let mut declaration = decl();
        declaration.primary_dimension = Dimension::Unit;
        declaration.specification = TestSpecification::Unit {
            example_ids: Vec::new(),
        };
        let result = record_campaign(
            declaration,
            FuzzCampaign::Unavailable {
                reason: "not run".to_string(),
            },
        );
        assert!(matches!(result, Err(DeclarationError::DimensionMismatch)));
    }

    #[test]
    fn record_rejects_blank_claim() {
        let mut declaration = decl();
        declaration.claim.statement = "   ".to_string();
        let result = record_campaign(
            declaration,
            FuzzCampaign::Unavailable {
                reason: "not run".to_string(),
            },
        );
        assert!(matches!(result, Err(DeclarationError::EmptyClaim)));
    }

    #[test]
    fn assert_passed_accepts_passed_record() {
        let corpus: [&[u8]; 1] = [b"a"];
        let record = record_campaign(decl(), run_seed_corpus(&corpus, |_| {}))
            .expect("fuzz declaration must be valid");
        assert_passed(&record);
    }

    #[test]
    #[should_panic(expected = "did not pass")]
    fn assert_passed_rejects_unavailable_record() {
        let record = record_campaign(
            decl(),
            FuzzCampaign::Unavailable {
                reason: "cargo-fuzz not installed".to_string(),
            },
        )
        .expect("fuzz declaration must be valid");
        assert_passed(&record);
    }

    #[test]
    #[should_panic(expected = "boom input")]
    fn assert_passed_rejects_failed_record_with_diagnostic() {
        let corpus: [&[u8]; 1] = [b"boom"];
        let record = record_campaign(decl(), run_seed_corpus(&corpus, |_| panic!("boom input")))
            .expect("fuzz declaration must be valid");
        assert_passed(&record);
    }
}
