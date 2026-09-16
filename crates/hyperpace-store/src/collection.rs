//! Typed CRUD over one embedded collection.

use std::marker::PhantomData;
use std::sync::Arc;

use serde_json::Value;

use crate::error::StoreError;
use crate::record::Record;
use crate::worker::StoreHandle;

/// A handle to one collection, typed by the record it stores.
///
/// Cheap to clone: every clone shares the same owner thread (see the crate docs) through an
/// `Arc`, the same way `hyperpace-device`'s `DeviceHandle` shares its owner thread.
pub struct Collection<T: Record> {
    pub(crate) handle: Arc<StoreHandle>,
    pub(crate) _marker: PhantomData<fn() -> T>,
}

impl<T: Record> Clone for Collection<T> {
    fn clone(&self) -> Self {
        Self {
            handle: Arc::clone(&self.handle),
            _marker: PhantomData,
        }
    }
}

impl<T: Record> Collection<T> {
    /// Create a new document and return its id.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Codec`] when `record` fails to serialize, and
    /// [`StoreError::Query`] or [`StoreError::Worker`] when the embedded node rejects the
    /// request or cannot be reached.
    pub fn create(&self, record: &T) -> Result<String, StoreError> {
        let data = serde_json::to_value(record).map_err(StoreError::Codec)?;
        let query = format!(
            "mutation($input: {coll}MutationInputArg!) {{ add_{coll}(input: $input) {{ _docID }} }}",
            coll = T::COLLECTION
        );
        let request = query::QueryRequest::new(query)
            .with_variables(serde_json::json!({ "input": { "data": data } }));
        let response = self.handle.execute(request)?;
        let items = response_field(&response, &format!("add_{}", T::COLLECTION))?;
        items
            .as_array()
            .and_then(|array| array.first())
            .and_then(|item| item.get("_docID"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| {
                StoreError::UnexpectedResponse(format!(
                    "add_{} did not return a _docID",
                    T::COLLECTION
                ))
            })
    }

    /// Read one document by id.
    ///
    /// Returns `Ok(None)` when no document has that id, rather than an error: a missing
    /// document is an expected outcome for a lookup, not a failure.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Codec`] when the stored document fails to deserialize into `T`,
    /// and [`StoreError::Query`] or [`StoreError::Worker`] when the embedded node rejects the
    /// request or cannot be reached.
    pub fn get(&self, doc_id: &str) -> Result<Option<T>, StoreError> {
        let query = format!(
            "query($id: [ID!]) {{ {coll}(docID: $id) {{ _docID data }} }}",
            coll = T::COLLECTION
        );
        let request =
            query::QueryRequest::new(query).with_variables(serde_json::json!({ "id": [doc_id] }));
        let response = self.handle.execute(request)?;
        let items = response_field(&response, T::COLLECTION)?;
        let Some(item) = items.as_array().and_then(|array| array.first()) else {
            return Ok(None);
        };
        decode_document(item).map(Some)
    }

    /// Read every document in this collection.
    ///
    /// The macro library, profile snapshots, app settings and the firmware index are all
    /// kilobyte scale (see `docs/plans/hyperpace.md`), so a full scan is the right shape here;
    /// an unbounded event log is the one collection where a caller should periodically prune
    /// old entries with [`Collection::delete`] rather than let this grow without limit.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Codec`] when a stored document fails to deserialize into `T`, and
    /// [`StoreError::Query`] or [`StoreError::Worker`] when the embedded node rejects the
    /// request or cannot be reached.
    pub fn list(&self) -> Result<Vec<(String, T)>, StoreError> {
        let query = format!("query {{ {coll} {{ _docID data }} }}", coll = T::COLLECTION);
        let response = self.handle.execute(query::QueryRequest::new(query))?;
        let items = response_field(&response, T::COLLECTION)?;
        let array = items.as_array().ok_or_else(|| {
            StoreError::UnexpectedResponse(format!("{} did not return a list", T::COLLECTION))
        })?;
        array
            .iter()
            .map(|item| {
                let doc_id = item
                    .get("_docID")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .ok_or_else(|| {
                        StoreError::UnexpectedResponse(format!(
                            "a {} document had no _docID",
                            T::COLLECTION
                        ))
                    })?;
                decode_document(item).map(|record| (doc_id, record))
            })
            .collect()
    }

    /// Replace an existing document's content.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Codec`] when `record` fails to serialize,
    /// [`StoreError::NotFound`] when `doc_id` names no document, and [`StoreError::Query`] or
    /// [`StoreError::Worker`] when the embedded node rejects the request or cannot be reached.
    pub fn update(&self, doc_id: &str, record: &T) -> Result<(), StoreError> {
        let data = serde_json::to_value(record).map_err(StoreError::Codec)?;
        let query = format!(
            "mutation($id: ID, $data: JSON) {{ update_{coll}(docID: $id, input: {{ data: $data }}) {{ _docID }} }}",
            coll = T::COLLECTION
        );
        let request = query::QueryRequest::new(query)
            .with_variables(serde_json::json!({ "id": doc_id, "data": data }));
        let response = self.handle.execute(request)?;
        let items = response_field(&response, &format!("update_{}", T::COLLECTION))?;
        if items.as_array().is_some_and(|array| !array.is_empty()) {
            Ok(())
        } else {
            Err(StoreError::NotFound(doc_id.to_string()))
        }
    }

    /// Delete a document by id.
    ///
    /// Returns `true` when a document was deleted and `false` when `doc_id` already named
    /// none: deleting something that may already be gone is a legitimate, idempotent call, not
    /// a caller error.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Query`] or [`StoreError::Worker`] when the embedded node rejects
    /// the request or cannot be reached.
    pub fn delete(&self, doc_id: &str) -> Result<bool, StoreError> {
        let query = format!(
            "mutation($id: ID) {{ delete_{coll}(docID: $id) {{ _docID }} }}",
            coll = T::COLLECTION
        );
        let request =
            query::QueryRequest::new(query).with_variables(serde_json::json!({ "id": doc_id }));
        let response = self.handle.execute(request)?;
        let items = response_field(&response, &format!("delete_{}", T::COLLECTION))?;
        Ok(items.as_array().is_some_and(|array| !array.is_empty()))
    }
}

/// Decode one document's `data` field into `T`.
fn decode_document<T: Record>(item: &Value) -> Result<T, StoreError> {
    let data = item.get("data").cloned().unwrap_or_default();
    serde_json::from_value(data).map_err(StoreError::Codec)
}

/// Pull `field` out of a query response, turning a GraphQL-level error into [`StoreError::Query`]
/// and a missing field into [`StoreError::UnexpectedResponse`].
fn response_field<'a>(
    response: &'a query::QueryResponse,
    field: &str,
) -> Result<&'a Value, StoreError> {
    if response.has_errors() {
        let messages = response
            .errors
            .iter()
            .map(|error| error.message.clone())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(StoreError::Query(messages));
    }
    response
        .data
        .as_ref()
        .and_then(|data| data.get(field))
        .ok_or_else(|| StoreError::UnexpectedResponse(format!("response had no \"{field}\" field")))
}

#[cfg(test)]
mod tests {
    // Tests may assert: the doctrine bans panics on production paths, not in tests.
    #![allow(clippy::unwrap_used)]

    use crate::Store;
    use crate::record::{
        AppSetting, EventRecord, FirmwareRecord, MacroEventRecord, MacroRecord, ProfileSnapshot,
    };

    fn open_temp_store() -> (Store, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        (store, dir)
    }

    #[test]
    fn macro_records_round_trip_through_create_get_update_delete() {
        let (store, _dir) = open_temp_store();
        let macros = store.macros();

        let record = MacroRecord {
            name: "burst fire".to_string(),
            slot: Some(3),
            events: vec![MacroEventRecord {
                press: true,
                kind: "Key".to_string(),
                value: 4,
                delay_ms: 12,
            }],
        };
        let doc_id = macros.create(&record).unwrap();

        let fetched = macros.get(&doc_id).unwrap();
        assert_eq!(fetched, Some(record.clone()));

        let listed = macros.list().unwrap();
        assert_eq!(listed, vec![(doc_id.clone(), record)]);

        let updated = MacroRecord {
            name: "renamed".to_string(),
            slot: None,
            events: vec![],
        };
        macros.update(&doc_id, &updated).unwrap();
        assert_eq!(macros.get(&doc_id).unwrap(), Some(updated));

        assert!(macros.delete(&doc_id).unwrap());
        assert_eq!(macros.get(&doc_id).unwrap(), None);
        assert!(macros.list().unwrap().is_empty());
    }

    #[test]
    fn deleting_a_missing_document_returns_false_not_an_error() {
        let (store, _dir) = open_temp_store();
        assert!(!store.macros().delete("bae-does-not-exist").unwrap());
    }

    #[test]
    fn updating_a_missing_document_is_a_not_found_error() {
        let (store, _dir) = open_temp_store();
        let record = MacroRecord {
            name: "ghost".to_string(),
            slot: None,
            events: vec![],
        };
        let error = store
            .macros()
            .update("bae-does-not-exist", &record)
            .unwrap_err();
        assert!(matches!(error, crate::StoreError::NotFound(_)));
    }

    #[test]
    fn getting_a_missing_document_returns_none_not_an_error() {
        let (store, _dir) = open_temp_store();
        assert_eq!(store.macros().get("bae-does-not-exist").unwrap(), None);
    }

    #[test]
    fn every_collection_is_independently_addressable() {
        let (store, _dir) = open_temp_store();

        let profile_id = store
            .profiles()
            .create(&ProfileSnapshot {
                name: "fps".to_string(),
                index: 1,
                captured_at: 1_726_000_000,
                settings: serde_json::json!({"polling_hz": 1000}),
            })
            .unwrap();
        let setting_id = store
            .settings()
            .create(&AppSetting {
                key: "autostart".to_string(),
                value: serde_json::json!(true),
            })
            .unwrap();
        let firmware_id = store
            .firmware()
            .create(&FirmwareRecord {
                product: "Hyperpace mouse".to_string(),
                version: "1.0a".to_string(),
                cid: 102,
                mid: 1,
                path: "firmware/1.0a.bin".to_string(),
                sha256: "0".repeat(64),
                imported_at: 1_726_000_000,
            })
            .unwrap();
        let event_id = store
            .events()
            .create(&EventRecord {
                at: 1_726_000_000,
                kind: "connected".to_string(),
                message: "device connected".to_string(),
                detail: None,
            })
            .unwrap();

        // Each collection is a distinct document namespace: the same store instance holds all
        // four ids without collision, and each is only visible through its own accessor.
        assert!(store.profiles().get(&profile_id).unwrap().is_some());
        assert!(store.settings().get(&setting_id).unwrap().is_some());
        assert!(store.firmware().get(&firmware_id).unwrap().is_some());
        assert!(store.events().get(&event_id).unwrap().is_some());
        assert!(store.profiles().get(&setting_id).unwrap().is_none());
    }

    #[test]
    fn a_collection_handle_is_cheap_to_clone_and_shares_the_owner_thread() {
        let (store, _dir) = open_temp_store();
        let macros = store.macros();
        let cloned = macros.clone();

        let doc_id = macros
            .create(&MacroRecord {
                name: "shared".to_string(),
                slot: None,
                events: vec![],
            })
            .unwrap();
        assert!(cloned.get(&doc_id).unwrap().is_some());
    }
}
