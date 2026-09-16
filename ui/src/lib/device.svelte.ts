// Central device/app state, built with Svelte 5 runes. Holds exactly one
// shadow of the connected device's settings, per the scale note in
// docs/plans/hyperpace.md 5 ("Hyperpace holds exactly one shadow per
// connected device and never accumulates history in memory"). Nothing here
// invents a value: every field starts null/empty and is filled only from a
// real command response.

import {
	invoke,
	openDeviceEventChannel,
	openFirmwareProgressChannel,
	openPairStateChannel,
	runningInTauri
} from './tauri';
import {
	AUTOSTART_KEY,
	DEFAULT_LOW_BATTERY_THRESHOLD_PERCENT,
	FIRMWARE_WATCH_ENABLED_KEY,
	LOW_BATTERY_THRESHOLD_KEY,
	MINIMIZE_TO_TRAY_KEY
} from './types';
import type {
	Access,
	AppSettings,
	AppSettingsRequest,
	AppSettingsResponse,
	Battery,
	ButtonAction,
	ConnectRequest,
	DeviceBackend,
	DeviceDescriptor,
	DeviceEvent,
	DeviceIdentity,
	DeviceState,
	FirmwareProgress,
	FirmwareRecord,
	FirmwareWatchReport,
	Keystroke,
	MacroRecord,
	PairState,
	ReceiverLight,
	ReceiverLightState,
	SaveMacroRequest,
	Settings,
	SetButtonRequest,
	WriteSettingRequest
} from './types';

/// The `keyof AppSettings` <-> `app_settings` store key mapping. This UI's own convention
/// (`AppSetting::value` is opaque JSON on the store side; see types.ts), kept in one place so a
/// screen never spells a store key literal itself.
const APP_SETTING_KEYS: [keyof AppSettings, string][] = [
	['autostart', AUTOSTART_KEY],
	['lowBatteryThresholdPercent', LOW_BATTERY_THRESHOLD_KEY],
	['minimizeToTray', MINIMIZE_TO_TRAY_KEY],
	['firmwareWatchEnabled', FIRMWARE_WATCH_ENABLED_KEY]
];

function extractAppSettings(settings: Record<string, unknown>): AppSettings {
	const autostart = settings[AUTOSTART_KEY];
	const threshold = settings[LOW_BATTERY_THRESHOLD_KEY];
	const minimizeToTray = settings[MINIMIZE_TO_TRAY_KEY];
	const firmwareWatchEnabled = settings[FIRMWARE_WATCH_ENABLED_KEY];
	return {
		autostart: typeof autostart === 'boolean' ? autostart : false,
		lowBatteryThresholdPercent:
			typeof threshold === 'number' ? threshold : DEFAULT_LOW_BATTERY_THRESHOLD_PERCENT,
		minimizeToTray: typeof minimizeToTray === 'boolean' ? minimizeToTray : false,
		// Absent means enabled, matching the Rust side's own default
		// (`commands::firmware_watch::watch_enabled`).
		firmwareWatchEnabled: typeof firmwareWatchEnabled === 'boolean' ? firmwareWatchEnabled : true
	};
}

/**
 * The `write_setting` requests needed to turn `before` into `after`: one request per field that
 * actually differs, since the command writes exactly one named field per call (no batch variant).
 * Button bindings are not diffed here; `set_button` is the preferred path for those
 * (`WriteSettingRequest::Button`'s own doc comment).
 */
export function diffSettings(before: Settings, after: Settings): WriteSettingRequest[] {
	const requests: WriteSettingRequest[] = [];
	const eq = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);

	if (after.pollingHz !== before.pollingHz) requests.push({ key: 'polling', hz: after.pollingHz });
	if (after.debounceMs !== before.debounceMs) requests.push({ key: 'debounce', ms: after.debounceMs });
	if (after.motionSync !== before.motionSync) {
		requests.push({ key: 'motionSync', on: after.motionSync });
	}
	if (after.angleSnap !== before.angleSnap) requests.push({ key: 'angleSnap', on: after.angleSnap });
	if (after.ripple !== before.ripple) requests.push({ key: 'ripple', on: after.ripple });
	if (!eq(after.lod, before.lod)) requests.push({ key: 'lod', value: after.lod });
	if (after.performance.on !== before.performance.on) {
		requests.push({ key: 'performanceOn', on: after.performance.on });
	}
	if (!eq(after.performance.timeout, before.performance.timeout)) {
		requests.push({ key: 'performanceTimeout', value: after.performance.timeout });
	}
	if (!eq(after.sleep, before.sleep)) requests.push({ key: 'sleep', value: after.sleep });
	if (after.sensorMode !== before.sensorMode) {
		requests.push({ key: 'sensorMode', mode: after.sensorMode });
	}
	if (after.currentStage !== before.currentStage) {
		requests.push({ key: 'currentDpiStage', index: after.currentStage });
	}
	if (after.dpiStages.length !== before.dpiStages.length) {
		requests.push({ key: 'dpiStageCount', count: after.dpiStages.length });
	}
	after.dpiStages.forEach((stage, index) => {
		const previous = before.dpiStages[index];
		if (!previous || stage.dpi !== previous.dpi || !eq(stage.color, previous.color)) {
			requests.push({ key: 'dpiStage', index, dpi: stage.dpi, color: stage.color });
		}
	});
	if (!eq(after.lighting, before.lighting)) requests.push({ key: 'lighting', value: after.lighting });
	if (!eq(after.dpiIndicator.mode, before.dpiIndicator.mode)) {
		requests.push({ key: 'dpiIndicatorMode', value: after.dpiIndicator.mode });
	}
	if (after.dpiIndicator.brightness !== before.dpiIndicator.brightness) {
		requests.push({ key: 'dpiIndicatorBrightness', level: after.dpiIndicator.brightness });
	}
	if (after.dpiIndicator.speed !== before.dpiIndicator.speed) {
		requests.push({ key: 'dpiIndicatorSpeed', speed: after.dpiIndicator.speed });
	}
	if (after.dpiIndicator.on !== before.dpiIndicator.on) {
		requests.push({ key: 'dpiIndicatorOn', on: after.dpiIndicator.on });
	}
	// 'unsupported' is never something a toggle can set (see PerformanceScreen); this guard just
	// keeps a SetLongRangeMode from ever being sent for a device that reported it does not have
	// the feature.
	if (after.longRange !== before.longRange && after.longRange !== 'unsupported') {
		requests.push({ key: 'longRange', on: after.longRange === 'on' });
	}
	return requests;
}

/**
 * Label for an "apply changes" button given whether a real reading exists (`hasData`), whether
 * the draft differs from it (`dirty`), and whether a write is in flight (`saving`). Never returns
 * "Up to date" when there is no device to be up to date with: that reads as a claim about the
 * device's state, and with no device connected nothing is actually known (see
 * docs/decs/hyperpace_DECS.md, the "Up to date" honesty-fence fix). One function so the Performance
 * and Lighting screens' apply buttons cannot drift on this rule.
 */
export function applyButtonLabel(hasData: boolean, dirty: boolean, saving: boolean): string {
	if (saving) return 'Applying...';
	if (!hasData) return 'Apply changes';
	return dirty ? 'Apply changes' : 'Up to date';
}

class DeviceStore {
	devices = $state<DeviceDescriptor[]>([]);
	scanning = $state(false);

	connecting = $state(false);
	connected = $state(false);
	backend = $state<DeviceBackend | null>(null);
	access = $state<Access | null>(null);
	/** False while a connected device is asleep/unresponsive (DeviceEvent Offline). */
	online = $state(true);
	/** A cable is attached that the app cannot open; see `status.ts` for what is shown. */
	cableBlocked = $state(false);
	identity = $state<DeviceIdentity | null>(null);
	battery = $state<Battery | null>(null);

	settings = $state<Settings | null>(null);
	settingsLoading = $state(false);
	/** Another settings read is owed once the one in flight finishes; see `refreshSettings`. */
	private settingsRefreshQueued = false;

	/** The receiver's own light as `read_receiver_light` last reported it: null before the first
	 * read completes, never a guessed default (see `readReceiverLight`). */
	receiverLight = $state<ReceiverLightState | null>(null);
	receiverLightLoading = $state(false);
	/** Another receiver-light read is owed once the one in flight finishes; see `readReceiverLight`. */
	private receiverLightRefreshQueued = false;

	macros = $state<MacroRecord[]>([]);
	macrosLoading = $state(false);

	firmware = $state<FirmwareRecord[]>([]);
	firmwareLoading = $state(false);
	/** Result of the most recent `checkForFirmwareUpdates`; archived packages newer than what the
	 * connected device currently reports, not the whole archive. */
	firmwareUpdates = $state<FirmwareRecord[]>([]);
	firmwareProgress = $state<FirmwareProgress | null>(null);
	/** Every `GetPairState` update streamed while the most recent `pairReceiver` call runs, live
	 * (phase and seconds remaining), not just its final result. Null before pairing has ever run. */
	pairState = $state<PairState | null>(null);
	/** Result of the most recent `checkFirmwarePublication` (route 1, "watch for publication").
	 * Never implies a package was found; see its own `hasFindings`/`disclaimer` fields. */
	firmwareWatchReport = $state<FirmwareWatchReport | null>(null);

	appSettings = $state<AppSettings | null>(null);

	/** The most recent command failure, verbatim from the backend. Never a guess. */
	lastError = $state<string | null>(null);

	private channel: ReturnType<typeof openDeviceEventChannel> | null = null;
	private initialized = false;

	/** What `lastError` was reporting on, so a later success of the same operation can clear it. */
	private lastErrorContext: string | null = null;

	private fail(context: string, err: unknown): never {
		const message = err instanceof Error ? err.message : String(err);
		this.lastError = `${context}: ${message}`;
		this.lastErrorContext = context;
		throw err;
	}

	/**
	 * Clear `lastError` if it reports a failure of `context`. Called when that same operation
	 * later succeeds, so a failure that has since been recovered from stops being shown as though
	 * it still applied. A failure of anything else stays up.
	 */
	private recovered(context: string) {
		if (this.lastErrorContext !== context) return;
		this.lastError = null;
		this.lastErrorContext = null;
	}

	clearError() {
		this.lastError = null;
	}

	private applyDeviceState(state: DeviceState) {
		this.connected = state.connected;
		this.backend = state.backend;
		this.access = state.access;
		this.identity = state.identity;
		this.battery = state.battery;
		this.online = state.online;
		this.cableBlocked = state.cableBlocked;
	}

	/**
	 * Opens this window's device event channel and registers it with `device_state`, the
	 * contract's subscribe path (`connect` and `disconnect` carry no channel). Idempotent and
	 * safe to call from more than one screen's `onMount`; only the first call does anything.
	 */
	async init() {
		if (this.initialized || !runningInTauri) return;
		this.initialized = true;
		this.channel = openDeviceEventChannel((event) => this.handleEvent(event));
		try {
			const state = await invoke<DeviceState>('device_state', { channel: this.channel });
			this.applyDeviceState(state);
			// The app connects to the mouse itself at startup, so a window opening afterwards can
			// find a device already connected with nobody having called `connect` here. The identity
			// is what says the connection is ready: it is only reported once the device has answered
			// and its settings are readable. Before that, reading would fail with "no device is
			// connected", and the `connected` event this window is now subscribed to will do the read.
			if (state.connected && state.identity) {
				this.refreshSettingsInBackground();
				this.readReceiverLightInBackground();
			}
		} catch (err) {
			this.fail('Could not read device state', err);
		}
	}

	async refreshDevices() {
		this.scanning = true;
		try {
			this.devices = await invoke<DeviceDescriptor[]>('list_devices');
		} catch (err) {
			this.fail('Could not list devices', err);
		} finally {
			this.scanning = false;
		}
	}

	/** Connect to the simulator, or the operator's real device when `realDevice` is set. Access
	 * defaults to read-only, matching the Rust side's own default (`AccessDto::default`), so a
	 * caller that does not ask for write access can never get it by omission. */
	async connect(realDevice: boolean, access: Access = 'readOnly') {
		this.connecting = true;
		try {
			await this.init();
			const request: ConnectRequest = { realDevice, access };
			const state = await invoke<DeviceState>('connect', { request });
			this.applyDeviceState(state);
			if (state.connected) {
				await this.refreshSettings();
				this.readReceiverLightInBackground();
			}
		} catch (err) {
			this.fail('Could not connect', err);
		} finally {
			this.connecting = false;
		}
	}

	async disconnect() {
		try {
			const state = await invoke<DeviceState>('disconnect');
			this.applyDeviceState(state);
		} catch (err) {
			this.fail('Could not disconnect cleanly', err);
		} finally {
			this.settings = null;
			this.receiverLight = null;
		}
	}

	/**
	 * Re-reads the whole settings shadow. Concurrency-safe: a call made while a read is already in
	 * flight does not start a second one, it marks that another read is owed, so a push that
	 * arrives mid-read is never lost and never turns into a burst of reads.
	 */
	async refreshSettings() {
		if (this.settingsLoading) {
			this.settingsRefreshQueued = true;
			return;
		}
		this.settingsLoading = true;
		try {
			this.settings = await invoke<Settings>('read_settings');
			this.recovered('Could not read settings');
		} catch (err) {
			this.fail('Could not read settings', err);
		} finally {
			this.settingsLoading = false;
			if (this.settingsRefreshQueued) {
				this.settingsRefreshQueued = false;
				this.refreshSettingsInBackground();
			}
		}
	}

	/**
	 * Starts a settings read that nothing is waiting on, for the paths that learn the device
	 * became readable from an event rather than from their own call. `refreshSettings` reports its
	 * own failure through `lastError` before it rethrows, so the rethrow has nowhere useful to go
	 * here and is dropped rather than left as an unhandled rejection.
	 */
	private refreshSettingsInBackground() {
		void this.refreshSettings().catch(() => {});
	}

	/**
	 * Re-reads the receiver's own light as `read_receiver_light` currently reports it. The
	 * receiver answers this itself even while the mouse sleeps. Concurrency-safe like
	 * `refreshSettings`: a call made while a read is already in flight does not start a second
	 * one, it marks that another read is owed, so a push that arrives mid-read is never lost and
	 * never turns into a burst of reads.
	 */
	async readReceiverLight() {
		if (this.receiverLightLoading) {
			this.receiverLightRefreshQueued = true;
			return;
		}
		this.receiverLightLoading = true;
		try {
			this.receiverLight = await invoke<ReceiverLightState>('read_receiver_light');
			this.recovered('Could not read receiver light');
		} catch (err) {
			this.fail('Could not read receiver light', err);
		} finally {
			this.receiverLightLoading = false;
			if (this.receiverLightRefreshQueued) {
				this.receiverLightRefreshQueued = false;
				this.readReceiverLightInBackground();
			}
		}
	}

	/** Same rationale as `refreshSettingsInBackground`: a receiver-light read triggered by a
	 * connection becoming ready rather than by a caller waiting on its result. */
	private readReceiverLightInBackground() {
		void this.readReceiverLight().catch(() => {});
	}

	/** Sends one `write_setting` command per request, then reconciles from a fresh read. */
	async writeSettings(requests: WriteSettingRequest[]) {
		if (requests.length === 0) return;
		try {
			for (const request of requests) {
				await invoke('write_setting', { request });
			}
			await this.refreshSettings();
		} catch (err) {
			this.fail('Could not write settings', err);
		}
	}

	/**
	 * The instant-apply write path: writes one setting, or a small group of settings that only
	 * make sense written together (growing the DPI stage count and filling the new stage's value
	 * in the same call), and reconciles from a fresh read, all through `writeSettings` above so
	 * there is exactly one place that turns a request into a device write (see the "one function"
	 * rule). A screen calls this the moment an operator commits a value, the same way the vendor's
	 * own configurator does, instead of collecting edits behind an "Apply" button.
	 *
	 * Unlike `writeSettings`, a failure is returned as a message rather than thrown: the caller
	 * already shows its control at the new value optimistically, so on failure it puts that one
	 * control back to `device.settings`'s last known value (unchanged here, since the write never
	 * landed) and shows this message beside just that control, never a screen-wide banner
	 * (design.md "Microinteractions stance"). A success returns null and changes nothing else,
	 * since a landed write is silent.
	 */
	async commitSetting(request: WriteSettingRequest | WriteSettingRequest[]): Promise<string | null> {
		try {
			await this.writeSettings(Array.isArray(request) ? request : [request]);
			return null;
		} catch (err) {
			return err instanceof Error ? err.message : String(err);
		}
	}

	async setButton(index: number, action: ButtonAction, keystroke?: Keystroke) {
		try {
			const request: SetButtonRequest = keystroke ? { index, action, keystroke } : { index, action };
			await invoke('set_button', { request });
			await this.refreshSettings();
		} catch (err) {
			this.fail('Could not set button', err);
		}
	}

	async setProfile(index: number) {
		try {
			await invoke('set_profile', { index });
			await this.refreshSettings();
		} catch (err) {
			this.fail('Could not switch profile', err);
		}
	}

	async factoryReset() {
		try {
			await invoke('factory_reset');
			await this.refreshSettings();
		} catch (err) {
			this.fail('Could not factory reset', err);
		}
	}

	/** Starts pairing and streams every `GetPairState` poll into `pairState` as it happens
	 * (`docs/research/mouse-protocol-v2.md` section 10.1), so the caller can render the real
	 * phase and seconds remaining instead of a bare "starting" spinner. */
	async pairReceiver(): Promise<PairState | undefined> {
		this.pairState = null;
		try {
			const channel = openPairStateChannel((state) => {
				this.pairState = state;
			});
			const final = await invoke<PairState>('pair_receiver', { channel });
			this.pairState = final;
			return final;
		} catch (err) {
			this.fail('Could not start pairing', err);
		}
	}

	/** Reads the keystroke or media chord currently bound to button `index`'s slot, regardless of
	 * that button's current action type (the slot exists at a fixed address either way). */
	async getButtonKeystroke(index: number): Promise<Keystroke | undefined> {
		try {
			return await invoke<Keystroke>('get_button_keystroke', { index });
		} catch (err) {
			this.fail('Could not read the keystroke slot', err);
		}
	}

	async setReceiverLight(light: ReceiverLight) {
		try {
			await invoke('receiver_light', { light });
			await this.readReceiverLight();
		} catch (err) {
			this.fail('Could not set receiver light', err);
		}
	}

	async exportConfig(): Promise<Uint8Array> {
		try {
			const bytes = await invoke<number[]>('export_config');
			return Uint8Array.from(bytes);
		} catch (err) {
			this.fail('Could not export configuration', err);
		}
	}

	async importConfig(bytes: Uint8Array) {
		try {
			await invoke('import_config', { bytes: Array.from(bytes) });
			await this.refreshSettings();
		} catch (err) {
			this.fail('Could not import configuration', err);
		}
	}

	async refreshMacros() {
		this.macrosLoading = true;
		try {
			this.macros = await invoke<MacroRecord[]>('list_macros');
		} catch (err) {
			this.fail('Could not list macros', err);
		} finally {
			this.macrosLoading = false;
		}
	}

	async saveMacro(request: SaveMacroRequest): Promise<MacroRecord | undefined> {
		try {
			const saved = await invoke<MacroRecord>('save_macro', { request });
			await this.refreshMacros();
			return saved;
		} catch (err) {
			this.fail('Could not save macro', err);
		}
	}

	async deleteMacro(id: string) {
		try {
			await invoke('delete_macro', { id });
			await this.refreshMacros();
		} catch (err) {
			this.fail('Could not delete macro', err);
		}
	}

	async refreshFirmware() {
		this.firmwareLoading = true;
		try {
			this.firmware = await invoke<FirmwareRecord[]>('firmware_list');
		} catch (err) {
			this.fail('Could not list firmware', err);
		} finally {
			this.firmwareLoading = false;
		}
	}

	async importFirmware(bytes: Uint8Array) {
		try {
			await invoke('firmware_import', { bytes: Array.from(bytes) });
			await this.refreshFirmware();
		} catch (err) {
			this.fail('Could not import firmware package', err);
		}
	}

	async installFirmware(id: string) {
		this.firmwareProgress = null;
		try {
			const channel = openFirmwareProgressChannel((progress) => {
				this.firmwareProgress = progress;
			});
			await invoke('firmware_install', { id, channel });
		} catch (err) {
			this.fail('Could not install firmware', err);
		}
	}

	async checkForFirmwareUpdates() {
		try {
			this.firmwareUpdates = await invoke<FirmwareRecord[]>('firmware_check_for_updates');
		} catch (err) {
			this.fail('Could not check for firmware updates', err);
		}
	}

	/** Route 1, "watch for publication": fetches the vendor's own config files and firmware
	 * directory paths and reports what changed. Never downloads or installs anything, and a
	 * result never implies firmware was found; render `summary`/`disclaimer` as returned. */
	async checkFirmwarePublication() {
		try {
			this.firmwareWatchReport = await invoke<FirmwareWatchReport>('firmware_watch_check');
		} catch (err) {
			this.fail('Could not check for firmware publication', err);
		}
	}

	async refreshAppSettings() {
		try {
			const request: AppSettingsRequest = { action: 'get' };
			const response = await invoke<AppSettingsResponse>('app_settings', { request });
			this.appSettings = extractAppSettings(response.settings);
		} catch (err) {
			this.fail('Could not read app settings', err);
		}
	}

	/** Writes every field present in `patch`, one `app_settings` call per key (no batch variant). */
	async writeAppSettings(patch: Partial<AppSettings>) {
		try {
			let response: AppSettingsResponse | null = null;
			for (const [field, storeKey] of APP_SETTING_KEYS) {
				if (!(field in patch)) continue;
				const request: AppSettingsRequest = { action: 'set', key: storeKey, value: patch[field] };
				response = await invoke<AppSettingsResponse>('app_settings', { request });
			}
			if (response) this.appSettings = extractAppSettings(response.settings);
		} catch (err) {
			this.fail('Could not save app settings', err);
		}
	}

	private handleEvent(event: DeviceEvent) {
		switch (event.type) {
			case 'disconnected':
				this.connected = false;
				this.online = true;
				this.identity = null;
				this.battery = null;
				this.backend = null;
				this.access = null;
				return;
			case 'offline':
				this.online = false;
				return;
			case 'connected':
				this.identity = event.identity;
				this.connected = true;
				this.online = true;
				// This is the only signal a connection the app made on its own (at startup, or
				// when the receiver was plugged back in) produces, so it is where those
				// connections learn their settings (and the receiver's own light) are now readable.
				this.refreshSettingsInBackground();
				this.readReceiverLightInBackground();
				return;
			case 'battery':
				this.battery = event.battery;
				return;
			case 'changed': {
				// The mouse changed a setting itself (a DPI button press, a profile switch): re-read
				// rather than guess which fields moved. The device layer re-reads its own copy before
				// answering, so this shows what the mouse holds now. A battery-only push changes no
				// setting and is left to the battery poll.
				const { dpi, polling, profile, dpiIndicator, lighting } = event.changed;
				if (dpi || polling || profile || dpiIndicator || lighting) {
					this.refreshSettingsInBackground();
				}
				return;
			}
		}
	}
}

export const device = new DeviceStore();
export const isTauriShell = runningInTauri;
