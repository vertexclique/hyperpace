//! The 720 byte `ComUsbUpgradeFile` container header, one per image, and its parser.
//!
//! `docs/research/firmware-update-spec.md` section 1. Little-endian header at offset 0 of an
//! image: four u32 fields, a version u32, three identity bytes, then eleven NUL-padded 64 byte
//! ASCII fields. The eleven fields span `23 + 64*11 = 727` bytes, seven more than the declared
//! `headLength` of 720, so the field region is validated to 727 bytes even though the checksum
//! only ever covers the first 720 (section 1's correction note); the header struct is sized to
//! 727 accordingly, never to 720.

use core::fmt;

use hyperpace_protocol::{REPORT_ID, Version};

use crate::error::FirmwareError;

/// Byte length of the checksummed header region, and the only value `headLength` may hold.
pub(crate) const HEADER_LEN: u32 = 720;
/// Byte length of the eleven ASCII command and identity fields, which run seven bytes past
/// `HEADER_LEN` (section 1's correction note).
pub(crate) const FIELD_REGION_LEN: usize = 23 + 64 * 11;
/// Byte width of each of the eleven ASCII fields.
pub(crate) const FIELD_LEN: usize = 64;
/// Byte offset of the first ASCII field, relative to an image's own start.
pub(crate) const FIELD_START: usize = 23;
/// Fixed image offset an image's payload starts at, relative to that image's own start.
pub(crate) const PAYLOAD_OFFSET: usize = 8192;

/// What kind of component an image targets, from its `DeviceType` byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// Byte 210: the mouse.
    Mouse,
    /// Byte 211: the receiver.
    Dongle,
    /// Any other byte. 209 has been observed on a Lofree keyboard image; its meaning is inferred,
    /// not confirmed (section 1).
    Other(u8),
}

impl DeviceType {
    fn from_byte(byte: u8) -> Self {
        match byte {
            210 => Self::Mouse,
            211 => Self::Dongle,
            other => Self::Other(other),
        }
    }
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mouse => write!(f, "mouse"),
            Self::Dongle => write!(f, "receiver"),
            Self::Other(byte) => write!(f, "unknown device type {byte}"),
        }
    }
}

/// A USB vendor and product id, parsed from a `vid_XXXX&pid_XXXX...` endpoint field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbEndpoint {
    /// USB vendor id.
    pub vendor_id: u16,
    /// USB product id.
    pub product_id: u16,
}

/// One parsed image header, plus the on-wire command bytes and endpoint identity it carries.
///
/// `reset_cmd`, `prepare_cmd` and `data_cmd` are the report bytes verbatim: the report id
/// followed by the command's declared body, copied unchanged from the header (section 1's
/// command field sub-encoding; section 7's Q1, "nothing is recomputed"). `data_cmd` is a
/// template: the flash state machine rewrites its chunk marker, length, address and payload per
/// packet, but the report id and starting address come from these bytes unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageHeader {
    /// Stored header checksum.
    pub head_crc: u32,
    /// Declared header length; always `HEADER_LEN` (720) once parsed successfully.
    pub head_len: u32,
    /// Payload length in bytes.
    pub fw_len: u32,
    /// Absolute byte offset of a chained second image, or 0 for none.
    pub next: u32,
    /// Firmware version this image carries.
    pub version: Version,
    /// What kind of component this image targets.
    pub device_type: DeviceType,
    /// Component id the image declares; 0 when unspecified (the common case in the archive).
    pub cid: u8,
    /// Module id the image declares; 0 when unspecified.
    pub mid: u8,
    /// Controller name string, e.g. `"NRF52833"`.
    pub ic_name: String,
    /// The device's identity while sitting in the bootloader (`bootOutputEndPoint`, section 3's
    /// T1).
    pub boot_endpoint: UsbEndpoint,
    /// The device's identity in normal operation (`normalOutputEndPoint`, section 3's T1).
    pub normal_endpoint: UsbEndpoint,
    /// `resetToUpdateModeCmd` report bytes: report id plus a 16 byte
    /// [`hyperpace_protocol::Frame`] encoding `EnterUsbUpdateMode`.
    pub reset_cmd: Vec<u8>,
    /// `prepareDownLoadCmd` report bytes, sent verbatim to start the transfer.
    pub prepare_cmd: Vec<u8>,
    /// `dataDownLoadCmd` report bytes: a template for every data packet.
    pub data_cmd: Vec<u8>,
    /// Product name string from the header.
    pub product: String,

    /// Absolute byte offset of this image's payload within the package's bytes. Not part of the
    /// header on disk (the header only carries lengths), kept for [`crate::package::Package`] to
    /// slice the payload without re-deriving it; not reachable outside this crate.
    pub(crate) payload_start: usize,
}

/// Parse one image header starting at `image_start` within `bytes`.
///
/// # Errors
///
/// See [`FirmwareError`]'s variants for checks V1 through V7 of
/// `docs/research/firmware-update-spec.md` section 2.
pub(crate) fn parse_at(bytes: &[u8], image_start: usize) -> Result<ImageHeader, FirmwareError> {
    let available = bytes.len().saturating_sub(image_start);
    if available < FIELD_REGION_LEN {
        return Err(FirmwareError::Truncated {
            needed: FIELD_REGION_LEN,
            got: available,
        });
    }
    let h = &bytes[image_start..];

    let head_crc = u32::from_le_bytes([h[0], h[1], h[2], h[3]]);
    let head_len = u32::from_le_bytes([h[4], h[5], h[6], h[7]]);
    let fw_len = u32::from_le_bytes([h[8], h[9], h[10], h[11]]);
    let next = u32::from_le_bytes([h[12], h[13], h[14], h[15]]);
    let ver_raw = u32::from_le_bytes([h[16], h[17], h[18], h[19]]);
    let device_type = DeviceType::from_byte(h[20]);
    let cid = h[21];
    let mid = h[22];

    if head_len != HEADER_LEN {
        return Err(FirmwareError::WrongHeaderLength { got: head_len });
    }

    let head_len_usize = head_len as usize;
    let span = &h[8..head_len_usize];
    let computed = compute_head_crc(span);
    if computed != head_crc {
        return Err(FirmwareError::ChecksumMismatch {
            stored: head_crc,
            computed,
        });
    }

    let field = |index: usize| -> &[u8] {
        &h[FIELD_START + FIELD_LEN * index..FIELD_START + FIELD_LEN * (index + 1)]
    };

    let file_id = trimmed_string(field(0));
    if file_id != "ComUsbUpgradeFile" {
        return Err(FirmwareError::WrongFileId { got: file_id });
    }
    let ic_name = trimmed_string(field(1));
    let boot_endpoint = parse_endpoint(field(3)).ok_or(FirmwareError::BadEndpoint {
        field: "bootOutputEndPoint",
    })?;
    let normal_endpoint = parse_endpoint(field(5)).ok_or(FirmwareError::BadEndpoint {
        field: "normalOutputEndPoint",
    })?;
    let reset_cmd = parse_command(field(6), "reset", 17)?;
    if reset_cmd[0] != REPORT_ID {
        return Err(FirmwareError::BadResetReportId { got: reset_cmd[0] });
    }
    let prepare_cmd = parse_command(field(7), "prepare", 49)?;
    let data_cmd = parse_command(field(8), "data", 49)?;
    if prepare_cmd[0] != data_cmd[0] {
        return Err(FirmwareError::ReportIdMismatch {
            prepare: prepare_cmd[0],
            data: data_cmd[0],
        });
    }
    let product = trimmed_string(field(10));

    let payload_start = image_start + PAYLOAD_OFFSET;
    let payload_end = payload_start.checked_add(fw_len as usize);
    if fw_len == 0 || payload_end.is_none_or(|end| end > bytes.len()) {
        return Err(FirmwareError::PayloadOutOfBounds {
            end: payload_end.unwrap_or(usize::MAX),
            len: bytes.len(),
        });
    }

    Ok(ImageHeader {
        head_crc,
        head_len,
        fw_len,
        next,
        version: Version {
            // Only the low 16 bits of the raw u32 are documented ("major<<8 | minor"); the high
            // half is zero in every archived header (section 1), so this intentionally discards
            // anything above it rather than rejecting a header over an undocumented field.
            #[allow(clippy::cast_possible_truncation)]
            major: (ver_raw >> 8) as u8,
            minor: (ver_raw & 0xff) as u8,
        },
        device_type,
        cid,
        mid,
        ic_name,
        boot_endpoint,
        normal_endpoint,
        reset_cmd,
        prepare_cmd,
        data_cmd,
        product,
        payload_start,
    })
}

/// Whether the header at `image_start` still checksums correctly.
///
/// Reuses [`compute_head_crc`], the same function [`parse_at`] checks against, so a re-check
/// after parsing can never disagree with the check parsing already performed.
pub(crate) fn head_crc_matches(bytes: &[u8], image_start: usize) -> bool {
    let Some(h) = bytes.get(image_start..) else {
        return false;
    };
    if h.len() < 12 {
        return false;
    }
    let stored = u32::from_le_bytes([h[0], h[1], h[2], h[3]]);
    let head_len = u32::from_le_bytes([h[4], h[5], h[6], h[7]]) as usize;
    let Some(span) = h.get(8..head_len) else {
        return false;
    };
    compute_head_crc(span) == stored
}

/// `(0x55555555 - sum(data)) mod 2^32`: the header checksum formula (V3), the same shape as
/// [`hyperpace_protocol::checksum`] and [`hyperpace_protocol::struct_check`] but over a u32.
pub(crate) fn compute_head_crc(data: &[u8]) -> u32 {
    let sum = data
        .iter()
        .fold(0u32, |acc, &byte| acc.wrapping_add(u32::from(byte)));
    0x5555_5555u32.wrapping_sub(sum)
}

/// Parse a `cmdlength`-prefixed command field: byte 0 is the on-wire length, byte 1 a feature
/// flag this crate does not act on (`Transport` exposes no way to select feature vs output
/// reports; see the module documentation of [`crate::flash`]), and the report bytes verbatim
/// follow at byte 2.
fn parse_command(
    field: &[u8],
    name: &'static str,
    expected_len: u8,
) -> Result<Vec<u8>, FirmwareError> {
    let cmdlen = field[0];
    if cmdlen != expected_len {
        return Err(FirmwareError::BadCommandLength {
            field: name,
            expected: expected_len,
            got: cmdlen,
        });
    }
    let len = usize::from(cmdlen);
    Ok(field[2..2 + len].to_vec())
}

/// Parse a `vid_XXXX&pid_XXXX...` endpoint field (check V6).
fn parse_endpoint(field: &[u8]) -> Option<UsbEndpoint> {
    let text = core::str::from_utf8(trim_nul(field)).ok()?;
    Some(UsbEndpoint {
        vendor_id: hex_after(text, "vid_")?,
        product_id: hex_after(text, "pid_")?,
    })
}

/// The hex run immediately following `marker` in `text`, parsed as a `u16`.
fn hex_after(text: &str, marker: &str) -> Option<u16> {
    let start = text.find(marker)? + marker.len();
    let rest = &text[start..];
    let end = rest
        .find(|c: char| !c.is_ascii_hexdigit())
        .unwrap_or(rest.len());
    if end == 0 {
        return None;
    }
    u16::from_str_radix(&rest[..end], 16).ok()
}

fn trim_nul(field: &[u8]) -> &[u8] {
    let end = field.iter().position(|&b| b == 0).unwrap_or(field.len());
    &field[..end]
}

fn trimmed_string(field: &[u8]) -> String {
    String::from_utf8_lossy(trim_nul(field)).into_owned()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::test_support::{HYPACE_MOUSE_ENDPOINT, header_bytes, synthetic_package};

    fn field_bytes(text: &str) -> [u8; FIELD_LEN] {
        let mut out = [0u8; FIELD_LEN];
        out[..text.len()].copy_from_slice(text.as_bytes());
        out
    }

    #[test]
    fn a_well_formed_header_parses() {
        let bytes = synthetic_package(64, 0);
        let header = parse_at(&bytes, 0).unwrap();
        assert_eq!(header.head_len, HEADER_LEN);
        assert_eq!(header.fw_len, 64);
        assert_eq!(header.ic_name, "NRF52833");
        assert_eq!(header.device_type, DeviceType::Mouse);
        assert_eq!(
            header.normal_endpoint,
            UsbEndpoint {
                vendor_id: 0x3554,
                product_id: 0xfb14
            }
        );
        assert_eq!(header.version.to_string(), "v2.18");
        assert_eq!(header.reset_cmd.len(), 17);
        assert_eq!(header.prepare_cmd.len(), 49);
        assert_eq!(header.data_cmd.len(), 49);
        assert_eq!(header.payload_start, PAYLOAD_OFFSET);
        assert!(head_crc_matches(&bytes, 0));
    }

    #[test]
    fn truncated_bytes_are_refused() {
        assert_eq!(
            parse_at(&[0; 10], 0),
            Err(FirmwareError::Truncated {
                needed: FIELD_REGION_LEN,
                got: 10
            })
        );
    }

    #[test]
    fn a_wrong_header_length_is_refused() {
        let mut bytes = synthetic_package(64, 0);
        bytes[4..8].copy_from_slice(&100u32.to_le_bytes()); // headLength != 720
        assert_eq!(
            parse_at(&bytes, 0),
            Err(FirmwareError::WrongHeaderLength { got: 100 })
        );
    }

    #[test]
    fn a_corrupted_checksum_is_refused() {
        let mut bytes = synthetic_package(64, 0);
        bytes[0] ^= 0xff;
        assert!(matches!(
            parse_at(&bytes, 0),
            Err(FirmwareError::ChecksumMismatch { .. })
        ));
        assert!(!head_crc_matches(&bytes, 0));
    }

    #[test]
    fn a_wrong_file_id_is_refused() {
        let mut bytes = synthetic_package(64, 0);
        bytes[FIELD_START..FIELD_START + 4].copy_from_slice(b"NOPE");
        let crc = compute_head_crc(&bytes[8..HEADER_LEN as usize]);
        bytes[0..4].copy_from_slice(&crc.to_le_bytes());
        assert_eq!(
            parse_at(&bytes, 0),
            Err(FirmwareError::WrongFileId {
                got: "NOPEsbUpgradeFile".to_owned()
            })
        );
    }

    #[test]
    fn a_payload_that_does_not_fit_is_refused() {
        let bytes = synthetic_package(64, 0);
        let truncated = &bytes[..bytes.len() - 1];
        assert!(matches!(
            parse_at(truncated, 0),
            Err(FirmwareError::PayloadOutOfBounds { .. })
        ));
    }

    #[test]
    fn a_zero_length_payload_is_refused() {
        let bytes = synthetic_package(0, 0);
        assert!(matches!(
            parse_at(&bytes, 0),
            Err(FirmwareError::PayloadOutOfBounds { .. })
        ));
    }

    #[test]
    fn a_bad_endpoint_field_is_refused() {
        let mut header = header_bytes(64, 0, 210, 0, 0, HYPACE_MOUSE_ENDPOINT, 6);
        header[FIELD_START + 5 * FIELD_LEN..FIELD_START + 5 * FIELD_LEN + 4]
            .copy_from_slice(b"nope");
        let crc = compute_head_crc(&header[8..HEADER_LEN as usize]);
        header[0..4].copy_from_slice(&crc.to_le_bytes());
        header.resize(PAYLOAD_OFFSET + 64, 0);
        assert_eq!(
            parse_at(&header, 0),
            Err(FirmwareError::BadEndpoint {
                field: "normalOutputEndPoint"
            })
        );
    }

    #[test]
    fn device_type_maps_documented_bytes_and_keeps_others() {
        assert_eq!(DeviceType::from_byte(210), DeviceType::Mouse);
        assert_eq!(DeviceType::from_byte(211), DeviceType::Dongle);
        assert_eq!(DeviceType::from_byte(209), DeviceType::Other(209));
    }

    #[test]
    fn endpoint_parsing_reads_the_documented_format() {
        let field = field_bytes("vid_3710&pid_5406&mi_01&col05");
        assert_eq!(
            parse_endpoint(&field),
            Some(UsbEndpoint {
                vendor_id: 0x3710,
                product_id: 0x5406
            })
        );
    }

    #[test]
    fn endpoint_parsing_rejects_missing_markers() {
        let field = field_bytes("not_an_endpoint");
        assert_eq!(parse_endpoint(&field), None);
    }
}
