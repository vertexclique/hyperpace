//! The `.bin` config file format: a full shadow dump plus a trailer identifying what wrote it.
//!
//! `docs/research/mouse-protocol-v2.md` section 11. Only the OLD driver ever exported a file, as
//! `[8192-byte shadow][32-byte "Compx Inc" magic][16-byte device type][16-byte sensor type]`.
//! Hyperpace's own shadow is [`crate::settings::SHADOW_LEN`] bytes (section 2: the NEW driver's
//! larger flash map), so the shadow region here is that size instead of the OLD driver's 8192,
//! and the trailer adds the reporting cid and mid so [`import`] can check device identity
//! directly rather than by an opaque type string. The `"Compx Inc"` magic itself is kept
//! byte-for-byte: it is not a product string Hyperpace ever displays, only a file-format marker
//! compared against bytes a genuine vendor export already carries, kept so a `.bin` written by
//! the vendor's own tool is still recognizable as the same family of file.
//!
//! # The clamp bug, fixed
//!
//! Section 11.2: on import, the vendor clamps the current DPI stage to `count - 1` but writes its
//! check byte from the *stage count*, not the clamped stage: `c[5] = 85 - c[2]`. A device that
//! honors the check byte would reject a legitimately clamped write. [`import`] derives each
//! scalar's check byte from its own clamped value instead.

use crate::encoding::scalar_pair;
use crate::response::DeviceIdentity;
use crate::settings::{SHADOW_LEN, Shadow, offset};

use core::fmt;

use crate::model::ModelTable;

const MAGIC: &[u8] = b"Compx Inc";
const MAGIC_FIELD_LEN: usize = 32;
const SENSOR_FIELD_LEN: usize = 16;
const TRAILER_LEN: usize = 64;
/// Total size of a Hyperpace `.bin` config file.
pub const FILE_LEN: usize = SHADOW_LEN + TRAILER_LEN;

/// Export `shadow` as a `.bin` config file for `device`, labelled with `sensor`.
#[must_use]
pub fn export(shadow: &Shadow, device: &DeviceIdentity, sensor: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(FILE_LEN);
    for addr in 0..u16::try_from(SHADOW_LEN).unwrap_or(u16::MAX) {
        bytes.push(shadow.scalar(addr));
    }
    // Bytes at or beyond u16::MAX cannot be reached by `scalar`; SHADOW_LEN is 16384, well under
    // that ceiling, so the loop above always covers the whole shadow.
    debug_assert_eq!(bytes.len(), SHADOW_LEN);

    write_padded(&mut bytes, MAGIC, MAGIC_FIELD_LEN);
    bytes.push(device.cid);
    bytes.push(device.mid);
    write_padded(&mut bytes, sensor.as_bytes(), SENSOR_FIELD_LEN);
    bytes.resize(FILE_LEN, 0);
    bytes
}

fn write_padded(out: &mut Vec<u8>, value: &[u8], field_len: usize) {
    let take = value.len().min(field_len);
    out.extend_from_slice(&value[..take]);
    out.extend(std::iter::repeat_n(0u8, field_len - take));
}

fn trimmed_string(field: &[u8]) -> String {
    let end = field.iter().position(|&b| b == 0).unwrap_or(field.len());
    String::from_utf8_lossy(&field[..end]).into_owned()
}

/// A config file, parsed and validated against a target [`ModelTable`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedConfig {
    /// The shadow the file's flash dump decodes to, with the DPI stage clamp bug fixed.
    pub shadow: Shadow,
    /// Sensor name recorded in the file's trailer.
    pub sensor: String,
}

/// Why a config file could not be imported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// The file was shorter than [`FILE_LEN`].
    Truncated {
        /// Bytes actually present.
        got: usize,
    },
    /// The trailer's magic bytes were not `"Compx Inc"`.
    BadMagic,
    /// The file's recorded cid/mid does not match the target model table.
    DeviceMismatch {
        /// cid recorded in the file.
        cid: u8,
        /// mid recorded in the file.
        mid: u8,
    },
    /// The file's recorded sensor does not match the target model table's sensor.
    SensorMismatch {
        /// Sensor name recorded in the file.
        file: String,
        /// Sensor name the target model table expects.
        expected: &'static str,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { got } => {
                write!(f, "a config file is {FILE_LEN} bytes, got {got}")
            }
            Self::BadMagic => write!(f, "not a recognized config file"),
            Self::DeviceMismatch { cid, mid } => {
                write!(
                    f,
                    "config file is for a different device (cid {cid}, mid {mid})"
                )
            }
            Self::SensorMismatch { file, expected } => {
                write!(f, "config file sensor '{file}' does not match '{expected}'")
            }
        }
    }
}

impl core::error::Error for ConfigError {}

/// Import a `.bin` config file for `table`.
///
/// # Errors
///
/// Returns [`ConfigError::Truncated`] if `bytes` is shorter than [`FILE_LEN`],
/// [`ConfigError::BadMagic`] if the trailer's magic does not match, and
/// [`ConfigError::DeviceMismatch`] or [`ConfigError::SensorMismatch`] if the file targets a
/// different model.
pub fn import(bytes: &[u8], table: &ModelTable) -> Result<ImportedConfig, ConfigError> {
    if bytes.len() < FILE_LEN {
        return Err(ConfigError::Truncated { got: bytes.len() });
    }
    let magic_field = &bytes[SHADOW_LEN..SHADOW_LEN + MAGIC_FIELD_LEN];
    if !magic_field.starts_with(MAGIC) {
        return Err(ConfigError::BadMagic);
    }
    let cid = bytes[SHADOW_LEN + MAGIC_FIELD_LEN];
    let mid = bytes[SHADOW_LEN + MAGIC_FIELD_LEN + 1];
    if cid != table.cid || mid != table.mid {
        return Err(ConfigError::DeviceMismatch { cid, mid });
    }
    let sensor_start = SHADOW_LEN + MAGIC_FIELD_LEN + 2;
    let sensor = trimmed_string(&bytes[sensor_start..sensor_start + SENSOR_FIELD_LEN]);
    if sensor != table.sensor {
        return Err(ConfigError::SensorMismatch {
            file: sensor.clone(),
            expected: table.sensor,
        });
    }

    let mut shadow = Shadow::new();
    shadow.apply_read(0, &bytes[..SHADOW_LEN]);

    // Fix the vendor's clamp bug (section 11.2): clamp count and current stage independently,
    // each with its own correctly derived check byte, instead of sharing the count's.
    let stage_count = shadow.scalar(offset::DPI_STAGE_COUNT).clamp(1, 8);
    let current_stage = shadow.scalar(offset::CURRENT_DPI).min(stage_count - 1);
    shadow.apply_read(offset::DPI_STAGE_COUNT, &scalar_pair(stage_count));
    shadow.apply_read(offset::CURRENT_DPI, &scalar_pair(current_stage));

    Ok(ImportedConfig { shadow, sensor })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::model::CID_62_MID_1;
    use crate::response::LinkType;
    use proptest::prelude::*;

    fn identity() -> DeviceIdentity {
        DeviceIdentity {
            cid: 62,
            mid: 1,
            link: LinkType::Wireless1k,
        }
    }

    #[test]
    fn export_has_the_documented_total_length() {
        let shadow = Shadow::new();
        let bytes = export(&shadow, &identity(), "3950");
        assert_eq!(bytes.len(), FILE_LEN);
        assert_eq!(&bytes[SHADOW_LEN..SHADOW_LEN + MAGIC.len()], MAGIC);
    }

    #[test]
    fn export_then_import_round_trips_the_shadow() {
        let mut shadow = Shadow::new();
        shadow.apply_read(offset::POLLING, &[1, 0x54]);
        shadow.apply_read(offset::DPI_STAGE_COUNT, &[3, 0x52]);
        shadow.apply_read(offset::CURRENT_DPI, &[1, 0x54]);
        let bytes = export(&shadow, &identity(), "3950");
        let imported = import(&bytes, &CID_62_MID_1).unwrap();
        assert_eq!(imported.shadow.scalar(offset::POLLING), 1);
        assert_eq!(imported.sensor, "3950");
    }

    #[test]
    fn import_refuses_a_truncated_file() {
        assert_eq!(
            import(&[0; 10], &CID_62_MID_1),
            Err(ConfigError::Truncated { got: 10 })
        );
    }

    #[test]
    fn import_refuses_a_bad_magic() {
        let mut bytes = vec![0u8; FILE_LEN];
        bytes[SHADOW_LEN..SHADOW_LEN + 4].copy_from_slice(b"NOPE");
        assert_eq!(import(&bytes, &CID_62_MID_1), Err(ConfigError::BadMagic));
    }

    #[test]
    fn import_refuses_a_file_for_a_different_device() {
        let shadow = Shadow::new();
        let mut wrong = identity();
        wrong.cid = 102;
        let bytes = export(&shadow, &wrong, "3950");
        assert_eq!(
            import(&bytes, &CID_62_MID_1),
            Err(ConfigError::DeviceMismatch { cid: 102, mid: 1 })
        );
    }

    #[test]
    fn import_refuses_a_sensor_mismatch() {
        let shadow = Shadow::new();
        let bytes = export(&shadow, &identity(), "9999");
        assert_eq!(
            import(&bytes, &CID_62_MID_1),
            Err(ConfigError::SensorMismatch {
                file: "9999".to_owned(),
                expected: "3950",
            })
        );
    }

    #[test]
    fn import_fixes_the_vendor_clamp_bug() {
        // A stage count of 9 (over the true 8-stage capacity) and a current stage past count - 1,
        // as a corrupt or hand-edited file might carry.
        let mut shadow = Shadow::new();
        shadow.apply_read(offset::DPI_STAGE_COUNT, &scalar_pair(9));
        shadow.apply_read(offset::CURRENT_DPI, &scalar_pair(9));
        let bytes = export(&shadow, &identity(), "3950");
        let imported = import(&bytes, &CID_62_MID_1).unwrap();

        let clamped_count = imported.shadow.scalar(offset::DPI_STAGE_COUNT);
        let clamped_stage = imported.shadow.scalar(offset::CURRENT_DPI);
        assert_eq!(clamped_count, 8);
        assert_eq!(clamped_stage, 7);
        // Each scalar's own check byte, not the vendor bug's shared one.
        assert_eq!(
            imported.shadow.scalar(offset::CURRENT_DPI + 1),
            0x55u8.wrapping_sub(clamped_stage)
        );
    }

    proptest! {
        #[test]
        fn export_then_import_always_recovers_the_polling_byte(byte: u8) {
            let mut shadow = Shadow::new();
            shadow.apply_read(offset::POLLING, &scalar_pair(byte));
            let bytes = export(&shadow, &identity(), "3950");
            let imported = import(&bytes, &CID_62_MID_1).unwrap();
            prop_assert_eq!(imported.shadow.scalar(offset::POLLING), byte);
        }
    }
}
