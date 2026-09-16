//! Every Tauri command in `docs/architecture/api-contract.md`, grouped by what each talks to.
//!
//! Every command is thin: it converts a request DTO (see [`crate::dto`]), calls one
//! [`crate::state::AppState`] or [`hyperpace_store::Store`] method, and converts the result back
//! to `Result<T, String>`, holding no protocol knowledge of its own
//! (`docs/architecture/api-contract.md`'s crate boundary table). See `src/command_list.rs` for the
//! full command name list this module's functions must stay in sync with.

pub mod device;
pub mod firmware;
pub mod firmware_watch;
pub mod macros;
pub mod settings;
