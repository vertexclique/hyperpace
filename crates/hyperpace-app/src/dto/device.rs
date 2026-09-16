//! Device connection, identity and event shapes.

use hyperpace_device::{Access, DeviceEvent};
use hyperpace_protocol::{
    Battery, DeviceIdentity, LinkType, PairPhase, PairState, ReceiverLight, StatusChanged,
};
use serde::{Deserialize, Serialize};

/// Mirrors [`LinkType`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LinkTypeDto {
    /// See [`LinkType::Wireless1k`].
    Wireless1k,
    /// See [`LinkType::Wireless4k`].
    Wireless4k,
    /// See [`LinkType::Wired1k`].
    Wired1k,
    /// See [`LinkType::Wired8k`].
    Wired8k,
    /// See [`LinkType::Wireless2k`].
    Wireless2k,
    /// See [`LinkType::Wireless8k`].
    Wireless8k,
    /// See [`LinkType::Unknown`].
    Unknown {
        /// The raw, unrecognized link byte.
        byte: u8,
    },
}

impl From<LinkType> for LinkTypeDto {
    fn from(link: LinkType) -> Self {
        match link {
            LinkType::Wireless1k => Self::Wireless1k,
            LinkType::Wireless4k => Self::Wireless4k,
            LinkType::Wired1k => Self::Wired1k,
            LinkType::Wired8k => Self::Wired8k,
            LinkType::Wireless2k => Self::Wireless2k,
            LinkType::Wireless8k => Self::Wireless8k,
            LinkType::Unknown(byte) => Self::Unknown { byte },
        }
    }
}

/// Whether this link is a wired connection, for [`crate::dto::DeviceIdentityDto::wired`].
fn is_wired(link: LinkType) -> bool {
    matches!(link, LinkType::Wired1k | LinkType::Wired8k)
}

/// Mirrors [`DeviceIdentity`], plus [`LinkType::max_polling_hz`] computed once so the UI never
/// has to re-derive it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceIdentityDto {
    /// Component id.
    pub cid: u8,
    /// Module id.
    pub mid: u8,
    /// Link type.
    pub link: LinkTypeDto,
    /// [`LinkType::max_polling_hz`] for `link`.
    pub max_polling_hz: u16,
    /// Whether `link` is a wired connection.
    pub wired: bool,
}

impl From<DeviceIdentity> for DeviceIdentityDto {
    fn from(identity: DeviceIdentity) -> Self {
        Self {
            cid: identity.cid,
            mid: identity.mid,
            link: identity.link.into(),
            max_polling_hz: identity.link.max_polling_hz(),
            wired: is_wired(identity.link),
        }
    }
}

/// Mirrors [`Battery`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryDto {
    /// Charge percent, before any smoothing.
    pub percent: u8,
    /// Whether the device is charging.
    pub charging: bool,
    /// Battery voltage in millivolts.
    pub millivolts: u16,
}

impl From<Battery> for BatteryDto {
    fn from(battery: Battery) -> Self {
        Self {
            percent: battery.percent,
            charging: battery.charging,
            millivolts: battery.millivolts,
        }
    }
}

/// Mirrors [`StatusChanged`].
// Six independent, orthogonal change flags mirroring the device's own bitmask exactly (see
// `hyperpace_protocol::StatusChanged`, which carries the same allow for the same reason).
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusChangedDto {
    /// Current DPI stage changed.
    pub dpi: bool,
    /// Polling rate changed.
    pub polling: bool,
    /// Active profile changed.
    pub profile: bool,
    /// DPI indicator settings changed.
    pub dpi_indicator: bool,
    /// Lighting settings changed.
    pub lighting: bool,
    /// Battery state changed.
    pub battery: bool,
}

impl From<StatusChanged> for StatusChangedDto {
    fn from(changed: StatusChanged) -> Self {
        Self {
            dpi: changed.dpi,
            polling: changed.polling,
            profile: changed.profile,
            dpi_indicator: changed.dpi_indicator,
            lighting: changed.lighting,
            battery: changed.battery,
        }
    }
}

/// Mirrors [`Access`]. Defaults to [`Self::ReadOnly`], matching [`Access::default`]: a
/// [`crate::dto::ConnectRequest`] that omits this field never grants write access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AccessDto {
    /// See [`Access::ReadOnly`].
    #[default]
    ReadOnly,
    /// See [`Access::ReadWrite`].
    ReadWrite,
}

impl From<AccessDto> for Access {
    fn from(access: AccessDto) -> Self {
        match access {
            AccessDto::ReadOnly => Self::ReadOnly,
            AccessDto::ReadWrite => Self::ReadWrite,
        }
    }
}

impl From<Access> for AccessDto {
    fn from(access: Access) -> Self {
        match access {
            Access::ReadOnly => Self::ReadOnly,
            Access::ReadWrite => Self::ReadWrite,
        }
    }
}

/// Which transport a connection uses. Mirrors nothing in `hyperpace_device` directly (that crate
/// exposes the two transports as types, not as a tag); this is this crate's own name for the
/// choice a caller of [`crate::dto::ConnectRequest`] makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeviceBackendDto {
    /// [`hyperpace_device::SimTransport`]: never touches real hardware.
    Simulator,
    /// [`hyperpace_device::HidTransport`]: the operator's own device.
    RealDevice,
}

/// One connection option [`crate::commands::device::list_devices`] reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDescriptor {
    /// Which backend this option connects through.
    pub backend: DeviceBackendDto,
    /// A short, non-branded label for this option.
    pub label: String,
    /// One sentence describing this option.
    pub description: String,
}

/// A `connect` request.
///
/// Both fields default to the safe choice when omitted: `real_device: false` selects the
/// simulator (`docs/architecture/api-contract.md`: "the simulator backend is used unless the
/// caller passes an explicit real-device flag"), and `access` defaults to
/// [`AccessDto::ReadOnly`] via [`Access::default`], so a request naming nothing at all can never
/// write to anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConnectRequest {
    /// Connect to the operator's real device instead of the simulator.
    pub real_device: bool,
    /// The write access to request.
    pub access: AccessDto,
}

/// A snapshot of the current connection, returned by `connect` and `device_state`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStateDto {
    /// Whether a device is currently connected.
    pub connected: bool,
    /// Which backend the current connection uses, when one is connected.
    pub backend: Option<DeviceBackendDto>,
    /// The write access the current connection has, when one is connected.
    pub access: Option<AccessDto>,
    /// The identity the connect sequence resolved, once it has.
    pub identity: Option<DeviceIdentityDto>,
    /// The most recent battery reading.
    pub battery: Option<BatteryDto>,
    /// Whether the device last reported itself online.
    pub online: bool,
}

/// One event streamed to a window's `Channel`, mirroring [`DeviceEvent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DeviceEventPayload {
    /// See [`DeviceEvent::Connected`].
    Connected {
        /// The identity the device reported.
        identity: DeviceIdentityDto,
    },
    /// See [`DeviceEvent::Battery`].
    Battery {
        /// The battery reading.
        battery: BatteryDto,
    },
    /// See [`DeviceEvent::Changed`].
    Changed {
        /// What changed.
        changed: StatusChangedDto,
    },
    /// See [`DeviceEvent::Disconnected`].
    Disconnected,
    /// See [`DeviceEvent::Offline`].
    Offline,
}

impl From<DeviceEvent> for DeviceEventPayload {
    fn from(event: DeviceEvent) -> Self {
        match event {
            DeviceEvent::Connected(identity) => Self::Connected {
                identity: identity.into(),
            },
            DeviceEvent::Battery(battery) => Self::Battery {
                battery: battery.into(),
            },
            DeviceEvent::Changed(changed) => Self::Changed {
                changed: changed.into(),
            },
            DeviceEvent::Disconnected => Self::Disconnected,
            DeviceEvent::Offline => Self::Offline,
        }
    }
}

/// Mirrors [`PairPhase`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "camelCase")]
pub enum PairPhaseDto {
    /// See [`PairPhase::Pairing`].
    Pairing,
    /// See [`PairPhase::Failed`].
    Failed,
    /// See [`PairPhase::Succeeded`].
    Succeeded,
    /// See [`PairPhase::Other`].
    Other {
        /// The raw, unrecognized phase byte.
        byte: u8,
    },
}

impl From<PairPhase> for PairPhaseDto {
    fn from(phase: PairPhase) -> Self {
        match phase {
            PairPhase::Pairing => Self::Pairing,
            PairPhase::Failed => Self::Failed,
            PairPhase::Succeeded => Self::Succeeded,
            PairPhase::Other(byte) => Self::Other { byte },
        }
    }
}

/// Mirrors [`PairState`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairStateDto {
    /// Pairing phase.
    pub state: PairPhaseDto,
    /// Seconds left in the pairing window.
    pub seconds_left: u8,
}

impl From<PairState> for PairStateDto {
    fn from(state: PairState) -> Self {
        Self {
            state: state.state.into(),
            seconds_left: state.seconds_left,
        }
    }
}

/// Mirrors [`ReceiverLight`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiverLightDto {
    /// Effect mode; the receiver has no documented mode names.
    pub mode: u8,
    /// RGB color.
    pub color: (u8, u8, u8),
    /// Effect speed.
    pub speed: u8,
    /// Effect brightness.
    pub brightness: u8,
    /// Effect time.
    pub time: u8,
}

impl From<ReceiverLightDto> for ReceiverLight {
    fn from(light: ReceiverLightDto) -> Self {
        Self {
            mode: light.mode,
            color: light.color,
            speed: light.speed,
            brightness: light.brightness,
            time: light.time,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn link_type_round_trips_every_named_variant() {
        for link in [
            LinkType::Wireless1k,
            LinkType::Wireless4k,
            LinkType::Wired1k,
            LinkType::Wired8k,
            LinkType::Wireless2k,
            LinkType::Wireless8k,
            LinkType::Unknown(7),
        ] {
            let dto: LinkTypeDto = link.into();
            let json = serde_json::to_string(&dto).unwrap();
            let back: LinkTypeDto = serde_json::from_str(&json).unwrap();
            assert_eq!(back, dto);
        }
    }

    #[test]
    fn identity_dto_carries_the_derived_polling_cap_and_wired_flag() {
        let identity = DeviceIdentity {
            cid: 62,
            mid: 1,
            link: LinkType::Wired8k,
        };
        let dto: DeviceIdentityDto = identity.into();
        assert_eq!(dto.max_polling_hz, 8000);
        assert!(dto.wired);

        let wireless = DeviceIdentityDto::from(DeviceIdentity {
            cid: 62,
            mid: 1,
            link: LinkType::Wireless2k,
        });
        assert!(!wireless.wired);
    }

    #[test]
    fn access_dto_default_is_read_only() {
        assert_eq!(AccessDto::default(), AccessDto::ReadOnly);
        assert_eq!(Access::from(AccessDto::default()), Access::ReadOnly);
    }

    #[test]
    fn connect_request_defaults_to_simulator_and_read_only() {
        let request: ConnectRequest = serde_json::from_str("{}").unwrap();
        assert!(!request.real_device);
        assert_eq!(request.access, AccessDto::ReadOnly);
    }

    #[test]
    fn device_event_payload_serializes_with_a_type_tag() {
        let payload = DeviceEventPayload::from(DeviceEvent::Offline);
        let json = serde_json::to_value(payload).unwrap();
        assert_eq!(json["type"], "offline");
    }
}
