//! The store's owner thread.
//!
//! One dedicated OS thread owns the embedded node and the Tokio runtime it needs, mirroring
//! `hyperpace-device`'s "one owner thread per device" pattern. Every [`crate::Collection`]
//! operation crosses into that thread as a plain data message over a channel and blocks for
//! the reply, so `hyperpace-store`'s public API stays fully synchronous.
//!
//! This is not just a style match: it is load-bearing. `Store::open` is a synchronous
//! function (see the API contract), so something has to drive the embedded node's async API
//! underneath it. Doing that with a `Runtime::block_on` called directly on whatever thread the
//! caller happens to be on would panic the moment that caller is itself already inside a Tokio
//! runtime (for example, a Tauri command handler in `hyperpace-app`, which the contract states
//! runs every command async). Running the node on its own thread, with its own runtime, makes
//! every `Store`/`Collection` call safe to make from any context, async or not.

use std::fmt::Write as _;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

use embedded::{EmbeddedNode, EmbeddedStore, TransportConfig};

use crate::error::StoreError;
use crate::record::{AppSetting, EventRecord, FirmwareRecord, MacroRecord, ProfileSnapshot};

/// GraphQL collection names this crate defines, in the order the schema is declared in. Built
/// from each record type's own [`crate::Record::COLLECTION`] so the SDL and the collection
/// accessors can never drift apart (see `schema_sdl` below).
const COLLECTIONS: [&str; 5] = [
    <MacroRecord as crate::Record>::COLLECTION,
    <ProfileSnapshot as crate::Record>::COLLECTION,
    <AppSetting as crate::Record>::COLLECTION,
    <FirmwareRecord as crate::Record>::COLLECTION,
    <EventRecord as crate::Record>::COLLECTION,
];

/// The SDL defining all five collections.
///
/// Every collection has the same one-field shape: a `data: JSON` document. Storing each
/// record as a single JSON blob, rather than a field per Rust struct field, means this crate
/// never has to keep a hand-written SDL type in sync with a Rust struct, and never has to
/// serialize a nested `Vec`/`Option`/enum field into a bespoke GraphQL input shape by hand.
///
/// `vertexia:` this trades away server-side filtering and indexing on individual record
/// fields: every query in `collection.rs` reads a whole document or the whole collection and
/// filters in Rust. That is the right tradeoff at today's scale (the macro library, profile
/// snapshots, the firmware index and the app settings are all kilobytes; see
/// `docs/plans/hyperpace.md`), and the ceiling is real: if a collection ever needs a
/// server-side filter or an index, the upgrade path is to declare that field as its own typed
/// SDL field alongside `data` and mirror it on write, not to redesign this module.
fn schema_sdl() -> String {
    let mut sdl = String::new();
    for name in COLLECTIONS {
        // `write!` to a `String` never fails; nothing meaningful to do with the result.
        let _ = writeln!(sdl, "type {name} {{ data: JSON }}");
    }
    sdl
}

/// One unit of work sent to the owner thread: run this GraphQL request and send back the raw
/// response.
struct Job {
    request: query::QueryRequest,
    reply: mpsc::Sender<query::QueryResponse>,
}

/// A live handle to the owner thread.
///
/// Held behind an `Arc` shared by `Store` and every [`crate::Collection`] cloned from it. The
/// thread and its embedded node shut down when the last handle drops.
pub(crate) struct StoreHandle {
    jobs: Option<mpsc::Sender<Job>>,
    thread: Option<thread::JoinHandle<()>>,
}

impl StoreHandle {
    /// Start the owner thread, open the embedded node at `root`, and wait for that to finish
    /// before returning: `Store::open` reports a failed open synchronously, not on the first
    /// later operation.
    pub(crate) fn open(root: &Path) -> Result<Self, StoreError> {
        let root = root.to_path_buf();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<(), StoreError>>();
        let (jobs_tx, jobs_rx) = mpsc::channel::<Job>();

        let thread = thread::Builder::new()
            .name("hyperpace-store".to_string())
            .spawn(move || run(&root, &ready_tx, jobs_rx))
            .map_err(|error| StoreError::Worker(error.to_string()))?;

        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Self {
                jobs: Some(jobs_tx),
                thread: Some(thread),
            }),
            Ok(Err(error)) => {
                let _ = thread.join();
                Err(error)
            }
            Err(_) => {
                let _ = thread.join();
                Err(StoreError::Worker(
                    "owner thread exited before it finished opening the store".to_string(),
                ))
            }
        }
    }

    /// Run one GraphQL request on the owner thread and return its response.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Worker`] when the owner thread is not running.
    pub(crate) fn execute(
        &self,
        request: query::QueryRequest,
    ) -> Result<query::QueryResponse, StoreError> {
        let jobs = self
            .jobs
            .as_ref()
            .ok_or_else(|| StoreError::Worker("store owner thread is not running".to_string()))?;
        let (reply_tx, reply_rx) = mpsc::channel();
        jobs.send(Job {
            request,
            reply: reply_tx,
        })
        .map_err(|_| StoreError::Worker("store owner thread is not running".to_string()))?;
        reply_rx.recv().map_err(|_| {
            StoreError::Worker("store owner thread stopped before it replied".to_string())
        })
    }
}

impl Drop for StoreHandle {
    fn drop(&mut self) {
        // Drop the sender first. The owner thread's `for job in jobs` loop below only ends
        // once every `Sender` is gone, and a struct's fields do not drop until after this
        // function returns, so without this explicit drop `join` would hang forever waiting
        // for a loop that is waiting on a channel this same call is still holding open.
        self.jobs.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// The owner thread's body: build the runtime and the embedded node, report whether that
/// worked, then serve requests until every [`StoreHandle`] has dropped.
fn run(root: &Path, ready: &mpsc::Sender<Result<(), StoreError>>, jobs: mpsc::Receiver<Job>) {
    // `new_multi_thread` with a small worker count, not `new_current_thread`: the embedded
    // node's own background tasks (its downsample task, for one) are spawned Tokio tasks that
    // need to make progress independently of whichever request this thread is blocked on, and
    // this is the flavor the embedded node's own test suite runs under.
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            let _ = ready.send(Err(StoreError::Worker(error.to_string())));
            return;
        }
    };

    let node = match runtime.block_on(open_node(root)) {
        Ok(node) => node,
        Err(error) => {
            let _ = ready.send(Err(error));
            return;
        }
    };

    if ready.send(Ok(())).is_err() {
        // Store::open gave up waiting (its own thread spawn or channel send failed after this
        // one started); still shut the node down cleanly before this thread exits.
        runtime.block_on(node.shutdown());
        return;
    }

    for job in jobs {
        let response = runtime.block_on(node.query_runner.execute(job.request));
        let _ = job.reply.send(response);
    }

    runtime.block_on(node.shutdown());
}

/// Open the embedded node at `root` and make sure the schema this crate needs exists.
async fn open_node(root: &Path) -> Result<EmbeddedNode<EmbeddedStore>, StoreError> {
    // `Persistence::Persistent` follows from passing a `data_path` here; `NodeBuilder` has no
    // separate setter for it (see `crates/embedded/src/node.rs::NodeBuilder::build` upstream).
    let node = EmbeddedNode::<EmbeddedStore>::builder()
        .data_path(root)
        .with_transport(TransportConfig::None)
        .build()
        .await
        .map_err(|error| StoreError::Node(error.to_string()))?;

    // Idempotent across restarts: a persistent root from an earlier run already has this
    // schema, and `add_schema` errors on a collection that already exists. Checking the first
    // collection is enough because this crate is the only writer of this schema and always
    // defines all five together.
    let schema_present = node
        .database
        .get_collection(COLLECTIONS[0])
        .map_err(|error| StoreError::Node(error.to_string()))?
        .is_some();
    if !schema_present {
        node.add_schema(&schema_sdl())
            .await
            .map_err(|error| StoreError::Node(error.to_string()))?;
    }

    Ok(node)
}
