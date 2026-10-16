//! Declarations, execution evidence, and policy interfaces for testes.
//! Draft API: native test backends and public test macros are not implemented.
#![forbid(unsafe_code)]
pub mod traits;
pub mod types;
pub use traits::*;
pub use types::*;
