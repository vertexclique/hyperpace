// Mirrors the Rust DTOs in crates/hyperpace-app/src/dto/*.rs field-for-field: every field name
// here is that struct's serde `rename_all = "camelCase"` name, and every tagged enum uses the
// same `tag` key and camelCase variant name serde actually produces (see
// docs/architecture/api-contract.md's "Wire format" note). This file IS the frontend half of the
// contract; it does not invent a shape the backend does not send.

/// An RGB color. Every Rust color field is a `(u8, u8, u8)` tuple, which serde serializes as a
/// 3-element JSON array, not an object; this is the wire shape; components that want a `{r,g,b}`
/// object convert at their own boundary (see ColorSwatchPicker).
export type RgbColor = [number, number, number];

// ---- dto/device.rs ----

export type LinkType =
	| { kind: 'wireless1k' }
	| { kind: 'wireless4k' }
	| { kind: 'wired1k' }
	| { kind: 'wired8k' }
	| { kind: 'wireless2k' }
	| { kind: 'wireless8k' }
	| { kind: 'unknown'; byte: number };

export interface DeviceIdentity {
	cid: number;
	mid: number;
	link: LinkType;
	maxPollingHz: number;
	wired: boolean;
}

export interface Battery {
	percent: number;
	charging: boolean;
	millivolts: number;
}

export interface StatusChanged {
	dpi: boolean;
	polling: boolean;
	profile: boolean;
	dpiIndicator: boolean;
	lighting: boolean;
	battery: boolean;
}

// AccessDto and DeviceBackendDto carry no serde `tag`, so a fieldless variant serializes as a
// bare (renamed) string, not `{ "...": ... }`.
export type Access = 'readOnly' | 'readWrite';
export type DeviceBackend = 'simulator' | 'realDevice';

export interface DeviceDescriptor {
	backend: DeviceBackend;
	label: string;
	description: string;
}

/// A `connect` request. `access` defaults to `'readOnly'` on the Rust side
/// (`AccessDto::default`); this UI always sends both fields explicitly so a real device is never
/// opened without an explicit, visible choice.
export interface ConnectRequest {
	realDevice: boolean;
	access: Access;
}

export interface DeviceState {
	connected: boolean;
	backend: DeviceBackend | null;
	access: Access | null;
	identity: DeviceIdentity | null;
	battery: Battery | null;
	online: boolean;
}

export type DeviceEvent =
	| { type: 'connected'; identity: DeviceIdentity }
	| { type: 'battery'; battery: Battery }
	| { type: 'changed'; changed: StatusChanged }
	| { type: 'disconnected' }
	| { type: 'offline' };

export type PairPhase =
	| { phase: 'pairing' }
	| { phase: 'failed' }
	| { phase: 'succeeded' }
	| { phase: 'other'; byte: number };

export interface PairState {
	state: PairPhase;
	secondsLeft: number;
}

export interface ReceiverLight {
	mode: number;
	color: RgbColor;
	speed: number;
	brightness: number;
	time: number;
}

// ---- dto/settings.rs ----

export type Lod =
	| { value: 'oneMillimeter' }
	| { value: 'twoMillimeters' }
	| { value: 'pointSevenMillimeters' }
	| { value: 'other'; byte: number };

export type SleepTime =
	| { value: 'tenSeconds' }
	| { value: 'thirtySeconds' }
	| { value: 'oneMinute' }
	| { value: 'twoMinutes' }
	| { value: 'fiveMinutes' }
	| { value: 'tenMinutes' }
	| { value: 'fifteenMinutes' }
	| { value: 'other'; byte: number };

export type LightModeName =
	| 'off'
	| 'rainbow'
	| 'singleColorBreath'
	| 'fixed'
	| 'neon'
	| 'rainbowBreath';

export type LightMode = { mode: LightModeName } | { mode: 'other'; byte: number };

// Mode names from mouse-protocol-v2.md 7.7. Mode 6 exists in the vendor disable-table but carries
// no label in either vendor language file and has no named LightMode variant on the Rust side
// either (it decodes to `LightMode::Other(6)`), so it is not offered as a selectable option here.
export const LIGHTING_MODES: { value: LightModeName; label: string }[] = [
	{ value: 'off', label: 'Off' },
	{ value: 'rainbow', label: 'Rainbow' },
	{ value: 'singleColorBreath', label: 'Single color breathe' },
	{ value: 'fixed', label: 'Fixed color' },
	{ value: 'neon', label: 'Neon' },
	{ value: 'rainbowBreath', label: 'Rainbow breathe' }
];

// ReceiverLightDto.mode is a raw, undocumented byte (no protocol enum backs it), unlike the
// mouse's own tagged LightMode; this offers the same effect labels against the raw numeric codes
// mouse-protocol-v2.md 7.7 documents, mode 6 included, for that raw field only.
export const RECEIVER_LIGHT_MODES: { value: number; label: string }[] = [
	{ value: 0, label: 'Off' },
	{ value: 1, label: 'Rainbow' },
	{ value: 2, label: 'Single color breathe' },
	{ value: 3, label: 'Fixed color' },
	{ value: 4, label: 'Neon' },
	{ value: 5, label: 'Rainbow breathe' },
	{ value: 6, label: 'Unnamed effect 6' }
];

export type DpiIndicatorModeName = 'off' | 'steady' | 'breathing';

export type DpiIndicatorMode = { mode: DpiIndicatorModeName } | { mode: 'other'; byte: number };

// Mode names from mouse-protocol-v2.md 7.6.
export const DPI_INDICATOR_MODES: { value: DpiIndicatorModeName; label: string }[] = [
	{ value: 'off', label: 'Off' },
	{ value: 'steady', label: 'Steady' },
	{ value: 'breathing', label: 'Breathing' }
];

export interface DpiIndicator {
	mode: DpiIndicatorMode;
	brightness: number;
	speed: number;
	on: boolean;
}

// LongRangeDto is fully fieldless, so like Access and DeviceBackend it serializes as a bare
// camelCase string, not a tagged object. 'unsupported' means the connected model answered status
// 1 to GetLongRangeMode; a screen must show that honestly, never render it as an 'off' toggle.
export type LongRange = 'on' | 'off' | 'unsupported';

// ProfileDto::Active carries data, so unlike LongRange this stays a tagged object even for the
// fieldless 'unsupported' variant (the "Wire format" note). 'unsupported' means the connected
// model answered status 1 to GetCurrentConfig; a screen must show that honestly, never a silent
// profile 0.
export type Profile = { state: 'active'; index: number } | { state: 'unsupported' };

export interface Performance {
	on: boolean;
	timeout: SleepTime;
}

export interface DpiStage {
	dpi: number;
	color: RgbColor;
}

export interface Lighting {
	mode: LightMode;
	color: RgbColor;
	speed: number;
	brightness: number;
	on: boolean;
}

export type MouseButton = 'left' | 'right' | 'middle' | 'backward' | 'forward';
export type DpiAction = 'loop' | 'increase' | 'decrease';
export type ScrollDirection = 'left' | 'right';

export type MacroCycles =
	| { cycles: 'times'; n: number }
	| { cycles: 'untilReleased' }
	| { cycles: 'untilAnyPress' };

// docs/architecture/api-contract.md buttons.rs, tagged by ButtonActionDto's "type" key.
export type ButtonAction =
	| { type: 'disabled' }
	| { type: 'mouse'; button: MouseButton }
	| { type: 'dpi'; action: DpiAction }
	| { type: 'scroll'; direction: ScrollDirection }
	| { type: 'fire'; times: number; intervalMs: number }
	| { type: 'keystroke' }
	| { type: 'macro'; slot: number; cycles: MacroCycles }
	| { type: 'pollingCycle' }
	| { type: 'media'; usage: number }
	| { type: 'unknown'; kind: number; param: number };

export const ASSIGNABLE_BUTTON_ACTION_TYPES = [
	'disabled',
	'mouse',
	'dpi',
	'scroll',
	'fire',
	'keystroke',
	'macro',
	'pollingCycle',
	'media'
] as const;

export interface Settings {
	pollingHz: number;
	dpiStages: DpiStage[];
	currentStage: number;
	lod: Lod;
	debounceMs: number;
	motionSync: boolean;
	angleSnap: boolean;
	ripple: boolean;
	performance: Performance;
	sleep: SleepTime;
	lighting: Lighting;
	buttons: ButtonAction[];
	sensorMode: number;
	dpiIndicator: DpiIndicator;
	longRange: LongRange;
	profile: Profile;
}

export type Modifier =
	| 'leftCtrl'
	| 'leftShift'
	| 'leftAlt'
	| 'leftWin'
	| 'rightCtrl'
	| 'rightShift'
	| 'rightAlt'
	| 'rightWin';

export interface Keystroke {
	modifiers: Modifier[];
	key: number | null;
	media: number | null;
}

export interface SetButtonRequest {
	index: number;
	action: ButtonAction;
	keystroke?: Keystroke;
}

// A `write_setting` request: one named field of Settings and its new value, tagged by
// WriteSettingRequest's "key". One command call writes exactly one field; there is no batch
// variant, so a screen applying several edits at once sends one request per changed field (see
// `diffSettings` in device.svelte.ts).
export type WriteSettingRequest =
	| { key: 'polling'; hz: number }
	| { key: 'debounce'; ms: number }
	| { key: 'motionSync'; on: boolean }
	| { key: 'angleSnap'; on: boolean }
	| { key: 'ripple'; on: boolean }
	| { key: 'lod'; value: Lod }
	| { key: 'performanceOn'; on: boolean }
	| { key: 'performanceTimeout'; value: SleepTime }
	| { key: 'sleep'; value: SleepTime }
	| { key: 'sensorMode'; mode: number }
	| { key: 'currentDpiStage'; index: number }
	| { key: 'dpiStageCount'; count: number }
	| { key: 'dpiStage'; index: number; dpi: number; color: RgbColor }
	| { key: 'lighting'; value: Lighting }
	| { key: 'button'; index: number; action: ButtonAction }
	| { key: 'dpiIndicatorMode'; value: DpiIndicatorMode }
	| { key: 'dpiIndicatorBrightness'; level: number }
	| { key: 'dpiIndicatorSpeed'; speed: number }
	| { key: 'dpiIndicatorOn'; on: boolean }
	| { key: 'longRange'; on: boolean };

// ---- dto/macros.rs ----

// MacroEventRecord::kind is a plain String on the Rust side (hyperpace-store never depends on
// hyperpace-protocol), interpreted by `commands::macros::kind_from_string`, which recognizes
// exactly these four names and falls back to `Other(0)` for anything else.
export const MACRO_EVENT_KINDS = ['Modifier', 'Key', 'Consumer', 'Mouse'] as const;

export interface MacroEvent {
	press: boolean;
	kind: string;
	value: number;
	delayMs: number;
}

/// A saved macro as returned by `list_macros`/`save_macro`, keyed by its store document id (not
/// by device slot: `slot` is optional and only meaningful when the macro is bound to a device
/// macro slot).
export interface MacroRecord {
	id: string;
	name: string;
	slot?: number;
	events: MacroEvent[];
}

export interface SaveMacroRequest {
	id?: string;
	name: string;
	slot?: number;
	events: MacroEvent[];
}

// ---- dto/firmware.rs ----

export interface FirmwareRecord {
	id: string;
	product: string;
	/// Already formatted `"{major}.{minor:02x}"` by the backend; render as-is.
	version: string;
	cid: number;
	mid: number;
	sha256: string;
	/// Unix seconds.
	importedAt: number;
}

export interface FirmwareProgress {
	imageIndex: number;
	imageCount: number;
	bytesSent: number;
	imageLen: number;
	percent: number;
}

// Route 1 ("watch for publication"): the result of `firmware_watch_check`. Every field here
// mirrors `dto::firmware::FirmwareWatchReportDto` and its nested shapes field-for-field; `summary`
// is already the full human-readable line (including the evidence disclaimer where relevant), so a
// screen renders it as-is rather than reconstructing wording itself.

export interface FirmwareWatchConfig {
	url: string;
	description: string;
	changed: boolean;
	evidence: string[];
	summary: string;
}

export interface FirmwareWatchDirectory {
	url: string;
	description: string;
	notable: boolean;
	summary: string;
}

export interface FirmwareWatchFetchError {
	url: string;
	reason: string;
}

export interface FirmwareWatchReport {
	configs: FirmwareWatchConfig[];
	directories: FirmwareWatchDirectory[];
	fetchErrors: FirmwareWatchFetchError[];
	hasFindings: boolean;
	disclaimer: string;
	/// Unix seconds.
	checkedAt: number;
}

// ---- dto/app_settings.rs ----

export type AppSettingsRequest = { action: 'get' } | { action: 'set'; key: string; value: unknown };

export interface AppSettingsResponse {
	settings: Record<string, unknown>;
}

// The app-setting keys this UI reads and writes through `app_settings`. Nothing else in this
// workspace enforces this exact key set (AppSetting::value is opaque JSON on the store side), so
// this is this UI's own convention, not part of the fixed contract.
export const LOW_BATTERY_THRESHOLD_KEY = 'low_battery_threshold_percent';
export const AUTOSTART_KEY = 'autostart';
export const MINIMIZE_TO_TRAY_KEY = 'minimize_to_tray';
// Mirrors `commands::firmware_watch::FIRMWARE_WATCH_ENABLED_KEY`. Absent means enabled, matching
// the Rust side's own default (all three acquisition routes ship enabled).
export const FIRMWARE_WATCH_ENABLED_KEY = 'firmware_watch_enabled';

/// The app settings this UI's Settings screen edits, extracted from `AppSettingsResponse.settings`
/// with the same defaults `hyperpace-app` itself falls back to when a key has never been written
/// (`state::DEFAULT_LOW_BATTERY_THRESHOLD`, `commands::firmware_watch::watch_enabled`).
export interface AppSettings {
	autostart: boolean;
	lowBatteryThresholdPercent: number;
	minimizeToTray: boolean;
	firmwareWatchEnabled: boolean;
}

export const DEFAULT_LOW_BATTERY_THRESHOLD_PERCENT = 15;

export const MACRO_NAME_MAX_BYTES = 30;
export const MACRO_EVENT_MAX_COUNT = 70;

export type ScreenId = 'buttons' | 'performance' | 'macros' | 'lighting' | 'firmware' | 'settings';
