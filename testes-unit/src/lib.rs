//! `testes-unit` owns the `unit!` dimension.
//!
//! Per ADR-0002, `unit!` is the only dimension with no external engine and no
//! `TestBackend` implementation needed — only its PORT BOUNDARY is designed so
//! far. No macro or backend is implemented yet; that is separate, future,
//! out-of-scope work (see ADR-0002 and ADR-0003).
