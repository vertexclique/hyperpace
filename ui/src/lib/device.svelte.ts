// Central device/app state, built with Svelte 5 runes. Holds exactly one
// shadow of the connected device's settings, per the scale note in
// docs/plans/hyperpace.md 5 ("Hyperpace holds exactly one shadow per
// connected device and never accumulates history in memory"). Nothing here
// invents a value: every field starts null/empty and is filled only from a
// real command response.

import { invoke, openDeviceEventChannel, runningInTauri } from './tauri';
import type {
	AppSettings,
	Battery,
	ButtonAction,
	DeviceEvent,
	DeviceIdentity,
	DeviceSummary,
	FirmwareRecord,
	MacroSlot,
	ReceiverLight,
	Settings
} from './types';

class DeviceStore {
	devices = $state<DeviceSummary[]>([]);
	scanning = $state(false);

	selectedDeviceId = $state<string | null>(null);
	connecting = $state(false);
	connected = $state(false);
	/** False while a connected device is asleep/unresponsive (DeviceEvent::Offline). */
	online = $state(true);
	identity = $state<DeviceIdentity | null>(null);
	battery = $state<Battery | null>(null);

	settings = $state<Settings | null>(null);
	settingsLoading = $state(false);

	macros = $state<MacroSlot[]>([]);
	macrosLoading = $state(false);

	firmware = $state<FirmwareRecord[]>([]);
	firmwareLoading = $state(false);

	appSettings = $state<AppSettings | null>(null);

	/** The most recent command failure, verbatim from the backend. Never a guess. */
	lastError = $state<string | null>(null);

	private channel: ReturnType<typeof openDeviceEventChannel> | null = null;

	private fail(context: string, err: unknown): never {
		const message = err instanceof Error ? err.message : String(err);
		this.lastError = `${context}: ${message}`;
		throw err;
	}

	clearError() {
		this.lastError = null;
	}

	async refreshDevices() {
		this.scanning = true;
		try {
			this.devices = await invoke<DeviceSummary[]>('list_devices');
		} catch (err) {
			this.fail('Could not list devices', err);
		} finally {
			this.scanning = false;
		}
	}

	async connect(deviceId: string) {
		this.connecting = true;
		try {
			if (!this.channel) {
				this.channel = openDeviceEventChannel((event) => this.handleEvent(event));
			}
			await invoke('connect', { deviceId, events: this.channel });
			this.selectedDeviceId = deviceId;
			await this.refreshDeviceState();
			await this.refreshSettings();
			this.connected = true;
		} catch (err) {
			this.connected = false;
			this.fail('Could not connect', err);
		} finally {
			this.connecting = false;
		}
	}

	async disconnect() {
		try {
			await invoke('disconnect');
		} catch (err) {
			this.fail('Could not disconnect cleanly', err);
		} finally {
			this.connected = false;
			this.identity = null;
			this.battery = null;
			this.settings = null;
			this.selectedDeviceId = null;
		}
	}

	async refreshDeviceState() {
		try {
			const state = await invoke<{
				identity: DeviceIdentity | null;
				battery: Battery | null;
				connected: boolean;
			}>('device_state');
			this.identity = state.identity;
			this.battery = state.battery;
			this.connected = state.connected;
		} catch (err) {
			this.fail('Could not read device state', err);
		}
	}

	async refreshSettings() {
		this.settingsLoading = true;
		try {
			this.settings = await invoke<Settings>('read_settings');
		} catch (err) {
			this.fail('Could not read settings', err);
		} finally {
			this.settingsLoading = false;
		}
	}

	/** Writes a partial settings patch, then reconciles from a fresh read. */
	async writeSetting(patch: Partial<Settings>) {
		try {
			await invoke('write_setting', { patch });
			await this.refreshSettings();
		} catch (err) {
			this.fail('Could not write setting', err);
		}
	}

	async setButton(index: number, action: ButtonAction) {
		try {
			await invoke('set_button', { index, action });
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

	async pairReceiver() {
		try {
			await invoke('pair_receiver');
		} catch (err) {
			this.fail('Could not start pairing', err);
		}
	}

	async setReceiverLight(light: ReceiverLight) {
		try {
			await invoke('receiver_light', { light });
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
			this.macros = await invoke<MacroSlot[]>('list_macros');
		} catch (err) {
			this.fail('Could not list macros', err);
		} finally {
			this.macrosLoading = false;
		}
	}

	async saveMacro(macro: MacroSlot) {
		try {
			await invoke('save_macro', { macro });
			await this.refreshMacros();
		} catch (err) {
			this.fail('Could not save macro', err);
		}
	}

	async deleteMacro(slot: number) {
		try {
			await invoke('delete_macro', { slot });
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
		try {
			await invoke('firmware_install', { id });
		} catch (err) {
			this.fail('Could not install firmware', err);
		}
	}

	async checkForFirmwareUpdates() {
		try {
			await invoke('firmware_check_for_updates');
			await this.refreshFirmware();
		} catch (err) {
			this.fail('Could not check for firmware updates', err);
		}
	}

	async refreshAppSettings() {
		try {
			this.appSettings = await invoke<AppSettings>('app_settings');
		} catch (err) {
			this.fail('Could not read app settings', err);
		}
	}

	async writeAppSettings(patch: Partial<AppSettings>) {
		try {
			await invoke('app_settings', { patch });
			await this.refreshAppSettings();
		} catch (err) {
			this.fail('Could not save app settings', err);
		}
	}

	private handleEvent(event: DeviceEvent) {
		if (event === 'Disconnected') {
			this.connected = false;
			this.online = true;
			this.identity = null;
			this.battery = null;
			return;
		}
		if (event === 'Offline') {
			this.online = false;
			return;
		}
		if ('Connected' in event) {
			this.identity = event.Connected;
			this.connected = true;
			this.online = true;
		} else if ('Battery' in event) {
			this.battery = event.Battery;
		} else if ('Changed' in event) {
			// A settings-affecting push arrived; re-read the shadow rather than
			// guess which fields moved.
			void this.refreshSettings();
		}
	}
}

export const device = new DeviceStore();
export const isTauriShell = runningInTauri;
