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
		draft = $state.snapshot(macro);
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
	<aside class="plate list-plate">
		<div class="plate-head">
			<div class="plate-title-group">
				<span class="plate-stub"></span>
				<span class="plate-title">Macro library</span>
			</div>
		</div>
		<p class="field-hint plate-subtitle">Stored locally, bound to buttons from the Buttons screen.</p>
		<button class="btn btn-primary new-btn" onclick={newMacro}>New macro</button>
		{#if device.macrosLoading}
			<p class="field-hint">Loading...</p>
		{:else if device.macros.length === 0}
			<p class="field-hint">No macros saved yet.</p>
		{:else}
			<ul class="macro-list">
				{#each device.macros as macro (macro.id)}
					{@const active = selectedId === macro.id}
					<li>
						<button type="button" class="list-row" class:active onclick={() => selectMacro(macro)}>
							{#if active}<span class="plate-stub row-stub"></span>{/if}
							<span class="row-text">
								<span class="row-label"
									>{macro.name || (macro.slot !== undefined ? `Slot ${macro.slot}` : 'Untitled')}</span
								>
								<span class="field-hint mono">{macro.events.length} events</span>
							</span>
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	</aside>

	<section class="editor-column">
		<div class="plate editor-plate">
			<div class="plate-head">
				<div class="plate-title-group">
					{#if draft}<span class="plate-stub"></span>{/if}
					<span class="plate-title">Macro editor</span>
				</div>
			</div>
			{#if !draft}
				<EmptyState
					title="No macro selected"
					message="Pick a macro from the library or create a new one to start editing."
				/>
			{:else}
				<div class="field-row">
					<div class="field name-field">
						<label class="field-label" for="macro-name">Name</label>
						<input id="macro-name" class="text-input" bind:value={draft.name} />
						<span class="field-hint mono" class:error-text={nameTooLong}>
							{nameBytes}/{MACRO_NAME_MAX_BYTES} bytes
						</span>
					</div>
					<div class="field">
						<span class="field-label">Slot</span>
						<input
							class="text-input mono slot-input"
							type="number"
							min="0"
							max="63"
							bind:value={draft.slot}
						/>
					</div>
				</div>

				<div class="plate-head events-head">
					<span class="plate-title">Events</span>
					<span class="field-hint mono">{draft.events.length}/{MACRO_EVENT_MAX_COUNT}</span>
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
								<span class="field-hint mono event-index">{i + 1}</span>
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
								<input
									class="text-input mono"
									type="number"
									min="0"
									max="65535"
									bind:value={ev.value}
								/>
								<input
									class="text-input mono"
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

				<button
					class="btn add-event-btn"
					disabled={draft.events.length >= MACRO_EVENT_MAX_COUNT}
					onclick={addEvent}
				>
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
			{/if}
		</div>
	</section>
</div>

<style>
	.layout {
		display: flex;
		gap: var(--space-md);
		padding: 0 var(--space-lg) var(--space-lg);
		align-items: flex-start;
	}

	.list-plate {
		width: 300px;
		flex-shrink: 0;
	}

	/* Groups the active-panel stub with its title so `.plate-head`'s own space-between keeps
	   them together at the head's leading edge (design.md "A panel header is a strip above a
	   hairline. The active panel carries a 2px accent stub..."). */
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

	.plate-subtitle {
		margin-bottom: var(--space-sm);
	}

	.new-btn {
		margin: var(--space-sm) 0 var(--space-md);
	}

	.macro-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		/* Bounds the library to its own scroll region instead of growing the screen past the
		   window: the store can hold more macros than fit at once, so this list scrolls, the
		   page never does. */
		max-height: 480px;
		overflow-y: auto;
	}

	.macro-list > li + li {
		border-top: 1px solid var(--color-rule);
	}

	.list-row {
		width: 100%;
		display: flex;
		align-items: center;
		gap: var(--space-2xs);
		background: transparent;
		border: none;
		padding: var(--space-xs) var(--space-2xs);
		text-align: left;
		font-size: var(--text-sm);
	}

	.list-row:hover {
		background: var(--color-paper-3);
	}

	.list-row.active {
		background: var(--color-paper-3);
	}

	.row-stub {
		margin-right: 2px;
	}

	.row-text {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 2px;
		min-width: 0;
	}

	.row-label {
		font-weight: 500;
		color: var(--color-ink);
	}

	.editor-column {
		flex: 1;
		min-width: 420px;
	}

	.name-field {
		flex: 1;
	}

	.slot-input {
		width: 80px;
	}

	.events-head {
		margin-top: var(--space-md);
	}

	.event-table {
		display: flex;
		flex-direction: column;
		margin-bottom: var(--space-sm);
		/* A macro can hold up to 70 events: bound the visible list to its own scroll region
		   (contained scroll) rather than letting the screen grow past the window, matching the
		   same "the page never scrolls, a bounded sub-list does" rule as the library above. The
		   horizontal scroll is a second, independent safety net for a squeezed width. */
		max-height: 360px;
		overflow-y: auto;
		overflow-x: auto;
	}

	.event-head,
	.event-row {
		display: grid;
		grid-template-columns: 22px 110px 130px 90px 90px 1fr;
		gap: var(--space-2xs);
		align-items: center;
		min-width: 560px;
		padding: var(--space-3xs) 0;
	}

	/* Square rows separated by hairlines, not individually boxed (design.md: depth comes from
	   the paper steps and the hairlines, never a border-per-row). */
	.event-row {
		border-top: 1px solid var(--color-rule);
	}

	.event-head {
		font-size: var(--text-xs);
		color: var(--color-muted);
		text-transform: uppercase;
		letter-spacing: 0.03em;
		padding-bottom: var(--space-2xs);
	}

	.event-index {
		text-align: center;
	}

	.event-actions {
		display: flex;
		gap: var(--space-3xs);
		justify-self: end;
	}

	.add-event-btn {
		margin-top: var(--space-2xs);
	}

	.actions-row {
		display: flex;
		gap: var(--space-2xs);
		margin-top: var(--space-md);
		align-items: center;
	}

	.error-text {
		color: var(--color-danger);
	}

	/* Same breakpoint as the Buttons screen: the library list stacks above the editor instead of
	   squeezing beside it. */
	@media (max-width: 900px) {
		.layout {
			flex-direction: column;
		}

		.list-plate {
			width: 100%;
		}
	}
</style>
