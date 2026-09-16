// The Data Management screen's own store, patterned on device.svelte.ts (Svelte 5 runes, the
// same lastError/fail/recovered convention). Nothing here invents a value: every field starts
// null/empty and is filled only from a real command response.
//
// Macro and firmware *listing* stays on the shared `device` store (device.macros,
// device.firmware): this screen reuses that state and its existing refresh/delete methods rather
// than holding a second copy that could drift, and only adds what device.svelte.ts does not
// already own (firmware archive deletion, profile snapshots, the event log, app-setting reset,
// and the whole-store backup).

import { invoke } from './tauri';
import { device } from './device.svelte';
import type { AppSettingsResponse } from './types';
import type {
	DataOverview,
	EventLogEntry,
	EventLogPage,
	ImportStoreResult,
	ProfileSnapshotRecord
} from './data-types';
import { EVENT_LOG_PAGE_SIZE } from './data-types';

class DataStore {
	overview = $state<DataOverview | null>(null);
	overviewLoading = $state(false);

	events = $state<EventLogEntry[]>([]);
	eventsTotal = $state(0);
	eventsOffset = $state(0);
	eventsLoading = $state(false);
	clearingEvents = $state(false);

	profileSnapshots = $state<ProfileSnapshotRecord[]>([]);
	profileSnapshotsLoading = $state(false);
	savingSnapshot = $state(false);
	restoringSnapshotId = $state<string | null>(null);

	/** The raw app-settings map, every stored key as-is (not the curated AppSettings shape
	 * device.svelte.ts exposes), so this screen can list and reset a key it otherwise knows
	 * nothing about (the GPU workaround key, for one). */
	appSettings = $state<Record<string, unknown>>({});
	appSettingsLoading = $state(false);

	exportingStore = $state(false);
	importingStore = $state(false);
	lastImportResult = $state<ImportStoreResult | null>(null);

	/** The most recent command failure, verbatim from the backend. Never a guess. */
	lastError = $state<string | null>(null);
	/** What `lastError` was reporting on, so a later success of the same operation can clear it. */
	private lastErrorContext: string | null = null;

	private fail(context: string, err: unknown): never {
		const message = err instanceof Error ? err.message : String(err);
		this.lastError = `${context}: ${message}`;
		this.lastErrorContext = context;
		throw err;
	}

	private recovered(context: string) {
		if (this.lastErrorContext !== context) return;
		this.lastError = null;
		this.lastErrorContext = null;
	}

	clearError() {
		this.lastError = null;
	}

	async refreshOverview() {
		this.overviewLoading = true;
		try {
			this.overview = await invoke<DataOverview>('data_overview');
			this.recovered('Could not read the data overview');
		} catch (err) {
			this.fail('Could not read the data overview', err);
		} finally {
			this.overviewLoading = false;
		}
	}

	async loadEventsPage(offset: number) {
		this.eventsLoading = true;
		try {
			const page = await invoke<EventLogPage>('list_events', {
				offset,
				limit: EVENT_LOG_PAGE_SIZE
			});
			this.events = page.events;
			this.eventsTotal = page.total;
			this.eventsOffset = offset;
			this.recovered('Could not read the event log');
		} catch (err) {
			this.fail('Could not read the event log', err);
		} finally {
			this.eventsLoading = false;
		}
	}

	async nextEventsPage() {
		const next = this.eventsOffset + EVENT_LOG_PAGE_SIZE;
		if (next >= this.eventsTotal) return;
		await this.loadEventsPage(next);
	}

	async previousEventsPage() {
		const previous = Math.max(0, this.eventsOffset - EVENT_LOG_PAGE_SIZE);
		await this.loadEventsPage(previous);
	}

	async clearEvents() {
		this.clearingEvents = true;
		try {
			await invoke('clear_events');
			await this.loadEventsPage(0);
			await this.refreshOverview();
		} catch (err) {
			this.fail('Could not clear the event log', err);
		} finally {
			this.clearingEvents = false;
		}
	}

	async refreshProfileSnapshots() {
		this.profileSnapshotsLoading = true;
		try {
			this.profileSnapshots = await invoke<ProfileSnapshotRecord[]>('list_profile_snapshots');
			this.recovered('Could not read profile snapshots');
		} catch (err) {
			this.fail('Could not read profile snapshots', err);
		} finally {
			this.profileSnapshotsLoading = false;
		}
	}

	/** Captures the connected mouse's current settings as a new named snapshot; the caller (the
	 * Data screen) is responsible for disabling this when no device is connected, the same way
	 * SettingsScreen's export button does, since a doomed call is better refused up front than
	 * surfaced only as a lastError after the fact. */
	async saveProfileSnapshot(name: string) {
		this.savingSnapshot = true;
		try {
			await invoke<ProfileSnapshotRecord>('save_profile_snapshot', { name });
			await this.refreshProfileSnapshots();
			await this.refreshOverview();
		} catch (err) {
			this.fail('Could not save a profile snapshot', err);
		} finally {
			this.savingSnapshot = false;
		}
	}

	async restoreProfileSnapshot(id: string) {
		this.restoringSnapshotId = id;
		try {
			await invoke('restore_profile_snapshot', { id });
			await device.refreshSettings();
		} catch (err) {
			this.fail('Could not restore the profile snapshot', err);
		} finally {
			this.restoringSnapshotId = null;
		}
	}

	async renameProfileSnapshot(id: string, name: string) {
		try {
			await invoke('rename_profile_snapshot', { id, name });
			await this.refreshProfileSnapshots();
		} catch (err) {
			this.fail('Could not rename the profile snapshot', err);
		}
	}

	async deleteProfileSnapshot(id: string) {
		try {
			await invoke('delete_profile_snapshot', { id });
			await this.refreshProfileSnapshots();
			await this.refreshOverview();
		} catch (err) {
			this.fail('Could not delete the profile snapshot', err);
		}
	}

	/** Deletes an archived firmware package. Firmware *listing* stays on `device.firmware`; this
	 * refreshes that shared list afterward so the Firmware screen never shows a package this
	 * screen just deleted. */
	async deleteFirmwareEntry(id: string) {
		try {
			await invoke('firmware_delete', { id });
			await device.refreshFirmware();
			await this.refreshOverview();
		} catch (err) {
			this.fail('Could not delete the firmware entry', err);
		}
	}

	async refreshAppSettings() {
		this.appSettingsLoading = true;
		try {
			const response = await invoke<AppSettingsResponse>('app_settings', {
				request: { action: 'get' }
			});
			this.appSettings = response.settings;
			this.recovered('Could not read app settings');
		} catch (err) {
			this.fail('Could not read app settings', err);
		} finally {
			this.appSettingsLoading = false;
		}
	}

	async resetAppSetting(key: string) {
		try {
			const response = await invoke<AppSettingsResponse>('reset_app_setting', { key });
			this.appSettings = response.settings;
			// autostart/lowBatteryThresholdPercent/firmwareWatchEnabled are also cached on the
			// shared device store (device.appSettings); re-read through its own command so a reset
			// here is reflected on the Settings screen too, instead of this store reaching into
			// device.svelte.ts's own state directly.
			await device.refreshAppSettings();
		} catch (err) {
			this.fail('Could not reset the app setting', err);
		}
	}

	async exportStore(): Promise<Uint8Array> {
		this.exportingStore = true;
		try {
			const bytes = await invoke<number[]>('export_store');
			return Uint8Array.from(bytes);
		} catch (err) {
			this.fail('Could not export the local store', err);
		} finally {
			this.exportingStore = false;
		}
	}

	async importStore(bytes: Uint8Array) {
		this.importingStore = true;
		try {
			this.lastImportResult = await invoke<ImportStoreResult>('import_store', {
				bytes: Array.from(bytes)
			});
			await Promise.all([
				this.refreshOverview(),
				this.loadEventsPage(0),
				this.refreshProfileSnapshots(),
				this.refreshAppSettings(),
				device.refreshMacros(),
				device.refreshFirmware()
			]);
		} catch (err) {
			this.fail('Could not import the backup', err);
		} finally {
			this.importingStore = false;
		}
	}
}

export const data = new DataStore();
