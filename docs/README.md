# Testes testing model

Testes defines seven behavior-verification dimensions plus mutation analysis. Dimensions are complementary obligations, not mandatory sequential steps for every function.

| Dimension                                | Question                                                   | Macro          |
| ---------------------------------------- | ---------------------------------------------------------- | -------------- |
| [Unit](dimensions/unit.md)               | Does this example behave as expected?                      | `unit!`        |
| [Property](dimensions/property.md)       | Does the invariant survive generated inputs?               | `property!`    |
| [Fuzz](dimensions/fuzz.md)               | Does the boundary withstand adversarial input?             | `fuzz!`        |
| [Model check](dimensions/model-check.md) | Can a modeled execution violate the invariant?             | `prove!`       |
| [Conformance](dimensions/conformance.md) | Does an implementation satisfy its contract?               | `conformance!` |
| [Integration](dimensions/integration.md) | Do connected components preserve behavior across a seam?   | `integration!` |
| [Regression](dimensions/regression.md)   | Is a previously observed defect still prevented?           | `regression!`  |
| [Mutation](dimensions/mutation.md)       | Do selected checks detect deliberately introduced defects? | `mutate!`      |

Read the [execution evidence contract](evidence.md) before interpreting a passing result.

## Implementation status

Seven of the eight macros are implemented: `unit!`, `property!`, `fuzz!`, `prove!`, `integration!`, `regression!`, and `mutate!`. `conformance!` is not implemented. Backend and runner status varies by dimension: `fuzz!` is an in-process seed-corpus runner, `prove!` takes a `ModelCheckBackend` port with no bundled verifier, and `mutate!` runs no mutation campaign. Evidence persistence and policy evaluators are not implemented. For the authoritative per-crate status, see [current state](agent-context/AGENTS-current-state.md). These documents specify intended behavior and the current status of each dimension.

## Ownership

- Testes owns generic execution, reporting, and assertion mechanics.
- Trait owners own semantic conformance suites.
- Projects own input strategies, fuzz targets, proof assumptions, fixtures, and regression artifacts.
- Orchestration selects obligations and records executions.

A dimension describes the verification question; a backend describes execution. Regression may wrap another dimension without rerunning it. Mutation assesses only checks actually executed.
