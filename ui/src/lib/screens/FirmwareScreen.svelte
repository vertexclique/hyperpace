<script lang="ts">
	import { onMount } from 'svelte';
	import { device } from '../device.svelte';

	let checking = $state(false);
	let importing = $state(false);
	let installingId = $state<string | null>(null);
	let fileInput = $state<HTMLInputElement | null>(null);

	onMount(() => {
		void device.refreshFirmware();
	});

	async function checkForUpdates() {
		checking = true;
		try {
			await device.checkForFirmwareUpdates();
		} finally {
			checking = false;
		}
	}

	async function onFileChosen(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		importing = true;
		try {
			const bytes = new Uint8Array(await file.arrayBuffer());
			await device.importFirmware(bytes);
		} finally {
			importing = false;
			input.value = '';
		}
	}

	async function install(id: string) {
		installingId = id;
		try {
			await device.installFirmware(id);
		} finally {
			installingId = null;
		}
	}

	function formatVersion(v: { major: number; minor: number }) {
		return `v${v.major}.${v.minor.toString(16).padStart(2, '0')}`;
	}
</script>

<div class="page-pad">
	<section class="panel">
		<div class="panel-title">Firmware archive</div>
		<div class="panel-subtitle">Images this app has verified and can flash to a mouse or receiver.</div>

		{#if device.firmwareLoading}
			<p class="field-hint">Loading...</p>
		{:else if device.firmware.length === 0}
			<div class="empty-state firmware-empty">
				<strong>No firmware available for this device yet</strong>
				<p>
					No genuine firmware package for this mouse has been located. Install and rollback are
					built and ready, they simply have nothing in the archive to act on. Importing a
					vendor-built package below is the way to add one.
				</p>
				<div class="disabled-actions">
					<button class="btn btn-primary" disabled>Install latest</button>
					<button class="btn" disabled>Roll back</button>
				</div>
			</div>
		{:else}
			<ul class="firmware-list">
				{#each device.firmware as fw (fw.id)}
					<li class="firmware-row">
						<div>
							<div class="field-label">{fw.product} - {formatVersion(fw.version)}</div>
							<div class="field-hint">
								{fw.device_type}, cid {fw.cid} mid {fw.mid}, imported {fw.imported_at}
							</div>
						</div>
						<button
							class="btn btn-primary"
							disabled={installingId !== null}
							onclick={() => install(fw.id)}
						>
							{installingId === fw.id ? 'Installing...' : 'Install'}
						</button>
					</li>
				{/each}
			</ul>
			<p class="field-hint">
				Installing an older entry for the same component is how a rollback is performed; there
				is no separate rollback command.
			</p>
		{/if}

		<div class="divider"></div>

		<div class="field-row">
			<div>
				<div class="field-label">Check for updates</div>
				<div class="field-hint">Looks at the configured acquisition routes for a new package.</div>
			</div>
			<button class="btn" disabled={checking} onclick={checkForUpdates}>
				{checking ? 'Checking...' : 'Check for updates'}
			</button>
		</div>

		<div class="divider"></div>

		<div class="field-row">
			<div>
				<div class="field-label">Import a package</div>
				<div class="field-hint">
					Accepts only a package whose identity markers match this hardware.
				</div>
			</div>
			<button class="btn" disabled={importing} onclick={() => fileInput?.click()}>
				{importing ? 'Importing...' : 'Choose file'}
			</button>
			<input
				bind:this={fileInput}
				type="file"
				accept=".bin"
				class="hidden-input"
				onchange={onFileChosen}
			/>
		</div>

		{#if device.lastError}
			<p class="field-hint error-text">{device.lastError}</p>
		{/if}
	</section>
</div>

<style>
	.page-pad {
		padding: 0 32px 32px;
	}

	.firmware-empty {
		text-align: left;
		align-items: flex-start;
		gap: 10px;
	}

	.firmware-empty p {
		color: var(--text-muted);
		font-size: 12.5px;
		max-width: 60ch;
	}

	.disabled-actions {
		display: flex;
		gap: 8px;
	}

	.firmware-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.firmware-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 12px;
		background: var(--bg-raised);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		padding: 10px 12px;
	}

	.divider {
		height: 1px;
		background: var(--border-soft);
		margin: 18px 0;
	}

	.hidden-input {
		display: none;
	}

	.error-text {
		color: var(--danger);
	}
</style>
