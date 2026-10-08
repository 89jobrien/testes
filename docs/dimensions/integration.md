# Integration

## Definition

Verify observable behavior across a named seam between connected components. Test wiring and boundary behavior rather than duplicating unit assertions.

## Declaration

`TestSpecification::Integration` identifies seam and components. Mark each boundary as real, fake, or simulated. Planned macro: `testes::integration!` (not implemented). Intended execution uses Rust tests and explicit process/I/O adapters.

## Execution evidence

`IntegrationEvidence` records seam, component identities, fixtures, observations, and cleanup outcome. The envelope records configuration, execution status, and sanitized diagnostics. Retain interaction inputs where replay requires them.

## Acceptance

The required interaction and observable assertions succeed in the declared environment. Failed setup, missing capabilities, and execution timeout must not masquerade as seam verification. Required cleanup failures remain visible.

## Example obligation

A CLI request reaches the application operation and yields the expected structured response.

## Limits

Do not claim a real transport was verified when substituted. Prefer child-process environment configuration to routine mutation of the test process environment.
