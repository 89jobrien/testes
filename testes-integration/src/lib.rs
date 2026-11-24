//! `testes-integration` owns the `integration!` dimension and the `integration!` macro.
//!
//! Like `unit!` (ADR-0002), `integration!` generates a native Rust `#[test]` with
//! evidence recording; no `TestBackend` is needed for native execution. The generated
//! test declares a `TestSpecification::Integration`, runs the scenario body panic-safely,
//! records `IntegrationEvidence` through `testes-core` types, and fails unless the
//! scenario actually passed. Unavailable and not-executed scenarios therefore show up
//! as failing tests and never as passing ones.
//!
//! Cleanup is not performed by the macro, so the recorded cleanup outcome is
//! `NotExecuted`. Fixtures and observations are not retained yet, so none are recorded.

#![forbid(unsafe_code)]

mod scenario;

pub use scenario::{IntegrationSpec, ScenarioError, assert_scenario_passed, run_scenario};
pub use testes_core::{
    BoundaryKind, ComponentIdentity, EvidenceRecord, ExecutionStatus, IntegrationEvidence,
    OutcomeReason, TestId,
};

/// Declares and executes an integration scenario with evidence recording.
///
/// # Grammar
///
/// ```text
/// integration! {
///     id: "seam-test-id",
///     subject: "checkout",
///     seam: "checkout->payments",
///     claim: "checkout charges through the payments port",
///     components: [("checkout", Real), ("payments", Fake)],
///     scenario: charges_through_payments => {
///         // Interaction and assertions; panics fail the scenario.
///         // `return Err(ScenarioError::Unavailable { .. })` reports an unavailable capability.
///         // `return Err(ScenarioError::NotExecuted { .. })` reports a scenario that did not run.
///     }
/// }
/// ```
///
/// Generates one `#[test] fn charges_through_payments()` that records the evidence and
/// panics unless the scenario passed. `components` entries use `BoundaryKind` variant names
/// (`Real`, `Fake`, `Simulated`); component versions are recorded as absent.
#[macro_export]
macro_rules! integration {
    (
        id: $id:expr,
        subject: $subject:expr,
        seam: $seam:expr,
        claim: $claim:expr,
        components: [ $( ($component:expr, $boundary:ident) ),* $(,)? ],
        scenario: $scenario:ident => $body:block $(,)?
    ) => {
        #[test]
        fn $scenario() {
            let spec = $crate::IntegrationSpec {
                id: ($id).to_string(),
                subject: ($subject).to_string(),
                seam: ($seam).to_string(),
                claim: ($claim).to_string(),
                components: vec![
                    $(
                        $crate::ComponentIdentity {
                            name: ($component).to_string(),
                            version: None,
                            boundary: $crate::BoundaryKind::$boundary,
                        }
                    ),*
                ],
            };
            let record = $crate::run_scenario(
                &spec,
                || -> ::core::result::Result<(), $crate::ScenarioError> {
                    $body;
                    ::core::result::Result::Ok(())
                },
            );
            $crate::assert_scenario_passed(&record);
        }
    };
}
