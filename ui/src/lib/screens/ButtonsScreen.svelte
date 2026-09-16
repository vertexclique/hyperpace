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

	const ASSIGNABLE_KINDS: { value: ButtonAction['kind']; label: string }[] = [
		{ value: 'Disabled', label: 'Disabled' },
		{ value: 'Mouse', label: 'Mouse button' },
		{ value: 'Dpi', label: 'DPI switch' },
		{ value: 'Scroll', label: 'Scroll' },
		{ value: 'Fire', label: 'Fire key (rapid click)' },
		{ value: 'Keystroke', label: 'Keystroke' },
		{ value: 'Macro', label: 'Macro' },
		{ value: 'PollingCycle', label: 'Cycle polling rate' },
		{ value: 'Media', label: 'Media key' }
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

	function setKind(kind: ButtonAction['kind']) {
		if (!draft) return;
		draft = defaultForKind(kind);
	}

	function defaultForKind(kind: ButtonAction['kind']): ButtonAction {
		switch (kind) {
			case 'Disabled':
				return { kind: 'Disabled' };
			case 'Mouse':
				return { kind: 'Mouse', button: 'Left' };
			case 'Dpi':
				return { kind: 'Dpi', action: 'Loop' };
			case 'Scroll':
				return { kind: 'Scroll', direction: 'Left' };
			case 'Fire':
				return { kind: 'Fire', times: 0, interval_ms: 20 };
			case 'Keystroke':
				return { kind: 'Keystroke' };
			case 'Macro':
				return { kind: 'Macro', slot: 0, cycles: { kind: 'Times', count: 1 } };
			case 'PollingCycle':
				return { kind: 'PollingCycle' };
			case 'Media':
				return { kind: 'Media', usage: MEDIA_KEYS[0].usage };
			default:
				return { kind: 'Disabled' };
		}
	}

	function setCyclesKind(kind: MacroCycles['kind']) {
		if (!draft || draft.kind !== 'Macro') return;
		draft = {
			...draft,
			cycles: kind === 'Times' ? { kind: 'Times', count: 1 } : { kind }
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
		switch (action.kind) {
			case 'Disabled':
				return 'Disabled';
			case 'Mouse':
				return `Mouse: ${action.button}`;
			case 'Dpi':
				return `DPI ${action.action}`;
			case 'Scroll':
				return `Scroll ${action.direction}`;
			case 'Fire':
				return action.times === 0 ? 'Fire while held' : `Fire x${action.times}`;
			case 'Keystroke':
				return 'Keystroke';
			case 'Macro':
				return `Macro slot ${action.slot}`;
			case 'PollingCycle':
				return 'Cycle polling rate';
			case 'Media':
				return MEDIA_KEYS.find((m) => m.usage === action.usage)?.label ?? `Media 0x${action.usage.toString(16)}`;
			case 'Unknown':
				return `Unrecognized (kind ${action.button_kind})`;
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
						value={draft.kind}
						options={ASSIGNABLE_KINDS}
						onchange={(k) => setKind(k)}
					/>

					{#if draft.kind === 'Mouse'}
						<SelectField
							label="Mouse button"
							value={draft.button}
							options={[
								{ value: 'Left', label: 'Left' },
								{ value: 'Right', label: 'Right' },
								{ value: 'Wheel', label: 'Wheel' },
								{ value: 'Backward', label: 'Backward' },
								{ value: 'Forward', label: 'Forward' }
							]}
							onchange={(v) => draft && draft.kind === 'Mouse' && (draft.button = v)}
						/>
					{:else if draft.kind === 'Dpi'}
						<SelectField
							label="DPI action"
							value={draft.action}
							options={[
								{ value: 'Loop', label: 'Loop through stages' },
								{ value: 'Increase', label: 'DPI +' },
								{ value: 'Decrease', label: 'DPI -' }
							]}
							onchange={(v) => draft && draft.kind === 'Dpi' && (draft.action = v)}
						/>
					{:else if draft.kind === 'Scroll'}
						<SelectField
							label="Scroll direction"
							value={draft.direction}
							options={[
								{ value: 'Left', label: 'Scroll left' },
								{ value: 'Right', label: 'Scroll right' }
							]}
							onchange={(v) => draft && draft.kind === 'Scroll' && (draft.direction = v)}
						/>
					{:else if draft.kind === 'Fire'}
						<RangeField
							label="Times"
							hint="0 repeats for as long as the button is held"
							min={0}
							max={3}
							value={draft.times}
							onchange={(v) => draft && draft.kind === 'Fire' && (draft.times = v)}
						/>
						<RangeField
							label="Interval"
							unit=" ms"
							min={10}
							max={255}
							value={draft.interval_ms}
							onchange={(v) => draft && draft.kind === 'Fire' && (draft.interval_ms = v)}
						/>
					{:else if draft.kind === 'Keystroke'}
						<p class="field-hint">
							Key assignment needs a keystroke-slot Tauri command; the fixed command surface in
							docs/architecture/api-contract.md does not yet expose one. The button can be set to
							type Keystroke, but its key content cannot be edited from this screen yet.
						</p>
					{:else if draft.kind === 'Macro'}
						<RangeField
							label="Macro slot"
							min={0}
							max={31}
							value={draft.slot}
							onchange={(v) => draft && draft.kind === 'Macro' && (draft.slot = v)}
						/>
						<SelectField
							label="Repeat"
							value={draft.cycles.kind}
							options={[
								{ value: 'Times', label: 'Fixed number of cycles' },
								{ value: 'UntilReleased', label: 'Until the button is released' },
								{ value: 'UntilAnyButton', label: 'Until any button is pressed' }
							]}
							onchange={(v) => setCyclesKind(v)}
						/>
						{#if draft.cycles.kind === 'Times'}
							<RangeField
								label="Cycles"
								min={1}
								max={250}
								value={draft.cycles.count}
								onchange={(v) =>
									draft &&
									draft.kind === 'Macro' &&
									draft.cycles.kind === 'Times' &&
									(draft.cycles.count = v)}
							/>
						{/if}
					{:else if draft.kind === 'Media'}
						<SelectField
							label="Media key"
							value={draft.usage}
							options={MEDIA_KEYS.map((m) => ({ value: m.usage, label: m.label }))}
							onchange={(v) => draft && draft.kind === 'Media' && (draft.usage = v)}
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
