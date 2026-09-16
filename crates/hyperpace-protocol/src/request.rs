//! Request builders: one constructor per command that a host actually sends.
//!
//! `docs/research/mouse-protocol-v2.md` section 5, with the worked frames of section 7.9 and 7.10
//! kept as golden-vector tests. Every builder here returns bytes straight from fixed struct
//! fields, never `Frame::with_payload` on data whose length is not already known to fit, so none
//! of them can fail except [`write_flash`], whose payload length is the caller's to bound.

use crate::encoding::scalar_pair;
use crate::frame::{Frame, FrameError, PAYLOAD_LEN};
use crate::settings::ReceiverLight;

/// An `EncryptionData` (command 1) request carrying a 4-byte challenge.
///
/// The device never compares the challenge bytes back (section 3.2), so any 4 bytes serve; a
/// caller that wants a fresh value each connect can fill `nonce` however it likes.
#[must_use]
pub fn handshake(nonce: [u8; 4]) -> Frame {
    let mut frame = Frame::command(1);
    frame.length = 8;
    frame.payload[..4].copy_from_slice(&nonce);
    frame
}

/// A `ReadFlashData` (command 8) request for `len` bytes starting at `address`.
///
/// `len` is clamped to [`PAYLOAD_LEN`] so this can never build a request the device would refuse,
/// matching the contract note that callers keep `len <= PAYLOAD_LEN`.
#[must_use]
pub fn read_flash(address: u16, len: u8) -> Frame {
    let mut frame = Frame::command(8);
    frame.address = address;
    frame.length = len.min(u8::try_from(PAYLOAD_LEN).unwrap_or(u8::MAX));
    frame
}

/// A `WriteFlashData` (command 7) request writing `data` at `address`.
///
/// One call is one frame: splitting a write larger than [`PAYLOAD_LEN`] bytes into multiple
/// frames is the device layer's job (`docs/architecture/api-contract.md`), not this crate's.
///
/// # Errors
///
/// Returns [`FrameError::PayloadTooLong`] when `data` is longer than [`PAYLOAD_LEN`] bytes.
pub fn write_flash(address: u16, data: &[u8]) -> Result<Frame, FrameError> {
    let mut frame = Frame::with_payload(7, data)?;
    frame.address = address;
    Ok(frame)
}

/// A `WriteFlashData` request writing the scalar pair `[value, 0x55 - value]` at `address`
/// (section 7.1).
#[must_use]
pub fn set_scalar(address: u16, value: u8) -> Frame {
    let mut frame = Frame::command(7);
    frame.address = address;
    frame.length = 2;
    frame.payload[..2].copy_from_slice(&scalar_pair(value));
    frame
}

/// A `DongleEnterPair` (command 5) request asking the receiver to pair with a mouse of model
/// `cid`.
///
/// Protocol reference section 10.1: `05 00 00 00 02 00 00 <cid>`. The declared length is 2 while
/// the cid sits at payload byte 2, outside it; the receiver reads it there regardless. A bare
/// command-5 frame with no cid is refused with status 1, which is easy to misread as "pairing is
/// unsupported" when the request itself was the problem.
#[must_use]
pub fn enter_pair(cid: u8) -> Frame {
    let mut frame = Frame::command(5);
    frame.length = 2;
    frame.payload[2] = cid;
    frame
}

/// A `SetCurrentConfig` (command 15) request selecting profile `index`.
#[must_use]
pub fn set_profile(index: u8) -> Frame {
    let mut frame = Frame::command(15);
    frame.length = 1;
    frame.payload[0] = index;
    frame
}

/// A `SetLongRangeMode` (command 22) request. Section 7.9: a fixed 10-byte payload with only
/// byte 0 significant.
#[must_use]
pub fn set_long_range(on: bool) -> Frame {
    let mut frame = Frame::command(22);
    frame.length = 10;
    frame.payload[0] = u8::from(on);
    frame
}

/// A `SetDongleLight` (command 24) request, renamed: sets the receiver's indicator light.
///
/// Section 10.5: a fixed 10-byte payload `[mode, r, g, b, speed, brightness, time]`, padded.
#[must_use]
pub fn receiver_light(light: &ReceiverLight) -> Frame {
    let mut frame = Frame::command(24);
    frame.length = 10;
    frame.payload[0] = light.mode;
    frame.payload[1] = light.color.0;
    frame.payload[2] = light.color.1;
    frame.payload[3] = light.color.2;
    frame.payload[4] = light.speed;
    frame.payload[5] = light.brightness;
    frame.payload[6] = light.time;
    frame
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn enter_pair_matches_the_documented_frame() {
        // Section 10.1: `05 00 00 00 02 00 00 <cid> 00*7 <ck>`.
        let bytes = enter_pair(102).encode();
        assert_eq!(&bytes[..8], &[5, 0, 0, 0, 2, 0, 0, 102]);
        assert!(bytes[8..15].iter().all(|&byte| byte == 0));
    }

    #[test]
    fn handshake_matches_the_eight_byte_payload_shape() {
        let frame = handshake([1, 2, 3, 4]);
        assert_eq!(frame.command, 1);
        assert_eq!(frame.length, 8);
        assert_eq!(&frame.payload[..4], &[1, 2, 3, 4]);
        assert_eq!(&frame.payload[4..8], &[0, 0, 0, 0]);
    }

    #[test]
    fn read_flash_matches_the_first_connect_read() {
        // Section 7.10: 08 00 00 00 0A 00*10 3B.
        let frame = read_flash(0, 10);
        assert_eq!(
            frame.encode(),
            [0x08, 0, 0, 0, 0x0a, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x3b]
        );
    }

    #[test]
    fn read_flash_matches_the_new_last_read() {
        // Section 7.10: 08 00 00 FA 06 00*10 45.
        let frame = read_flash(250, 6);
        assert_eq!(
            frame.encode(),
            [0x08, 0, 0, 0xfa, 0x06, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x45]
        );
    }

    #[test]
    fn read_flash_clamps_an_oversized_length() {
        let frame = read_flash(0, 200);
        assert_eq!(frame.length, u8::try_from(PAYLOAD_LEN).unwrap());
    }

    #[test]
    fn write_flash_refuses_an_oversized_payload() {
        assert_eq!(
            write_flash(0, &[0; PAYLOAD_LEN + 1]),
            Err(FrameError::PayloadTooLong(PAYLOAD_LEN + 1))
        );
    }

    #[test]
    fn set_scalar_matches_the_polling_rate_worked_example() {
        // Section 7.3: rate 1000 at offset 0 -> 07 00 00 00 02 01 54 00*8 EF.
        let frame = set_scalar(0, 1);
        assert_eq!(
            frame.encode(),
            [
                0x07, 0, 0, 0, 0x02, 0x01, 0x54, 0, 0, 0, 0, 0, 0, 0, 0, 0xef
            ]
        );
    }

    #[test]
    fn set_scalar_matches_the_debounce_worked_example() {
        // Section 7.8: debounce 8 ms at offset 169 -> 07 00 00 A9 02 08 4D 00*8 46.
        let frame = set_scalar(169, 8);
        assert_eq!(
            frame.encode(),
            [
                0x07, 0, 0, 0xa9, 0x02, 0x08, 0x4d, 0, 0, 0, 0, 0, 0, 0, 0, 0x46
            ]
        );
    }

    #[test]
    fn set_profile_matches_the_documented_shape() {
        let frame = set_profile(2);
        assert_eq!(frame.command, 15);
        assert_eq!(frame.length, 1);
        assert_eq!(frame.payload[0], 2);
    }

    #[test]
    fn set_long_range_matches_the_worked_examples() {
        // Section 7.9.
        assert_eq!(
            set_long_range(true).encode(),
            [0x16, 0, 0, 0, 0x0a, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x2c]
        );
        assert_eq!(
            set_long_range(false).encode(),
            [0x16, 0, 0, 0, 0x0a, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x2d]
        );
    }

    #[test]
    fn receiver_light_lays_out_the_fixed_record() {
        let light = ReceiverLight {
            mode: 0,
            color: (0xff, 0, 0),
            speed: 5,
            brightness: 5,
            time: 1,
        };
        let frame = receiver_light(&light);
        assert_eq!(frame.command, 24);
        assert_eq!(frame.length, 10);
        assert_eq!(&frame.payload[..7], &[0, 0xff, 0, 0, 5, 5, 1]);
    }
}
