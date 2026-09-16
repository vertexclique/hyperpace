//! The flash state machine: streams a parsed [`Package`] to a device already sitting in its
//! bootloader, over a caller-supplied [`Transport`].
//!
//! `docs/research/firmware-update-spec.md` sections 5 through 11.
//!
//! # Scope: what this module does not do
//!
//! This crate never opens a device (`docs/architecture/api-contract.md`). Reaching the
//! bootloader needs sending the reset command to the *normal-mode* device, then waiting for it to
//! re-enumerate under a *different* USB identity and opening a connection to that (section 5 and
//! 6): both are device-opening operations. [`flash`] therefore assumes `target` is already
//! connected to the device in its bootloader; the caller (`hyperpace-device`) owns reset,
//! re-enumeration and reconnecting. [`crate::header::ImageHeader::reset_cmd`] carries the reset
//! command's exact bytes, a plain [`hyperpace_protocol::Frame`] encoding
//! [`hyperpace_protocol::Command::EnterUpdateMode`], for that caller to send over its own
//! existing connection.
//!
//! For the same reason, a chained package whose second image is reached over the *normal* PID
//! rather than the boot PID (section 12: "image 2 ... downloaded first over the normal PID") is
//! outside this function's reach when it and the first image need different connections; the plan
//! (`docs/plans/hyperpace.md`) already lists the dual-MCU receiver chain as untestable without
//! hardware and an image. [`flash`] streams every image in `pkg.images` over the one `target` it
//! is given, which is correct whenever a chain shares one boot connection and is this module's
//! documented limit otherwise.
//!
//! # Scope: the `Transport` size mismatch
//!
//! [`Transport::send`] and [`Transport::recv`] move exactly [`FRAME_LEN`] (16) bytes: the size
//! `hyperpace-protocol`'s 16 byte settings frame needs. The bootloader's prepare and data
//! packets carry a 48 byte body (`cmdlength` 49 minus the report id byte). `Transport` has no
//! parameter to carry more per call, and this crate cannot change it (owned by
//! `hyperpace-protocol`). Since 48 is exactly three [`FRAME_LEN`] units, each logical body is
//! sent as three consecutive [`Transport::send`] calls and read back the same way; this is a
//! Hyperpace-internal framing the concrete `Transport` on the other end must reassemble into one
//! real HID report, not a claim about the vendor's own wire format (which this crate cannot
//! verify for this hardware regardless; see the crate documentation). The single-frame reset
//! command needs no such splitting: stripped of its report id, it is already exactly
//! [`FRAME_LEN`] bytes, the same shape as every other command this crate's sibling protocol crate
//! sends.

use core::time::Duration;

use hyperpace_protocol::{FRAME_LEN, Transport};

use crate::error::FirmwareError;
use crate::header::ImageHeader;
use crate::package::Package;

/// Length of a prepare or data packet's body, after its report id: `cmdlength` (49) minus 1.
const BODY_LEN: usize = 48;
/// Bytes of image payload one data packet carries (section 8's D1/D2).
const DATA_CHUNK_LEN: usize = 32;
/// How long one `recv` call waits for a reply before this module treats it as "nothing yet" and
/// polls again.
const RECV_TIMEOUT: Duration = Duration::from_millis(500);
/// Bound on polling for the prepare phase's erase-done state. `20 * RECV_TIMEOUT` is 10 s,
/// matching section 10's R1 download watchdog; no HYPACE erase timing is known to budget this
/// more precisely (section 10's closing note).
const MAX_ERASE_POLLS: usize = 20;
/// Bound on polling for the final `0x88` Success state after the last data packet, on the same
/// 10 s budget as [`MAX_ERASE_POLLS`].
const MAX_SUCCESS_POLLS: usize = 20;
/// Per-packet resend budget for a data packet (section 10's R6: ten tries at one queue position).
const MAX_PACKET_RETRIES: usize = 10;

const OP_DATA_ECHO: u8 = 0xB1;
const STATE_OPCODE: u8 = 0x5B;
const STATE_MARKER: u8 = 0xB5;
const ERROR_OPCODE: u8 = 0x5A;
const ERROR_MARKER: u8 = 0xA5;
const SUCCESS_STATE: u8 = 0x88;
/// Erase-done states that let the queue advance past the prepare packet (section 9's queue
/// advance rule): `1` (`Erase_Backup`, CX only) and `2` (`Erase_Main`).
const ERASE_DONE_STATES: [u8; 2] = [1, 2];

/// Progress reported after every data packet the device acknowledges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// Index of the image currently streaming, within [`Package::images`].
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

/// Stream `pkg` to `target`, a device already connected in its bootloader.
///
/// Sends each image's prepare packet verbatim, waits for the device to report erase done, then
/// streams the payload as data packets read directly from the package's own bytes
/// (`DATA_CHUNK_LEN` bytes at a time; the whole image is never buffered), calling `progress` after
/// every acknowledged packet. See the module documentation for what this function assumes about
/// `target` and does not attempt.
///
/// # Errors
///
/// Returns [`FirmwareError::Empty`] for a package with no images,
/// [`FirmwareError::NotPreflighted`] if the package's own checksums no longer hold (the one
/// preflight condition this function can re-verify itself; a caller must have already confirmed
/// identity and battery through [`crate::guards::preflight`], since this function has neither),
/// [`FirmwareError::DeviceError`] if the device reports an error state,
/// [`FirmwareError::RetryExhausted`] or [`FirmwareError::Timeout`] if the device stops
/// responding, and [`FirmwareError::Transport`] for a transport failure.
pub fn flash<T: Transport>(
    pkg: &Package,
    target: &mut T,
    progress: &mut dyn FnMut(Progress),
) -> Result<(), FirmwareError> {
    if pkg.images.is_empty() {
        return Err(FirmwareError::Empty);
    }
    if !pkg.checksums_hold() {
        return Err(FirmwareError::NotPreflighted);
    }

    let image_count = pkg.images.len();
    for (image_index, header) in pkg.images.iter().enumerate() {
        stream_image(pkg, header, image_index, image_count, target, progress)?;
        // Only the last image flashed emits the final 0x88 Success (section 12); an
        // intermediate image's completion is signalled some other, undocumented way (section 12
        // names no byte for it), so this only waits for Success after the last image.
        if image_index + 1 == image_count {
            wait_for_success(target)?;
        }
    }

    Ok(())
}

fn stream_image<T: Transport>(
    pkg: &Package,
    header: &ImageHeader,
    image_index: usize,
    image_count: usize,
    target: &mut T,
    progress: &mut dyn FnMut(Progress),
) -> Result<(), FirmwareError> {
    // Q1: the prepare packet is copied verbatim, nothing recomputed; `prepare_cmd[0]` is the
    // report id, which this crate's Transport framing does not carry (see the module docs).
    send_body(target, &header.prepare_cmd[1..])?;
    wait_for_erase(target)?;

    // D3: the starting flash address is the template's own bytes, ground-truthed against the
    // archive as a 4 byte big-endian field starting one byte after the opcode.
    let base_address = u32::from_be_bytes([
        header.data_cmd[5],
        header.data_cmd[6],
        header.data_cmd[7],
        header.data_cmd[8],
    ]);
    let payload = pkg.payload(header);
    let total_chunks = payload.chunks(DATA_CHUNK_LEN).count().max(1);

    for (chunk_index, chunk) in payload.chunks(DATA_CHUNK_LEN).enumerate() {
        let is_final = chunk_index + 1 == total_chunks;
        let marker = if is_final { 0xC1 } else { 0xC0 };
        let chunk_offset = u32::try_from(chunk_index * DATA_CHUNK_LEN).unwrap_or(u32::MAX);
        let address = base_address.wrapping_add(chunk_offset);
        let len = u8::try_from(chunk.len()).unwrap_or(0);
        let body = data_body(marker, len, address, chunk);
        send_and_ack(target, &body)?;

        let sent_len = (chunk_index + 1) * DATA_CHUNK_LEN;
        let bytes_sent = u32::try_from(sent_len)
            .unwrap_or(header.fw_len)
            .min(header.fw_len);
        progress(Progress {
            image_index,
            image_count,
            bytes_sent,
            image_len: header.fw_len,
            percent: overall_percent(image_index, image_count, bytes_sent, header.fw_len),
        });
    }

    Ok(())
}

/// Build one data packet body (section 8's D2, ground-truthed against the archive): opcode
/// [`OP_DATA_ECHO`] forced, the chunk marker, `payload`'s length, the running flash address, then
/// `payload` itself, 0xFF-padded past a short final chunk (D5).
fn data_body(marker: u8, len: u8, address: u32, payload: &[u8]) -> [u8; BODY_LEN] {
    let mut body = [0u8; BODY_LEN];
    body[0] = OP_DATA_ECHO;
    body[1] = marker;
    body[2] = len;
    body[4..8].copy_from_slice(&address.to_be_bytes());
    body[16..16 + payload.len()].copy_from_slice(payload);
    for byte in &mut body[16 + payload.len()..BODY_LEN] {
        *byte = 0xFF;
    }
    body
}

/// Send `body` as consecutive [`FRAME_LEN`]-sized [`Transport::send`] calls (see the module
/// documentation for why).
fn send_body<T: Transport>(target: &mut T, body: &[u8]) -> Result<(), FirmwareError> {
    for chunk in body.chunks(FRAME_LEN) {
        let mut buf = [0u8; FRAME_LEN];
        buf[..chunk.len()].copy_from_slice(chunk);
        target.send(&buf).map_err(FirmwareError::Transport)?;
    }
    Ok(())
}

fn recv_chunk<T: Transport>(target: &mut T) -> Result<Option<[u8; FRAME_LEN]>, FirmwareError> {
    target.recv(RECV_TIMEOUT).map_err(FirmwareError::Transport)
}

fn is_error(reply: &[u8; FRAME_LEN]) -> bool {
    reply[0] == ERROR_OPCODE && reply[1] == ERROR_MARKER
}

fn is_state(reply: &[u8; FRAME_LEN]) -> bool {
    reply[0] == STATE_OPCODE && reply[1] == STATE_MARKER
}

fn device_error(reply: &[u8; FRAME_LEN]) -> FirmwareError {
    FirmwareError::DeviceError { state: reply[2] }
}

/// Send one data packet's body and wait for its acknowledgment, resending up to
/// [`MAX_PACKET_RETRIES`] times on silence (section 10's R6). Section 4's "echo-comparison trap"
/// warns against comparing echoed content on a native transport; any non-error reply is treated
/// as the acknowledgment advancing the queue, matching that guidance.
fn send_and_ack<T: Transport>(target: &mut T, body: &[u8]) -> Result<(), FirmwareError> {
    for _ in 0..MAX_PACKET_RETRIES {
        send_body(target, body)?;
        let Some(reply) = recv_chunk(target)? else {
            continue;
        };
        if is_error(&reply) {
            return Err(device_error(&reply));
        }
        return Ok(());
    }
    Err(FirmwareError::RetryExhausted { phase: "data" })
}

/// Poll for the erase-done state that lets the queue advance past the prepare packet (section
/// 9's queue advance rule; the prepare echo (`0xB0`) and the undocumented progress states 3/4/5 are
/// ignored and simply re-polled, matching every reference implementation).
fn wait_for_erase<T: Transport>(target: &mut T) -> Result<(), FirmwareError> {
    for _ in 0..MAX_ERASE_POLLS {
        let Some(reply) = recv_chunk(target)? else {
            continue;
        };
        if is_error(&reply) {
            return Err(device_error(&reply));
        }
        if is_state(&reply) && ERASE_DONE_STATES.contains(&reply[2]) {
            return Ok(());
        }
    }
    Err(FirmwareError::Timeout { phase: "prepare" })
}

/// Poll for the final `0x88` Success state after the last data packet (section 11's C1).
fn wait_for_success<T: Transport>(target: &mut T) -> Result<(), FirmwareError> {
    for _ in 0..MAX_SUCCESS_POLLS {
        let Some(reply) = recv_chunk(target)? else {
            continue;
        };
        if is_error(&reply) {
            return Err(device_error(&reply));
        }
        if is_state(&reply) && reply[2] == SUCCESS_STATE {
            return Ok(());
        }
    }
    Err(FirmwareError::Timeout { phase: "data" })
}

/// Overall percent across every image in the package, from `bytes_sent` of `image_len` in image
/// `image_index` of `image_count` (generalizes section 12's "0-50% for pass 1, 50-100% for pass
/// 2" to any image count).
fn overall_percent(image_index: usize, image_count: usize, bytes_sent: u32, image_len: u32) -> u8 {
    if image_count == 0 {
        return 0;
    }
    let image_progress = if image_len == 0 {
        0
    } else {
        u64::from(bytes_sent) * 100 / u64::from(image_len)
    };
    let index = u64::try_from(image_index).unwrap_or(u64::MAX);
    let count = u64::try_from(image_count).unwrap_or(1);
    let overall = (index * 100 + image_progress) / count;
    u8::try_from(overall.min(100)).unwrap_or(100)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::collections::VecDeque;
    use std::string::String;

    use hyperpace_protocol::TransportError;

    use super::*;
    use crate::package::Package;

    const OP_PREPARE_ECHO: u8 = 0xB0;

    /// A device sitting in its bootloader: reassembles the [`FRAME_LEN`]-chunked bodies this
    /// module sends and replies the way section 9's state machine describes, driven by a small,
    /// test-chosen script rather than real timing.
    struct FakeBootloader {
        inbound: Vec<u8>,
        pending: VecDeque<[u8; FRAME_LEN]>,
        data_packets_seen: usize,
        total_data_packets: usize,
        fail_prepare: bool,
        fail_on_data_packet: Option<usize>,
        silent: bool,
    }

    impl FakeBootloader {
        fn new(total_data_packets: usize) -> Self {
            Self {
                inbound: Vec::new(),
                pending: VecDeque::new(),
                data_packets_seen: 0,
                total_data_packets,
                fail_prepare: false,
                fail_on_data_packet: None,
                silent: false,
            }
        }
    }

    fn state_chunk(state: u8) -> [u8; FRAME_LEN] {
        let mut chunk = [0u8; FRAME_LEN];
        chunk[0] = STATE_OPCODE;
        chunk[1] = STATE_MARKER;
        chunk[2] = state;
        chunk
    }

    fn error_chunk() -> [u8; FRAME_LEN] {
        let mut chunk = [0u8; FRAME_LEN];
        chunk[0] = ERROR_OPCODE;
        chunk[1] = ERROR_MARKER;
        chunk[2] = 0x01;
        chunk
    }

    fn echo_chunk(opcode: u8) -> [u8; FRAME_LEN] {
        let mut chunk = [0u8; FRAME_LEN];
        chunk[0] = opcode;
        chunk
    }

    impl Transport for FakeBootloader {
        fn send(&mut self, frame: &[u8; FRAME_LEN]) -> Result<(), TransportError> {
            self.inbound.extend_from_slice(frame);
            if self.inbound.len() < BODY_LEN {
                return Ok(());
            }
            let body: Vec<u8> = self.inbound.drain(..BODY_LEN).collect();
            if self.silent {
                return Ok(());
            }
            match body[0] {
                OP_PREPARE_ECHO => {
                    self.pending.push_back(if self.fail_prepare {
                        error_chunk()
                    } else {
                        state_chunk(2)
                    });
                }
                OP_DATA_ECHO => {
                    self.data_packets_seen += 1;
                    if self.fail_on_data_packet == Some(self.data_packets_seen) {
                        self.pending.push_back(error_chunk());
                    } else {
                        self.pending.push_back(echo_chunk(OP_DATA_ECHO));
                        if self.data_packets_seen == self.total_data_packets {
                            self.pending.push_back(state_chunk(SUCCESS_STATE));
                        }
                    }
                }
                _ => {}
            }
            Ok(())
        }

        fn recv(&mut self, _timeout: Duration) -> Result<Option<[u8; FRAME_LEN]>, TransportError> {
            Ok(self.pending.pop_front())
        }

        fn description(&self) -> String {
            "fake bootloader".to_owned()
        }
    }

    fn data_packet_count(fw_len: u32) -> usize {
        (fw_len as usize).div_ceil(DATA_CHUNK_LEN).max(1)
    }

    #[test]
    fn flashing_a_single_image_streams_every_packet_and_reaches_full_progress() {
        let bytes = crate::test_support::synthetic_package(96, 0);
        let pkg = Package::parse(&bytes).unwrap();
        let mut transport = FakeBootloader::new(data_packet_count(96));
        let mut seen = Vec::new();
        flash(&pkg, &mut transport, &mut |p| seen.push(p)).unwrap();

        assert_eq!(seen.len(), 3); // 96 / 32
        assert_eq!(seen.last().unwrap().percent, 100);
        assert_eq!(seen.last().unwrap().bytes_sent, 96);
        assert!(seen.windows(2).all(|w| w[0].bytes_sent <= w[1].bytes_sent));
    }

    #[test]
    fn flashing_a_short_final_chunk_still_reaches_full_progress() {
        let bytes = crate::test_support::synthetic_package(40, 0); // 32 + 8
        let pkg = Package::parse(&bytes).unwrap();
        let mut transport = FakeBootloader::new(data_packet_count(40));
        let mut seen = Vec::new();
        flash(&pkg, &mut transport, &mut |p| seen.push(p)).unwrap();
        assert_eq!(seen.len(), 2);
        assert_eq!(seen.last().unwrap().bytes_sent, 40);
        assert_eq!(seen.last().unwrap().percent, 100);
    }

    #[test]
    fn a_device_error_during_prepare_is_reported() {
        let bytes = crate::test_support::synthetic_package(32, 0);
        let pkg = Package::parse(&bytes).unwrap();
        let mut transport = FakeBootloader::new(data_packet_count(32));
        transport.fail_prepare = true;
        let result = flash(&pkg, &mut transport, &mut |_| {});
        assert!(matches!(result, Err(FirmwareError::DeviceError { .. })));
    }

    #[test]
    fn a_device_error_during_data_is_reported() {
        let bytes = crate::test_support::synthetic_package(64, 0);
        let pkg = Package::parse(&bytes).unwrap();
        let mut transport = FakeBootloader::new(data_packet_count(64));
        transport.fail_on_data_packet = Some(1);
        let result = flash(&pkg, &mut transport, &mut |_| {});
        assert!(matches!(result, Err(FirmwareError::DeviceError { .. })));
    }

    #[test]
    fn a_silent_bootloader_times_out_instead_of_hanging() {
        let bytes = crate::test_support::synthetic_package(32, 0);
        let pkg = Package::parse(&bytes).unwrap();
        let mut transport = FakeBootloader::new(1);
        transport.silent = true; // never replies to anything, including the prepare packet
        let result = flash(&pkg, &mut transport, &mut |_| {});
        assert_eq!(result, Err(FirmwareError::Timeout { phase: "prepare" }));
    }

    #[test]
    fn an_empty_package_is_refused() {
        let bytes = crate::test_support::synthetic_package(32, 0);
        let mut pkg = Package::parse(&bytes).unwrap();
        pkg.images.clear();
        let mut transport = FakeBootloader::new(0);
        assert_eq!(
            flash(&pkg, &mut transport, &mut |_| {}),
            Err(FirmwareError::Empty)
        );
    }

    #[test]
    fn a_chained_two_image_package_streams_both_images_in_order() {
        let bytes = crate::test_support::synthetic_chained_package(64, 32);
        let pkg = Package::parse(&bytes).unwrap();
        assert_eq!(pkg.images.len(), 2);
        let total = data_packet_count(64) + data_packet_count(32);
        let mut transport = FakeBootloader::new(total);
        let mut seen = Vec::new();
        flash(&pkg, &mut transport, &mut |p| seen.push(p)).unwrap();

        assert!(seen.iter().any(|p| p.image_index == 0));
        assert!(seen.iter().any(|p| p.image_index == 1));
        assert_eq!(seen.last().unwrap().image_index, 1);
        assert_eq!(seen.last().unwrap().percent, 100);
    }

    #[test]
    fn overall_percent_spans_every_image_evenly() {
        assert_eq!(overall_percent(0, 2, 0, 100), 0);
        assert_eq!(overall_percent(0, 2, 100, 100), 50);
        assert_eq!(overall_percent(1, 2, 100, 100), 100);
    }
}
