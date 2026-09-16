//! Synthetic package bytes shared by this crate's own unit and integration tests.
//!
//! Every field shape mirrors `docs/research/firmware-update-spec.md` section 1, but no HYPACE
//! image exists anywhere to source real values from (the crate documentation and
//! `firmware/archive/SOURCES.md` say so plainly): these bytes prove the parser and the state
//! machine against a package with the right shape, never against a genuine one. The endpoint
//! constants use the HYPACE's own passive USB descriptor read (`firmware/archive/SOURCES.md`),
//! since that much is directly observed, not invented.
#![cfg(test)]

use hyperpace_protocol::Command;

use crate::header::{self, FIELD_LEN, FIELD_REGION_LEN, FIELD_START, HEADER_LEN, PAYLOAD_OFFSET};

/// A single-MCU reset frame: [`Command::EnterUpdateMode`] with no payload (section 5's B2), the
/// same shape [`crate::header::ImageHeader::reset_cmd`] parses out of a real image.
fn single_reset_frame() -> hyperpace_protocol::Frame {
    hyperpace_protocol::Frame::command(Command::EnterUpdateMode as u8)
}

/// The mouse's own normal-mode USB identity, wired.
pub(crate) const HYPACE_MOUSE_ENDPOINT: &str = "vid_3554&pid_fb14&mi_01&col05";
/// The receiver's normal-mode USB identity.
pub(crate) const HYPACE_RECEIVER_ENDPOINT: &str = "vid_3554&pid_fb16&mi_01&col05";
/// A synthetic boot-mode identity; no real HYPACE boot PID is known (spec section 16, item 1).
pub(crate) const BOOT_ENDPOINT: &str = "vid_3554&pid_f400&col01";

fn field_bytes(text: &str) -> [u8; FIELD_LEN] {
    let mut out = [0u8; FIELD_LEN];
    out[..text.len()].copy_from_slice(text.as_bytes());
    out
}

fn command_field(cmdlen: u8, report: &[u8]) -> [u8; FIELD_LEN] {
    let mut out = [0u8; FIELD_LEN];
    out[0] = cmdlen;
    out[2..2 + report.len()].copy_from_slice(report);
    out
}

/// Build one well-formed 727 byte header region (not padded to a full image), with a correct
/// `headCRC`, for the given fields.
pub(crate) fn header_bytes(
    fw_len: u32,
    next: u32,
    device_type_byte: u8,
    cid: u8,
    mid: u8,
    normal_endpoint: &str,
    data_report_id: u8,
) -> Vec<u8> {
    let mut out = vec![0u8; FIELD_REGION_LEN];
    out[4..8].copy_from_slice(&HEADER_LEN.to_le_bytes());
    out[8..12].copy_from_slice(&fw_len.to_le_bytes());
    out[12..16].copy_from_slice(&next.to_le_bytes());
    out[16..20].copy_from_slice(&0x0218u32.to_le_bytes()); // raw 536: v2.18
    out[20] = device_type_byte;
    out[21] = cid;
    out[22] = mid;

    let set_field = |out: &mut [u8], index: usize, bytes: [u8; FIELD_LEN]| {
        out[FIELD_START + FIELD_LEN * index..FIELD_START + FIELD_LEN * (index + 1)]
            .copy_from_slice(&bytes);
    };

    set_field(&mut out, 0, field_bytes("ComUsbUpgradeFile"));
    set_field(&mut out, 1, field_bytes("NRF52833"));
    set_field(&mut out, 3, field_bytes(BOOT_ENDPOINT));
    set_field(&mut out, 5, field_bytes(normal_endpoint));

    let reset_frame = single_reset_frame().encode();
    let mut reset_report = vec![hyperpace_protocol::REPORT_ID];
    reset_report.extend_from_slice(&reset_frame);
    set_field(&mut out, 6, command_field(17, &reset_report));

    let mut prepare_report = vec![data_report_id, 0xb0, 0, 0, 0];
    prepare_report.extend_from_slice(&fw_len.to_be_bytes());
    prepare_report.extend_from_slice(&0x1234_5678u32.to_be_bytes());
    prepare_report.resize(49, 0);
    set_field(&mut out, 7, command_field(49, &prepare_report));

    let mut data_report = vec![data_report_id, 0xb1, 0, 0, 0];
    data_report.extend_from_slice(&0u32.to_be_bytes());
    data_report.resize(49, 0);
    set_field(&mut out, 8, command_field(49, &data_report));

    set_field(&mut out, 10, field_bytes("Test Device"));

    let crc = header::compute_head_crc(&out[8..HEADER_LEN as usize]);
    out[0..4].copy_from_slice(&crc.to_le_bytes());
    out
}

/// A full single-image package: a wired-mouse header plus `fw_len` bytes of payload.
pub(crate) fn synthetic_package(fw_len: u32, next: u32) -> Vec<u8> {
    let mut bytes = header_bytes(fw_len, next, 210, 0, 0, HYPACE_MOUSE_ENDPOINT, 6);
    bytes.resize(PAYLOAD_OFFSET, 0);
    bytes.extend(std::iter::repeat_n(0xABu8, fw_len as usize));
    bytes
}

/// Like [`synthetic_package`], but with an explicit, nonzero declared cid/mid, for tests of the
/// identity checks that only bite when a package actually asserts an identity.
pub(crate) fn synthetic_package_with_identity(fw_len: u32, cid: u8, mid: u8) -> Vec<u8> {
    let mut bytes = header_bytes(fw_len, 0, 210, cid, mid, HYPACE_MOUSE_ENDPOINT, 6);
    bytes.resize(PAYLOAD_OFFSET, 0);
    bytes.extend(std::iter::repeat_n(0xABu8, fw_len as usize));
    bytes
}

/// A full single-image receiver package.
pub(crate) fn synthetic_receiver_package(fw_len: u32) -> Vec<u8> {
    let mut bytes = header_bytes(fw_len, 0, 211, 0, 0, HYPACE_RECEIVER_ENDPOINT, 6);
    bytes.resize(PAYLOAD_OFFSET, 0);
    bytes.extend(std::iter::repeat_n(0xCDu8, fw_len as usize));
    bytes
}

/// A two-image chained mouse package: main MCU (report id 6) then a sub MCU (report id 9), as
/// section 12 documents for a chained pair.
pub(crate) fn synthetic_chained_package(fw_len_a: u32, fw_len_b: u32) -> Vec<u8> {
    let next = u32::try_from(PAYLOAD_OFFSET + fw_len_a as usize).unwrap_or(u32::MAX);
    let mut first = header_bytes(fw_len_a, next, 210, 0, 0, HYPACE_MOUSE_ENDPOINT, 6);
    first.resize(PAYLOAD_OFFSET, 0);
    first.extend(std::iter::repeat_n(0xAAu8, fw_len_a as usize));

    let mut second = header_bytes(fw_len_b, 0, 210, 0, 0, HYPACE_MOUSE_ENDPOINT, 9);
    second.resize(PAYLOAD_OFFSET, 0);
    second.extend(std::iter::repeat_n(0xBBu8, fw_len_b as usize));

    first.extend(second);
    first
}
