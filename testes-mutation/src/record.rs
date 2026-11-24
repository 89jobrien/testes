use crate::assessment::{MutationAssessment, assess};
use std::collections::BTreeMap;
use testes_core::{
    ArtifactReference, Claim, CommandSpec, DeclarationError, Dimension, DimensionEvidence,
    EvidenceRecord, ExecutionMetadata, IdError, MutationEvidence, MutationPolicy,
    ObligationDisposition, Provenance, ResourceLimits, Scope, SubjectId, TestDeclaration, TestId,
    TestSpecification,
};

/// Declared mutation specification, before any campaign has run.
#[derive(Clone, Debug)]
pub struct MutationSpec {
    pub id: String,
    pub subject: String,
    pub claim: String,
    pub source_selectors: Vec<String>,
    pub operators: Vec<String>,
    pub selected_checks: Vec<String>,
    pub commands: Vec<CommandSpec>,
}

/// What a campaign runner actually produced. Nothing here is invented by testes.
#[derive(Clone, Debug)]
pub struct MutationCampaign {
    pub provenance: Provenance,
    pub execution: ExecutionMetadata,
    pub artifacts: Vec<ArtifactReference>,
    pub evidence: MutationEvidence,
}

/// Evidence record plus its claim assessment. Not a verdict on its own.
#[must_use]
#[derive(Clone, Debug)]
pub struct MutationRecord {
    pub evidence: EvidenceRecord,
    pub assessment: MutationAssessment,
}

impl MutationRecord {
    pub fn is_satisfied(&self) -> bool {
        self.assessment.is_satisfied()
    }

    /// Panics with the recorded gaps when the mutation claim is not satisfied.
    pub fn assert_satisfied(&self) {
        if let MutationAssessment::Unsatisfied { gaps, .. } = &self.assessment {
            panic!("mutation claim not satisfied: {gaps:?}");
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MutationDeclarationError {
    Id(IdError),
    Declaration(DeclarationError),
}

impl std::fmt::Display for MutationDeclarationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(error) => write!(f, "invalid mutation identifier: {error}"),
            Self::Declaration(error) => write!(f, "invalid mutation declaration: {error}"),
        }
    }
}

impl std::error::Error for MutationDeclarationError {}

/// Builds a `CommandSpec` with no working directory or environment overrides.
pub fn command(program: &str, arguments: &[&str]) -> CommandSpec {
    CommandSpec {
        program: program.to_string(),
        arguments: arguments
            .iter()
            .map(|argument| argument.to_string())
            .collect(),
        working_directory: None,
        environment: BTreeMap::new(),
    }
}

/// Declares the mutation, records the campaign's evidence, and assesses it.
///
/// The record's `outcome` is the baseline's execution outcome. Claim satisfaction
/// lives only in `MutationRecord::assessment`.
pub fn record_mutation(
    spec: MutationSpec,
    campaign: MutationCampaign,
    policy: &MutationPolicy,
) -> Result<MutationRecord, MutationDeclarationError> {
    let MutationSpec {
        id,
        subject,
        claim,
        source_selectors,
        operators,
        selected_checks,
        commands,
    } = spec;

    let selected_checks = selected_checks
        .into_iter()
        .map(|check| TestId::new(check).map_err(MutationDeclarationError::Id))
        .collect::<Result<Vec<_>, _>>()?;

    let declaration = TestDeclaration {
        id: TestId::new(id).map_err(MutationDeclarationError::Id)?,
        primary_dimension: Dimension::Mutation,
        secondary_dimensions: Vec::new(),
        subject: SubjectId::new(subject).map_err(MutationDeclarationError::Id)?,
        claim: Claim {
            statement: claim,
            obligations: Vec::new(),
        },
        required_scope: Scope {
            input_domain: source_selectors.join(", "),
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
            source_selectors,
            operators,
            selected_checks,
            commands,
        },
    };
    declaration
        .validate()
        .map_err(MutationDeclarationError::Declaration)?;

    let assessment = assess(&declaration, &campaign.evidence, policy);
    let outcome = campaign.evidence.baseline.clone();
    let evidence = EvidenceRecord {
        schema_version: 1,
        declaration,
        provenance: campaign.provenance,
        execution: campaign.execution,
        outcome,
        artifacts: campaign.artifacts,
        dimension_evidence: DimensionEvidence::Mutation(campaign.evidence),
    };

    Ok(MutationRecord {
        evidence,
        assessment,
    })
}
