//! Local store for Hyperpace: the macro library, profile snapshots, app settings, the firmware
//! archive index and the event log.
//!
//! This crate knows nothing about HID or the device wire protocol (see the crate boundary
//! table in `docs/architecture/api-contract.md`): it persists whatever serde-shaped records a
//! caller gives it, under five fixed collections. [`Store`] is the entry point; its own
//! documentation covers how it is built on an embedded `defradb.rs` node and why, and the
//! internal `worker` module documents the owner-thread design that keeps this crate's API
//! synchronous and safe to call from any context.
//!
//! # Layers
//!
//! - `record` (private, re-exported): the five record types this crate stores, and the
//!   [`Record`] trait tying each to its collection.
//! - `worker` (private): the embedded node's owner thread.
//! - `collection` (private, re-exported as [`Collection`]): typed CRUD over one collection.
//! - `store` (private, re-exported as [`Store`]): [`Store`] itself.
//! - `error` (private, re-exported as [`StoreError`]): [`StoreError`].

mod collection;
mod error;
mod record;
mod store;
mod worker;

pub use collection::Collection;
pub use error::StoreError;
pub use record::{
    AppSetting, EventRecord, FirmwareRecord, MacroEventRecord, MacroRecord, ProfileSnapshot, Record,
};
pub use store::Store;
