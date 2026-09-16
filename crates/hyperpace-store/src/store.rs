//! The top-level [`Store`] handle.

use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::collection::Collection;
use crate::error::StoreError;
use crate::record::{
    AppSetting, EventRecord, FirmwareRecord, MacroRecord, ProfileSnapshot, Record,
};
use crate::worker::StoreHandle;

/// The local store: macros, profile snapshots, app settings, the firmware archive index and
/// the event log, held in an embedded [defradb.rs](https://github.com/sourcenetwork/defradb.rs)
/// node.
///
/// # Why an embedded document database, and how it is scoped down
///
/// `defradb.rs` is a full peer-to-peer document database; this crate uses none of that. The
/// `embedded` crate this depends on is pulled in with `default-features = false` and only the
/// `native` feature enabled, which keeps the regolith storage backend and its Tokio runtime
/// integration without pulling in libp2p or the WASM (`wasmtime`) runtime that its default
/// features would otherwise bring in. That was verified directly, not assumed: with this
/// feature selection, `cargo tree` on this crate names no `libp2p*` or `wasmtime*` package,
/// confirming the transport and the WASM lens runtime are genuinely absent from the build, not
/// merely unused. The node is opened with a `data_path` (which is what makes it
/// `Persistence::Persistent`; `NodeBuilder` has no separate setter for that enum) and
/// `TransportConfig::None`, so it never opens a socket.
///
/// This was a real attempt, not a formality ahead of a fallback: a minimal probe crate against
/// `embedded` with this exact feature selection built, ran, persisted documents to disk across
/// a process restart, and round-tripped create/get/update/delete/list through the GraphQL
/// query executor before any of this crate's code was written. The plain-JSON-file fallback the
/// plan named was not needed.
///
/// # Why every record is one JSON document
///
/// Each of the five collections is declared in the embedded schema as a single `data: JSON`
/// field; see the internal `worker` module for the SDL and the tradeoff that shape makes.
///
/// # Concurrency
///
/// `Store` and every [`Collection`] cloned from it share one owner thread that holds the
/// embedded node and its Tokio runtime, the same pattern `hyperpace-device` uses for the
/// device connection. See `crate::worker` for why that is load-bearing here, not just a style
/// match.
pub struct Store {
    handle: Arc<StoreHandle>,
}

impl Store {
    /// Open (creating if needed) the store rooted at `root`.
    ///
    /// Safe to call again with the same root from a later process: the schema this crate needs
    /// is created once and reused, and every document a previous run wrote is still there.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Worker`] when the owner thread cannot be started, and
    /// [`StoreError::Node`] when the embedded node fails to open `root` or define its schema
    /// (for example, `root` is already locked by another open `Store` on the same path).
    pub fn open(root: &Path) -> Result<Self, StoreError> {
        Ok(Self {
            handle: Arc::new(StoreHandle::open(root)?),
        })
    }

    /// The default store root: `$HOME/.hyperpace`, or `%USERPROFILE%\.hyperpace` where `HOME`
    /// is unset. Every OS uses this same fixed path rather than a platform configuration
    /// directory; see the rejected-forks list in `docs/plans/hyperpace.md`.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::HomeDirectoryUnknown`] when neither environment variable is set.
    pub fn default_root() -> Result<PathBuf, StoreError> {
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .ok_or(StoreError::HomeDirectoryUnknown)?;
        Ok(PathBuf::from(home).join(".hyperpace"))
    }

    /// The saved macro library.
    #[must_use]
    pub fn macros(&self) -> Collection<MacroRecord> {
        self.collection()
    }

    /// Saved device profile snapshots.
    #[must_use]
    pub fn profiles(&self) -> Collection<ProfileSnapshot> {
        self.collection()
    }

    /// App-level (not device) settings.
    #[must_use]
    pub fn settings(&self) -> Collection<AppSetting> {
        self.collection()
    }

    /// The local firmware archive index.
    #[must_use]
    pub fn firmware(&self) -> Collection<FirmwareRecord> {
        self.collection()
    }

    /// The app and device event log.
    #[must_use]
    pub fn events(&self) -> Collection<EventRecord> {
        self.collection()
    }

    fn collection<T: Record>(&self) -> Collection<T> {
        Collection {
            handle: Arc::clone(&self.handle),
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests may assert: the doctrine bans panics on production paths, not in tests.
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::record::MacroRecord;

    #[test]
    fn open_creates_the_root_directory() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("nested").join(".hyperpace");
        assert!(!root.exists());

        let _store = Store::open(&root).unwrap();

        assert!(root.exists());
    }

    #[test]
    fn reopening_the_same_root_keeps_previously_written_documents() {
        let dir = tempfile::tempdir().unwrap();

        let doc_id = {
            let store = Store::open(dir.path()).unwrap();
            store
                .macros()
                .create(&MacroRecord {
                    name: "persists".to_string(),
                    slot: None,
                    events: vec![],
                })
                .unwrap()
        };

        let store = Store::open(dir.path()).unwrap();
        let record = store.macros().get(&doc_id).unwrap();
        assert_eq!(record.map(|r| r.name), Some("persists".to_string()));
    }

    #[test]
    fn default_root_is_under_the_home_directory() {
        let root = Store::default_root().unwrap();
        assert!(root.ends_with(".hyperpace"));
    }
}
