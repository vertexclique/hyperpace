<script lang="ts">
	import { onMount } from 'svelte';
	import { device } from '../device.svelte';
	import type { MacroEvent, SaveMacroRequest } from '../types';
	import { MACRO_EVENT_KINDS, MACRO_EVENT_MAX_COUNT, MACRO_NAME_MAX_BYTES } from '../types';
	import EmptyState from '../components/EmptyState.svelte';
	import SelectField from '../components/SelectField.svelte';

	// MacroEventRecord::kind is a free-form string, not a serde-tagged enum (hyperpace-store
	// never depends on hyperpace-protocol); these are the only names
	// `commands::macros::kind_from_string` recognizes.
	const EVENT_KINDS = MACRO_EVENT_KINDS.map((kind) => ({ value: kind, label: kind }));

	let selectedId = $state<string | null>(null);
	let draft = $state<SaveMacroRequest | null>(null);
	let saving = $state(false);
	let confirmingDelete = $state(false);

	onMount(() => {
		void device.refreshMacros();
	});

	let nameBytes = $derived(draft ? new TextEncoder().encode(draft.name).length : 0);
	let nameTooLong = $derived(nameBytes > MACRO_NAME_MAX_BYTES);

	function selectMacro(macro: SaveMacroRequest & { id: string }) {
		selectedId = macro.id;
		draft = structuredClone(macro);
		confirmingDelete = false;
	}

	function newMacro() {
		const usedSlots = new Set(
			device.macros.map((m) => m.slot).filter((slot): slot is number => slot !== undefined)
		);
		let slot = 0;
		while (usedSlots.has(slot)) slot += 1;
		selectedId = null;
		draft = { name: 'New macro', slot, events: [] };
		confirmingDelete = false;
	}

	function addEvent() {
		if (!draft || draft.events.length >= MACRO_EVENT_MAX_COUNT) return;
		const event: MacroEvent = { press: true, kind: 'Key', value: 0, delayMs: 10 };
		draft.events = [...draft.events, event];
	}

	function removeEvent(index: number) {
		if (!draft) return;
		draft.events = draft.events.filter((_, i) => i !== index);
	}

	function moveEvent(index: number, dir: -1 | 1) {
		if (!draft) return;
		const target = index + dir;
		if (target < 0 || target >= draft.events.length) return;
		const events = [...draft.events];
		[events[index], events[target]] = [events[target], events[index]];
		draft.events = events;
	}

	async function save() {
		if (!draft || nameTooLong) return;
		saving = true;
		try {
			const saved = await device.saveMacro(draft);
			// A newly created macro only gets its store id back from the response; adopt it so a
			// second save updates this record instead of creating another one.
			if (saved) selectedId = saved.id;
		} finally {
			saving = false;
		}
	}

	async function remove() {
		if (selectedId === null) return;
		await device.deleteMacro(selectedId);
		selectedId = null;
		draft = null;
		confirmingDelete = false;
	}
</script>

<div class="layout">
	<aside class="panel list-panel">
		<div class="panel-title">Macro library</div>
		<div class="panel-subtitle">Stored locally, bound to buttons from the Buttons screen.</div>
		<button class="btn btn-primary new-btn" onclick={newMacro}>New macro</button>
		{#if device.macrosLoading}
			<p class="field-hint">Loading...</p>
		{:else if device.macros.length === 0}
			<p class="field-hint">No macros saved yet.</p>
		{:else}
			<ul class="macro-list">
				{#each device.macros as macro (macro.id)}
					<li>
						<button
							type="button"
							class="list-row"
							class:active={selectedId === macro.id}
							onclick={() => selectMacro(macro)}
						>
							<span>{macro.name || (macro.slot !== undefined ? `Slot ${macro.slot}` : 'Untitled')}</span>
							<span class="field-hint">{macro.events.length} events</span>
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	</aside>

	<section class="editor-column">
		{#if !draft}
			<EmptyState
				title="No macro selected"
				message="Pick a macro from the library or create a new one to start editing."
			/>
		{:else}
			<div class="panel">
				<div class="field-row">
					<div class="field" style="flex:1">
						<label class="field-label" for="macro-name">Name</label>
						<input id="macro-name" class="text-input" bind:value={draft.name} />
						<span class="field-hint" class:error-text={nameTooLong}>
							{nameBytes}/{MACRO_NAME_MAX_BYTES} bytes
						</span>
					</div>
					<div class="field">
						<span class="field-label">Slot</span>
						<input
							class="text-input"
							type="number"
							min="0"
							max="63"
							style="width:80px"
							bind:value={draft.slot}
						/>
					</div>
				</div>

				<div class="events-header">
					<span class="panel-title" style="margin:0">Events</span>
					<span class="field-hint">{draft.events.length}/{MACRO_EVENT_MAX_COUNT}</span>
				</div>

				{#if draft.events.length === 0}
					<p class="field-hint">No events yet. Add one to begin.</p>
				{:else}
					<div class="event-table">
						<div class="event-head">
							<span></span>
							<span>Press/release</span>
							<span>Kind</span>
							<span>Value</span>
							<span>Delay (ms)</span>
							<span></span>
						</div>
						{#each draft.events as ev, i (i)}
							<div class="event-row">
								<span class="field-hint event-index">{i + 1}</span>
								<SelectField
									label={`Event ${i + 1} press or release`}
									hideLabel
									value={ev.press ? 'press' : 'release'}
									options={[
										{ value: 'press', label: 'Press' },
										{ value: 'release', label: 'Release' }
									]}
									onchange={(v) => (ev.press = v === 'press')}
								/>
								<SelectField
									label={`Event ${i + 1} kind`}
									hideLabel
									value={ev.kind}
									options={EVENT_KINDS}
									onchange={(v) => (ev.kind = v)}
								/>
								<input class="text-input" type="number" min="0" max="65535" bind:value={ev.value} />
								<input
									class="text-input"
									type="number"
									min="0"
									max="65535"
									bind:value={ev.delayMs}
								/>
								<div class="event-actions">
									<button class="btn" onclick={() => moveEvent(i, -1)} disabled={i === 0}>Up</button>
									<button
										class="btn"
										onclick={() => moveEvent(i, 1)}
										disabled={i === draft.events.length - 1}>Down</button
									>
									<button class="btn btn-danger" onclick={() => removeEvent(i)}>Remove</button>
								</div>
							</div>
						{/each}
					</div>
				{/if}

				<button class="btn" disabled={draft.events.length >= MACRO_EVENT_MAX_COUNT} onclick={addEvent}>
					Add event
				</button>

				<div class="actions-row">
					<button class="btn btn-primary" disabled={saving || nameTooLong} onclick={save}>
						{saving ? 'Saving...' : 'Save macro'}
					</button>
					{#if confirmingDelete}
						<button class="btn btn-danger" onclick={remove}>Confirm delete</button>
						<button class="btn" onclick={() => (confirmingDelete = false)}>Cancel</button>
					{:else}
						<button class="btn btn-danger" onclick={() => (confirmingDelete = true)}>Delete macro</button>
					{/if}
				</div>
				{#if device.lastError}
					<p class="field-hint error-text">{device.lastError}</p>
				{/if}
			</div>
		{/if}
	</section>
</div>

<style>
	.layout {
		display: flex;
		gap: 20px;
		padding: 0 32px 32px;
		align-items: flex-start;
	}

	.list-panel {
		width: 260px;
		flex-shrink: 0;
	}

	.new-btn {
		width: 100%;
		justify-content: center;
		margin: 10px 0 14px;
	}

	.macro-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.list-row {
		width: 100%;
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 2px;
		background: var(--bg-raised);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		padding: 8px 10px;
		text-align: left;
		font-size: 12.5px;
	}

	.list-row.active {
		border-color: var(--accent);
	}

	.editor-column {
		flex: 1;
		min-width: 0;
	}

	.events-header {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		margin: 18px 0 10px;
	}

	.event-table {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin-bottom: 12px;
	}

	.event-head,
	.event-row {
		display: grid;
		grid-template-columns: 22px 110px 130px 90px 90px 1fr;
		gap: 8px;
		align-items: center;
	}

	.event-head {
		font-size: 11px;
		color: var(--text-faint);
		text-transform: uppercase;
		letter-spacing: 0.03em;
	}

	.event-index {
		text-align: center;
	}

	.event-actions {
		display: flex;
		gap: 4px;
		justify-self: end;
	}

	.actions-row {
		display: flex;
		gap: 8px;
		margin-top: 18px;
		align-items: center;
	}

	.error-text {
		color: var(--danger);
	}
</style>
