<script lang="ts">
	import { onMount } from 'svelte';
	import { getName, getTauriVersion, getVersion } from '@tauri-apps/api/app';
	import { device, isTauriShell } from '../device.svelte';
	import { accessWarningDetail, statusLine } from '../status';
	import { invoke } from '../tauri';
	import { DEFAULT_LOW_BATTERY_THRESHOLD_PERCENT } from '../types';
	import type { AppSettings, AppSettingsResponse, DeviceBackend, LinkType } from '../types';
	import EmptyState from '../components/EmptyState.svelte';
	import HelpTip from '../components/HelpTip.svelte';
	import Toggle from '../components/Toggle.svelte';
	import RangeField from '../components/RangeField.svelte';
	import SelectField from '../components/SelectField.svelte';
	import SegmentDial from '../controls/SegmentDial.svelte';

	const PROFILE_OPTIONS = Array.from({ length: 8 }, (_, i) => ({ value: i, label: String(i) }));

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
	let otherDevices = $derived(device.devices.filter((d) => d.backend !== device.backend));
	let profileUnknown = $derived(!hasData || device.settings?.profile.state === 'unsupported');

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
			// minimizeToTray has no control on this screen any more (closing the window always
			// keeps the app in the tray now), so this never writes back a preference the operator
			// cannot see or change here.
			const patch: Partial<AppSettings> = {
				autostart: appDraft.autostart,
				lowBatteryThresholdPercent: appDraft.lowBatteryThresholdPercent,
				firmwareWatchEnabled: appDraft.firmwareWatchEnabled
			};
			await device.writeAppSettings(patch);
			appDirty = false;
		} finally {
			appSaving = false;
		}
	}

	async function switchProfile(index: number) {
		const previous = profileIndex;
		profileIndex = index;
		profileSaving = true;
		try {
			await device.setProfile(index);
		} catch {
			profileIndex = previous;
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
			<h2 class="plate-title">Device<HelpTip topic="deviceConnection" /></h2>
		</div>

		{#if !isTauriShell}
			<p class="plate-subtitle">Connect to a Hyperpace mouse to read and edit its settings.</p>
			<EmptyState
				title="Preview mode"
				message="This build is not running inside the Hyperpace desktop app, so no device commands are available."
			/>
		{:else if device.connected}
			<p class="plate-subtitle">{statusLine(device, isTauriShell)}</p>

			{#if accessWarningDetail(device)}
				<p class="cable-warning">{accessWarningDetail(device)}</p>
				<p class="cable-command mono">
					sudo install -m 644 70-hyperpace.rules /usr/lib/udev/rules.d/ && sudo udevadm control
					--reload-rules && sudo udevadm trigger --subsystem-match=hidraw
				</p>
			{/if}

			<div class="identifiers-row">
				<span class="field-hint">Identifiers</span>
				<span class="field-hint mono">
					{#if device.identity}
						cid {device.identity.cid} &middot; mid {device.identity.mid} &middot; link {formatLinkType(
							device.identity.link
						)} &middot; {device.backend === 'simulator' ? 'simulator' : 'real device'}
					{:else}
						not read yet
					{/if}
				</span>
			</div>

			<div class="field-row device-actions">
				<button class="btn btn-danger" onclick={() => device.disconnect()}>Disconnect</button>
				{#each otherDevices as d (d.backend)}
					<button
						class="btn"
						disabled={connectingBackend !== null}
						onclick={() => connectTo(d.backend)}
					>
						{connectingBackend === d.backend ? 'Connecting...' : `Use ${d.label}`}
					</button>
				{/each}
			</div>
		{:else}
			<p class="plate-subtitle">Connect to a Hyperpace mouse to read and edit its settings.</p>
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
			<h2 class="plate-title">Profile<HelpTip topic="profile" /></h2>
		</div>
		<p class="plate-subtitle">
			Pick the device's active on-board profile.
			{#if !device.connected}
				<span class="caption-note">No mouse connected.</span>
			{:else if device.settings?.profile.state === 'unsupported'}
				<span class="caption-note">Not supported on this device.</span>
			{/if}
		</p>
		<div class="dial-field">
			<SegmentDial
				values={PROFILE_OPTIONS}
				value={profileIndex}
				ariaLabel="Active profile"
				unknown={profileUnknown}
				disabled={profileSaving}
				size="default"
				centerLabel={String(profileIndex)}
				onchange={switchProfile}
			/>
			<p class="field-hint">
				{#if profileSaving}
					Switching...
				{:else if hasData && device.settings?.profile.state === 'active'}
					Currently profile <span class="mono">{profileIndex}</span>.
				{:else}
					Currently: -
				{/if}
			</p>
		</div>
		{#if device.lastError}
			<p class="field-hint error-text">{device.lastError}</p>
		{/if}
	</section>

	<section class="plate">
		<div class="plate-head">
			<h2 class="plate-title">Configuration backup<HelpTip topic="configExport" /></h2>
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
		<div class="pref-list">
			<div class="pref-row">
				<div class="tip-relabel">
					<span class="field-label">Launch at login<HelpTip topic="launchAtLogin" /></span>
					<div class="relabeled">
						<Toggle
							label="Launch at login"
							checked={appDraft.autostart}
							disabled={!isTauriShell}
							onchange={(v) => {
								appDraft.autostart = v;
								appDirty = true;
							}}
						/>
					</div>
				</div>
			</div>
			<div class="pref-row">
				<div class="tip-relabel">
					<span class="field-label"
						>Watch for firmware publication<HelpTip topic="firmwareWatch" /></span
					>
					<div class="relabeled">
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
					</div>
				</div>
			</div>
			<div class="pref-row">
				<div class="tip-relabel">
					<span class="field-label">Low battery warning<HelpTip topic="lowBatteryWarning" /></span
					>
					<div class="relabeled relabeled-range">
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
				</div>
			</div>
		</div>
		<button
			class="btn btn-primary save-btn"
			disabled={!isTauriShell || !appDirty || appSaving}
			onclick={saveAppSettings}
		>
			{appSaving ? 'Saving...' : appDirty ? 'Save preferences' : 'Up to date'}
		</button>

		<div class="rule"></div>

		<div class="pref-row gpu-row">
			<span class="field-label"
				>Linux NVIDIA renderer workaround<HelpTip topic="rendererWorkaround" /></span
			>
			<SelectField
				label="Linux NVIDIA renderer workaround"
				hideLabel
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
	/* Shown when the mouse is plugged in by cable but the operating system refuses this app access
	   to it: the one connection problem an operator can fix themselves, so it names the fix. */
	.cable-warning {
		margin-top: var(--space-2xs);
		color: var(--color-warning);
		font-size: var(--text-sm);
		line-height: 1.45;
	}

	.cable-command {
		margin-top: var(--space-2xs);
		padding: var(--space-2xs) var(--space-xs);
		background: var(--color-paper-3);
		border-left: 2px solid var(--color-warning);
		color: var(--color-ink-2);
		font-size: var(--text-xs);
		line-height: 1.5;
		overflow-wrap: anywhere;
	}

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

	/* The clean list the operator asked for: one preference per row, label and description on the
	   left (each Toggle/RangeField's own internal layout already puts the control on the right),
	   separated by a hairline instead of a wrapping multi-column grid that scattered a row's
	   label away from its control at odd widths. */
	.pref-list {
		display: flex;
		flex-direction: column;
	}

	.pref-row {
		padding: var(--space-sm) 0;
	}

	.pref-row + .pref-row {
		border-top: 1px solid var(--color-rule);
	}

	/* Toggle and RangeField render their own label inline with the control, so their built-in
	   label has no slot for a HelpTip beside it (unlike SelectField's `hideLabel`). This renders
	   the label ourselves, with the tip, and hides the control's own copy (sr-only, not
	   display:none, so RangeField's real label-for association still names the slider for
	   assistive tech); Toggle's copy carries no such association, its aria-label already covers
	   it independently. */
	.tip-relabel {
		display: flex;
		flex-direction: column;
		gap: var(--space-3xs);
	}

	.relabeled :global(.field-label) {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}

	/* RangeField's own label is itself the flex item that carries the value readout's
	   space-between partner; absolute-positioning it removes it from the flex layout entirely
	   (an absolutely positioned flex item takes no part in flex sizing), which would otherwise
	   leave the value readout alone and pulled to the start instead of staying where it read
	   before, at the end next to the slider below it. Toggle's own label is nested one level
	   deeper (inside a wrapper that stays a flex item, hint text and all), so it needs no such
	   correction. */
	.relabeled-range :global(.field-row) {
		justify-content: flex-end;
	}

	.save-btn {
		margin-top: var(--space-2xs);
	}

	.gpu-row {
		max-width: 32rem;
	}

	.dial-field {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-2xs);
		text-align: center;
		padding: var(--space-sm) 0 var(--space-2xs);
	}

	.identifiers-row {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		justify-content: space-between;
		gap: var(--space-2xs) var(--space-sm);
		margin-top: var(--space-sm);
	}

	.device-actions {
		justify-content: flex-start;
		margin-top: var(--space-sm);
		gap: var(--space-2xs);
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
