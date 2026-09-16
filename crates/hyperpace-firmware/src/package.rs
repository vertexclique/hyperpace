//! A parsed firmware package: one or more chained [`ImageHeader`]s plus the bytes they came from,
//! and the identity checks that decide whether a package targets this hardware.
//!
//! `docs/research/firmware-update-spec.md` sections 3 and 12. `Package` never copies a payload
//! region into a second field; `ImageHeader::payload_start` is an offset into the one byte
//! buffer [`Package::parse`] takes ownership of, and `Package::payload` slices it on demand.

use core::fmt;

use hyperpace_protocol::DeviceIdentity;

use crate::error::FirmwareError;
use crate::header::{self, DeviceType, ImageHeader};

/// Compx's USB vendor id, shared by every component in this hardware family.
pub const VENDOR_ID: u16 = 0x3554;
/// Product id the mouse enumerates as over a USB cable (`firmware/archive/SOURCES.md`'s passive
/// descriptor read).
pub const WIRED_PRODUCT_ID: u16 = 0xFB14;
/// Product id the receiver enumerates as.
pub const RECEIVER_PRODUCT_ID: u16 = 0xFB16;

/// The USB vendor and product id of whichever component is reachable right now, however it was
/// enumerated. Distinct from [`crate::header::UsbEndpoint`], which is what an image *declares* it
/// expects; `UsbIds` is what a caller observed on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbIds {
    /// USB vendor id.
    pub vendor_id: u16,
    /// USB product id.
    pub product_id: u16,
}

/// Whether a package's declared identity targets the connected hardware.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Match {
    /// Every identity marker checked (USB endpoint, component type, and cid/mid when the image
    /// declares one) is consistent with the connected device.
    Target,
    /// The package's declared identity markers clearly belong to different hardware.
    ForeignDevice {
        /// Why: names the marker that disagreed and what it held.
        reason: String,
    },
    /// The available evidence does not confirm a match, but does not contradict one either.
    Unknown {
        /// Why the match could not be confirmed.
        reason: String,
    },
}

impl fmt::Display for Match {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Target => write!(f, "target"),
            Self::ForeignDevice { reason } => write!(f, "foreign device ({reason})"),
            Self::Unknown { reason } => write!(f, "unknown ({reason})"),
        }
    }
}

/// A parsed firmware package: one image, or a chain the first image's `nextFileAddress` leads to
/// (section 12).
pub struct Package {
    /// Every image in the package, in chain order. Never empty once parsed.
    pub images: Vec<ImageHeader>,
    /// The package's raw bytes, held once so payloads can be sliced on demand; never duplicated
    /// into a second, payload-sized field.
    bytes: Vec<u8>,
}

impl fmt::Debug for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Package")
            .field("images", &self.images)
            .field("bytes_len", &self.bytes.len())
            .finish()
    }
}

impl Package {
    /// Parse a firmware package from `bytes`, validating every chained header (checks V1 through
    /// V7 of `docs/research/firmware-update-spec.md` section 2).
    ///
    /// # Errors
    ///
    /// Returns [`FirmwareError`] on the first header that fails a check, or
    /// [`FirmwareError::ChainTooLong`] if `nextFileAddress` chains past a safety bound (every
    /// archived package chains at most two images).
    pub fn parse(bytes: &[u8]) -> Result<Self, FirmwareError> {
        /// Real packages chain at most two images; this only guards a crafted cycle.
        const MAX_CHAIN: usize = 8;

        let mut images = Vec::new();
        let mut offset = 0usize;
        loop {
            let parsed = header::parse_at(bytes, offset)?;
            let next = parsed.next;
            let payload_end = parsed.payload_start + parsed.fw_len as usize;
            images.push(parsed);
            if next == 0 {
                break;
            }
            if images.len() >= MAX_CHAIN {
                return Err(FirmwareError::ChainTooLong { limit: MAX_CHAIN });
            }
            let next_offset = next as usize;
            if next_offset != payload_end || next_offset >= bytes.len() {
                return Err(FirmwareError::ChainOutOfBounds {
                    next,
                    expected: payload_end,
                });
            }
            offset = next_offset;
        }

        Ok(Self {
            images,
            bytes: bytes.to_vec(),
        })
    }

    /// The payload bytes for one of this package's own images.
    ///
    /// Bounds were already validated by [`Package::parse`] (check V4), so this never panics for
    /// an `ImageHeader` this `Package` produced.
    pub(crate) fn payload(&self, header: &ImageHeader) -> &[u8] {
        let start = header.payload_start.min(self.bytes.len());
        let end = start
            .saturating_add(header.fw_len as usize)
            .min(self.bytes.len());
        &self.bytes[start..end]
    }

    /// Whether every image's stored `headCRC` still matches a fresh recomputation.
    ///
    /// Always true for a `Package` [`Package::parse`] produced, since parsing already enforces
    /// this; re-checked here (not assumed) so [`crate::guards::preflight`] and
    /// [`crate::flash::flash`] verify the real bytes rather than trusting a stale invariant.
    pub(crate) fn checksums_hold(&self) -> bool {
        self.images.iter().all(|image| {
            let image_start = image.payload_start - header::PAYLOAD_OFFSET;
            header::head_crc_matches(&self.bytes, image_start)
        })
    }

    /// Whether this package's declared identity targets the connected hardware, using both the
    /// USB endpoint it declares and the protocol-level identity the device reports.
    ///
    /// Checked, in order: the connected `usb` ids are a recognized Hyperpace identity at all;
    /// the package's first image declares the same normal-mode endpoint (`docs/research/
    /// firmware-update-spec.md` section 3's T5); its `DeviceType` matches which component `usb`
    /// identifies; and, when the image declares a nonzero cid/mid, it agrees with `id`. A zero
    /// cid/mid (the norm in the archive; section 3's identity handshake note) is not treated as
    /// an automatic match, unlike the vendor's own implementation.
    #[must_use]
    pub fn matches(&self, id: &DeviceIdentity, usb: &UsbIds) -> Match {
        let Some(first) = self.images.first() else {
            return Match::Unknown {
                reason: "package has no images".to_owned(),
            };
        };

        if usb.vendor_id != VENDOR_ID
            || (usb.product_id != WIRED_PRODUCT_ID && usb.product_id != RECEIVER_PRODUCT_ID)
        {
            return Match::Unknown {
                reason: format!(
                    "connected device {:04x}:{:04x} is not a recognized Hyperpace identity",
                    usb.vendor_id, usb.product_id
                ),
            };
        }

        if first.normal_endpoint.vendor_id != usb.vendor_id
            || first.normal_endpoint.product_id != usb.product_id
        {
            return Match::ForeignDevice {
                reason: format!(
                    "image targets {:04x}:{:04x}, connected device is {:04x}:{:04x}",
                    first.normal_endpoint.vendor_id,
                    first.normal_endpoint.product_id,
                    usb.vendor_id,
                    usb.product_id
                ),
            };
        }

        let expected_type = if usb.product_id == RECEIVER_PRODUCT_ID {
            DeviceType::Dongle
        } else {
            DeviceType::Mouse
        };
        if first.device_type != expected_type {
            return Match::Unknown {
                reason: format!(
                    "image declares device type '{}', connected component is a {expected_type}",
                    first.device_type
                ),
            };
        }

        if (first.cid, first.mid) != (0, 0) && (first.cid, first.mid) != (id.cid, id.mid) {
            return Match::Unknown {
                reason: format!(
                    "image declares cid {} mid {}, device reports cid {} mid {}",
                    first.cid, first.mid, id.cid, id.mid
                ),
            };
        }

        Match::Target
    }

    /// A weaker identity check using only the mouse's own protocol identity, for
    /// [`crate::guards::preflight`], whose fixed signature carries no [`UsbIds`].
    ///
    /// `id` only ever describes the mouse (command 1, `EncryptionData`, answered on the vendor
    /// collection); a receiver package always classifies as [`Match::Unknown`] here, since
    /// nothing in that reply describes a receiver. **This alone is not a safety gate**: callers
    /// must additionally confirm [`Package::matches`] returns [`Match::Target`] using the
    /// connected device's real USB ids before flashing; see the module documentation of
    /// [`crate::guards`].
    #[must_use]
    pub(crate) fn matches_identity(&self, id: DeviceIdentity) -> Match {
        let Some(first) = self.images.first() else {
            return Match::Unknown {
                reason: "package has no images".to_owned(),
            };
        };
        if first.device_type != DeviceType::Mouse {
            return Match::Unknown {
                reason: "no receiver identity is available from the mouse's own handshake"
                    .to_owned(),
            };
        }
        match (first.cid, first.mid) {
            // Unspecified in the header (the archive's norm): not a positive contradiction, so
            // this does not by itself block a flash; the USB endpoint check in `matches` is the
            // real gate for this case.
            (0, 0) => Match::Target,
            (cid, mid) if cid == id.cid && mid == id.mid => Match::Target,
            (cid, mid) => Match::ForeignDevice {
                reason: format!(
                    "image declares cid {cid} mid {mid}, device reports cid {} mid {}",
                    id.cid, id.mid
                ),
            },
        }
    }
}

/// Order `packages` so every receiver package precedes every mouse package, per the documented
/// policy that a receiver must be updated before the mouse it pairs with (`docs/research/
/// firmware-update-spec.md` section 12). A stable sort, so packages of the same component type
/// keep their relative order; a package whose first image is neither a mouse nor a receiver sorts
/// after both.
#[must_use]
pub fn order_for_multi_target(packages: Vec<Package>) -> Vec<Package> {
    let mut ordered = packages;
    ordered.sort_by_key(
        |package| match package.images.first().map(|h| h.device_type) {
            Some(DeviceType::Dongle) => 0,
            Some(DeviceType::Mouse) => 1,
            _ => 2,
        },
    );
    ordered
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use hyperpace_protocol::LinkType;

    fn identity() -> DeviceIdentity {
        DeviceIdentity {
            cid: 0,
            mid: 0,
            link: LinkType::Wired1k,
        }
    }

    fn hyperpace_usb(is_receiver: bool) -> UsbIds {
        UsbIds {
            vendor_id: VENDOR_ID,
            product_id: if is_receiver {
                RECEIVER_PRODUCT_ID
            } else {
                WIRED_PRODUCT_ID
            },
        }
    }

    #[test]
    fn matches_rejects_a_foreign_vendor_and_product() {
        let bytes = crate::test_support::synthetic_package(64, 0);
        let pkg = Package::parse(&bytes).unwrap();
        let foreign = UsbIds {
            vendor_id: 0x3710,
            product_id: 0x5406,
        };
        assert!(matches!(
            pkg.matches(&identity(), &foreign),
            Match::Unknown { .. }
        ));
    }

    #[test]
    fn matches_accepts_a_consistent_wired_mouse_package() {
        let bytes = crate::test_support::synthetic_package(64, 0);
        let pkg = Package::parse(&bytes).unwrap();
        assert_eq!(
            pkg.matches(&identity(), &hyperpace_usb(false)),
            Match::Target
        );
    }

    #[test]
    fn matches_rejects_a_hyperpace_receiver_against_a_mouse_image() {
        let bytes = crate::test_support::synthetic_package(64, 0);
        let pkg = Package::parse(&bytes).unwrap();
        assert!(matches!(
            pkg.matches(&identity(), &hyperpace_usb(true)),
            Match::ForeignDevice { .. }
        ));
    }

    #[test]
    fn order_for_multi_target_moves_receivers_first() {
        let mouse = Package::parse(&crate::test_support::synthetic_package(32, 0)).unwrap();
        let receiver =
            Package::parse(&crate::test_support::synthetic_receiver_package(32)).unwrap();
        let ordered = order_for_multi_target(vec![mouse, receiver]);
        assert_eq!(ordered[0].images[0].device_type, DeviceType::Dongle);
        assert_eq!(ordered[1].images[0].device_type, DeviceType::Mouse);
    }

    #[test]
    fn checksums_hold_is_true_for_a_freshly_parsed_package() {
        let bytes = crate::test_support::synthetic_package(64, 0);
        let pkg = Package::parse(&bytes).unwrap();
        assert!(pkg.checksums_hold());
    }

    #[test]
    fn payload_slices_the_declared_region() {
        let bytes = crate::test_support::synthetic_package(64, 0);
        let pkg = Package::parse(&bytes).unwrap();
        let payload = pkg.payload(&pkg.images[0]);
        assert_eq!(payload.len(), 64);
        assert!(payload.iter().all(|&b| b == 0xAB));
    }
}
