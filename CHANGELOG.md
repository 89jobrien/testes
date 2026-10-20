# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-10-20

### Added

- **Complete unit! evidence path** — Comprehensive test cases demonstrating all four ExecutionStatus values (Passed, Failed, Inconclusive, NotExecuted)
  - Regression test proving declaration alone does not produce success
  - All ExecutionStatus states covered with working examples
  - Macro-generated test coverage for evidence recording

### Fixed

- **License field completeness** — Added license declarations to all 9 crates for future publishing support
- **Security: unmaintained dependency** — Replaced `paste 1.0.15` (RUSTSEC-2024-0436) with maintained `pastey 0.2`
  - Drop-in replacement for macro identifier concatenation syntax
  - All eight dimension macros (unit!, property!, fuzz!, prove!, conformance!, integration!, regression!, mutate!) updated

## [0.3.0] - 2026-10-16

### Fixed

- **Test suite completeness** — Removed intentional failing test cases that were blocking test execution
- **Dimension test coverage** — Added missing test files for conformance! and prove! macros
  - testes-conformance/tests/conformance_macro.rs — tests trait contract conformance declarations
  - testes-model-check/tests/model_check_macro.rs — tests property invariant proof declarations
- **Evidence recording consistency** — Fixed placeholder harness_id and contract identity values in model-check and conformance evidence structures

### Added

- All eight test dimension macros now have complete macro-generated test coverage
- Evidence shape validation tests for conformance and model-check dimensions

## [0.2.0] - 2026-10-16

### Added

- **property! macro** — Property-based testing declaration and execution using proptest strategies
  - Real property execution with atomic case counting
  - Honest evidence recording for measured and unavailable fields
  - Shape validation ensuring dimension-evidence pairing
- **unit! macro** — Native unit test evidence recording with assertion introspection
  - Test isolation and independent execution per test case
  - Regression coverage for evidence shape validation
  - Baseline unit coverage for core declaration and validation types
- **Core types and traits** — Multidimensional test framework foundation
  - Eight test dimensions (unit, property, fuzz, model-check, conformance, integration, regression, mutation)
  - Declaration/execution/evidence separation
  - Shape validation and evidence record integrity checks
  - Error types with miette diagnostics
- **Nine-crate workspace structure** — One crate per dimension plus testes-core

### Changed

- Reorganized workspace into focused dimension crates for future backend independence

## [0.1.0] - Initial draft (unpublished)

- Established project structure and core abstractions
