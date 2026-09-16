//! The error type shared by container parsing, identity matching, preflight guards and the flash
//! state machine.
//!
//! One enum covers the whole crate, unlike `hyperpace-protocol`'s per-module errors, because
//! [`crate::package::Package::parse`], [`crate::guards::preflight`] and [`crate::flash::flash`]
//! form one pipeline a caller runs in sequence, and a single error type lets it propagate through
//! all three with `?` unchanged.

use core::fmt;

use hyperpace_protocol::TransportError;

use crate::package::Match;

/// Why a firmware package could not be parsed, preflighted or flashed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirmwareError {
    /// Fewer bytes were available from an image's own start than its declared header region
    /// needs (`docs/research/firmware-update-spec.md` section 1's 727 byte field span).
    Truncated {
        /// Bytes the parser needed to read the full field region.
        needed: usize,
        /// Bytes actually available from the image's start.
        got: usize,
    },
    /// The header's `fileId` field was not `ComUsbUpgradeFile` (check V1).
    WrongFileId {
        /// What the field actually held.
        got: String,
    },
    /// `headLength` was not exactly 720 (check V2).
    WrongHeaderLength {
        /// The declared length.
        got: u32,
    },
    /// The stored `headCRC` did not match the recomputed checksum over bytes 8..`headLength`
    /// (check V3).
    ChecksumMismatch {
        /// Checksum stored in the header.
        stored: u32,
        /// Checksum recomputed from the header bytes.
        computed: u32,
    },
    /// The declared payload does not fit inside the file (check V4).
    PayloadOutOfBounds {
        /// Byte offset the payload would need to end at.
        end: usize,
        /// Bytes actually available in the file.
        len: usize,
    },
    /// `nextFileAddress` did not point exactly past the first image's payload, or pointed at or
    /// past the end of the file (check V5).
    ChainOutOfBounds {
        /// The declared next-image offset.
        next: u32,
        /// The offset it was required to equal.
        expected: usize,
    },
    /// More chained images than the safety bound allows; guards against a crafted cycle in
    /// `nextFileAddress`, since every archived package chains at most two.
    ChainTooLong {
        /// The chain length limit that was hit.
        limit: usize,
    },
    /// An endpoint field was not `vid_XXXX&pid_XXXX` (check V6).
    BadEndpoint {
        /// Which field failed: `"bootOutputEndPoint"` or `"normalOutputEndPoint"`.
        field: &'static str,
    },
    /// A command field's declared on-wire length did not match section 1's fixed lengths (check
    /// V7): 17 for the reset command, 49 for prepare and data.
    BadCommandLength {
        /// Which command field: `"reset"`, `"prepare"` or `"data"`.
        field: &'static str,
        /// The length section 1 requires.
        expected: u8,
        /// The length the header declared.
        got: u8,
    },
    /// The reset command's report id was not [`hyperpace_protocol::REPORT_ID`] (check V7, B1).
    BadResetReportId {
        /// The report id the header declared.
        got: u8,
    },
    /// The prepare and data command fields declared different report ids, so no single boot
    /// collection could carry both (check V7, section 4's X1).
    ReportIdMismatch {
        /// Report id the prepare command declared.
        prepare: u8,
        /// Report id the data command declared.
        data: u8,
    },
    /// The package has no images at all.
    Empty,
    /// [`crate::guards::preflight`] found the package's own checksums no longer hold.
    ChecksumFailed,
    /// [`crate::guards::preflight`] found the package does not target the connected hardware.
    NotTarget {
        /// What the identity guard found instead of [`Match::Target`].
        found: Match,
    },
    /// [`crate::guards::preflight`] found the battery too low, or unknown, to risk a flash that
    /// cannot be paused or resumed mid-image.
    BatteryTooLow {
        /// Battery percent, if it was known.
        percent: Option<u8>,
        /// The minimum percent a flash requires.
        required: u8,
    },
    /// [`crate::flash::flash`] was called on a package whose own checksum re-check failed; it
    /// refuses to start. This is the one preflight condition `flash` can re-verify itself, since
    /// it is the only one that needs no external state (see the module documentation).
    NotPreflighted,
    /// A byte-moving failure from the [`hyperpace_protocol::Transport`] carrying the flash.
    Transport(TransportError),
    /// The device reported an error state during the transfer (opcode `0x5A`, marker `0xA5`,
    /// section 9).
    DeviceError {
        /// The state byte the device reported alongside the error marker.
        state: u8,
    },
    /// A data packet was not acknowledged within its retry budget (section 10's R6: ten tries at
    /// one queue position).
    RetryExhausted {
        /// Which phase exhausted its retries: `"data"`.
        phase: &'static str,
    },
    /// No reply arrived before the phase's bounded wait ran out.
    Timeout {
        /// Which phase timed out: `"prepare"` or `"data"`.
        phase: &'static str,
    },
}

impl fmt::Display for FirmwareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { needed, got } => {
                write!(f, "firmware header needs {needed} bytes, got {got}")
            }
            Self::WrongFileId { got } => {
                write!(f, "not a firmware container (fileId '{got}')")
            }
            Self::WrongHeaderLength { got } => {
                write!(f, "firmware header declares length {got}, expected 720")
            }
            Self::ChecksumMismatch { stored, computed } => {
                write!(
                    f,
                    "firmware header checksum {stored:#010x} does not match computed {computed:#010x}"
                )
            }
            Self::PayloadOutOfBounds { end, len } => {
                write!(f, "firmware payload needs {end} bytes, file has {len}")
            }
            Self::ChainOutOfBounds { next, expected } => {
                write!(
                    f,
                    "chained image offset {next} does not match the expected {expected}"
                )
            }
            Self::ChainTooLong { limit } => {
                write!(f, "firmware package chains more than {limit} images")
            }
            Self::BadEndpoint { field } => {
                write!(
                    f,
                    "firmware header field '{field}' is not a valid USB endpoint"
                )
            }
            Self::BadCommandLength {
                field,
                expected,
                got,
            } => {
                write!(
                    f,
                    "firmware header '{field}' command declares length {got}, expected {expected}"
                )
            }
            Self::BadResetReportId { got } => {
                write!(f, "firmware reset command uses report id {got}, expected 8")
            }
            Self::ReportIdMismatch { prepare, data } => {
                write!(
                    f,
                    "firmware prepare and data commands disagree on report id ({prepare} vs {data})"
                )
            }
            Self::Empty => write!(f, "firmware package has no images"),
            Self::ChecksumFailed => write!(f, "firmware package failed its checksum check"),
            Self::NotTarget { found } => {
                write!(f, "firmware package does not target this device: {found}")
            }
            Self::BatteryTooLow { percent, required } => match percent {
                Some(percent) => write!(
                    f,
                    "battery at {percent}% is below the {required}% minimum for a flash"
                ),
                None => write!(f, "battery level is unknown, required at least {required}%"),
            },
            Self::NotPreflighted => {
                write!(
                    f,
                    "firmware package failed its own checksum re-check; refusing to flash"
                )
            }
            Self::Transport(err) => write!(f, "{err}"),
            Self::DeviceError { state } => {
                write!(
                    f,
                    "device reported an error during the flash (state {state:#04x})"
                )
            }
            Self::RetryExhausted { phase } => {
                write!(
                    f,
                    "firmware {phase} packet was not acknowledged after every retry"
                )
            }
            Self::Timeout { phase } => {
                write!(f, "firmware {phase} phase timed out waiting for the device")
            }
        }
    }
}

impl core::error::Error for FirmwareError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Transport(err) => Some(err),
            _ => None,
        }
    }
}
