# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **mutate! macro** — Mutation declaration, evidence recording, and claim assessment in `testes-mutation` (ADR-0006)
  - Records a `DimensionEvidence::Mutation` result from a campaign supplied by a runner; no campaign runs during expansion
  - Assessment requires a passing baseline, selected-check attribution for caught mutants, a timeout/exclusion policy, and a non-zero assessed count
  - Zero assessed experiments is never reported as full detection

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
