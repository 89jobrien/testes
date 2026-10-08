# Testes testing model

Testes defines seven behavior-verification dimensions plus mutation analysis. Dimensions are complementary obligations, not mandatory sequential steps for every function.

| Dimension | Question | Planned macro |
|---|---|---|
| [Unit](dimensions/unit.md) | Does this example behave as expected? | `unit!` |
| [Property](dimensions/property.md) | Does the invariant survive generated inputs? | `property!` |
| [Fuzz](dimensions/fuzz.md) | Does the boundary withstand adversarial input? | `fuzz!` |
| [Model check](dimensions/model-check.md) | Can a modeled execution violate the invariant? | `prove!` |
| [Conformance](dimensions/conformance.md) | Does an implementation satisfy its contract? | `conformance!` |
| [Integration](dimensions/integration.md) | Do connected components preserve behavior across a seam? | `integration!` |
| [Regression](dimensions/regression.md) | Is a previously observed defect still prevented? | `regression!` |
| [Mutation](dimensions/mutation.md) | Do selected checks detect deliberately introduced defects? | `mutate!` |

Read the [execution evidence contract](evidence.md) before interpreting a passing result.

## Implementation status

The repository contains draft core types and trait interfaces. The public macros above, native backends, evidence persistence, runners, and policy evaluators are not implemented. The initial core commit has not been compiled or tested as part of this publication. These documents specify intended behavior, not completed integrations.

## Ownership

- Testes owns generic execution, reporting, and assertion mechanics.
- Trait owners own semantic conformance suites.
- Projects own input strategies, fuzz targets, proof assumptions, fixtures, and regression artifacts.
- Orchestration selects obligations and records executions.

A dimension describes the verification question; a backend describes execution. Regression may wrap another dimension without rerunning it. Mutation assesses only checks actually executed.
