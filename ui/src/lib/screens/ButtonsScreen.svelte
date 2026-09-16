<script lang="ts">
	import { device } from '../device.svelte';
	import type { ButtonAction, MacroCycles } from '../types';
	import MouseArt from '../art/MouseArt.svelte';
	import EmptyState from '../components/EmptyState.svelte';
	import SelectField from '../components/SelectField.svelte';
	import RangeField from '../components/RangeField.svelte';

	// docs/research/mouse-protocol-v2.md 8.5, NEW medias list. HID Consumer
	// usage ids; labelled by usage, not by the vendor's (sometimes swapped)
	// icon names.
	const MEDIA_KEYS: { label: string; usage: number }[] = [
		{ label: 'Brightness up', usage: 0x006f },
		{ label: 'Brightness down', usage: 0x0070 },
		{ label: 'Mute', usage: 0x00e2 },
		{ label: 'Volume up', usage: 0x00e9 },
		{ label: 'Volume down', usage: 0x00ea },
		{ label: 'Play / pause', usage: 0x00cd },
		{ label: 'Previous track', usage: 0x00b6 },
		{ label: 'Next track', usage: 0x00b5 },
		{ label: 'Stop', usage: 0x00b7 },
		{ label: 'Media player', usage: 0x0183 },
		{ label: 'Email', usage: 0x018a },
		{ label: 'My computer', usage: 0x0194 },
		{ label: 'Calculator', usage: 0x0192 },
		{ label: 'Home page', usage: 0x0223 },
		{ label: 'Search', usage: 0x0221 },
		{ label: 'Favorites', usage: 0x022a },
		{ label: 'AC Back (0x0225)', usage: 0x0225 },
		{ label: 'AC Forward (0x0224)', usage: 0x0224 },
		{ label: 'Stop (web)', usage: 0x0226 },
		{ label: 'Refresh', usage: 0x0227 }
	];

	const BUTTON_LABELS = ['Left click', 'Right click', 'Wheel click', 'Back', 'Forward'];

	const ASSIGNABLE_KINDS: { value: ButtonAction['type']; label: string }[] = [
		{ value: 'disabled', label: 'Disabled' },
		{ value: 'mouse', label: 'Mouse button' },
		{ value: 'dpi', label: 'DPI switch' },
		{ value: 'scroll', label: 'Scroll' },
		{ value: 'fire', label: 'Fire key (rapid click)' },
		{ value: 'keystroke', label: 'Keystroke' },
		{ value: 'macro', label: 'Macro' },
		{ value: 'pollingCycle', label: 'Cycle polling rate' },
		{ value: 'media', label: 'Media key' }
	];

	let selected = $state<number | null>(null);
	let draft = $state<ButtonAction | null>(null);
	let saving = $state(false);

	let buttons = $derived(device.settings?.buttons ?? []);
	let selectedAction = $derived(selected !== null ? (buttons[selected] ?? null) : null);

	$effect(() => {
		draft = selectedAction ? structuredClone(selectedAction) : null;
	});

	function selectHotspot(index: number) {
		selected = index;
	}

	function setKind(type: ButtonAction['type']) {
		if (!draft) return;
		draft = defaultForKind(type);
	}

	function defaultForKind(type: ButtonAction['type']): ButtonAction {
		switch (type) {
			case 'disabled':
				return { type: 'disabled' };
			case 'mouse':
				return { type: 'mouse', button: 'left' };
			case 'dpi':
				return { type: 'dpi', action: 'loop' };
			case 'scroll':
				return { type: 'scroll', direction: 'left' };
			case 'fire':
				return { type: 'fire', times: 0, intervalMs: 20 };
			case 'keystroke':
				return { type: 'keystroke' };
			case 'macro':
				return { type: 'macro', slot: 0, cycles: { cycles: 'times', n: 1 } };
			case 'pollingCycle':
				return { type: 'pollingCycle' };
			case 'media':
				return { type: 'media', usage: MEDIA_KEYS[0].usage };
			default:
				return { type: 'disabled' };
		}
	}

	function setCyclesKind(cycles: MacroCycles['cycles']) {
		if (!draft || draft.type !== 'macro') return;
		draft = {
			...draft,
			cycles: cycles === 'times' ? { cycles: 'times', n: 1 } : { cycles }
		};
	}

	async function apply() {
		if (selected === null || !draft) return;
		saving = true;
		try {
			await device.setButton(selected, draft);
		} finally {
			saving = false;
		}
	}

	function actionSummary(action: ButtonAction): string {
		switch (action.type) {
			case 'disabled':
				return 'Disabled';
			case 'mouse':
				return `Mouse: ${action.button}`;
			case 'dpi':
				return `DPI ${action.action}`;
			case 'scroll':
				return `Scroll ${action.direction}`;
			case 'fire':
				return action.times === 0 ? 'Fire while held' : `Fire x${action.times}`;
			case 'keystroke':
				return 'Keystroke';
			case 'macro':
				return `Macro slot ${action.slot}`;
			case 'pollingCycle':
				return 'Cycle polling rate';
			case 'media':
				return MEDIA_KEYS.find((m) => m.usage === action.usage)?.label ?? `Media 0x${action.usage.toString(16)}`;
			case 'unknown':
				return `Unrecognized (kind ${action.kind})`;
		}
	}
</script>

<div class="layout">
	<div class="art-column panel">
		<MouseArt
			selected={selected}
			onselect={selectHotspot}
			labels={BUTTON_LABELS}
			activeCount={buttons.length || 5}
		/>
		<p class="field-hint art-caption">Select a button to edit its action.</p>
	</div>

	<div class="content-column">
		{#if !device.connected}
			<EmptyState
				title="No device connected"
				message="Connect a Hyperpace mouse from Settings to read and edit its button assignments."
			/>
		{:else if device.settingsLoading}
			<EmptyState title="Reading settings" message="Fetching the current button layout from the device." />
		{:else if selected === null}
			<div class="panel">
				<div class="panel-title">Buttons</div>
				<div class="panel-subtitle">{buttons.length} button{buttons.length === 1 ? '' : 's'} reported by this device.</div>
				<ul class="button-list">
					{#each buttons as action, i (i)}
						<li>
							<button type="button" class="list-row" onclick={() => selectHotspot(i)}>
								<span>{BUTTON_LABELS[i] ?? `Button ${i + 1}`}</span>
								<span class="field-hint">{actionSummary(action)}</span>
							</button>
						</li>
					{/each}
				</ul>
			</div>
		{:else if draft}
			<div class="panel">
				<div class="panel-title">{BUTTON_LABELS[selected] ?? `Button ${selected + 1}`}</div>
				<div class="panel-subtitle">Currently: {selectedAction ? actionSummary(selectedAction) : 'unknown'}</div>

				<div class="grid" style="gap: 14px">
					<SelectField
						label="Action"
						value={draft.type}
						options={ASSIGNABLE_KINDS}
						onchange={(k) => setKind(k)}
					/>

					{#if draft.type === 'mouse'}
						<SelectField
							label="Mouse button"
							value={draft.button}
							options={[
								{ value: 'left', label: 'Left' },
								{ value: 'right', label: 'Right' },
								{ value: 'middle', label: 'Middle (wheel click)' },
								{ value: 'backward', label: 'Backward' },
								{ value: 'forward', label: 'Forward' }
							]}
							onchange={(v) => draft && draft.type === 'mouse' && (draft.button = v)}
						/>
					{:else if draft.type === 'dpi'}
						<SelectField
							label="DPI action"
							value={draft.action}
							options={[
								{ value: 'loop', label: 'Loop through stages' },
								{ value: 'increase', label: 'DPI +' },
								{ value: 'decrease', label: 'DPI -' }
							]}
							onchange={(v) => draft && draft.type === 'dpi' && (draft.action = v)}
						/>
					{:else if draft.type === 'scroll'}
						<SelectField
							label="Scroll direction"
							value={draft.direction}
							options={[
								{ value: 'left', label: 'Scroll left' },
								{ value: 'right', label: 'Scroll right' }
							]}
							onchange={(v) => draft && draft.type === 'scroll' && (draft.direction = v)}
						/>
					{:else if draft.type === 'fire'}
						<RangeField
							label="Times"
							hint="0 repeats for as long as the button is held"
							min={0}
							max={3}
							value={draft.times}
							onchange={(v) => draft && draft.type === 'fire' && (draft.times = v)}
						/>
						<RangeField
							label="Interval"
							unit=" ms"
							min={10}
							max={255}
							value={draft.intervalMs}
							onchange={(v) => draft && draft.type === 'fire' && (draft.intervalMs = v)}
						/>
					{:else if draft.type === 'keystroke'}
						<p class="field-hint">
							Key assignment needs a keystroke-slot Tauri command; the fixed command surface in
							docs/architecture/api-contract.md does not yet expose one. The button can be set to
							type Keystroke, but its key content cannot be edited from this screen yet.
						</p>
					{:else if draft.type === 'macro'}
						<RangeField
							label="Macro slot"
							min={0}
							max={31}
							value={draft.slot}
							onchange={(v) => draft && draft.type === 'macro' && (draft.slot = v)}
						/>
						<SelectField
							label="Repeat"
							value={draft.cycles.cycles}
							options={[
								{ value: 'times', label: 'Fixed number of cycles' },
								{ value: 'untilReleased', label: 'Until the button is released' },
								{ value: 'untilAnyPress', label: 'Until any button is pressed' }
							]}
							onchange={(v) => setCyclesKind(v)}
						/>
						{#if draft.cycles.cycles === 'times'}
							<RangeField
								label="Cycles"
								min={1}
								max={250}
								value={draft.cycles.n}
								onchange={(v) =>
									draft &&
									draft.type === 'macro' &&
									draft.cycles.cycles === 'times' &&
									(draft.cycles.n = v)}
							/>
						{/if}
					{:else if draft.type === 'media'}
						<SelectField
							label="Media key"
							value={draft.usage}
							options={MEDIA_KEYS.map((m) => ({ value: m.usage, label: m.label }))}
							onchange={(v) => draft && draft.type === 'media' && (draft.usage = v)}
						/>
					{/if}
				</div>

				<div class="actions-row">
					<button class="btn btn-primary" disabled={saving} onclick={apply}>
						{saving ? 'Applying...' : 'Apply'}
					</button>
					<button class="btn" onclick={() => (selected = null)}>Back to list</button>
				</div>
				{#if device.lastError}
					<p class="field-hint error-text">{device.lastError}</p>
				{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	.layout {
		display: flex;
		gap: 20px;
		padding: 0 32px 32px;
		align-items: flex-start;
	}

	.art-column {
		width: 260px;
		flex-shrink: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		position: sticky;
		top: 20px;
	}

	.art-caption {
		text-align: center;
	}

	.content-column {
		flex: 1;
		min-width: 0;
	}

	.button-list {
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
		justify-content: space-between;
		gap: 12px;
		background: var(--bg-raised);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		padding: 10px 12px;
		text-align: left;
		font-size: 12.5px;
	}

	.list-row:hover {
		border-color: var(--text-faint);
	}

	.actions-row {
		display: flex;
		gap: 8px;
		margin-top: 18px;
	}

	.error-text {
		color: var(--danger);
		margin-top: 10px;
	}
</style>
