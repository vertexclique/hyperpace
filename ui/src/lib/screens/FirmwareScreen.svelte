<script lang="ts">
	import { onMount } from 'svelte';
	import { device } from '../device.svelte';

	// Route 2 ("the vendor request"): the ready-to-send message, kept in sync by hand with
	// docs/firmware-request.md (the same convention ui/src/lib/generated/commands.ts uses to stay
	// in sync with command_list::COMMANDS). This is the short, copyable core of that document; the
	// full document explains why each line is worded the way it is.
	const VENDOR_REQUEST_MESSAGE = `Subject: Firmware update packages needed for HYPACE mouse and receiver (all versions)

Hello,

I own a Lofree HYPACE mouse and would like to request the official firmware update packages for it and its 2.4 GHz receiver, including every version you have available, not only the current one.

My hardware identifies itself as:
- Mouse, wired connection: USB vendor id 0x3554, product id 0xFB14 (reported device version 0x0300)
- Receiver, 2.4 GHz dongle: USB vendor id 0x3554, product id 0xFB16 (reported device version 0x0216)
- Device model id reported by your own driver: cid 102

Could you send:
1. Every firmware package you have for the mouse, every version, not only the latest.
2. Every firmware package you have for the receiver, every version, for whichever receiver variant (1K, 2K, 4K or 8K) matches product id 0xFB16 on my unit. Could you also confirm which variant that is?
3. Please send the complete, vendor-built update package exactly as your own updater tool would use it, not a raw firmware payload and not a rebuilt or re-packaged image. A raw payload or a rebuilt image cannot be flashed: the update procedure needs the whole package your build process produces, not just the firmware bytes inside it.

Having more than one version lets me roll back if a newer release causes a problem, which is why I am asking for the full version history rather than only the newest release.

Thank you for your help.`;

	let checking = $state(false);
	let importing = $state(false);
	let installingId = $state<string | null>(null);
	let fileInput = $state<HTMLInputElement | null>(null);
	let watching = $state(false);
	let copied = $state(false);

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

	async function checkPublication() {
		watching = true;
		try {
			await device.checkFirmwarePublication();
		} finally {
			watching = false;
		}
	}

	async function copyVendorRequest() {
		try {
			await navigator.clipboard.writeText(VENDOR_REQUEST_MESSAGE);
			copied = true;
			setTimeout(() => (copied = false), 2000);
		} catch {
			// Clipboard access can be refused (no permission, no secure context); the text is
			// still on screen and selectable by hand, so this is not a failure worth surfacing
			// through device.lastError, which is reserved for backend command failures.
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

	function formatUnixSeconds(unixSeconds: number) {
		return new Date(unixSeconds * 1000).toLocaleString();
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
					built and ready, they simply have nothing in the archive to act on. Rollback needs at
					least two vendor-built packages per target before it is even possible. Asking the
					vendor or importing a package below are the ways to add one.
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
							<div class="field-label">{fw.product} - v{fw.version}</div>
							<div class="field-hint">
								cid {fw.cid} mid {fw.mid}, imported {formatUnixSeconds(fw.importedAt)}
							</div>
						</div>
						<button
							class="btn btn-primary"
							disabled={installingId !== null}
							onclick={() => install(fw.id)}
						>
							{installingId === fw.id
								? device.firmwareProgress
									? `Installing... ${device.firmwareProgress.percent}%`
									: 'Installing...'
								: 'Install'}
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
				<div class="field-hint">
					Compares the local archive above against what the connected device reports, for an
					already-imported package newer than the installed version.
				</div>
			</div>
			<button class="btn" disabled={checking} onclick={checkForUpdates}>
				{checking ? 'Checking...' : 'Check for updates'}
			</button>
		</div>
		{#if !checking && device.firmwareUpdates.length > 0}
			<p class="field-hint">
				{device.firmwareUpdates.length} update{device.firmwareUpdates.length === 1 ? '' : 's'} available
				for the connected device.
			</p>
		{/if}

		<div class="divider"></div>

		<div class="panel-title">Ask the vendor</div>
		<div class="panel-subtitle">
			The route most likely to yield a real, rollback-capable version history. See
			<code>docs/firmware-request.md</code> for the full explanation.
		</div>
		<pre class="vendor-request">{VENDOR_REQUEST_MESSAGE}</pre>
		<button class="btn" onclick={copyVendorRequest}>
			{copied ? 'Copied' : 'Copy request message'}
		</button>

		<div class="divider"></div>

		<div class="field-row">
			<div>
				<div class="field-label">Watch for publication</div>
				<div class="field-hint">
					Checks the vendor's own configuration files and firmware directory paths for a change,
					and reports what it finds. Never downloads or installs anything on its own; a changed
					file is evidence to look at, not proof that firmware is available. Runs once at app
					start plus a slow periodic check in the background; can be turned off in app
					preferences.
				</div>
			</div>
			<button class="btn" disabled={watching} onclick={checkPublication}>
				{watching ? 'Checking...' : 'Check for publication'}
			</button>
		</div>
		{#if device.firmwareWatchReport}
			{@const report = device.firmwareWatchReport}
			<div class="watch-report">
				<p class="field-hint">
					Checked {formatUnixSeconds(report.checkedAt)}.
					{report.hasFindings
						? 'Something changed since the research recorded it, listed below.'
						: 'Nothing has changed since the research recorded it.'}
				</p>
				{#each report.configs.filter((c) => c.changed) as finding (finding.url)}
					<div class="watch-finding">
						<div class="field-label">{finding.description}</div>
						<p class="field-hint">{finding.summary}</p>
						{#each finding.evidence as snippet (snippet)}
							<code class="evidence-snippet">{snippet}</code>
						{/each}
					</div>
				{/each}
				{#each report.directories.filter((d) => d.notable) as finding (finding.url)}
					<div class="watch-finding">
						<div class="field-label">{finding.description}</div>
						<p class="field-hint">{finding.summary}</p>
					</div>
				{/each}
				{#each report.fetchErrors as err (err.url)}
					<p class="field-hint error-text">{err.url}: could not be checked ({err.reason})</p>
				{/each}
				{#if !report.hasFindings}
					<p class="field-hint">{report.disclaimer}</p>
				{/if}
			</div>
		{/if}

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
		flex-wrap: wrap;
		justify-content: space-between;
		align-items: center;
		gap: 8px 12px;
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

	.vendor-request {
		background: var(--bg-raised);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		padding: 12px;
		margin: 10px 0;
		font-family: var(--font-mono);
		font-size: 11.5px;
		line-height: 1.5;
		white-space: pre-wrap;
		max-height: 260px;
		overflow-y: auto;
	}

	.watch-report {
		margin-top: 10px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.watch-finding {
		background: var(--bg-raised);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		padding: 10px 12px;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.evidence-snippet {
		display: block;
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		background: var(--border-soft);
		border-radius: var(--radius-sm);
		padding: 4px 6px;
		white-space: pre-wrap;
		word-break: break-word;
	}
</style>
