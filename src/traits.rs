use crate::types::*;
use std::time::{Duration, SystemTime};

#[derive(Clone, Debug)]
pub enum BackendError {
    Unsupported { reason: String },
    InvalidRequest { reason: String },
    Infrastructure { reason: String },
    ArtifactFailure { reason: String },
}
impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported { reason }
            | Self::InvalidRequest { reason }
            | Self::Infrastructure { reason }
            | Self::ArtifactFailure { reason } => f.write_str(reason),
        }
    }
}
impl std::error::Error for BackendError {}

/// Assertion violations belong in evidence, not BackendError.
pub trait TestBackend {
    fn identity(&self) -> BackendIdentity;
    fn supports(&self, declaration: &TestDeclaration) -> bool;
    fn execute(
        &self,
        declaration: &TestDeclaration,
        request: &RunRequest,
        context: &mut dyn ExecutionContext,
    ) -> Result<EvidenceRecord, BackendError>;
}
pub trait ExecutionContext {
    fn now(&self) -> SystemTime;
    fn is_cancelled(&self) -> bool;
    fn capabilities(&self) -> Vec<CapabilityObservation>;
    fn retain_artifact(
        &mut self,
        request: &ArtifactRequest,
        bytes: &[u8],
    ) -> Result<ArtifactReference, BackendError>;
}
pub trait ArtifactStore {
    fn retain(
        &mut self,
        request: &ArtifactRequest,
        bytes: &[u8],
    ) -> Result<ArtifactReference, BackendError>;
    fn verify(&self, reference: &ArtifactReference) -> Result<(), BackendError>;
}
pub trait EvidenceSink {
    fn record(&mut self, evidence: &EvidenceRecord) -> Result<(), BackendError>;
}
#[derive(Clone, Debug)]
pub struct EvidenceRequirements {
    pub expected_revision: RepositoryRevision,
    pub require_clean_working_tree: bool,
    pub require_artifact_verification: bool,
    pub mutation: Option<MutationPolicy>,
}
#[derive(Clone, Debug)]
pub struct MutationPolicy {
    pub minimum_assessed: u64,
    pub maximum_missed: u64,
    pub allow_timeouts: bool,
}
#[derive(Clone, Debug)]
pub enum EvidenceGap {
    RevisionMismatch,
    WorkingTreeNotReproducible,
    RequiredScopeNotSatisfied { detail: String },
    MissingObligation { id: ObligationId },
    ObligationNotPassed { id: ObligationId },
    ArtifactUnavailable { id: ArtifactId },
    ArtifactIntegrityFailure { id: ArtifactId },
    BaselineNotPassed,
    InsufficientMutationExperiments,
    UnresolvedMutation { id: MutationId },
    InvalidEvidenceShape,
    Other { detail: String },
}
#[derive(Clone, Debug)]
pub enum ClaimVerdict {
    Supported,
    Refuted { reason: String },
    Insufficient { gaps: Vec<EvidenceGap> },
}
/// Evaluates verification claims independently of deferrals.
pub trait EvidenceEvaluator {
    fn evaluate(
        &self,
        declaration: &TestDeclaration,
        evidence: &[EvidenceRecord],
        requirements: &EvidenceRequirements,
        artifacts: &dyn ArtifactStore,
    ) -> ClaimVerdict;
}
#[derive(Clone, Debug)]
pub enum GovernanceVerdict {
    Satisfied {
        claim_supported: bool,
        rationale: String,
    },
    Unsatisfied {
        reason: String,
    },
}
/// An accepted deferral must not become ClaimVerdict::Supported.
pub trait GovernancePolicy {
    fn assess(&self, declaration: &TestDeclaration, claim: &ClaimVerdict) -> GovernanceVerdict;
}
#[derive(Clone, Debug)]
pub enum CheckFailure {
    Violation { message: String },
    Infrastructure { message: String },
    Inconclusive { message: String },
}
pub trait CaseContext {
    fn is_cancelled(&self) -> bool;
    fn retain_artifact(
        &mut self,
        request: &ArtifactRequest,
        bytes: &[u8],
    ) -> Result<ArtifactReference, BackendError>;
}
pub struct ContractCaseDeclaration {
    pub id: TestId,
    pub obligation: ObligationId,
    pub description: String,
    pub required_capabilities: Vec<CapabilityRequirement>,
}
/// Concrete cases belong to the project that owns the port contract.
pub trait ConformanceCase<Port: ?Sized> {
    fn declaration(&self) -> &ContractCaseDeclaration;
    fn check(&self, subject: &mut Port, context: &mut dyn CaseContext) -> Result<(), CheckFailure>;
}
/// Constructs a fresh, isolated subject for each case.
pub trait SubjectFactory<Port: ?Sized> {
    fn implementation(&self) -> ImplementationIdentity;
    fn capabilities(&self) -> Vec<CapabilityObservation>;
    fn create(&self, context: &mut dyn CaseContext) -> Result<Box<Port>, CheckFailure>;
}
pub trait ConformanceSuite<Port: ?Sized> {
    fn contract(&self) -> ContractIdentity;
    fn cases(&self) -> &[Box<dyn ConformanceCase<Port>>];
}
/// Explicit cleanup when Drop alone is insufficient.
pub trait SubjectCleanup<Port: ?Sized> {
    fn cleanup(
        &self,
        subject: Box<Port>,
        context: &mut dyn CaseContext,
    ) -> Result<(), CheckFailure>;
}
#[derive(Clone, Debug)]
pub struct CaseTiming {
    pub setup: Duration,
    pub execution: Duration,
    pub cleanup: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_error_display_shows_reason_for_each_variant() {
        assert_eq!(
            BackendError::Unsupported {
                reason: "no fuzz toolchain".to_string()
            }
            .to_string(),
            "no fuzz toolchain"
        );
        assert_eq!(
            BackendError::InvalidRequest {
                reason: "missing subject".to_string()
            }
            .to_string(),
            "missing subject"
        );
        assert_eq!(
            BackendError::Infrastructure {
                reason: "process spawn failed".to_string()
            }
            .to_string(),
            "process spawn failed"
        );
        assert_eq!(
            BackendError::ArtifactFailure {
                reason: "digest mismatch".to_string()
            }
            .to_string(),
            "digest mismatch"
        );
    }
}
