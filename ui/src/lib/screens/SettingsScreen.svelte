<script lang="ts">
	import { onMount } from 'svelte';
	import { device, isTauriShell } from '../device.svelte';
	import type { AppSettings, DeviceBackend, LinkType } from '../types';
	import EmptyState from '../components/EmptyState.svelte';
	import Toggle from '../components/Toggle.svelte';
	import RangeField from '../components/RangeField.svelte';

	let connectingBackend = $state<DeviceBackend | null>(null);
	let appDraft = $state<AppSettings | null>(null);
	let appDirty = $state(false);
	let appSaving = $state(false);
	let profileIndex = $state(0);
	let profileSaving = $state(false);
	let exporting = $state(false);
	let importing = $state(false);
	let configFileInput = $state<HTMLInputElement | null>(null);

	onMount(() => {
		void device.refreshDevices();
		void device.refreshAppSettings();
	});

	$effect(() => {
		appDraft = device.appSettings ? { ...device.appSettings } : null;
		appDirty = false;
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
</script>

<div class="page-pad grid" style="grid-template-columns: repeat(auto-fit, minmax(300px, 1fr))">
	<section class="panel">
		<div class="panel-title">Device</div>
		<div class="panel-subtitle">Connect to a Hyperpace mouse to read and edit its settings.</div>

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
							cid {device.identity.cid}, mid {device.identity.mid}, link {formatLinkType(
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
				<p class="field-hint" style="margin-top:10px">No devices found.</p>
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

	<section class="panel">
		<div class="panel-title">Profile</div>
		<div class="panel-subtitle">Switch the device's active on-board profile.</div>
		<div class="field-row">
			<input
				class="text-input"
				type="number"
				min="0"
				max="7"
				style="width:80px"
				bind:value={profileIndex}
				disabled={!device.connected}
			/>
			<button class="btn btn-primary" disabled={!device.connected || profileSaving} onclick={switchProfile}>
				{profileSaving ? 'Switching...' : 'Switch profile'}
			</button>
		</div>
		<p class="field-hint">The active profile is not yet reported by read_settings.</p>
	</section>

	<section class="panel">
		<div class="panel-title">Configuration backup</div>
		<div class="panel-subtitle">Export or import the full settings shadow as a .bin file.</div>
		<div class="field-row">
			<span class="field-label">Export current configuration</span>
			<button class="btn" disabled={!device.connected || exporting} onclick={exportConfig}>
				{exporting ? 'Exporting...' : 'Export'}
			</button>
		</div>
		<div class="field-row" style="margin-top:12px">
			<span class="field-label">Import a configuration file</span>
			<button class="btn" disabled={!device.connected || importing} onclick={() => configFileInput?.click()}>
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

	<section class="panel">
		<div class="panel-title">App preferences</div>
		<div class="panel-subtitle">Behaviour of the Hyperpace app itself, independent of any device.</div>
		{#if !appDraft}
			<p class="field-hint">
				{isTauriShell ? 'Loading...' : 'Not available in preview mode.'}
			</p>
		{:else}
			<div class="field-stack">
				<Toggle
					label="Launch at login"
					checked={appDraft.autostart}
					onchange={(v) => {
						if (appDraft) appDraft.autostart = v;
						appDirty = true;
					}}
				/>
				<Toggle
					label="Minimize to tray on close"
					checked={appDraft.minimizeToTray}
					onchange={(v) => {
						if (appDraft) appDraft.minimizeToTray = v;
						appDirty = true;
					}}
				/>
				<Toggle
					label="Watch for firmware publication"
					hint="Checks the vendor's own config files and firmware directory paths at app start and periodically; never downloads or installs anything."
					checked={appDraft.firmwareWatchEnabled}
					onchange={(v) => {
						if (appDraft) appDraft.firmwareWatchEnabled = v;
						appDirty = true;
					}}
				/>
				<RangeField
					label="Low battery warning"
					unit="%"
					min={5}
					max={50}
					value={appDraft.lowBatteryThresholdPercent}
					onchange={(v) => {
						if (appDraft) appDraft.lowBatteryThresholdPercent = v;
						appDirty = true;
					}}
				/>
				<button class="btn btn-primary" disabled={!appDirty || appSaving} onclick={saveAppSettings}>
					{appSaving ? 'Saving...' : appDirty ? 'Save preferences' : 'Up to date'}
				</button>
			</div>
		{/if}
	</section>
</div>

<style>
	.page-pad {
		padding: 0 32px 32px;
	}

	.field-stack {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.device-list {
		list-style: none;
		margin: 10px 0 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.device-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 12px;
		background: var(--bg-raised);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		padding: 8px 10px;
		font-size: 12.5px;
	}

	.hidden-input {
		display: none;
	}

	.error-text {
		color: var(--danger);
	}
</style>
