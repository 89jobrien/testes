# Model check

## Definition

Verify assertions over executions represented by a symbolic harness. Claims are scoped by modeled inputs, assumptions, bounds, and supported operations.

## Declaration

`TestSpecification::ModelCheck` identifies harness and proof obligations. Scope records justified assumptions and bounds. Planned macro: `testes::prove!` (not implemented). Intended backend: Kani. Keep symbolic inputs, assumptions, and unwind annotations visible in project-owned harnesses.

## Execution evidence

`ModelCheckEvidence` records symbolic inputs, assumptions, bounds, per-obligation statuses, available non-vacuity checks, and report/trace artifacts. Record verifier version/configuration.

## Acceptance

All required assertions are established for the declared scope. Refuted obligations fail. Insufficient bounds, unsupported verification, or resource exhaustion are inconclusive. Unexpectedly empty reachable domains require investigation.

## Example obligation

External-input budget arithmetic cannot wrap and bypass the limit within the modeled domain.

## Limits

Do not claim unrestricted correctness from a bounded result. Designate proof-critical obligations explicitly; do not silently add assumptions or defer required proofs until a bug appears.
