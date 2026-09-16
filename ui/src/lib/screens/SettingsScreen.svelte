<script lang="ts">
	import { onMount } from 'svelte';
	import { getName, getTauriVersion, getVersion } from '@tauri-apps/api/app';
	import { device, isTauriShell } from '../device.svelte';
	import { invoke } from '../tauri';
	import { DEFAULT_LOW_BATTERY_THRESHOLD_PERCENT } from '../types';
	import type { AppSettings, AppSettingsResponse, DeviceBackend, LinkType } from '../types';
	import EmptyState from '../components/EmptyState.svelte';
	import Toggle from '../components/Toggle.svelte';
	import RangeField from '../components/RangeField.svelte';
	import SelectField from '../components/SelectField.svelte';

	// The app's own fallback when a preference has never been written (matches
	// `extractAppSettings` in device.svelte.ts and the Rust side's own defaults), not a device
	// reading: these preferences exist independent of any device, so there is nothing dishonest
	// about showing them immediately instead of behind a "Loading..." placeholder while the
	// near-instant local-store round trip completes.
	const DEFAULT_APP_SETTINGS: AppSettings = {
		autostart: false,
		lowBatteryThresholdPercent: DEFAULT_LOW_BATTERY_THRESHOLD_PERCENT,
		minimizeToTray: false,
		firmwareWatchEnabled: true
	};

	// Mirrors crates/hyperpace-app/src/gpu_workaround.rs::APP_SETTING_KEY (the WebKitGTK DMABUF
	// renderer workaround for NVIDIA/Linux, `docs/decs/hyperpace_DECS.md` #43): read and written
	// directly through the generic `app_settings` command here rather than folded into this
	// screen's AppSettings/APP_SETTING_KEYS draft-and-save flow (device.svelte.ts, types.ts),
	// since wiring it into that shared convention needs edits to both of those files, outside
	// this restyle pass's file list. `gpu_workaround::resolve` only ever reads this at the next
	// launch (before the window exists), so a change here cannot take effect live.
	type GpuWorkaroundValue = 'auto' | 'on' | 'off';
	const GPU_WORKAROUND_KEY = 'gpu_dmabuf_workaround';
	const GPU_WORKAROUND_OPTIONS: { value: GpuWorkaroundValue; label: string }[] = [
		{ value: 'auto', label: 'Auto: detect at each launch' },
		{ value: 'on', label: 'On: always disable it' },
		{ value: 'off', label: 'Off: never disable it' }
	];

	function parseGpuWorkaroundValue(raw: unknown): GpuWorkaroundValue {
		return raw === 'on' || raw === 'off' ? raw : 'auto';
	}

	let connectingBackend = $state<DeviceBackend | null>(null);
	let appDraft = $state<AppSettings>({ ...DEFAULT_APP_SETTINGS });
	let appDirty = $state(false);
	let appSaving = $state(false);
	let profileIndex = $state(0);
	let profileSaving = $state(false);
	let exporting = $state(false);
	let importing = $state(false);
	let configFileInput = $state<HTMLInputElement | null>(null);
	let hasData = $derived(device.settings !== null);

	let gpuWorkaround = $state<GpuWorkaroundValue>('auto');
	let gpuWorkaroundSaving = $state(false);
	let gpuWorkaroundError = $state<string | null>(null);

	// Read live from the running desktop binary (@tauri-apps/api/app), never hardcoded, so this
	// never drifts from what is actually installed. Left null (rendered as "-") on any failure or
	// outside the desktop shell, rather than a guessed version string.
	let appName = $state<string | null>(null);
	let appVersion = $state<string | null>(null);
	let tauriVersion = $state<string | null>(null);

	onMount(() => {
		void device.refreshDevices();
		void device.refreshAppSettings();
		void loadGpuWorkaround();
		void loadAppInfo();
	});

	$effect(() => {
		appDraft = device.appSettings ? { ...device.appSettings } : { ...DEFAULT_APP_SETTINGS };
		appDirty = false;
	});

	$effect(() => {
		const profile = device.settings?.profile;
		if (profile && profile.state === 'active') profileIndex = profile.index;
	});

	// Real-device access always defaults to read-only (`device.connect`'s own default), matching
	// the safety property: choosing "This device" is the explicit opt-in the contract requires,
	// but it never implies write access.
	async function connectTo(backend: DeviceBackend) {
		connectingBackend = backend;
		try {
			await device.connect(backend === 'realDevice');
		} finally {
			connectingBackend = null;
		}
	}

	function formatLinkType(link: LinkType): string {
		return link.kind === 'unknown' ? `unknown (0x${link.byte.toString(16)})` : link.kind;
	}

	async function saveAppSettings() {
		if (!appDraft) return;
		appSaving = true;
		try {
			await device.writeAppSettings(appDraft);
			appDirty = false;
		} finally {
			appSaving = false;
		}
	}

	async function switchProfile() {
		profileSaving = true;
		try {
			await device.setProfile(profileIndex);
		} finally {
			profileSaving = false;
		}
	}

	async function exportConfig() {
		exporting = true;
		try {
			const bytes = await device.exportConfig();
			const blob = new Blob([new Uint8Array(bytes)], { type: 'application/octet-stream' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = 'hyperpace-config.bin';
			a.click();
			URL.revokeObjectURL(url);
		} finally {
			exporting = false;
		}
	}

	async function onImportChosen(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		importing = true;
		try {
			const bytes = new Uint8Array(await file.arrayBuffer());
			await device.importConfig(bytes);
		} finally {
			importing = false;
			input.value = '';
		}
	}

	async function loadGpuWorkaround() {
		if (!isTauriShell) return;
		try {
			const response = await invoke<AppSettingsResponse>('app_settings', {
				request: { action: 'get' }
			});
			gpuWorkaround = parseGpuWorkaroundValue(response.settings[GPU_WORKAROUND_KEY]);
		} catch (err) {
			gpuWorkaroundError = err instanceof Error ? err.message : String(err);
		}
	}

	async function setGpuWorkaround(value: GpuWorkaroundValue) {
		gpuWorkaroundSaving = true;
		gpuWorkaroundError = null;
		try {
			const response = await invoke<AppSettingsResponse>('app_settings', {
				request: { action: 'set', key: GPU_WORKAROUND_KEY, value }
			});
			gpuWorkaround = parseGpuWorkaroundValue(response.settings[GPU_WORKAROUND_KEY]);
		} catch (err) {
			gpuWorkaroundError = err instanceof Error ? err.message : String(err);
		} finally {
			gpuWorkaroundSaving = false;
		}
	}

	async function loadAppInfo() {
		if (!isTauriShell) return;
		try {
			const [name, version, tauri] = await Promise.all([getName(), getVersion(), getTauriVersion()]);
			appName = name;
			appVersion = version;
			tauriVersion = tauri;
		} catch {
			// Diagnostic-only read; a failure here just leaves the About panel's fields blank,
			// never a fabricated version string.
		}
	}
</script>

<div class="page-pad grid settings-grid">
	<section class="plate">
		<div class="plate-head">
			<h2 class="plate-title">Device</h2>
		</div>
		<p class="plate-subtitle">Connect to a Hyperpace mouse to read and edit its settings.</p>

		{#if !isTauriShell}
			<EmptyState
				title="Preview mode"
				message="This build is not running inside the Hyperpace desktop app, so no device commands are available."
			/>
		{:else if device.connected}
			<div class="field-row">
				<div>
					<div class="field-label">Connected</div>
					<div class="field-hint">
						{#if device.identity}
							cid <span class="mono">{device.identity.cid}</span>, mid
							<span class="mono">{device.identity.mid}</span>, link {formatLinkType(
								device.identity.link
							)}
						{:else}
							Identity not read yet.
						{/if}
					</div>
				</div>
				<button class="btn btn-danger" onclick={() => device.disconnect()}>Disconnect</button>
			</div>
		{:else}
			<div class="field-row">
				<span class="field-hint">{device.devices.length} connection option(s).</span>
				<button class="btn" disabled={device.scanning} onclick={() => device.refreshDevices()}>
					{device.scanning ? 'Scanning...' : 'Rescan'}
				</button>
			</div>
			{#if device.devices.length === 0}
				<p class="field-hint device-none">No devices found.</p>
			{:else}
				<ul class="device-list">
					{#each device.devices as d (d.backend)}
						<li class="device-row">
							<span>{d.label}: {d.description}</span>
							<button
								class="btn btn-primary"
								disabled={connectingBackend !== null}
								onclick={() => connectTo(d.backend)}
							>
								{connectingBackend === d.backend ? 'Connecting...' : 'Connect'}
							</button>
						</li>
					{/each}
				</ul>
			{/if}
		{/if}
		{#if device.lastError}
			<p class="field-hint error-text">{device.lastError}</p>
		{/if}
	</section>

	<section class="plate">
		<div class="plate-head">
			<h2 class="plate-title">Profile</h2>
		</div>
		<p class="plate-subtitle">
			Switch the device's active on-board profile.
			{#if !device.connected}
				<span class="caption-note">No mouse connected.</span>
			{:else if device.settings?.profile.state === 'unsupported'}
				<span class="caption-note">Not supported on this device.</span>
			{/if}
		</p>
		<div class="field-row">
			<input
				class="text-input mono"
				type="number"
				min="0"
				max="7"
				style="width:80px"
				disabled={!hasData || device.settings?.profile.state === 'unsupported'}
				value={hasData ? profileIndex : ''}
				oninput={(e) => (profileIndex = Number((e.target as HTMLInputElement).value))}
			/>
			<button
				class="btn btn-primary"
				disabled={!hasData || device.settings?.profile.state === 'unsupported' || profileSaving}
				onclick={switchProfile}
			>
				{profileSaving ? 'Switching...' : 'Switch profile'}
			</button>
		</div>
		<p class="field-hint">
			{#if hasData && device.settings?.profile.state === 'active'}
				Currently profile <span class="mono">{device.settings.profile.index}</span>.
			{:else}
				Currently: -
			{/if}
		</p>
	</section>

	<section class="plate">
		<div class="plate-head">
			<h2 class="plate-title">Configuration backup</h2>
		</div>
		<p class="plate-subtitle">Export or import the full settings shadow as a .bin file.</p>
		<div class="field-row">
			<span class="field-label">Export current configuration</span>
			<button class="btn" disabled={!device.connected || exporting} onclick={exportConfig}>
				{exporting ? 'Exporting...' : 'Export'}
			</button>
		</div>
		<div class="field-row backup-row">
			<span class="field-label">Import a configuration file</span>
			<button
				class="btn"
				disabled={!device.connected || importing}
				onclick={() => configFileInput?.click()}
			>
				{importing ? 'Importing...' : 'Choose file'}
			</button>
			<input
				bind:this={configFileInput}
				type="file"
				accept=".bin"
				class="hidden-input"
				onchange={onImportChosen}
			/>
		</div>
	</section>

	<section class="plate">
		<div class="plate-head">
			<h2 class="plate-title">About and diagnostics</h2>
		</div>
		{#if !isTauriShell}
			<p class="field-hint">
				Version and diagnostics are read from the desktop app itself; not available in preview
				mode.
			</p>
		{:else}
			<div class="about-list">
				<div class="about-row">
					<span class="field-hint">App</span>
					<span class="mono">{appName ?? '-'}</span>
				</div>
				<div class="about-row">
					<span class="field-hint">Version</span>
					<span class="mono">{appVersion ?? '-'}</span>
				</div>
				<div class="about-row">
					<span class="field-hint">Tauri runtime</span>
					<span class="mono">{tauriVersion ?? '-'}</span>
				</div>
				<div class="about-row">
					<span class="field-hint">Data directory</span>
					<span class="mono">$HOME/.hyperpace</span>
				</div>
			</div>
		{/if}
	</section>

	<section class="plate wide">
		<div class="plate-head">
			<h2 class="plate-title">App preferences</h2>
		</div>
		<p class="plate-subtitle">
			Behaviour of the Hyperpace app itself, independent of any device.
			{#if !isTauriShell}
				<span class="caption-note">Not available in preview mode.</span>
			{/if}
		</p>
		<div class="pref-grid">
			<Toggle
				label="Launch at login"
				checked={appDraft.autostart}
				disabled={!isTauriShell}
				onchange={(v) => {
					appDraft.autostart = v;
					appDirty = true;
				}}
			/>
			<Toggle
				label="Minimize to tray on close"
				checked={appDraft.minimizeToTray}
				disabled={!isTauriShell}
				onchange={(v) => {
					appDraft.minimizeToTray = v;
					appDirty = true;
				}}
			/>
			<Toggle
				label="Watch for firmware publication"
				hint="Checks the vendor's own config files and firmware directory paths at app start and periodically; never downloads or installs anything."
				checked={appDraft.firmwareWatchEnabled}
				disabled={!isTauriShell}
				onchange={(v) => {
					appDraft.firmwareWatchEnabled = v;
					appDirty = true;
				}}
			/>
			<RangeField
				label="Low battery warning"
				unit="%"
				min={5}
				max={50}
				value={appDraft.lowBatteryThresholdPercent}
				disabled={!isTauriShell}
				onchange={(v) => {
					appDraft.lowBatteryThresholdPercent = v;
					appDirty = true;
				}}
			/>
		</div>
		<button
			class="btn btn-primary"
			disabled={!isTauriShell || !appDirty || appSaving}
			onclick={saveAppSettings}
		>
			{appSaving ? 'Saving...' : appDirty ? 'Save preferences' : 'Up to date'}
		</button>

		<div class="rule"></div>

		<div class="field gpu-field">
			<SelectField
				label="Linux NVIDIA renderer workaround"
				hint="Works around a WebKitGTK blank-window bug on NVIDIA GPUs under Wayland or X11 (Linux only) by disabling its DMABUF renderer. Auto detects the failure condition at each launch. Takes effect on the next launch, not live."
				options={GPU_WORKAROUND_OPTIONS}
				value={gpuWorkaround}
				disabled={!isTauriShell || gpuWorkaroundSaving}
				onchange={setGpuWorkaround}
			/>
			{#if gpuWorkaroundSaving}<p class="field-hint">Saving...</p>{/if}
			{#if gpuWorkaroundError}<p class="field-hint error-text">{gpuWorkaroundError}</p>{/if}
		</div>
	</section>
</div>

<style>
	.page-pad {
		padding: 0 var(--space-lg) var(--space-lg);
	}

	.settings-grid {
		grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
		align-items: start;
	}

	.wide {
		grid-column: 1 / -1;
	}

	.plate-title {
		margin: 0;
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--color-ink);
	}

	.plate-subtitle {
		margin: 0 0 var(--space-sm);
		font-size: var(--text-xs);
		color: var(--color-muted);
	}

	.rule {
		height: 1px;
		background: var(--color-rule-2);
		margin: var(--space-md) 0;
	}

	.pref-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
		gap: var(--space-sm) var(--space-md);
		margin-bottom: var(--space-md);
	}

	.gpu-field {
		max-width: 32rem;
	}

	.backup-row {
		margin-top: var(--space-sm);
	}

	.device-list {
		list-style: none;
		margin: var(--space-sm) 0 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: var(--space-2xs);
	}

	.device-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-sm);
		background: var(--color-paper-3);
		border: 1px solid var(--color-rule);
		padding: var(--space-2xs) var(--space-xs);
		font-size: var(--text-xs);
	}

	.device-none {
		margin-top: var(--space-sm);
	}

	.about-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-2xs);
	}

	.about-row {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: var(--space-sm);
	}

	.hidden-input {
		display: none;
	}

	.error-text {
		color: var(--color-danger);
	}

	.caption-note {
		color: var(--color-faint);
	}
</style>
