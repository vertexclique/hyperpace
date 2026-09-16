//! Firmware archive and flash progress shapes.

use hyperpace_firmware::Progress;
use hyperpace_store::FirmwareRecord;
use serde::{Deserialize, Serialize};

/// One entry in the local firmware archive, mirroring [`FirmwareRecord`] plus its store document
/// id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareRecordDto {
    /// Store document id.
    pub id: String,
    /// Product name as printed in the package header. Carries no vendor or model branding.
    pub product: String,
    /// Firmware version, formatted `"{major}.{minor:02x}"`.
    pub version: String,
    /// Component id this image targets.
    pub cid: u8,
    /// Module id this image targets.
    pub mid: u8,
    /// sha256 of the package bytes, hex encoded.
    pub sha256: String,
    /// Unix seconds when this package was imported into the archive.
    pub imported_at: i64,
}

impl FirmwareRecordDto {
    /// Pair a store id with the record it names.
    #[must_use]
    pub fn from_record(id: String, record: FirmwareRecord) -> Self {
        Self {
            id,
            product: record.product,
            version: record.version,
            cid: record.cid,
            mid: record.mid,
            sha256: record.sha256,
            imported_at: record.imported_at,
        }
    }
}

/// One progress update streamed while `firmware_install` runs, mirroring [`Progress`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareProgressPayload {
    /// Index of the image currently streaming.
    pub image_index: usize,
    /// Total images in the package being flashed.
    pub image_count: usize,
    /// Payload bytes of the current image sent and acknowledged so far.
    pub bytes_sent: u32,
    /// Total payload bytes in the current image.
    pub image_len: u32,
    /// Overall progress across every image in the package, 0 to 100.
    pub percent: u8,
}

impl From<Progress> for FirmwareProgressPayload {
    fn from(progress: Progress) -> Self {
        Self {
            image_index: progress.image_index,
            image_count: progress.image_count,
            bytes_sent: progress.bytes_sent,
            image_len: progress.image_len,
            percent: progress.percent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firmware_record_dto_pairs_the_store_id() {
        let record = FirmwareRecord {
            product: "Hyperpace mouse".to_owned(),
            version: "1.0a".to_owned(),
            cid: 102,
            mid: 1,
            path: "firmware/1.0a.bin".to_owned(),
            sha256: "0".repeat(64),
            imported_at: 1_726_000_000,
        };
        let dto = FirmwareRecordDto::from_record("bae-1".to_owned(), record);
        assert_eq!(dto.id, "bae-1");
        assert_eq!(dto.product, "Hyperpace mouse");
    }

    #[test]
    fn progress_payload_mirrors_every_field() {
        let progress = Progress {
            image_index: 0,
            image_count: 1,
            bytes_sent: 32,
            image_len: 64,
            percent: 50,
        };
        let payload = FirmwareProgressPayload::from(progress);
        assert_eq!(payload.bytes_sent, 32);
        assert_eq!(payload.percent, 50);
    }
}
