# testes-conformance

The `conformance!` dimension. It is intended as a generic runner for contract suites, where the trait owner keeps the case semantics.

Status: not implemented. This crate has no macro and no backend yet. Its only public surface is the crate documentation.

## Planned shape

- The generic case lifecycle and reporting live here.
- Concrete conformance cases stay with the project that owns the trait. Testes does not define what a contract means.
- The interfaces already in `testes-core` are `ConformanceCase`, `ConformanceSuite`, `SubjectFactory`, `SubjectCleanup`, and `CaseContext`.

The owner decision on the async adapter strategy is still open, so the macro's shape is not settled. See [ADR-0003](../docs/decisions/0003-eight-way-dimension-crate-split.md) for the crate split.

Until the macro lands, `conformance!` must not be reported as passing anything.

## Dependencies

- `testes-core`
