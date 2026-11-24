//! `testes-mutation` owns the `mutate!` dimension.
//!
//! Per ADR-0006, `mutate!` declares a `TestSpecification::Mutation`, records a
//! `DimensionEvidence::Mutation` result from a campaign that was supplied by a
//! runner, and assesses that result against an explicit `MutationPolicy`.
//! The macro never runs a mutation campaign. cargo-mutants integration is not
//! implemented here and must stay outside `testes-core`.

#![forbid(unsafe_code)]

mod assessment;
mod record;

pub use assessment::{MutationAssessment, MutationCounts, assess};
pub use record::{
    MutationCampaign, MutationDeclarationError, MutationRecord, MutationSpec, command,
    record_mutation,
};
pub use testes_core::{
    ArtifactReference, BackendConfiguration, BackendIdentity, ClaimVerdict, CommandSpec,
    DimensionEvidence, EvidenceGap, EvidenceRecord, ExecutionMetadata, ExecutionOutcome,
    ExecutionStatus, IdError, MutationDefinition, MutationEvidence, MutationOutcome,
    MutationPolicy, MutationResult, Observed, Provenance, RepositoryRevision, ResourceLimits,
    RunId, Scope, SubjectId, TestId, WorkingTreeState,
};

/// Declares a mutation campaign and records its evidence.
///
/// # Grammar
///
/// ```text
/// mutate! {
///     id: "mutation-id",
///     subject: "module::item",
///     claim: "claim statement",
///     sources: ["src/lib.rs"],
///     operators: ["delete-statement"],
///     checks: ["check-id"],
///     commands: [("cargo", ["nextest", "run"])],
///     policy: MutationPolicy {
///         minimum_assessed: 1,
///         maximum_missed: 0,
///         allow_timeouts: false,
///     },
///     campaign: campaign_value,
/// }
/// ```
///
/// Evaluates to a [`MutationRecord`]. The record is `#[must_use]`; call
/// [`MutationRecord::assert_satisfied`] or inspect `assessment` before reporting
/// the claim as verified. Panics with a contextual message if the declaration
/// (id, subject, claim, or check identifiers) is invalid.
#[macro_export]
macro_rules! mutate {
    (
        id: $id:expr,
        subject: $subject:expr,
        claim: $claim:expr,
        sources: [$($source:expr),* $(,)?],
        operators: [$($operator:expr),* $(,)?],
        checks: [$($check:expr),* $(,)?],
        commands: [$(($program:expr, [$($argument:expr),* $(,)?])),* $(,)?],
        policy: $policy:expr,
        campaign: $campaign:expr
        $(,)?
    ) => {
        $crate::record_mutation(
            $crate::MutationSpec {
                id: ::std::string::ToString::to_string(&$id),
                subject: ::std::string::ToString::to_string(&$subject),
                claim: ::std::string::ToString::to_string(&$claim),
                source_selectors: vec![$(::std::string::ToString::to_string(&$source)),*],
                operators: vec![$(::std::string::ToString::to_string(&$operator)),*],
                selected_checks: vec![$(::std::string::ToString::to_string(&$check)),*],
                commands: vec![$($crate::command($program, &[$($argument),*])),*],
            },
            $campaign,
            &$policy,
        )
        .expect("mutate! declaration must be valid")
    };
}
