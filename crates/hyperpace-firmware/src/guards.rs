//! Preflight guards: the checks Hyperpace runs before ever attempting a flash.
//!
//! The plan names four guards: checksum, identity, battery, and a do-not-unplug warning (the
//! last is a UI concern with nothing to check in code). [`preflight`]'s signature is fixed to
//! `(pkg, id, battery)`, so it carries no [`crate::package::UsbIds`] and its identity guard is
//! necessarily `Package::matches_identity`, the weaker of this crate's two
//! identity checks. **This alone is not a complete safety gate**: a caller must additionally
//! confirm [`crate::package::Package::matches`] returns [`Match::Target`] using the connected
//! device's real USB vendor and product id before ever calling [`preflight`] or
//! [`crate::flash::flash`].

use hyperpace_protocol::DeviceIdentity;

use crate::error::FirmwareError;
use crate::package::{Match, Package};

/// Minimum battery percent before flashing is allowed.
///
/// No vendor tool defines a threshold (`docs/research/firmware-update-spec.md` precondition P3:
/// "Threshold undefined anywhere"); Hyperpace sets this floor itself, since a flash interrupted
/// by power loss can leave the device unable to boot and cannot be paused or resumed.
pub const MIN_BATTERY_PERCENT: u8 = 20;

/// The observable state of every preflight guard, for a caller that wants to show a checklist
/// rather than only a pass or fail result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guards {
    /// Whether the package's own checksums still hold. Always true for a [`Package`]
    /// [`Package::parse`] produced (parsing already enforces this); recomputed here rather than
    /// assumed, so this reflects a real check rather than a trusted invariant.
    pub checksum_ok: bool,
    /// Whether the package's declared identity is consistent with the connected mouse.
    pub identity: Match,
    /// Battery percent as last read, or `None` when it is unknown.
    pub battery_percent: Option<u8>,
}

/// Evaluate every guard without failing fast, so a caller can render all three states at once
/// (for example, a confirmation checklist before an operator-approved flash).
#[must_use]
pub fn guards(pkg: &Package, id: &DeviceIdentity, battery: Option<u8>) -> Guards {
    Guards {
        checksum_ok: pkg.checksums_hold(),
        identity: pkg.matches_identity(*id),
        battery_percent: battery,
    }
}

/// Refuse to proceed unless every guard passes.
///
/// # Errors
///
/// Returns [`FirmwareError::ChecksumFailed`] if the package's checksum no longer holds,
/// [`FirmwareError::NotTarget`] if its identity does not resolve to [`Match::Target`], or
/// [`FirmwareError::BatteryTooLow`] if the battery is below [`MIN_BATTERY_PERCENT`] or unknown.
/// Checks run in that order and stop at the first failure.
pub fn preflight(
    pkg: &Package,
    id: &DeviceIdentity,
    battery: Option<u8>,
) -> Result<(), FirmwareError> {
    let evaluated = guards(pkg, id, battery);
    if !evaluated.checksum_ok {
        return Err(FirmwareError::ChecksumFailed);
    }
    if !matches!(evaluated.identity, Match::Target) {
        return Err(FirmwareError::NotTarget {
            found: evaluated.identity,
        });
    }
    match evaluated.battery_percent {
        Some(percent) if percent >= MIN_BATTERY_PERCENT => Ok(()),
        percent => Err(FirmwareError::BatteryTooLow {
            percent,
            required: MIN_BATTERY_PERCENT,
        }),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use hyperpace_protocol::LinkType;

    fn wired_mouse_identity() -> DeviceIdentity {
        DeviceIdentity {
            cid: 0,
            mid: 0,
            link: LinkType::Wired1k,
        }
    }

    #[test]
    fn every_guard_passing_preflights_clean() {
        let pkg = Package::parse(&crate::test_support::synthetic_package(64, 0)).unwrap();
        assert_eq!(preflight(&pkg, &wired_mouse_identity(), Some(80)), Ok(()));
    }

    #[test]
    fn guards_reports_each_field_independently() {
        let pkg = Package::parse(&crate::test_support::synthetic_package(64, 0)).unwrap();
        let evaluated = guards(&pkg, &wired_mouse_identity(), Some(5));
        assert!(evaluated.checksum_ok);
        assert_eq!(evaluated.identity, Match::Target);
        assert_eq!(evaluated.battery_percent, Some(5));
    }

    #[test]
    fn low_battery_refuses_even_with_a_matching_identity() {
        let pkg = Package::parse(&crate::test_support::synthetic_package(64, 0)).unwrap();
        assert_eq!(
            preflight(&pkg, &wired_mouse_identity(), Some(5)),
            Err(FirmwareError::BatteryTooLow {
                percent: Some(5),
                required: MIN_BATTERY_PERCENT
            })
        );
    }

    #[test]
    fn unknown_battery_refuses() {
        let pkg = Package::parse(&crate::test_support::synthetic_package(64, 0)).unwrap();
        assert_eq!(
            preflight(&pkg, &wired_mouse_identity(), None),
            Err(FirmwareError::BatteryTooLow {
                percent: None,
                required: MIN_BATTERY_PERCENT
            })
        );
    }

    #[test]
    fn a_mismatched_cid_mid_refuses() {
        // A package that actually declares a cid/mid (unlike synthetic_package's default 0/0,
        // which matches_identity treats as unspecified rather than contradicted).
        let pkg = Package::parse(&crate::test_support::synthetic_package_with_identity(
            64, 102, 1,
        ))
        .unwrap();
        let mismatched = DeviceIdentity {
            cid: 62,
            mid: 1,
            link: LinkType::Wired1k,
        };
        assert!(matches!(
            preflight(&pkg, &mismatched, Some(80)),
            Err(FirmwareError::NotTarget { .. })
        ));
    }

    #[test]
    fn a_receiver_package_never_preflights_from_the_mouse_handshake_alone() {
        let pkg = Package::parse(&crate::test_support::synthetic_receiver_package(64)).unwrap();
        // matches_identity cannot confirm a receiver from a command-1 (mouse) reply; preflight
        // must refuse rather than assume, per this module's documentation.
        assert!(matches!(
            preflight(&pkg, &wired_mouse_identity(), Some(80)),
            Err(FirmwareError::NotTarget { .. })
        ));
    }
}
