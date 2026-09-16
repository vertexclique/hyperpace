// Mirrors the Rust types in docs/architecture/api-contract.md. Field names
// match the Rust struct fields verbatim (serde's default, un-renamed
// representation), so the JSON this UI sends and receives lines up with the
// backend without a translation layer.
//
// A few sub-structs (Lighting, Performance, SleepTime, Lod, ReceiverLight,
// DpiStage) are referenced by the contract but not spelled out in it; their
// shape here follows docs/research/mouse-protocol-v2.md sections 7.6-7.8,
// the byte-exact source for the device memory map. Until hyperpace-protocol
// ships these exact struct definitions, treat the shapes below as the UI's
// best-effort bridge, not the fixed contract.

export interface RgbColor {
	r: number;
	g: number;
	b: number;
}

export type LinkType =
	| 'Wireless1k'
	| 'Wireless4k'
	| 'Wired1k'
	| 'Wired8k'
	| 'Wireless2k'
	| 'Wireless8k'
	| { Unknown: number };

export interface DeviceIdentity {
	cid: number;
	mid: number;
	link: LinkType;
}

export interface Battery {
	percent: number;
	charging: boolean;
	millivolts: number;
}

export interface Version {
	major: number;
	minor: number;
}

export interface StatusChanged {
	dpi: boolean;
	polling: boolean;
	profile: boolean;
	dpi_indicator: boolean;
	lighting: boolean;
	battery: boolean;
}

export interface PairState {
	state: 'Idle' | 'Pairing' | 'Fail' | 'Success';
	seconds_left: number;
}

// A single DPI stage: value plus the indicator color shown for that stage,
// per mouse-protocol-v2.md 7.4/7.5 (records at 12+4i and 44+4i).
export interface DpiStage {
	dpi: number;
	color: RgbColor;
}

export type Lod = 'OneMm' | 'TwoMm' | 'PointSevenMm';

export interface Performance {
	on: boolean;
	timeout_s: number;
}

export interface SleepTime {
	seconds: number;
}

// Mode names from mouse-protocol-v2.md 7.7. Mode 6 exists in the vendor
// disable-table but carries no label in either vendor language file; it is
// offered here as "Unnamed effect 6" rather than invented branding.
export const LIGHTING_MODES = [
	{ value: 0, label: 'Off' },
	{ value: 1, label: 'Rainbow' },
	{ value: 2, label: 'Single color breathe' },
	{ value: 3, label: 'Fixed color' },
	{ value: 4, label: 'Neon' },
	{ value: 5, label: 'Rainbow breathe' },
	{ value: 6, label: 'Unnamed effect 6' }
] as const;

export interface Lighting {
	mode: number;
	color: RgbColor;
	speed: number;
	brightness: number;
	on: boolean;
}

export interface DpiIndicator {
	mode: 0 | 1 | 2; // off, steady, breathing
	brightness: number; // level 1..10
	speed: number;
	on: boolean;
}

export interface ReceiverLight {
	mode: number;
	color: RgbColor;
	speed: number;
	brightness: number;
	time: number;
}

// docs/architecture/api-contract.md buttons.rs
export type ButtonAction =
	| { kind: 'Disabled' }
	| { kind: 'Mouse'; button: 'Left' | 'Right' | 'Wheel' | 'Backward' | 'Forward' }
	| { kind: 'Dpi'; action: 'Loop' | 'Increase' | 'Decrease' }
	| { kind: 'Scroll'; direction: 'Left' | 'Right' }
	| { kind: 'Fire'; times: number; interval_ms: number }
	| { kind: 'Keystroke' }
	| { kind: 'Macro'; slot: number; cycles: MacroCycles }
	| { kind: 'PollingCycle' }
	| { kind: 'Media'; usage: number }
	| { kind: 'Unknown'; button_kind: number; param: number };

export type MacroCycles =
	| { kind: 'Times'; count: number }
	| { kind: 'UntilReleased' }
	| { kind: 'UntilAnyButton' };

export const BUTTON_ACTION_KINDS = [
	'Disabled',
	'Mouse',
	'Dpi',
	'Scroll',
	'Fire',
	'Keystroke',
	'Macro',
	'PollingCycle',
	'Media',
	'Unknown'
] as const;

export interface Settings {
	polling_hz: number;
	dpi_stages: DpiStage[];
	current_stage: number;
	lod: Lod;
	debounce_ms: number;
	motion_sync: boolean;
	angle_snap: boolean;
	ripple: boolean;
	performance: Performance;
	sleep: SleepTime;
	lighting: Lighting;
	buttons: ButtonAction[];
	sensor_mode: number;
	// Not part of Settings in the contract (long range lives outside the
	// mapped flash region per mouse-protocol-v2.md 7.9); surfaced alongside
	// it here because the UI has nowhere else honest to put it.
	long_range?: boolean;
	dpi_indicator?: DpiIndicator;
}

export interface Modifier {
	name: 'LCtrl' | 'LShift' | 'LAlt' | 'LWin' | 'RCtrl' | 'RShift' | 'RAlt' | 'RWin';
}

export interface Keystroke {
	modifiers: Modifier[];
	key: number | null;
	media: number | null;
}

export interface MacroEvent {
	press: boolean;
	kind: number;
	value: number;
	delay_ms: number;
}

export interface MacroSlot {
	slot: number;
	name: string;
	events: MacroEvent[];
}

export const MACRO_NAME_MAX_BYTES = 30;
export const MACRO_EVENT_MAX_COUNT = 70;

export interface ModelTable {
	cid: number;
	mid: number;
	buttons: number;
	max_dpi: number;
	sensor: string;
	dpi_ranges: DpiRange[];
	max_debounce_ms: number;
}

export interface DpiRange {
	min: number;
	max: number;
	step: number;
	flags: number;
}

// hyperpace-app Tauri command surface (docs/architecture/api-contract.md,
// "hyperpace-app" section). Every command is async and returns
// Result<T, String>, rendered here as a thrown error on the Err path.

export interface DeviceSummary {
	id: string;
	description: string;
}

export type DeviceEvent =
	| { Connected: DeviceIdentity }
	| { Battery: Battery }
	| { Changed: StatusChanged }
	| 'Disconnected'
	| 'Offline';

export interface DeviceState {
	identity: DeviceIdentity | null;
	battery: Battery | null;
	connected: boolean;
}

export interface FirmwareRecord {
	id: string;
	product: string;
	version: Version;
	device_type: 'Mouse' | 'Dongle';
	cid: number;
	mid: number;
	imported_at: string;
}

export type ScreenId =
	| 'buttons'
	| 'performance'
	| 'macros'
	| 'lighting'
	| 'firmware'
	| 'settings';

export interface AppSettings {
	autostart: boolean;
	low_battery_warning_percent: number;
	minimize_to_tray: boolean;
}
