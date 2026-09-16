<script lang="ts">
	import { onMount } from 'svelte';
	import { device } from '../device.svelte';
	import EmptyState from '../components/EmptyState.svelte';
	import HelpTip from '../components/HelpTip.svelte';

	let checking = $state(false);
	// Distinct from `checking`: whether a check has ever completed, so a screen freshly opened
	// never claims "no newer package found" before it has actually looked (device.firmwareUpdates
	// starts as [] regardless, which would otherwise read the same as a real zero-result check).
	let checked = $state(false);
	let importing = $state(false);
	let installingId = $state<string | null>(null);
	let fileInput = $state<HTMLInputElement | null>(null);
	let watching = $state(false);

	const PROGRESS_SEGMENTS = 24;
	const PROGRESS_SEGMENT_INDEXES = Array.from({ length: PROGRESS_SEGMENTS }, (_, i) => i);

	// `commands/firmware.rs::firmware_install` refuses outright unless the connection has write
	// access to a real device (it never even attempts the simulator, which never models the
	// update bootloader). Gating the CTA on the same condition, rather than letting a doomed click
	// surface only as a lastError after the fact, is what keeps this screen's one primary action
	// honest about when it can actually run.
	let installReady = $derived(
		device.connected && device.backend === 'realDevice' && device.access === 'readWrite'
	);

	function installBlockedReason(): string {
		if (!device.connected) return 'No mouse connected.';
		if (device.backend === 'simulator') return 'The simulator cannot be flashed.';
		return 'This connection is read-only; installing needs write access.';
	}

	onMount(() => {
		void device.refreshFirmware();
	});

	async function checkForUpdates() {
		checking = true;
		try {
			await device.checkForFirmwareUpdates();
			checked = true;
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
	<div class="grid firmware-grid">
		<section class="plate">
			<div class="plate-head">
				<h2 class="plate-title">Firmware archive<HelpTip topic="firmwareArchive" /></h2>
				<span class="chip chip-warning">Unverified on hardware</span>
			</div>
			<p class="plate-subtitle">
				Packages already imported to this app and matched against this hardware's identity
				markers.
			</p>
			<p class="field-hint caution">
				Installing runs the researched update procedure end to end; it has never been exercised
				against real hardware.
			</p>

			{#if device.firmwareLoading}
				<p class="field-hint">Loading...</p>
			{:else if device.firmware.length === 0}
				<EmptyState
					title="No firmware available for this device yet"
					message="No genuine firmware package for this mouse has been located. Install and rollback are built and ready, they simply have nothing in the archive to act on. Rollback needs at least two vendor-built packages per target before it is even possible. Asking the vendor or importing a package below are the ways to add one."
				>
					{#snippet actions()}
						<button class="btn btn-primary" disabled title="No package in the archive to install">
							Install
						</button>
						<button
							class="btn"
							disabled
							title="Achieved by installing an older archived package for the same target, not a separate command; needs at least two archived packages, and none are present."
						>
							Roll back
						</button>
					{/snippet}
				</EmptyState>
			{:else}
				{#if !installReady}
					<p class="field-hint error-text">{installBlockedReason()}</p>
				{/if}
				<ul class="firmware-list">
					{#each device.firmware as fw (fw.id)}
						<li class="firmware-item">
							<div class="firmware-row">
								<div>
									<div class="field-label">
										{fw.product} - <span class="mono">v{fw.version}</span>
									</div>
									<div class="field-hint mono">cid {fw.cid}, mid {fw.mid}</div>
									<div class="field-hint">imported {formatUnixSeconds(fw.importedAt)}</div>
								</div>
								<button
									class="btn btn-primary"
									disabled={installingId !== null || !installReady}
									title={installReady ? undefined : installBlockedReason()}
									onclick={() => install(fw.id)}
								>
									{installingId === fw.id ? 'Installing...' : 'Install'}
								</button>
							</div>
							{#if installingId === fw.id && device.firmwareProgress}
								{@const p = device.firmwareProgress}
								<div
									class="progress"
									role="progressbar"
									aria-valuenow={p.percent}
									aria-valuemin={0}
									aria-valuemax={100}
									aria-label={`Installing ${fw.product}`}
								>
									<div class="progress-track">
										{#each PROGRESS_SEGMENT_INDEXES as i (i)}
											<span
												class="progress-seg"
												class:filled={i < Math.round((p.percent / 100) * PROGRESS_SEGMENTS)}
											></span>
										{/each}
									</div>
									<div class="progress-readout mono">
										image {p.imageIndex + 1}/{p.imageCount}, {p.bytesSent}/{p.imageLen} B, {p.percent}%
									</div>
								</div>
							{/if}
						</li>
					{/each}
				</ul>
				<p class="field-hint">
					Installing an older entry for the same component is how a rollback is performed; there
					is no separate rollback command.
				</p>
			{/if}

			<div class="rule"></div>

			<div class="field-row">
				<div>
					<div class="field-label">Check for updates<HelpTip topic="firmwareCheckUpdates" /></div>
					<div class="field-hint">
						Compares the local archive above against what the connected device reports, for an
						already-imported package newer than the installed version.
					</div>
				</div>
				<button
					class="btn"
					disabled={checking || !device.connected}
					title={device.connected ? undefined : 'Connect a device first.'}
					onclick={checkForUpdates}
				>
					{checking ? 'Checking...' : 'Check for updates'}
				</button>
			</div>
			{#if checked && !checking}
				{#if device.firmwareUpdates.length > 0}
					<p class="field-hint">
						{device.firmwareUpdates.length} update{device.firmwareUpdates.length === 1 ? '' : 's'} available
						for the connected device.
					</p>
				{:else}
					<p class="field-hint">No newer package found in the archive.</p>
				{/if}
			{/if}

			{#if device.lastError}
				<p class="field-hint error-text">{device.lastError}</p>
			{/if}
		</section>

		<section class="plate">
			<div class="plate-head">
				<h2 class="plate-title">Acquire a package</h2>
			</div>

			<div class="acquire-block">
				<div class="field-row">
					<div>
						<div class="field-label">Watch for publication<HelpTip topic="firmwareWatch" /></div>
						<div class="field-hint">
							Checks the vendor's own configuration files and firmware directory paths for a
							change, and reports what it finds. Never downloads or installs anything on its own;
							a changed file is evidence to look at, not proof that firmware is available. Runs
							once at app start plus a slow periodic check in the background; can be turned off in
							app preferences.
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
								? 'Something has changed since the last known state, listed below.'
								: 'Nothing has changed since the last known state.'}
						</p>
						{#each report.configs.filter((c) => c.changed) as finding (finding.url)}
							<div class="watch-finding">
								<div class="field-label">{finding.description}</div>
								<p class="field-hint">{finding.summary}</p>
								{#each finding.evidence as snippet (snippet)}
									<code class="evidence-snippet mono">{snippet}</code>
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
			</div>

			<div class="rule"></div>

			<div class="acquire-block">
				<div class="field-row">
					<div>
						<div class="field-label">Import a package<HelpTip topic="firmwareImport" /></div>
						<div class="field-hint">
							Accepts only a package whose identity markers match this hardware.
						</div>
					</div>
					<button
						class="btn"
						disabled={importing || !device.connected}
						title={device.connected ? undefined : 'Connect a device first.'}
						onclick={() => fileInput?.click()}
					>
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
			</div>
		</section>
	</div>
</div>

<style>
	.page-pad {
		padding: 0 var(--space-lg) var(--space-lg);
	}

	/* Asymmetric on purpose: the archive panel carries the list, the empty state and the
	   check-for-updates row, while the acquire panel (now just publication watch and import,
	   the vendor-request block having been dropped) is lighter. An even split would leave the
	   acquire column's width unused by its own shorter content; this gives that width back to
	   the panel that needs it instead of stretching the lighter one to match. */
	.firmware-grid {
		grid-template-columns: minmax(380px, 3fr) minmax(320px, 2fr);
		align-items: start;
	}

	@media (max-width: 900px) {
		.firmware-grid {
			grid-template-columns: 1fr;
		}
	}

	.plate-title {
		margin: 0;
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--color-ink);
	}

	.plate-subtitle {
		margin: 0 0 var(--space-2xs);
		font-size: var(--text-xs);
		color: var(--color-muted);
	}

	.caution {
		color: var(--color-warning);
		margin-bottom: var(--space-sm);
	}

	.rule {
		height: 1px;
		background: var(--color-rule-2);
		margin: var(--space-md) 0;
	}

	.firmware-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: var(--space-2xs);
		max-height: 22rem;
		overflow-y: auto;
	}

	.firmware-item {
		background: var(--color-paper-3);
		border: 1px solid var(--color-rule);
		padding: var(--space-xs) var(--space-sm);
	}

	.firmware-row {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-2xs) var(--space-sm);
	}

	.progress {
		margin-top: var(--space-xs);
		display: flex;
		flex-direction: column;
		gap: var(--space-3xs);
	}

	.progress-track {
		display: flex;
		gap: 2px;
		height: 6px;
	}

	.progress-seg {
		flex: 1;
		background: var(--color-rule);
	}

	.progress-seg.filled {
		background: var(--color-accent);
	}

	.progress-readout {
		font-size: var(--text-xs);
		color: var(--color-muted);
	}

	.hidden-input {
		display: none;
	}

	.error-text {
		color: var(--color-danger);
	}

	.watch-report {
		margin-top: var(--space-2xs);
		display: flex;
		flex-direction: column;
		gap: var(--space-2xs);
		max-height: 16rem;
		overflow-y: auto;
	}

	.watch-finding {
		background: var(--color-paper-3);
		border: 1px solid var(--color-rule);
		padding: var(--space-xs) var(--space-sm);
		display: flex;
		flex-direction: column;
		gap: var(--space-3xs);
	}

	.evidence-snippet {
		display: block;
		font-size: var(--text-xs);
		color: var(--color-muted);
		background: var(--color-rule-2);
		padding: var(--space-3xs) var(--space-2xs);
		white-space: pre-wrap;
		word-break: break-word;
	}
</style>
