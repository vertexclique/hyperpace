//! Failure modes for every public operation in this crate.

use std::fmt;

/// Why a store operation failed.
///
/// No `unwrap` or `expect` reaches a caller through this crate: every failure path, from
/// opening the embedded node to a malformed response, ends up as one of these variants
/// instead of a panic.
#[derive(Debug)]
pub enum StoreError {
    /// Neither `HOME` nor `USERPROFILE` was set, so [`crate::Store::default_root`] could not
    /// resolve a root path.
    HomeDirectoryUnknown,
    /// The store's owner thread (see the crate docs) could not be started, or stopped
    /// answering requests. The string is the underlying I/O or channel failure.
    Worker(String),
    /// The embedded node reported an error while opening storage or defining the schema.
    Node(String),
    /// A GraphQL query or mutation the embedded node ran returned one or more errors.
    Query(String),
    /// A record failed to serialize to, or deserialize from, JSON.
    Codec(serde_json::Error),
    /// The embedded node's response did not have the shape this crate expects from it.
    UnexpectedResponse(String),
    /// `update` targeted a document id that no longer names a document.
    NotFound(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HomeDirectoryUnknown => {
                write!(
                    f,
                    "could not determine the home directory (HOME and USERPROFILE are both unset)"
                )
            }
            Self::Worker(reason) => write!(f, "store owner thread error: {reason}"),
            Self::Node(reason) => write!(f, "embedded store error: {reason}"),
            Self::Query(reason) => write!(f, "embedded store rejected the request: {reason}"),
            Self::Codec(error) => write!(f, "record did not encode or decode as JSON: {error}"),
            Self::UnexpectedResponse(reason) => {
                write!(
                    f,
                    "embedded store returned an unexpected response: {reason}"
                )
            }
            Self::NotFound(doc_id) => write!(f, "no document with id \"{doc_id}\""),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Codec(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn every_variant_has_a_non_empty_display() {
        let bad_json = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let variants = [
            StoreError::HomeDirectoryUnknown,
            StoreError::Worker("closed".to_string()),
            StoreError::Node("open failed".to_string()),
            StoreError::Query("field not found".to_string()),
            StoreError::Codec(bad_json),
            StoreError::UnexpectedResponse("missing data".to_string()),
            StoreError::NotFound("bae-1".to_string()),
        ];
        for variant in variants {
            assert!(!variant.to_string().is_empty());
        }
    }

    #[test]
    fn codec_error_exposes_its_source() {
        let bad_json = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let error = StoreError::Codec(bad_json);
        assert!(std::error::Error::source(&error).is_some());
    }
}
