// Integration tests for the unit! macro
// Per ADR-0004 Phase 3: examples demonstrating the macro generates honest tests.
//
// The macro generates:
// 1. Successful examples: test passes when assertions succeed
// 2. Failing examples: test fails when assertions fail
// 3. Validation: invalid declarations are caught before examples run
// 4. Evidence recording: each test stores evidence in a thread_local
//
// These tests verify that the generated #[test] functions themselves run correctly.
// See the example_usage module below for concrete examples.

// Example usage (can be run to verify macro generates valid tests)
#[cfg(test)]
mod example_usage {
    use testes_unit::unit;

    unit! {
        id: "basic-assertion",
        subject: "assert",
        claim: "basic assertion works",
        examples: {
            "simple_true" => {
                assert!(true);
            }
        }
    }

    unit! {
        id: "two-examples",
        subject: "unit macro",
        claim: "multiple examples work",
        examples: {
            "first" => {
                assert_eq!(1 + 1, 2);
            },
            "second" => {
                assert_ne!(1, 2);
            }
        }
    }
}
