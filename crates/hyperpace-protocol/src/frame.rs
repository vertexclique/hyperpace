//! The transport envelope of the device channel.
//!
//! Every exchange on the vendor collection is one 16 byte frame carried by HID report
//! [`REPORT_ID`], in both directions:
//!
//! | Byte | Meaning |
//! |---|---|
//! | 0 | command |
//! | 1 | status: 0 in a request; in a response 0 means decoded and 1 means unsupported |
//! | 2, 3 | settings address, big endian, used by the flash commands |
//! | 4 | payload length |
//! | 5 to 14 | payload, at most [`PAYLOAD_LEN`] bytes |
//! | 15 | checksum |
//!
//! The checksum makes the report id and all 16 bytes sum to [`CHECKSUM_TARGET`] modulo 256, so
//! the report id takes part in the sum even though it never appears in the frame itself.

use core::fmt;

/// HID report id that carries the device channel.
pub const REPORT_ID: u8 = 8;

/// Length of a frame, excluding the report id byte the HID stack prepends.
pub const FRAME_LEN: usize = 16;

/// Largest payload a single frame can carry, bytes 5 to 14.
pub const PAYLOAD_LEN: usize = 10;

/// Value that the report id plus every frame byte must sum to, modulo 256.
pub const CHECKSUM_TARGET: u8 = 0x55;

/// Checksum of the first 15 bytes of a frame.
///
/// The device accepts a frame when `REPORT_ID + sum(bytes) == CHECKSUM_TARGET` modulo 256, so the
/// checksum byte is whatever makes that identity hold.
#[must_use]
pub fn checksum(head: &[u8; FRAME_LEN - 1]) -> u8 {
    let sum = head.iter().fold(0u8, |acc, byte| acc.wrapping_add(*byte));
    CHECKSUM_TARGET.wrapping_sub(REPORT_ID).wrapping_sub(sum)
}

/// Status reported by the device in byte 1 of a response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The device decoded the command.
    Decoded,
    /// The device does not implement the command.
    Unsupported,
    /// Any other value, kept verbatim because the firmware's full set is not known.
    Other(u8),
}

impl From<u8> for Status {
    fn from(byte: u8) -> Self {
        match byte {
            0 => Self::Decoded,
            1 => Self::Unsupported,
            other => Self::Other(other),
        }
    }
}

/// A decoded frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    /// Command byte.
    pub command: u8,
    /// Status byte: zero in requests, meaningful in responses.
    pub status: u8,
    /// Settings address, meaningful for the flash commands only.
    pub address: u16,
    /// Number of payload bytes that carry meaning.
    pub length: u8,
    /// Payload bytes 5 to 14, zero padded.
    pub payload: [u8; PAYLOAD_LEN],
}

impl Frame {
    /// A frame that carries only a command, with an empty payload.
    #[must_use]
    pub fn command(command: u8) -> Self {
        Self {
            command,
            status: 0,
            address: 0,
            length: 0,
            payload: [0; PAYLOAD_LEN],
        }
    }

    /// A frame carrying `payload`.
    ///
    /// # Errors
    ///
    /// Returns [`FrameError::PayloadTooLong`] when `payload` exceeds [`PAYLOAD_LEN`].
    pub fn with_payload(command: u8, payload: &[u8]) -> Result<Self, FrameError> {
        if payload.len() > PAYLOAD_LEN {
            return Err(FrameError::PayloadTooLong(payload.len()));
        }
        let mut frame = Self::command(command);
        frame.length = u8::try_from(payload.len()).unwrap_or(0);
        frame.payload[..payload.len()].copy_from_slice(payload);
        Ok(frame)
    }

    /// Serialize the frame, filling in the checksum.
    #[must_use]
    pub fn encode(&self) -> [u8; FRAME_LEN] {
        let mut head = [0u8; FRAME_LEN - 1];
        head[0] = self.command;
        head[1] = self.status;
        head[2] = (self.address >> 8) as u8;
        head[3] = (self.address & 0xff) as u8;
        head[4] = self.length;
        head[5..].copy_from_slice(&self.payload);

        let mut out = [0u8; FRAME_LEN];
        out[..FRAME_LEN - 1].copy_from_slice(&head);
        out[FRAME_LEN - 1] = checksum(&head);
        out
    }

    /// Parse a frame received from the device.
    ///
    /// The checksum is not enforced here, because a response is useful even when its trailing
    /// byte is not what a request would carry; call [`Frame::checksum_valid`] to check it.
    ///
    /// # Errors
    ///
    /// Returns [`FrameError::Length`] when `bytes` is not [`FRAME_LEN`] long.
    pub fn decode(bytes: &[u8]) -> Result<Self, FrameError> {
        let bytes: &[u8; FRAME_LEN] = bytes
            .try_into()
            .map_err(|_| FrameError::Length(bytes.len()))?;
        let mut payload = [0u8; PAYLOAD_LEN];
        payload.copy_from_slice(&bytes[5..FRAME_LEN - 1]);
        Ok(Self {
            command: bytes[0],
            status: bytes[1],
            address: u16::from_be_bytes([bytes[2], bytes[3]]),
            length: bytes[4],
            payload,
        })
    }

    /// Whether `bytes` carries the checksum the device expects.
    #[must_use]
    pub fn checksum_valid(bytes: &[u8; FRAME_LEN]) -> bool {
        let head: &[u8; FRAME_LEN - 1] = &bytes[..FRAME_LEN - 1]
            .try_into()
            .unwrap_or([0; FRAME_LEN - 1]);
        checksum(head) == bytes[FRAME_LEN - 1]
    }

    /// Status reported by the device.
    #[must_use]
    pub fn status(&self) -> Status {
        Status::from(self.status)
    }

    /// The meaningful part of the payload, as declared by byte 4.
    ///
    /// The length byte is device supplied and is known to understate the payload for at least one
    /// command, so callers that need the raw bytes read [`Frame::payload`] instead.
    #[must_use]
    pub fn declared_payload(&self) -> &[u8] {
        let len = (self.length as usize).min(PAYLOAD_LEN);
        &self.payload[..len]
    }
}

/// Why a frame could not be built or parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    /// A buffer of the wrong size was handed to [`Frame::decode`].
    Length(usize),
    /// A payload longer than [`PAYLOAD_LEN`] was handed to [`Frame::with_payload`].
    PayloadTooLong(usize),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length(len) => {
                write!(f, "a device frame is {FRAME_LEN} bytes, got {len}")
            }
            Self::PayloadTooLong(len) => {
                write!(
                    f,
                    "a frame carries at most {PAYLOAD_LEN} payload bytes, got {len}"
                )
            }
        }
    }
}

impl core::error::Error for FrameError {}

#[cfg(test)]
mod tests {
    // Tests may assert: the doctrine bans panics on production paths, not in tests.
    #![allow(clippy::unwrap_used)]

    use super::*;

    // Vectors recomputed by hand from the vendor implementation: the checksum byte is
    // 0x55 - REPORT_ID - sum(bytes 0..14), modulo 256.

    #[test]
    fn command_only_frame_matches_the_vendor_vector() {
        // Command 3 with an empty payload: sum is 3, so the checksum is 0x55 - 8 - 3 = 74.
        let encoded = Frame::command(3).encode();
        assert_eq!(encoded, [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 74]);
    }

    #[test]
    fn flash_read_frame_matches_the_vendor_vector() {
        // Command 8, address 0, length 10: sum is 18, so the checksum is 0x55 - 8 - 18 = 59.
        let mut frame = Frame::command(8);
        frame.length = 10;
        assert_eq!(
            frame.encode(),
            [8, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 59]
        );
    }

    #[test]
    fn address_is_big_endian() {
        let mut frame = Frame::command(8);
        frame.address = 0x0103;
        let encoded = frame.encode();
        assert_eq!([encoded[2], encoded[3]], [0x01, 0x03]);
        assert_eq!(Frame::decode(&encoded).unwrap().address, 0x0103);
    }

    #[test]
    fn every_encoded_frame_satisfies_the_device_identity() {
        for command in 0..=u8::MAX {
            let encoded = Frame::with_payload(command, &[command; PAYLOAD_LEN])
                .unwrap()
                .encode();
            let total = encoded
                .iter()
                .fold(REPORT_ID, |acc, byte| acc.wrapping_add(*byte));
            assert_eq!(
                total, CHECKSUM_TARGET,
                "command {command} broke the identity"
            );
            assert!(Frame::checksum_valid(&encoded));
        }
    }

    #[test]
    fn encode_decode_round_trips() {
        let frame = Frame::with_payload(0x0f, &[1, 2, 3]).unwrap();
        let decoded = Frame::decode(&frame.encode()).unwrap();
        assert_eq!(decoded, frame);
        assert_eq!(decoded.declared_payload(), &[1, 2, 3]);
    }

    #[test]
    fn a_corrupted_checksum_is_detected() {
        let mut encoded = Frame::command(3).encode();
        encoded[FRAME_LEN - 1] = encoded[FRAME_LEN - 1].wrapping_add(1);
        assert!(!Frame::checksum_valid(&encoded));
    }

    #[test]
    fn oversized_payloads_and_buffers_are_refused() {
        assert_eq!(
            Frame::with_payload(1, &[0; PAYLOAD_LEN + 1]),
            Err(FrameError::PayloadTooLong(PAYLOAD_LEN + 1))
        );
        assert_eq!(
            Frame::decode(&[0; FRAME_LEN - 1]),
            Err(FrameError::Length(FRAME_LEN - 1))
        );
    }

    #[test]
    fn status_bytes_map_to_their_meaning() {
        assert_eq!(Status::from(0), Status::Decoded);
        assert_eq!(Status::from(1), Status::Unsupported);
        assert_eq!(Status::from(7), Status::Other(7));
    }

    #[test]
    fn a_declared_length_beyond_the_payload_is_clamped() {
        let mut encoded = Frame::command(4).encode();
        encoded[4] = 200;
        let frame = Frame::decode(&encoded).unwrap();
        assert_eq!(frame.declared_payload().len(), PAYLOAD_LEN);
    }
}
