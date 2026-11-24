# Dimension variants, evidence shape, and IDs

Read this before adding or changing a `Dimension`, `TestSpecification`, or `DimensionEvidence` variant, changing evidence shape validation, or adding a string ID type.

Each `Dimension` variant has a matching `TestSpecification` variant (the declared
intent) and `DimensionEvidence` variant (the recorded result) — e.g.
`TestSpecification::Fuzz` pairs with `DimensionEvidence::Fuzz(FuzzEvidence)`.
`EvidenceRecord::validate_shape()` checks these line up, including that
`Regression`'s wrapped `underlying_dimension` matches the nested
`DimensionEvidence::Regression.execution`'s own dimension. Shape validation is
intentionally partial — it does not establish scope coverage, artifact integrity, or
policy compliance; see "Known review questions" in `docs/handoff.md` before assuming
more than that.

`string_id!` (testes-core/src/types.rs) generates the newtype ID pattern shared by `TestId`,
`RunId`, `SubjectId`, `ObligationId`, `ContractId`, `ImplementationId`, `ArtifactId`,
`FailureId`, `MutationId` — all reject blank/control-character values through the same
`IdError`.
