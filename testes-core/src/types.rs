use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    time::{Duration, SystemTime},
};

macro_rules! string_id {
    ($($name:ident),+ $(,)?) => {$(
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);
        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, IdError> {
                let value = value.into();
                if value.trim().is_empty() { return Err(IdError::Empty); }
                if value.chars().any(char::is_control) { return Err(IdError::ControlCharacter); }
                Ok(Self(value))
            }
            pub fn as_str(&self) -> &str { &self.0 }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) }
        }
    )+};
}
string_id!(
    TestId,
    RunId,
    SubjectId,
    ObligationId,
    ContractId,
    ImplementationId,
    ArtifactId,
    FailureId,
    MutationId
);

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error, miette::Diagnostic)]
pub enum IdError {
    #[error("identifier must not be blank")]
    Empty,
    #[error("identifier must not contain control characters")]
    ControlCharacter,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Dimension {
    Unit,
    Property,
    Fuzz,
    ModelCheck,
    Conformance,
    Integration,
    Regression,
    Mutation,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claim {
    pub statement: String,
    pub obligations: Vec<ObligationDefinition>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObligationDefinition {
    pub id: ObligationId,
    pub description: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assumption {
    pub expression: String,
    pub justification: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityRequirement {
    pub name: String,
    pub description: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedBound {
    pub name: String,
    pub value: u64,
    pub justification: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceLimits {
    pub wall_time: Option<Duration>,
    pub memory_bytes: Option<u64>,
    pub max_input_bytes: Option<u64>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Scope {
    pub input_domain: String,
    pub assumptions: Vec<Assumption>,
    pub bounds: Vec<NamedBound>,
    pub capabilities: Vec<CapabilityRequirement>,
    pub resources: ResourceLimits,
}
/// Policy intent, not an execution result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObligationDisposition {
    Required,
    NotApplicable {
        rationale: String,
    },
    Deferred {
        rationale: String,
        owner: String,
        review_condition: String,
    },
}
#[derive(Clone, Debug)]
pub struct TestDeclaration {
    pub id: TestId,
    pub primary_dimension: Dimension,
    pub secondary_dimensions: Vec<Dimension>,
    pub subject: SubjectId,
    pub claim: Claim,
    pub required_scope: Scope,
    pub disposition: ObligationDisposition,
    pub specification: TestSpecification,
}
/// Describes execution requirements; executable code belongs to a backend.
#[derive(Clone, Debug)]
pub enum TestSpecification {
    Unit {
        example_ids: Vec<String>,
    },
    Property {
        strategy_id: String,
        minimum_cases: u64,
        replay_artifacts: Vec<ArtifactId>,
    },
    Fuzz {
        target_id: String,
        oracle_ids: Vec<String>,
        corpus_artifacts: Vec<ArtifactId>,
    },
    ModelCheck {
        harness_id: String,
        obligation_ids: Vec<ObligationId>,
    },
    Conformance {
        contract: ContractIdentity,
        implementation: ImplementationIdentity,
        case_ids: Vec<TestId>,
    },
    Integration {
        seam_id: String,
        components: Vec<ComponentIdentity>,
    },
    Regression {
        failure: FailureId,
        underlying_dimension: Dimension,
        reproducer_artifacts: Vec<ArtifactId>,
    },
    Mutation {
        source_selectors: Vec<String>,
        operators: Vec<String>,
        selected_checks: Vec<TestId>,
        commands: Vec<CommandSpec>,
    },
}
impl TestSpecification {
    pub fn dimension(&self) -> Dimension {
        match self {
            Self::Unit { .. } => Dimension::Unit,
            Self::Property { .. } => Dimension::Property,
            Self::Fuzz { .. } => Dimension::Fuzz,
            Self::ModelCheck { .. } => Dimension::ModelCheck,
            Self::Conformance { .. } => Dimension::Conformance,
            Self::Integration { .. } => Dimension::Integration,
            Self::Regression { .. } => Dimension::Regression,
            Self::Mutation { .. } => Dimension::Mutation,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error, miette::Diagnostic)]
pub enum DeclarationError {
    #[error("declaration's primary dimension does not match its specification's dimension")]
    DimensionMismatch,
    #[error("declaration's claim statement must not be blank")]
    EmptyClaim,
    #[error("duplicate obligation id in claim: {0}")]
    DuplicateObligation(ObligationId),
    #[error("regression specification's underlying dimension is invalid")]
    InvalidRegressionDimension,
}
impl TestDeclaration {
    pub fn validate(&self) -> Result<(), DeclarationError> {
        if self.primary_dimension != self.specification.dimension() {
            return Err(DeclarationError::DimensionMismatch);
        }
        if self.claim.statement.trim().is_empty() {
            return Err(DeclarationError::EmptyClaim);
        }
        let mut seen = BTreeSet::new();
        for obligation in &self.claim.obligations {
            if !seen.insert(&obligation.id) {
                return Err(DeclarationError::DuplicateObligation(obligation.id.clone()));
            }
        }
        if let TestSpecification::Regression {
            underlying_dimension,
            ..
        } = &self.specification
            && matches!(
                underlying_dimension,
                Dimension::Regression | Dimension::Mutation
            )
        {
            return Err(DeclarationError::InvalidRegressionDimension);
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionStatus {
    Passed,
    Failed,
    Inconclusive,
    NotExecuted,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutcomeReason {
    AssertionsSatisfied,
    AssertionViolation,
    CounterexampleFound,
    CrashDetected,
    BaselineFailed,
    CapabilityUnavailable,
    BudgetNotCompleted,
    InsufficientBounds,
    UnsupportedOperation,
    GenerationAborted,
    Cancelled,
    InfrastructureFailure,
    PolicySuppressed,
    BackendDefined(String),
}
#[derive(Clone, Debug)]
pub struct ObligationResult {
    pub id: ObligationId,
    pub status: ExecutionStatus,
    pub diagnostic: Option<String>,
}
#[derive(Clone, Debug)]
pub struct ExecutionOutcome {
    pub status: ExecutionStatus,
    pub reason: OutcomeReason,
    pub diagnostic: Option<String>,
    pub obligations: Vec<ObligationResult>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryRevision {
    pub repository: String,
    pub commit: String,
    pub working_tree: WorkingTreeState,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkingTreeState {
    Clean,
    Dirty { patch_artifact: ArtifactId },
    Unknown,
}
#[derive(Clone, Debug)]
pub struct Provenance {
    pub run_id: RunId,
    pub revision: RepositoryRevision,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendIdentity {
    pub name: String,
    pub version: String,
    pub toolchain: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigurationValue {
    Public(String),
    Redacted,
    Artifact(ArtifactId),
}
pub type BackendConfiguration = BTreeMap<String, ConfigurationValue>;
#[derive(Clone, Debug)]
pub struct ExecutionMetadata {
    pub backend: BackendIdentity,
    pub configuration: BackendConfiguration,
    pub started_at: SystemTime,
    pub duration: Duration,
    pub actual_scope: Scope,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactKind {
    Fixture,
    Counterexample,
    Corpus,
    CrashInput,
    ProofReport,
    ProofTrace,
    MutationReport,
    ProcessOutput,
    HistoricalFailure,
    WorkingTreePatch,
    Other(String),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentDigest {
    pub algorithm: String,
    pub value: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactDisclosure {
    Public,
    Restricted,
    Redacted { policy: String },
}
#[derive(Clone, Debug)]
pub struct ArtifactReference {
    pub id: ArtifactId,
    pub kind: ArtifactKind,
    pub location: String,
    pub digest: ContentDigest,
    pub disclosure: ArtifactDisclosure,
}
#[derive(Clone, Debug)]
pub struct ArtifactRequest {
    pub kind: ArtifactKind,
    pub media_type: String,
    pub disclosure: ArtifactDisclosure,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractIdentity {
    pub id: ContractId,
    pub version: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImplementationIdentity {
    pub id: ImplementationId,
    pub configuration_id: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentIdentity {
    pub name: String,
    pub version: Option<String>,
    pub boundary: BoundaryKind,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundaryKind {
    Real,
    Fake,
    Simulated,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandSpec {
    pub program: String,
    pub arguments: Vec<String>,
    pub working_directory: Option<String>,
    pub environment: BTreeMap<String, ConfigurationValue>,
}
#[derive(Clone, Debug)]
pub struct RunRequest {
    pub provenance: Provenance,
    pub configuration: BackendConfiguration,
    pub resource_limits: ResourceLimits,
}
/// An unavailable measurement is not zero.
#[derive(Clone, Debug)]
pub enum Observed<T> {
    Available(T),
    Unavailable { reason: String },
}
#[derive(Clone, Debug)]
pub struct UnitEvidence {
    pub executed_examples: Vec<String>,
    pub assertions_evaluated: Observed<u64>,
}
#[derive(Clone, Debug)]
pub struct PropertyEvidence {
    pub strategy_id: String,
    pub seed: Observed<String>,
    pub completed_cases: u64,
    pub rejected_cases: Observed<u64>,
    pub replayed_artifacts: Vec<ArtifactId>,
    pub minimized_counterexample: Option<ArtifactId>,
}
#[derive(Clone, Debug)]
pub struct CoverageMeasurement {
    pub metric: String,
    pub value: u64,
}
#[derive(Clone, Debug)]
pub struct FuzzEvidence {
    pub target_id: String,
    pub oracle_ids: Vec<String>,
    pub initial_corpus: Vec<ArtifactId>,
    pub actual_duration: Duration,
    pub executions: Observed<u64>,
    pub coverage: Observed<Vec<CoverageMeasurement>>,
    pub failure_inputs: Vec<ArtifactId>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProofStatus {
    Established,
    Refuted,
    Inconclusive,
    NotExecuted,
}
#[derive(Clone, Debug)]
pub struct ProofObligationResult {
    pub id: ObligationId,
    pub status: ProofStatus,
    pub diagnostic: Option<String>,
    pub counterexample: Option<ArtifactId>,
}
#[derive(Clone, Debug)]
pub struct ModelCheckEvidence {
    pub harness_id: String,
    pub symbolic_inputs: Vec<String>,
    pub assumptions: Vec<Assumption>,
    pub bounds: Vec<NamedBound>,
    pub obligations: Vec<ProofObligationResult>,
    pub non_vacuity: Observed<bool>,
    pub report: Option<ArtifactId>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CapabilityState {
    Available,
    Unavailable { reason: String },
}
#[derive(Clone, Debug)]
pub struct CapabilityObservation {
    pub name: String,
    pub state: CapabilityState,
}
#[derive(Clone, Debug)]
pub enum ContractCaseOutcome {
    Executed(ExecutionOutcome),
    NotApplicable { rationale: String },
    NotExecuted { reason: String },
}
#[derive(Clone, Debug)]
pub struct ContractCaseEvidence {
    pub case_id: TestId,
    pub obligation: ObligationId,
    pub duration: Duration,
    pub outcome: ContractCaseOutcome,
}
#[derive(Clone, Debug)]
pub struct ConformanceEvidence {
    pub contract: ContractIdentity,
    pub implementation: ImplementationIdentity,
    pub capabilities: Vec<CapabilityObservation>,
    pub cases: Vec<ContractCaseEvidence>,
}
#[derive(Clone, Debug)]
pub struct IntegrationEvidence {
    pub seam_id: String,
    pub components: Vec<ComponentIdentity>,
    pub fixtures: Vec<ArtifactId>,
    pub observations: Vec<ArtifactId>,
    pub cleanup: ExecutionOutcome,
}
#[derive(Clone, Debug)]
pub struct RegressionEvidence {
    pub failure: FailureId,
    pub reproducer_artifacts: Vec<ArtifactId>,
    pub historical_failure: Option<ArtifactId>,
    /// Reuses the underlying execution rather than running it twice.
    pub execution: Box<DimensionEvidence>,
}
#[derive(Clone, Debug)]
pub struct MutationDefinition {
    pub id: MutationId,
    pub location: String,
    pub operator: String,
    pub transformation: String,
}
#[derive(Clone, Debug)]
pub enum MutationOutcome {
    Caught {
        detecting_checks: Observed<Vec<TestId>>,
        diagnostic: Option<String>,
    },
    Missed,
    Unviable {
        diagnostic: String,
    },
    Timeout {
        limit: Duration,
    },
    Excluded {
        rationale: String,
    },
}
#[derive(Clone, Debug)]
pub struct MutationResult {
    pub mutation: MutationDefinition,
    pub outcome: MutationOutcome,
    pub duration: Duration,
    pub artifacts: Vec<ArtifactId>,
}
#[derive(Clone, Debug)]
pub struct MutationEvidence {
    pub source_selectors: Vec<String>,
    pub operators: Vec<String>,
    pub selected_checks: Vec<TestId>,
    pub commands: Vec<CommandSpec>,
    pub baseline: ExecutionOutcome,
    pub mutations: Vec<MutationResult>,
}
impl MutationEvidence {
    /// Excludes unviable, timeout, and excluded outcomes. None means no assessed experiments.
    pub fn detection_rate(&self) -> Option<f64> {
        let mut caught = 0_u64;
        let mut missed = 0_u64;
        for result in &self.mutations {
            match &result.outcome {
                MutationOutcome::Caught { .. } => caught += 1,
                MutationOutcome::Missed => missed += 1,
                _ => {}
            }
        }
        let assessed = caught + missed;
        if assessed == 0 {
            None
        } else {
            Some(caught as f64 / assessed as f64)
        }
    }
}
#[derive(Clone, Debug)]
pub enum DimensionEvidence {
    Unit(UnitEvidence),
    Property(PropertyEvidence),
    Fuzz(FuzzEvidence),
    ModelCheck(ModelCheckEvidence),
    Conformance(ConformanceEvidence),
    Integration(IntegrationEvidence),
    Regression(RegressionEvidence),
    Mutation(MutationEvidence),
}
impl DimensionEvidence {
    pub fn dimension(&self) -> Dimension {
        match self {
            Self::Unit(_) => Dimension::Unit,
            Self::Property(_) => Dimension::Property,
            Self::Fuzz(_) => Dimension::Fuzz,
            Self::ModelCheck(_) => Dimension::ModelCheck,
            Self::Conformance(_) => Dimension::Conformance,
            Self::Integration(_) => Dimension::Integration,
            Self::Regression(_) => Dimension::Regression,
            Self::Mutation(_) => Dimension::Mutation,
        }
    }
}
#[derive(Clone, Debug)]
pub struct EvidenceRecord {
    pub schema_version: u32,
    pub declaration: TestDeclaration,
    pub provenance: Provenance,
    pub execution: ExecutionMetadata,
    pub outcome: ExecutionOutcome,
    pub artifacts: Vec<ArtifactReference>,
    pub dimension_evidence: DimensionEvidence,
}
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error, miette::Diagnostic)]
pub enum EvidenceShapeError {
    #[error("evidence record's declaration failed validation: {0}")]
    InvalidDeclaration(#[source] DeclarationError),
    #[error("evidence record's dimension does not match its declaration's primary dimension")]
    DimensionMismatch,
    #[error(
        "regression payload's underlying execution dimension does not match the declared dimension"
    )]
    InvalidRegressionPayload,
}
impl EvidenceRecord {
    /// Structural validation only; does not establish claim satisfaction.
    pub fn validate_shape(&self) -> Result<(), EvidenceShapeError> {
        self.declaration
            .validate()
            .map_err(EvidenceShapeError::InvalidDeclaration)?;
        if self.declaration.primary_dimension != self.dimension_evidence.dimension() {
            return Err(EvidenceShapeError::DimensionMismatch);
        }
        if let (
            TestSpecification::Regression {
                underlying_dimension,
                ..
            },
            DimensionEvidence::Regression(regression),
        ) = (&self.declaration.specification, &self.dimension_evidence)
            && regression.execution.dimension() != *underlying_dimension
        {
            return Err(EvidenceShapeError::InvalidRegressionPayload);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claim(statement: &str) -> Claim {
        Claim {
            statement: statement.to_string(),
            obligations: Vec::new(),
        }
    }

    fn scope() -> Scope {
        Scope {
            input_domain: "test".to_string(),
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

    fn unit_declaration() -> TestDeclaration {
        TestDeclaration {
            id: TestId::new("subject-under-test").expect("id must not be blank"),
            primary_dimension: Dimension::Unit,
            secondary_dimensions: Vec::new(),
            subject: SubjectId::new("subject-under-test").expect("id must not be blank"),
            claim: claim("example behaves as expected"),
            required_scope: scope(),
            disposition: ObligationDisposition::Required,
            specification: TestSpecification::Unit {
                example_ids: vec!["example".to_string()],
            },
        }
    }

    fn execution_metadata() -> ExecutionMetadata {
        ExecutionMetadata {
            backend: BackendIdentity {
                name: "test-backend".to_string(),
                version: "0.0.0".to_string(),
                toolchain: None,
            },
            configuration: BackendConfiguration::new(),
            started_at: SystemTime::UNIX_EPOCH,
            duration: Duration::ZERO,
            actual_scope: scope(),
        }
    }

    fn passing_outcome() -> ExecutionOutcome {
        ExecutionOutcome {
            status: ExecutionStatus::Passed,
            reason: OutcomeReason::AssertionsSatisfied,
            diagnostic: None,
            obligations: Vec::new(),
        }
    }

    #[test]
    fn string_id_rejects_blank_value() {
        let result = TestId::new("   ");
        assert_eq!(result, Err(IdError::Empty));
    }

    #[test]
    fn string_id_rejects_control_characters() {
        let result = TestId::new("abc\u{0007}def");
        assert_eq!(result, Err(IdError::ControlCharacter));
    }

    #[test]
    fn string_id_accepts_valid_value() {
        let id = TestId::new("valid-id").expect("valid id must construct");
        assert_eq!(id.as_str(), "valid-id");
    }

    #[test]
    fn declaration_validate_rejects_dimension_mismatch() {
        let mut declaration = unit_declaration();
        declaration.primary_dimension = Dimension::Property;
        assert_eq!(
            declaration.validate(),
            Err(DeclarationError::DimensionMismatch)
        );
    }

    #[test]
    fn declaration_validate_rejects_empty_claim() {
        let mut declaration = unit_declaration();
        declaration.claim = claim("   ");
        assert_eq!(declaration.validate(), Err(DeclarationError::EmptyClaim));
    }

    #[test]
    fn declaration_validate_rejects_duplicate_obligation_ids() {
        let mut declaration = unit_declaration();
        let obligation_id = ObligationId::new("dup").expect("id must not be blank");
        declaration.claim.obligations = vec![
            ObligationDefinition {
                id: obligation_id.clone(),
                description: "first".to_string(),
            },
            ObligationDefinition {
                id: obligation_id.clone(),
                description: "second".to_string(),
            },
        ];
        assert_eq!(
            declaration.validate(),
            Err(DeclarationError::DuplicateObligation(obligation_id))
        );
    }

    #[test]
    fn declaration_validate_rejects_regression_wrapping_mutation() {
        let mut declaration = unit_declaration();
        declaration.primary_dimension = Dimension::Regression;
        declaration.specification = TestSpecification::Regression {
            failure: FailureId::new("failure").expect("id must not be blank"),
            underlying_dimension: Dimension::Mutation,
            reproducer_artifacts: Vec::new(),
        };
        assert_eq!(
            declaration.validate(),
            Err(DeclarationError::InvalidRegressionDimension)
        );
    }

    #[test]
    fn declaration_validate_accepts_well_formed_declaration() {
        assert_eq!(unit_declaration().validate(), Ok(()));
    }

    #[test]
    fn evidence_validate_shape_rejects_dimension_mismatch_with_evidence() {
        let record = EvidenceRecord {
            schema_version: 1,
            declaration: unit_declaration(),
            provenance: Provenance {
                run_id: RunId::new("run").expect("id must not be blank"),
                revision: RepositoryRevision {
                    repository: "testes".to_string(),
                    commit: "0000000".to_string(),
                    working_tree: WorkingTreeState::Clean,
                },
            },
            execution: execution_metadata(),
            outcome: passing_outcome(),
            artifacts: Vec::new(),
            dimension_evidence: DimensionEvidence::Property(PropertyEvidence {
                strategy_id: "wrong-dimension".to_string(),
                seed: Observed::Unavailable {
                    reason: "not applicable".to_string(),
                },
                completed_cases: 0,
                rejected_cases: Observed::Unavailable {
                    reason: "not applicable".to_string(),
                },
                replayed_artifacts: Vec::new(),
                minimized_counterexample: None,
            }),
        };
        assert_eq!(
            record.validate_shape(),
            Err(EvidenceShapeError::DimensionMismatch)
        );
    }

    #[test]
    fn evidence_validate_shape_rejects_regression_payload_dimension_mismatch() {
        let mut declaration = unit_declaration();
        declaration.primary_dimension = Dimension::Regression;
        declaration.specification = TestSpecification::Regression {
            failure: FailureId::new("failure").expect("id must not be blank"),
            underlying_dimension: Dimension::Unit,
            reproducer_artifacts: Vec::new(),
        };
        let record = EvidenceRecord {
            schema_version: 1,
            declaration,
            provenance: Provenance {
                run_id: RunId::new("run").expect("id must not be blank"),
                revision: RepositoryRevision {
                    repository: "testes".to_string(),
                    commit: "0000000".to_string(),
                    working_tree: WorkingTreeState::Clean,
                },
            },
            execution: execution_metadata(),
            outcome: passing_outcome(),
            artifacts: Vec::new(),
            dimension_evidence: DimensionEvidence::Regression(RegressionEvidence {
                failure: FailureId::new("failure").expect("id must not be blank"),
                reproducer_artifacts: Vec::new(),
                historical_failure: None,
                execution: Box::new(DimensionEvidence::Property(PropertyEvidence {
                    strategy_id: "mismatched-underlying-dimension".to_string(),
                    seed: Observed::Unavailable {
                        reason: "not applicable".to_string(),
                    },
                    completed_cases: 0,
                    rejected_cases: Observed::Unavailable {
                        reason: "not applicable".to_string(),
                    },
                    replayed_artifacts: Vec::new(),
                    minimized_counterexample: None,
                })),
            }),
        };
        assert_eq!(
            record.validate_shape(),
            Err(EvidenceShapeError::InvalidRegressionPayload)
        );
    }

    #[test]
    fn mutation_detection_rate_is_none_when_nothing_assessed() {
        let evidence = MutationEvidence {
            source_selectors: Vec::new(),
            operators: Vec::new(),
            selected_checks: Vec::new(),
            commands: Vec::new(),
            baseline: passing_outcome(),
            mutations: vec![MutationResult {
                mutation: MutationDefinition {
                    id: MutationId::new("m1").expect("id must not be blank"),
                    location: "src/lib.rs:1".to_string(),
                    operator: "delete-statement".to_string(),
                    transformation: "removed statement".to_string(),
                },
                outcome: MutationOutcome::Unviable {
                    diagnostic: "did not compile".to_string(),
                },
                duration: Duration::ZERO,
                artifacts: Vec::new(),
            }],
        };
        assert_eq!(evidence.detection_rate(), None);
    }

    #[test]
    fn mutation_detection_rate_handles_mixed_outcomes() {
        let evidence = MutationEvidence {
            source_selectors: Vec::new(),
            operators: Vec::new(),
            selected_checks: Vec::new(),
            commands: Vec::new(),
            baseline: passing_outcome(),
            mutations: vec![
                MutationResult {
                    mutation: MutationDefinition {
                        id: MutationId::new("caught").expect("id must not be blank"),
                        location: "src/lib.rs:1".to_string(),
                        operator: "delete-statement".to_string(),
                        transformation: "removed statement".to_string(),
                    },
                    outcome: MutationOutcome::Caught {
                        detecting_checks: Observed::Available(Vec::new()),
                        diagnostic: None,
                    },
                    duration: Duration::ZERO,
                    artifacts: Vec::new(),
                },
                MutationResult {
                    mutation: MutationDefinition {
                        id: MutationId::new("missed").expect("id must not be blank"),
                        location: "src/lib.rs:2".to_string(),
                        operator: "delete-statement".to_string(),
                        transformation: "removed statement".to_string(),
                    },
                    outcome: MutationOutcome::Missed,
                    duration: Duration::ZERO,
                    artifacts: Vec::new(),
                },
                MutationResult {
                    mutation: MutationDefinition {
                        id: MutationId::new("timeout").expect("id must not be blank"),
                        location: "src/lib.rs:3".to_string(),
                        operator: "delete-statement".to_string(),
                        transformation: "removed statement".to_string(),
                    },
                    outcome: MutationOutcome::Timeout {
                        limit: Duration::from_secs(1),
                    },
                    duration: Duration::ZERO,
                    artifacts: Vec::new(),
                },
            ],
        };
        assert_eq!(evidence.detection_rate(), Some(0.5));
    }

    #[test]
    fn id_error_display_shows_message_for_each_variant() {
        assert_eq!(IdError::Empty.to_string(), "identifier must not be blank");
        assert_eq!(
            IdError::ControlCharacter.to_string(),
            "identifier must not contain control characters"
        );
    }

    #[test]
    fn declaration_error_display_shows_message_for_each_variant() {
        assert_eq!(
            DeclarationError::DimensionMismatch.to_string(),
            "declaration's primary dimension does not match its specification's dimension"
        );
        assert_eq!(
            DeclarationError::EmptyClaim.to_string(),
            "declaration's claim statement must not be blank"
        );
        assert_eq!(
            DeclarationError::DuplicateObligation(
                ObligationId::new("dup").expect("id must not be blank")
            )
            .to_string(),
            "duplicate obligation id in claim: dup"
        );
        assert_eq!(
            DeclarationError::InvalidRegressionDimension.to_string(),
            "regression specification's underlying dimension is invalid"
        );
    }

    #[test]
    fn evidence_shape_error_display_shows_message_for_each_variant() {
        assert_eq!(
            EvidenceShapeError::InvalidDeclaration(DeclarationError::EmptyClaim).to_string(),
            "evidence record's declaration failed validation: declaration's claim statement must not be blank"
        );
        assert_eq!(
            EvidenceShapeError::DimensionMismatch.to_string(),
            "evidence record's dimension does not match its declaration's primary dimension"
        );
        assert_eq!(
            EvidenceShapeError::InvalidRegressionPayload.to_string(),
            "regression payload's underlying execution dimension does not match the declared dimension"
        );
    }
}
