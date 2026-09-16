<script lang="ts">
	import { onMount } from 'svelte';
	import { device } from '../device.svelte';
	import { data } from '../data.svelte';
	import { EVENT_LOG_PAGE_SIZE } from '../data-types';
	import type { EventLogEntry, ProfileSnapshotRecord } from '../data-types';
	import EmptyState from '../components/EmptyState.svelte';
	import HelpTip from '../components/HelpTip.svelte';

	type TabId = 'history' | 'profiles' | 'macros' | 'firmware' | 'settings' | 'backup';

	const TABS: { id: TabId; label: string }[] = [
		{ id: 'history', label: 'History' },
		{ id: 'profiles', label: 'Profiles' },
		{ id: 'macros', label: 'Macros' },
		{ id: 'firmware', label: 'Firmware archive' },
		{ id: 'settings', label: 'App settings' },
		{ id: 'backup', label: 'Backup' }
	];

	let activeTab = $state<TabId>('history');

	let confirmingClearHistory = $state(false);
	let confirmingRestoreId = $state<string | null>(null);
	let confirmingDeleteSnapshotId = $state<string | null>(null);
	let confirmingDeleteMacroId = $state<string | null>(null);
	let confirmingDeleteFirmwareId = $state<string | null>(null);

	let renamingSnapshotId = $state<string | null>(null);
	let renameDraft = $state('');

	function defaultSnapshotName(): string {
		return `Snapshot ${new Date().toLocaleString()}`;
	}
	let newSnapshotName = $state(defaultSnapshotName());

	let exporting = $state(false);
	let importing = $state(false);
	let backupFileInput = $state<HTMLInputElement | null>(null);

	onMount(() => {
		void data.refreshOverview();
		void data.loadEventsPage(0);
		void data.refreshProfileSnapshots();
		void data.refreshAppSettings();
		void device.refreshMacros();
		void device.refreshFirmware();
	});

	function formatUnixSeconds(unixSeconds: number): string {
		return new Date(unixSeconds * 1000).toLocaleString();
	}

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		const units = ['KB', 'MB', 'GB'];
		let value = bytes / 1024;
		let unitIndex = 0;
		while (value >= 1024 && unitIndex < units.length - 1) {
			value /= 1024;
			unitIndex += 1;
		}
		return `${value.toFixed(1)} ${units[unitIndex]}`;
	}

	function kindTone(kind: string): string {
		if (kind === 'connected') return 'chip-ok';
		if (kind === 'battery_low') return 'chip-danger';
		if (kind === 'disconnected' || kind === 'offline') return 'chip-warning';
		return 'chip-neutral';
	}

	async function clearHistory() {
		await data.clearEvents();
		confirmingClearHistory = false;
	}

	async function restoreSnapshot(id: string) {
		await data.restoreProfileSnapshot(id);
		confirmingRestoreId = null;
	}

	async function saveSnapshot() {
		const name = newSnapshotName.trim();
		if (!name) return;
		await data.saveProfileSnapshot(name);
		newSnapshotName = defaultSnapshotName();
	}

	function startRename(snapshot: ProfileSnapshotRecord) {
		renamingSnapshotId = snapshot.id;
		renameDraft = snapshot.name;
	}

	async function commitRename(id: string) {
		const name = renameDraft.trim();
		if (name) await data.renameProfileSnapshot(id, name);
		renamingSnapshotId = null;
	}

	async function deleteSnapshot(id: string) {
		await data.deleteProfileSnapshot(id);
		confirmingDeleteSnapshotId = null;
	}

	async function deleteMacro(id: string) {
		await device.deleteMacro(id);
		confirmingDeleteMacroId = null;
	}

	async function deleteFirmware(id: string) {
		await data.deleteFirmwareEntry(id);
		confirmingDeleteFirmwareId = null;
	}

	async function exportEverything() {
		exporting = true;
		try {
			const bytes = await data.exportStore();
			const blob = new Blob([new Uint8Array(bytes)], { type: 'application/json' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = 'hyperpace-backup.json';
			a.click();
			URL.revokeObjectURL(url);
		} finally {
			exporting = false;
		}
	}

	async function onBackupFileChosen(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		importing = true;
		try {
			const bytes = new Uint8Array(await file.arrayBuffer());
			await data.importStore(bytes);
		} finally {
			importing = false;
			input.value = '';
		}
	}

	function eventRowKey(event: EventLogEntry): string {
		return event.id;
	}

	let pageStart = $derived(data.eventsTotal === 0 ? 0 : data.eventsOffset + 1);
	let pageEnd = $derived(Math.min(data.eventsOffset + EVENT_LOG_PAGE_SIZE, data.eventsTotal));
	let hasPrevious = $derived(data.eventsOffset > 0);
	let hasNext = $derived(data.eventsOffset + EVENT_LOG_PAGE_SIZE < data.eventsTotal);

	let appSettingEntries = $derived(Object.entries(data.appSettings));
</script>

<div class="page-pad">
	<section class="plate overview-plate">
		<div class="plate-head">
			<div class="plate-title-group">
				<span class="plate-stub"></span>
				<span class="plate-title">Local store</span>
				<HelpTip topic="dataOverview" />
			</div>
		</div>
		{#if data.overviewLoading && !data.overview}
			<p class="field-hint">Loading...</p>
		{:else if data.overview}
			<div class="overview-grid">
				<div class="overview-tile">
					<span class="tile-count mono">{data.overview.macros}</span>
					<span class="tile-label">Macros</span>
				</div>
				<div class="overview-tile">
					<span class="tile-count mono">{data.overview.profiles}</span>
					<span class="tile-label">Profiles</span>
				</div>
				<div class="overview-tile">
					<span class="tile-count mono">{data.overview.settings}</span>
					<span class="tile-label">Settings</span>
				</div>
				<div class="overview-tile">
					<span class="tile-count mono">{data.overview.firmware}</span>
					<span class="tile-label">Firmware</span>
				</div>
				<div class="overview-tile">
					<span class="tile-count mono">{data.overview.events}</span>
					<span class="tile-label">Events</span>
				</div>
			</div>
			<div class="overview-meta">
				<span class="field-hint">Store size <span class="mono">{formatBytes(data.overview.storeSizeBytes)}</span></span>
				<span class="field-hint mono store-path">{data.overview.storeRoot}</span>
			</div>
		{/if}
		{#if data.lastError}<p class="field-hint error-text">{data.lastError}</p>{/if}
	</section>

	<nav class="tab-strip" aria-label="Data sections">
		{#each TABS as tab (tab.id)}
			<button
				type="button"
				class="tab-btn"
				class:active={activeTab === tab.id}
				onclick={() => (activeTab = tab.id)}
			>
				{tab.label}
			</button>
		{/each}
	</nav>

	<section class="plate tab-content">
		{#if activeTab === 'history'}
			<div class="tab-head-row">
				<div>
					<h2 class="tab-heading">Device and app history<HelpTip topic="dataHistory" /></h2>
					<p class="field-hint">Connects, disconnects, battery warnings and settings writes.</p>
				</div>
				{#if confirmingClearHistory}
					<div class="confirm-row">
						<button class="btn btn-danger" disabled={data.clearingEvents} onclick={clearHistory}>
							{data.clearingEvents ? 'Clearing...' : 'Confirm clear'}
						</button>
						<button class="btn" onclick={() => (confirmingClearHistory = false)}>Cancel</button>
					</div>
				{:else}
					<button
						class="btn btn-danger"
						disabled={data.eventsTotal === 0}
						onclick={() => (confirmingClearHistory = true)}
					>
						Clear history
					</button>
				{/if}
			</div>

			{#if data.eventsLoading && data.events.length === 0}
				<p class="field-hint">Loading...</p>
			{:else if data.events.length === 0}
				<EmptyState title="No history yet" message="Device connects, disconnects and warnings will appear here." />
			{:else}
				<div class="event-table">
					<div class="event-head">
						<span>Time</span>
						<span>Kind</span>
						<span>Message</span>
					</div>
					<div class="event-body">
						{#each data.events as event (eventRowKey(event))}
							<div class="event-row">
								<span class="field-hint mono">{formatUnixSeconds(event.at)}</span>
								<span class="chip {kindTone(event.kind)}">{event.kind}</span>
								<span class="event-message">{event.message}</span>
							</div>
						{/each}
					</div>
				</div>
				<div class="pager">
					<span class="field-hint mono">{pageStart}-{pageEnd} of {data.eventsTotal}</span>
					<div class="pager-actions">
						<button class="btn" disabled={!hasPrevious || data.eventsLoading} onclick={() => data.previousEventsPage()}>
							Previous
						</button>
						<button class="btn" disabled={!hasNext || data.eventsLoading} onclick={() => data.nextEventsPage()}>
							Next
						</button>
					</div>
				</div>
			{/if}
		{:else if activeTab === 'profiles'}
			<div class="tab-head-row">
				<div>
					<h2 class="tab-heading">Profile snapshots<HelpTip topic="dataProfiles" /></h2>
					<p class="field-hint">Capture the mouse's full settings, and restore or delete a saved copy.</p>
				</div>
			</div>
			<div class="field-row snapshot-save-row">
				<input
					class="text-input snapshot-name-input"
					bind:value={newSnapshotName}
					placeholder="Snapshot name"
					aria-label="New snapshot name"
				/>
				<button
					class="btn btn-primary"
					disabled={!device.connected || data.savingSnapshot || newSnapshotName.trim().length === 0}
					title={device.connected ? undefined : 'No mouse connected.'}
					onclick={saveSnapshot}
				>
					{data.savingSnapshot ? 'Saving...' : 'Save current settings'}
				</button>
			</div>

			{#if data.profileSnapshotsLoading && data.profileSnapshots.length === 0}
				<p class="field-hint">Loading...</p>
			{:else if data.profileSnapshots.length === 0}
				<EmptyState title="No snapshots saved yet" message="Save the connected mouse's current settings to create one." />
			{:else}
				<ul class="record-list">
					{#each data.profileSnapshots as snapshot (snapshot.id)}
						<li class="record-row">
							{#if renamingSnapshotId === snapshot.id}
								<div class="rename-row">
									<input class="text-input" bind:value={renameDraft} aria-label="Snapshot name" />
									<button class="btn btn-primary" onclick={() => commitRename(snapshot.id)}>Save</button>
									<button class="btn" onclick={() => (renamingSnapshotId = null)}>Cancel</button>
								</div>
							{:else}
								<div class="record-info">
									<span class="field-label">{snapshot.name}</span>
									<span class="field-hint">
										profile <span class="mono">{snapshot.index}</span> &middot; captured {formatUnixSeconds(
											snapshot.capturedAt
										)}
									</span>
								</div>
								<div class="record-actions">
									{#if confirmingRestoreId === snapshot.id}
										<button
											class="btn btn-primary"
											disabled={data.restoringSnapshotId === snapshot.id}
											onclick={() => restoreSnapshot(snapshot.id)}
										>
											{data.restoringSnapshotId === snapshot.id ? 'Restoring...' : 'Confirm restore'}
										</button>
										<button class="btn" onclick={() => (confirmingRestoreId = null)}>Cancel</button>
									{:else}
										<button
											class="btn"
											disabled={!device.connected}
											title={device.connected ? 'Overwrites the connected mouse\'s current settings.' : 'No mouse connected.'}
											onclick={() => (confirmingRestoreId = snapshot.id)}
										>
											Restore
										</button>
									{/if}
									<button class="btn" onclick={() => startRename(snapshot)}>Rename</button>
									{#if confirmingDeleteSnapshotId === snapshot.id}
										<button class="btn btn-danger" onclick={() => deleteSnapshot(snapshot.id)}>Confirm delete</button>
										<button class="btn" onclick={() => (confirmingDeleteSnapshotId = null)}>Cancel</button>
									{:else}
										<button class="btn btn-danger" onclick={() => (confirmingDeleteSnapshotId = snapshot.id)}>
											Delete
										</button>
									{/if}
								</div>
							{/if}
						</li>
					{/each}
				</ul>
			{/if}
		{:else if activeTab === 'macros'}
			<div class="tab-head-row">
				<div>
					<h2 class="tab-heading">Saved macros<HelpTip topic="dataMacros" /></h2>
					<p class="field-hint">The macro library, edited from the Macros screen.</p>
				</div>
			</div>
			{#if device.macrosLoading && device.macros.length === 0}
				<p class="field-hint">Loading...</p>
			{:else if device.macros.length === 0}
				<EmptyState title="No macros saved yet" message="Record one from the Macros screen." />
			{:else}
				<ul class="record-list">
					{#each device.macros as macro (macro.id)}
						<li class="record-row">
							<div class="record-info">
								<span class="field-label">{macro.name || 'Untitled'}</span>
								<span class="field-hint mono">{macro.events.length} events</span>
							</div>
							<div class="record-actions">
								{#if confirmingDeleteMacroId === macro.id}
									<button class="btn btn-danger" onclick={() => deleteMacro(macro.id)}>Confirm delete</button>
									<button class="btn" onclick={() => (confirmingDeleteMacroId = null)}>Cancel</button>
								{:else}
									<button class="btn btn-danger" onclick={() => (confirmingDeleteMacroId = macro.id)}>Delete</button>
								{/if}
							</div>
						</li>
					{/each}
				</ul>
			{/if}
		{:else if activeTab === 'firmware'}
			<div class="tab-head-row">
				<div>
					<h2 class="tab-heading">Firmware archive<HelpTip topic="firmwareArchive" /></h2>
					<p class="field-hint">Packages imported into the local archive.</p>
				</div>
			</div>
			{#if device.firmwareLoading && device.firmware.length === 0}
				<p class="field-hint">Loading...</p>
			{:else if device.firmware.length === 0}
				<EmptyState title="No firmware archived yet" message="Import a package from the Firmware screen." />
			{:else}
				<ul class="record-list">
					{#each device.firmware as fw (fw.id)}
						<li class="record-row">
							<div class="record-info">
								<span class="field-label">{fw.product} - <span class="mono">v{fw.version}</span></span>
								<span class="field-hint mono">cid {fw.cid}, mid {fw.mid}</span>
								<span class="field-hint">imported {formatUnixSeconds(fw.importedAt)}</span>
							</div>
							<div class="record-actions">
								{#if confirmingDeleteFirmwareId === fw.id}
									<button class="btn btn-danger" onclick={() => deleteFirmware(fw.id)}>Confirm delete</button>
									<button class="btn" onclick={() => (confirmingDeleteFirmwareId = null)}>Cancel</button>
								{:else}
									<button class="btn btn-danger" onclick={() => (confirmingDeleteFirmwareId = fw.id)}>Delete</button>
								{/if}
							</div>
						</li>
					{/each}
				</ul>
			{/if}
		{:else if activeTab === 'settings'}
			<div class="tab-head-row">
				<div>
					<h2 class="tab-heading">App settings<HelpTip topic="dataAppSettings" /></h2>
					<p class="field-hint">Every stored preference. Reset returns a key to its own default.</p>
				</div>
			</div>
			{#if data.appSettingsLoading && appSettingEntries.length === 0}
				<p class="field-hint">Loading...</p>
			{:else if appSettingEntries.length === 0}
				<EmptyState title="No app settings stored yet" message="Preferences appear here once they are changed from their defaults." />
			{:else}
				<ul class="record-list">
					{#each appSettingEntries as [key, value] (key)}
						<li class="record-row">
							<div class="record-info">
								<span class="field-label mono">{key}</span>
								<span class="field-hint mono">{JSON.stringify(value)}</span>
							</div>
							<div class="record-actions">
								<button class="btn" onclick={() => data.resetAppSetting(key)}>Reset</button>
							</div>
						</li>
					{/each}
				</ul>
			{/if}
		{:else if activeTab === 'backup'}
			<div class="tab-head-row">
				<div>
					<h2 class="tab-heading">Backup<HelpTip topic="dataBackup" /></h2>
					<p class="field-hint">
						Export every collection as one file, or import a backup: matching records already present
						are skipped, nothing is overwritten. Firmware entries carry their index metadata only,
						never the package bytes.
					</p>
				</div>
			</div>
			<div class="field-row backup-row">
				<span class="field-label">Export everything</span>
				<button class="btn btn-primary" disabled={exporting} onclick={exportEverything}>
					{exporting ? 'Exporting...' : 'Export'}
				</button>
			</div>
			<div class="field-row backup-row">
				<span class="field-label">Import a backup</span>
				<button class="btn" disabled={importing} onclick={() => backupFileInput?.click()}>
					{importing ? 'Importing...' : 'Choose file'}
				</button>
				<input
					bind:this={backupFileInput}
					type="file"
					accept=".json,application/json"
					class="hidden-input"
					onchange={onBackupFileChosen}
				/>
			</div>
			{#if data.lastImportResult}
				{@const result = data.lastImportResult}
				<p class="field-hint import-result">
					Added {result.macrosAdded} macro(s), {result.profilesAdded} profile(s), {result.settingsAdded}
					setting(s), {result.firmwareAdded} firmware entrie(s) and {result.eventsAdded} event(s).
				</p>
			{/if}
		{/if}
		{#if data.lastError}<p class="field-hint error-text">{data.lastError}</p>{/if}
	</section>
</div>

<style>
	.page-pad {
		padding: 0 var(--space-lg) var(--space-lg);
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.plate-title-group {
		display: flex;
		align-items: baseline;
		gap: var(--space-2xs);
	}

	.plate-title {
		font-family: var(--font-display);
		font-size: var(--text-md);
		font-weight: 600;
		color: var(--color-ink);
	}

	.overview-grid {
		display: grid;
		grid-template-columns: repeat(5, 1fr);
		gap: var(--space-sm);
	}

	.overview-tile {
		display: flex;
		flex-direction: column;
		gap: var(--space-3xs);
		background: var(--color-paper-3);
		border: 1px solid var(--color-rule);
		padding: var(--space-xs) var(--space-sm);
	}

	.tile-count {
		font-size: var(--text-xl);
		font-weight: 600;
		color: var(--color-ink);
	}

	.tile-label {
		font-size: var(--text-xs);
		color: var(--color-muted);
		text-transform: uppercase;
		letter-spacing: 0.03em;
	}

	.overview-meta {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		gap: var(--space-2xs) var(--space-sm);
		margin-top: var(--space-sm);
	}

	.store-path {
		overflow-wrap: anywhere;
	}

	.tab-strip {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-xs);
		border-bottom: 1px solid var(--color-rule);
	}

	.tab-btn {
		background: transparent;
		border: none;
		border-bottom: 2px solid transparent;
		color: var(--color-muted);
		padding: var(--space-2xs) var(--space-2xs) var(--space-xs);
		font-size: var(--text-sm);
		font-weight: 500;
	}

	.tab-btn:hover {
		color: var(--color-ink);
	}

	.tab-btn.active {
		color: var(--color-ink);
		border-bottom-color: var(--color-accent);
	}

	.tab-head-row {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		align-items: flex-start;
		gap: var(--space-sm);
		margin-bottom: var(--space-sm);
	}

	.tab-heading {
		font-family: var(--font-display);
		font-size: var(--text-md);
		font-weight: 600;
		color: var(--color-ink);
		margin-bottom: 2px;
	}

	.confirm-row {
		display: flex;
		gap: var(--space-2xs);
	}

	.event-table {
		max-height: 26rem;
		overflow-y: auto;
	}

	.event-head,
	.event-row {
		display: grid;
		grid-template-columns: 190px 130px 1fr;
		gap: var(--space-sm);
		align-items: center;
		padding: var(--space-2xs) 0;
	}

	.event-head {
		font-size: var(--text-xs);
		color: var(--color-muted);
		text-transform: uppercase;
		letter-spacing: 0.03em;
		border-bottom: 1px solid var(--color-rule);
		padding-bottom: var(--space-2xs);
	}

	.event-row {
		border-top: 1px solid var(--color-rule);
	}

	.event-message {
		font-size: var(--text-sm);
	}

	.pager {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-top: var(--space-sm);
		gap: var(--space-sm);
	}

	.pager-actions {
		display: flex;
		gap: var(--space-2xs);
	}

	.record-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		max-height: 24rem;
		overflow-y: auto;
	}

	.record-list > li + li {
		border-top: 1px solid var(--color-rule);
	}

	.record-row {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-2xs) var(--space-sm);
		padding: var(--space-xs) var(--space-2xs);
	}

	.record-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.record-actions {
		display: flex;
		gap: var(--space-2xs);
		flex-wrap: wrap;
	}

	.rename-row {
		display: flex;
		gap: var(--space-2xs);
		flex: 1;
	}

	.snapshot-save-row {
		margin-bottom: var(--space-sm);
	}

	.snapshot-name-input {
		flex: 1;
		max-width: 24rem;
	}

	.backup-row + .backup-row {
		margin-top: var(--space-sm);
	}

	.hidden-input {
		display: none;
	}

	.import-result {
		margin-top: var(--space-sm);
	}

	.error-text {
		color: var(--color-danger);
	}

	@media (max-width: 900px) {
		.overview-grid {
			grid-template-columns: repeat(2, 1fr);
		}

		.event-head,
		.event-row {
			grid-template-columns: 1fr;
		}
	}
</style>
