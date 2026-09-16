# Hyperpace API contract

Status: current truth for the autonomous build of `docs/plans/hyperpace.md`. Implementations are
written against these signatures so independently built crates fit together. Changing a signature
means changing it here first and logging the change in `docs/decs/hyperpace_DECS.md`.

Protocol numbers come from `docs/research/mouse-protocol-v2.md`; firmware numbers come from
`docs/research/firmware-update-spec.md`. No crate in this workspace opens the operator's device
outside `hyperpace-device`, and nothing writes to a real device without an explicit caller flag.

## Crate boundaries

| Crate | Depends on | Never does |
|---|---|---|
| `hyperpace-protocol` | nothing outside std | I/O, threads, async |
| `hyperpace-device` | `hyperpace-protocol`, hidapi | interpret settings semantics |
| `hyperpace-firmware` | `hyperpace-protocol` | open a device itself |
| `hyperpace-store` | serde, defradb.rs embedded | know about HID |
| `hyperpace-app` | all of the above, Tauri | hold protocol knowledge |

## hyperpace-protocol

```rust
// frame.rs (implemented)
pub const REPORT_ID: u8; pub const FRAME_LEN: usize; pub const PAYLOAD_LEN: usize;
pub struct Frame { pub command: u8, pub status: u8, pub address: u16, pub length: u8,
                   pub payload: [u8; PAYLOAD_LEN] }
impl Frame { pub fn command(u8) -> Self; pub fn with_payload(u8, &[u8]) -> Result<Self, FrameError>;
             pub fn encode(&self) -> [u8; FRAME_LEN]; pub fn decode(&[u8]) -> Result<Self, FrameError>;
             pub fn checksum_valid(&[u8; FRAME_LEN]) -> bool; pub fn status(&self) -> Status;
             pub fn declared_payload(&self) -> &[u8] }

// command.rs
#[repr(u8)] pub enum Command { Handshake = 1, DriverStatus = 2, Online = 3, Battery = 4,
    EnterPair = 5, PairState = 6, WriteFlash = 7, ReadFlash = 8, FactoryReset = 9,
    StatusChanged = 10, EnterUpdateMode = 13, GetProfile = 14, SetProfile = 15,
    ReadVersion = 18, SetLongRange = 22, GetLongRange = 23, SetReceiverLight = 24,
    GetReceiverLight = 25, GetReceiverVersion = 29 }
impl Command { pub fn from_byte(u8) -> Option<Self>; pub fn request(self) -> Frame }

// request.rs: one constructor per command, each returning Frame
pub fn handshake(nonce: [u8; 4]) -> Frame;
pub fn read_flash(address: u16, len: u8) -> Frame;          // len <= PAYLOAD_LEN
pub fn write_flash(address: u16, data: &[u8]) -> Result<Frame, FrameError>;
pub fn set_scalar(address: u16, value: u8) -> Frame;        // writes [v, 0x55 - v]
pub fn set_profile(index: u8) -> Frame;
pub fn set_long_range(on: bool) -> Frame;
pub fn receiver_light(light: &ReceiverLight) -> Frame;

// response.rs: parsers, each takes a Frame and returns a typed value or ProtocolError
pub struct DeviceIdentity { pub cid: u8, pub mid: u8, pub link: LinkType }
pub enum LinkType { Wireless1k, Wireless4k, Wired1k, Wired8k, Wireless2k, Wireless8k, Unknown(u8) }
impl LinkType { pub fn max_polling_hz(self) -> u16 }
pub struct Battery { pub percent: u8, pub charging: bool, pub millivolts: u16 }
pub struct Version { pub major: u8, pub minor: u8 }        // rendered "v{major}.{minor:02x}"
pub struct PairState { pub state: PairPhase, pub seconds_left: u8 }
pub struct StatusChanged { pub dpi: bool, pub polling: bool, pub profile: bool,
                           pub dpi_indicator: bool, pub lighting: bool, pub battery: bool }
pub fn identity(&Frame) -> Result<DeviceIdentity, ProtocolError>;
pub fn battery(&Frame) -> Result<Battery, ProtocolError>;   // reads 4 payload bytes, not the length byte
pub fn version(&Frame) -> Result<Version, ProtocolError>;
pub fn online(&Frame) -> Result<(bool, [u8; 3]), ProtocolError>;
pub fn long_range(&Frame) -> Result<bool, ProtocolError>;   // command 23; status 1 -> Err(Unsupported)

// model.rs
pub struct ModelTable { pub cid: u8, pub mid: u8, pub buttons: u8, pub max_dpi: u32,
                        pub sensor: &'static str, pub dpi_ranges: &'static [DpiRange],
                        pub default_buttons: &'static [ButtonAction], pub max_debounce_ms: u8 }
pub struct DpiRange { pub min: u32, pub max: u32, pub step: u32, pub flags: u8 }
pub fn table_for(cid: u8, mid: u8) -> Option<&'static ModelTable>;   // cid 102 and cid 62

// settings.rs: the device memory map and a shadow
pub mod offset { pub const POLLING: u16; pub const DPI_STAGE_COUNT: u16; pub const CURRENT_DPI: u16;
                 pub const LOD: u16; pub const DPI_VALUE: u16; pub const DPI_COLOR: u16;
                 pub const DPI_INDICATOR: u16; pub const DPI_INDICATOR_BRIGHTNESS: u16;
                 pub const DPI_INDICATOR_SPEED: u16; pub const DPI_INDICATOR_ON: u16;
                 pub const LIGHT_POWER_SAVE: u16; pub const BUTTONS: u16;
                 pub const LIGHTING: u16; pub const LIGHT_ON: u16; pub const DEBOUNCE: u16;
                 pub const MOTION_SYNC: u16; pub const SLEEP: u16; pub const ANGLE: u16;
                 pub const RIPPLE: u16; pub const LIGHT_OFF_MOVING: u16; pub const PERF_ON: u16;
                 pub const PERF_TIMEOUT: u16; pub const SENSOR_MODE: u16;
                 pub const KEYSTROKE: u16; pub const MACRO: u16 }
pub struct Shadow { /* 16 KiB, 0xff filled */ }
impl Shadow {
    pub fn new() -> Self;
    pub fn apply_read(&mut self, address: u16, data: &[u8]);
    pub fn settings(&self, table: &ModelTable) -> Result<Settings, ProtocolError>;
    pub fn scalar(&self, address: u16) -> u8;
}
pub struct Settings { pub polling_hz: u16, pub dpi_stages: Vec<DpiStage>, pub current_stage: u8,
                      pub lod: Lod, pub debounce_ms: u8, pub motion_sync: bool, pub angle_snap: bool,
                      pub ripple: bool, pub performance: Performance, pub sleep: SleepTime,
                      pub lighting: Lighting, pub buttons: Vec<ButtonAction>, pub sensor_mode: u8,
                      pub dpi_indicator: DpiIndicator }
pub enum DpiIndicatorMode { Off, Steady, Breathing, Other(u8) }
impl DpiIndicatorMode { pub fn from_byte(u8) -> Self; pub fn to_byte(self) -> u8 }
pub struct DpiIndicator { pub mode: DpiIndicatorMode, pub brightness: u8, pub speed: u8, pub on: bool }
// Long range (command 22/23, section 7.9) is not part of Settings/Shadow: it has no flash
// address, so it is read with response::long_range (below), not Shadow::settings.

// encoding.rs: every value conversion, each with an inverse and a property test
pub fn polling_to_byte(hz: u16) -> Option<u8>;  pub fn polling_from_byte(u8) -> Option<u16>;
pub fn dpi_to_bytes(dpi: u32, table: &ModelTable) -> Result<[u8; 4], ProtocolError>;
pub fn dpi_from_bytes(&[u8; 4], table: &ModelTable) -> Result<u32, ProtocolError>;
pub fn indicator_brightness_to_byte(level: u8) -> u8;  pub fn indicator_brightness_from_byte(u8) -> u8;

// buttons.rs
pub enum ButtonAction { Disabled, Mouse(MouseButton), Dpi(DpiAction), Scroll(ScrollDirection),
    Fire { times: u8, interval_ms: u8 }, Keystroke, Macro { slot: u8, cycles: MacroCycles },
    PollingCycle, Media(u16), Unknown { kind: u8, param: u16 } }
impl ButtonAction { pub fn encode(&self) -> [u8; 4]; pub fn decode(&[u8; 4]) -> Self }

// keystroke.rs and macros.rs: slot codecs, both round-trip tested
pub struct Keystroke { pub modifiers: Vec<Modifier>, pub key: Option<u8>, pub media: Option<u16> }
impl Keystroke { pub fn encode(&self) -> Vec<u8>; pub fn decode(&[u8]) -> Result<Self, ProtocolError> }
pub struct MacroSlot { pub name: String, pub events: Vec<MacroEvent> }   // name <= 30 bytes, events <= 70
pub struct MacroEvent { pub press: bool, pub kind: MacroEventKind, pub value: u16, pub delay_ms: u16 }
impl MacroSlot { pub fn encode(&self) -> Result<Vec<u8>, ProtocolError>;
                 pub fn decode(&[u8]) -> Result<Option<Self>, ProtocolError> }

// config_file.rs: the vendor's .bin export format, with its clamp bug fixed
pub fn export(shadow: &Shadow, device: &DeviceIdentity, sensor: &str) -> Vec<u8>;
pub fn import(bytes: &[u8], table: &ModelTable) -> Result<ImportedConfig, ConfigError>;

// transport.rs: the pure trait, so the firmware crate needs nothing from the device crate
pub trait Transport: Send {
    fn send(&mut self, frame: &[u8; FRAME_LEN]) -> Result<(), TransportError>;
    fn recv(&mut self, timeout: Duration) -> Result<Option<[u8; FRAME_LEN]>, TransportError>;
    fn description(&self) -> String;
}
```

Errors: every fallible function returns a crate error enum implementing `core::error::Error`; no
`unwrap` or `expect` outside tests; no panic reachable from a public function.

## hyperpace-device

```rust
// Transport itself is defined in hyperpace-protocol; this crate provides the implementations.
pub struct HidTransport;      // hidapi, vendor collection only, opened read-write
pub struct SimTransport;      // the simulator, used by every test in this workspace

pub struct DeviceHandle;      // cheap clone, talks to the owner thread over channels
impl DeviceHandle {
    pub fn request(&self, frame: Frame, timeout: Duration) -> Result<Frame, DeviceError>;
    pub fn read_settings(&self) -> Result<Shadow, DeviceError>;
    pub fn write_scalar(&self, address: u16, value: u8) -> Result<(), DeviceError>;
    pub fn write_block(&self, address: u16, data: &[u8]) -> Result<(), DeviceError>;
    pub fn events(&self) -> Receiver<DeviceEvent>;
    pub fn access(&self) -> Access;             // ReadOnly by default
}
pub enum Access { ReadOnly, ReadWrite }         // ReadWrite must be passed explicitly by the caller
pub enum DeviceEvent { Connected(DeviceIdentity), Battery(Battery), Changed(StatusChanged),
                       Disconnected, Offline }
pub fn spawn(transport: Box<dyn Transport>, access: Access) -> Result<DeviceHandle, DeviceError>;
pub fn watch() -> Result<Receiver<HotplugEvent>, DeviceError>;   // watcher starts before enumeration
```

Rules: one owner thread per device; writes serialized by construction; requests matched by command
byte; pushes (command 10) dispatched as events and never consumed as a reply; a write attempted
under `Access::ReadOnly` returns `DeviceError::ReadOnly` without touching the transport.

## hyperpace-firmware

```rust
pub struct ImageHeader { pub head_crc: u32, pub head_len: u32, pub fw_len: u32, pub next: u32,
    pub version: Version, pub device_type: DeviceType, pub cid: u8, pub mid: u8,
    pub ic_name: String, pub boot_endpoint: UsbEndpoint, pub normal_endpoint: UsbEndpoint,
    pub reset_cmd: Vec<u8>, pub prepare_cmd: Vec<u8>, pub data_cmd: Vec<u8>, pub product: String }
pub struct Package { pub images: Vec<ImageHeader>, /* payload offsets, never copied */ }
impl Package { pub fn parse(bytes: &[u8]) -> Result<Self, FirmwareError>;   // validates head CRC
               pub fn matches(&self, id: &DeviceIdentity, usb: &UsbIds) -> Match }
pub enum Match { Target, ForeignDevice { reason: String }, Unknown { reason: String } }
pub struct Guards { pub checksum_ok: bool, pub identity: Match, pub battery_percent: Option<u8> }
pub fn preflight(pkg: &Package, id: &DeviceIdentity, battery: Option<u8>) -> Result<(), FirmwareError>;
pub fn flash<T: Transport>(pkg: &Package, target: &mut T, progress: &mut dyn FnMut(Progress))
    -> Result<(), FirmwareError>;

// watch.rs: route 1 ("watch for publication"), pure evaluation; never fetches a URL itself
pub struct ConfigTarget { pub url: &'static str, pub description: &'static str,
                          pub recorded_sha256: &'static str }
pub const CONFIG_TARGETS: &[ConfigTarget];        // the two config files FIRMWARE-VERDICT.md names
pub struct DirectoryTarget { pub url: &'static str, pub description: &'static str }
pub const DIRECTORY_TARGETS: &[DirectoryTarget];  // the firmware directory paths it names
pub const RECORDED_SPA_INDEX_SHA256: &str;        // the directory oracle's "missing" fallback hash
pub const EVIDENCE_DISCLAIMER: &str;              // a hash change alone is not proof of firmware
pub struct ConfigFinding { pub url: &'static str, pub description: &'static str,
    pub recorded_sha256: &'static str, pub fetched_sha256: String, pub changed: bool,
    pub evidence: Vec<String> }                   // evidence: quoted only when changed is true
pub enum DirectoryState { MissingAsRecorded, ExistsAsDirectory,
    ContentChanged { fetched_sha256: String }, Unexpected { status: u16 } }
pub struct DirectoryFinding { pub url: &'static str, pub description: &'static str,
                              pub state: DirectoryState }
pub struct WatchReport { pub configs: Vec<ConfigFinding>, pub directories: Vec<DirectoryFinding>,
                         pub fetch_errors: Vec<(String, String)> }
impl WatchReport { pub fn has_findings(&self) -> bool }
pub fn evaluate_config(target: &ConfigTarget, body: &[u8]) -> ConfigFinding;
pub fn evaluate_directory(target: &DirectoryTarget, status: u16, body: &[u8]) -> DirectoryFinding;
```

`flash` streams 32 byte packets straight from the package bytes, never buffering an image, and
refuses to start unless `preflight` passed. Nothing in this crate opens a device; `watch` extends
that to never fetching a URL either. `hyperpace-app`'s `commands::firmware_watch` owns the actual
HTTP fetch (a bounded, timed-out `reqwest::blocking::Client`) and calls `evaluate_config` /
`evaluate_directory` with what it received.

## hyperpace-store

```rust
pub struct Store;             // embedded defradb.rs node, persistent, no transport
impl Store {
    pub fn open(root: &Path) -> Result<Self, StoreError>;      // default root: $HOME/.hyperpace
    pub fn macros(&self) -> Collection<MacroRecord>;
    pub fn profiles(&self) -> Collection<ProfileSnapshot>;
    pub fn settings(&self) -> Collection<AppSetting>;
    pub fn firmware(&self) -> Collection<FirmwareRecord>;
    pub fn events(&self) -> Collection<EventRecord>;
}
```

If the embedded node cannot be built storage-only, the fallback named in the plan is a plain JSON
file store behind the same `Store` API, and that switch is logged as a decision.

## hyperpace-app

Tauri commands (all async, all returning `Result<T, String>` rendered for the UI):
`list_devices`, `connect`, `disconnect`, `device_state`, `read_settings`, `write_setting`,
`set_button`, `save_macro`, `list_macros`, `delete_macro`, `set_profile`, `factory_reset`,
`pair_receiver`, `receiver_light`, `export_config`, `import_config`, `firmware_list`,
`firmware_import`, `firmware_install`, `firmware_check_for_updates`, `firmware_watch_check`,
`app_settings`.

`firmware_check_for_updates` compares the local archive against the connected device only; it is
not an acquisition route. `firmware_watch_check` is route 1 ("watch for publication"): it fetches
the vendor's own config files and firmware directory paths (`commands::firmware_watch`, pure
evaluation in `hyperpace_firmware::watch`) and returns a `FirmwareWatchReportDto`, gated by the
`firmware_watch_enabled` app setting (absent means enabled) and never downloading or installing
anything. It also runs automatically, at most once per app start plus a slow (24 hour) periodic
check in the background (`commands::firmware_watch::spawn_periodic_check`), never a busy loop.

Events to the UI go over one `Channel<DeviceEventPayload>` per window, re-subscribed when a
destroyed window is recreated. `device_state` is the subscribe command: its `channel` argument
registers this window's channel and its return value is the current connection snapshot; `connect`
and `disconnect` carry no channel of their own. The tray owns battery display; the window may be
closed without exiting.

### Wire format

Every request and response type crossing the IPC boundary lives in `hyperpace-app/src/dto/*.rs`
and derives `#[serde(rename_all = "camelCase")]`, so every JSON field name the UI sends or reads is
the Rust field name with its first letter lowercased and underscores removed (`real_device` ->
`realDevice`). A tagged enum (`WriteSettingRequest`, `ButtonActionDto`, `DeviceEventPayload`,
`LodDto`/`SleepTimeDto`/`LightModeDto`/`LinkTypeDto`/`PairPhaseDto`/`MacroCyclesDto`/
`DpiIndicatorModeDto`, `AppSettingsRequest`) always serializes to a JSON *object* carrying its own
`tag` key (`"type"`, `"key"`, `"mode"`, `"value"`, `"kind"`, `"phase"`, `"cycles"`, `"action"`
respectively) even for a variant with no data (`DeviceEvent::Offline` is `{"type":"offline"}`,
never the bare string `"Offline"`); an enum with no `tag` attribute (`AccessDto`,
`DeviceBackendDto`, `MouseButtonDto`, `DpiActionDto`, `ScrollDirectionDto`, `ModifierDto`,
`LongRangeDto`) serializes as a bare camelCase string instead, since every one of those is fully
fieldless. A `(u8, u8, u8)` color field serializes as a 3-element JSON array, never a `{r,g,b}`
object. `ui/src/lib/types.ts` mirrors every one of these shapes field-for-field and tag-for-tag;
`crates/hyperpace-app/tests/frontend_contract.rs` deserializes a handful of literal frontend
payloads against the real DTOs to keep the two from drifting apart again, and its command-name-list
check needs `ui/src/lib/generated/commands.ts` kept in sync with `command_list::COMMANDS` by hand.

`SettingsDto` carries `dpi_indicator: DpiIndicatorDto` (mirrors `hyperpace_protocol::DpiIndicator`
one-for-one, decoded from the flash shadow like every other settings field) and
`long_range: LongRangeDto` (`"on"` / `"off"` / `"unsupported"`). Long range is not part of the
flash shadow: `read_settings` fills it from a dedicated `GetLongRangeMode` (23) request issued
alongside the shadow read, and maps a status-1 reply to `"unsupported"`, never to `false` (the
honesty fence, `CLAUDE.md` section 8). `write_setting` gains five matching variants: `dpiIndicatorMode`
(`{key, value: DpiIndicatorModeDto}`), `dpiIndicatorBrightness`/`dpiIndicatorSpeed` (`{key, level|speed}`,
plain scalar writes), `dpiIndicatorOn` (`{key, on}`), and `longRange` (`{key, on}`, routed through
`DeviceHandle::request(request::set_long_range(..))`, not `write_scalar`, since it addresses no
flash offset).

`FirmwareWatchReportDto` (`firmware_watch_check`'s return) flattens `hyperpace_firmware::watch`'s
`ConfigFinding`/`DirectoryFinding` into plain structs carrying a pre-rendered `summary` (that
type's own `Display`, disclaimer included), so the UI never reconstructs the wording itself; a
`FirmwareWatchDirectoryDto` drops `DirectoryState`'s detail down to a `notable: bool` plus the same
`summary` text, rather than mirroring that enum tag-for-tag on the wire.

## UI

SvelteKit static SPA, dark only, original SVG artwork of mouse and receiver. Screens: Buttons,
Performance, Macros, Lighting and receiver, Firmware, Settings. The device is called "Hyperpace".
`ui/src/lib/types.ts` is the frontend half of the wire-format contract above; there is no code
generation between it and `hyperpace-app/src/dto/*.rs`, so a change to a command signature or DTO
must be reflected there by hand in the same change.
No vendor name, logo or model string appears anywhere in the interface.
