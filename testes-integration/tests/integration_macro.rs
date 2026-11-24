use testes_integration::{ScenarioError, integration};

integration! {
    id: "macro_seam_passes",
    subject: "macro subject",
    seam: "macro-subject->fake-port",
    claim: "a macro-declared scenario passes when its body succeeds",
    components: [("macro-subject", Real), ("fake-port", Fake)],
    scenario: macro_scenario_passes => {
        assert_eq!(2 + 2, 4);
    }
}

integration! {
    id: "macro_seam_propagates_scenario_errors",
    subject: "macro subject",
    seam: "macro-subject->fake-port",
    claim: "a scenario body can use `?` on fallible seam steps",
    components: [("macro-subject", Real)],
    scenario: macro_scenario_uses_question_mark => {
        let step: Result<u8, ScenarioError> = Ok(1);
        let value = step?;
        assert_eq!(value, 1);
    }
}
