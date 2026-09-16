<script lang="ts">
	import { device } from '../device.svelte';
	import type { ButtonAction, Keystroke, MacroCycles, Modifier } from '../types';
	import MouseArt from '../art/MouseArt.svelte';
	import SelectField from '../components/SelectField.svelte';
	import RangeField from '../components/RangeField.svelte';

	const MODIFIER_OPTIONS: { value: Modifier; label: string }[] = [
		{ value: 'leftCtrl', label: 'L Ctrl' },
		{ value: 'leftShift', label: 'L Shift' },
		{ value: 'leftAlt', label: 'L Alt' },
		{ value: 'leftWin', label: 'L Win' },
		{ value: 'rightCtrl', label: 'R Ctrl' },
		{ value: 'rightShift', label: 'R Shift' },
		{ value: 'rightAlt', label: 'R Alt' },
		{ value: 'rightWin', label: 'R Win' }
	];

	function keyRange(start: number, count: number, label: (i: number) => string) {
		return Array.from({ length: count }, (_, i) => ({ value: start + i, label: label(i) }));
	}

	// docs/research/mouse-protocol-v2.md 8.6: the shared keymap's 95 type-1 entries, HID
	// keyboard-page (0x07) usage ids. Built from ranges instead of listed by hand to stay compact;
	// each range's bounds are quoted from that section.
	const KEY_OPTIONS: { value: number; label: string }[] = [
		...keyRange(4, 26, (i) => String.fromCharCode(65 + i)), // A..Z, 4..29
		...keyRange(30, 9, (i) => String(i + 1)), // 1..9, 30..38
		{ value: 39, label: '0' },
		{ value: 40, label: 'Enter' },
		{ value: 41, label: 'Esc' },
		{ value: 42, label: 'Backspace' },
		{ value: 43, label: 'Tab' },
		{ value: 44, label: 'Space' },
		{ value: 45, label: '-' },
		{ value: 46, label: '=' },
		{ value: 47, label: '[' },
		{ value: 48, label: ']' },
		{ value: 49, label: '\\' },
		{ value: 51, label: ';' },
		{ value: 52, label: "'" },
		{ value: 53, label: '`' },
		{ value: 54, label: ',' },
		{ value: 55, label: '.' },
		{ value: 56, label: '/' },
		{ value: 57, label: 'Caps Lock' },
		...keyRange(58, 12, (i) => `F${i + 1}`), // F1..F12, 58..69
		{ value: 70, label: 'Print Screen' },
		{ value: 71, label: 'Scroll Lock' },
		{ value: 72, label: 'Pause' },
		{ value: 73, label: 'Insert' },
		{ value: 74, label: 'Home' },
		{ value: 75, label: 'Page Up' },
		{ value: 76, label: 'Delete' },
		{ value: 77, label: 'End' },
		{ value: 78, label: 'Page Down' },
		{ value: 79, label: 'Right' },
		{ value: 80, label: 'Left' },
		{ value: 81, label: 'Down' },
		{ value: 82, label: 'Up' },
		{ value: 83, label: 'Num Lock' },
		{ value: 84, label: 'Keypad /' },
		{ value: 85, label: 'Keypad *' },
		{ value: 86, label: 'Keypad -' },
		{ value: 87, label: 'Keypad +' },
		{ value: 88, label: 'Keypad Enter' },
		...keyRange(89, 9, (i) => `Keypad ${i + 1}`), // Keypad 1..9, 89..97
		{ value: 98, label: 'Keypad 0' },
		{ value: 99, label: 'Keypad .' }
	];

	const EMPTY_KEYSTROKE: Keystroke = { modifiers: [], key: null, media: null };

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
	let draft = $state<ButtonAction>({ type: 'disabled' });
	let saving = $state(false);

	// The keystroke slot at the selected button's index (docs/research/mouse-protocol-v2.md
	// section 8.4): read back through get_button_keystroke so the chord editor and the media-key
	// select start from what the device actually holds, not a blank slate.
	let keystrokeDraft = $state<Keystroke>({ ...EMPTY_KEYSTROKE });
	let keystrokeLoaded = $state(false);

	// `hasData` mirrors the other screens' own flag (see LightingScreen): true once this device's
	// settings, buttons included, have actually been read.
	let hasData = $derived(device.settings !== null);
	let buttons = $derived(device.settings?.buttons ?? []);
	let selectedAction = $derived(selected !== null ? (buttons[selected] ?? null) : null);

	// `draft` is never null: with no real assignment to show (no device, or a button index past
	// what this device reported), it falls back to the same structural "disabled" default every
	// unassigned button already displays with a device connected, and the Action select below
	// marks itself `unknown` in that case so "Disabled" is never shown as if it were a reading.
	$effect(() => {
		draft = selectedAction ? $state.snapshot(selectedAction) : defaultForKind('disabled');
	});

	async function loadKeystroke(index: number) {
		keystrokeLoaded = false;
		// No device, nothing to read: skip the round trip entirely rather than calling a command
		// that can only fail with "no device connected", which would surface as an error for a
		// state that is normal, not a failure.
		if (!device.connected) {
			keystrokeDraft = { ...EMPTY_KEYSTROKE };
			keystrokeLoaded = true;
			return;
		}
		const keystroke = await device.getButtonKeystroke(index);
		keystrokeDraft = keystroke ?? { ...EMPTY_KEYSTROKE };
		keystrokeLoaded = true;
		// A media-bound slot already carries its usage id; prefill the select with it instead of
		// defaulting to the first entry in the list.
		if (draft && draft.type === 'media' && keystrokeDraft.media !== null) {
			draft = { ...draft, usage: keystrokeDraft.media };
		}
	}

	function selectHotspot(index: number) {
		selected = index;
		void loadKeystroke(index);
	}

	function toggleModifier(modifier: Modifier, on: boolean) {
		const set = new Set(keystrokeDraft.modifiers);
		if (on) set.add(modifier);
		else set.delete(modifier);
		keystrokeDraft = { ...keystrokeDraft, modifiers: [...set] };
	}

	function setKind(type: ButtonAction['type']) {
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
		if (draft.type !== 'macro') return;
		draft = {
			...draft,
			cycles: cycles === 'times' ? { cycles: 'times', n: 1 } : { cycles }
		};
	}

	async function apply() {
		if (selected === null || !hasData) return;
		saving = true;
		try {
			let keystroke: Keystroke | undefined;
			if (draft.type === 'keystroke') {
				keystroke = keystrokeDraft;
			} else if (draft.type === 'media') {
				keystroke = { modifiers: [], key: null, media: draft.usage };
			}
			await device.setButton(selected, draft, keystroke);
			await loadKeystroke(selected);
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
	<div class="art-column">
		<MouseArt
			selected={selected}
			onselect={selectHotspot}
			labels={BUTTON_LABELS}
			activeCount={buttons.length || 5}
		/>
		<p class="field-hint art-caption">Select a button to edit its action.</p>
		{#if hasData && buttons.length < BUTTON_LABELS.length}
			<p class="field-hint unsupported-note">
				{BUTTON_LABELS.length - buttons.length} button{BUTTON_LABELS.length - buttons.length === 1
					? ''
					: 's'} not on this device.
			</p>
		{/if}
	</div>

	<div class="content-column">
		{#if selected === null}
			<div class="plate list-plate">
				<div class="plate-head">
					<div class="plate-title-group">
						<span class="plate-stub"></span>
						<span class="plate-title">Buttons</span>
					</div>
				</div>
				<p class="field-hint plate-subtitle">
					{#if hasData}
						{buttons.length} button{buttons.length === 1 ? '' : 's'} reported by this device.
					{:else}
						{BUTTON_LABELS.length} buttons shown (default layout).{#if !device.connected} No mouse connected.{/if}
					{/if}
				</p>
				<ul class="button-list">
					{#each BUTTON_LABELS as label, i (i)}
						{@const unavailable = hasData && i >= buttons.length}
						<li>
							<button
								type="button"
								class="list-row"
								disabled={unavailable}
								onclick={() => selectHotspot(i)}
							>
								<span class="row-label">{label}</span>
								<span class="field-hint">
									{#if !hasData}
										-
									{:else if unavailable}
										Not on this device
									{:else}
										{actionSummary(buttons[i])}
									{/if}
								</span>
							</button>
						</li>
					{/each}
				</ul>
			</div>
		{:else}
			<div class="plate editor-plate">
				<div class="plate-head">
					<div class="plate-title-group">
						<span class="plate-stub"></span>
						<span class="plate-title">{BUTTON_LABELS[selected] ?? `Button ${selected + 1}`}</span>
					</div>
				</div>
				<p class="field-hint plate-subtitle">
					Currently: {hasData && selectedAction ? actionSummary(selectedAction) : '-'}
				</p>

				<div class="grid">
					<SelectField
						label="Action"
						value={draft.type}
						options={ASSIGNABLE_KINDS}
						unknown={!hasData}
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
							disabled={!hasData}
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
							disabled={!hasData}
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
							disabled={!hasData}
							onchange={(v) => draft && draft.type === 'scroll' && (draft.direction = v)}
						/>
					{:else if draft.type === 'fire'}
						<RangeField
							label="Times"
							hint="0 repeats for as long as the button is held"
							min={0}
							max={3}
							value={draft.times}
							disabled={!hasData}
							onchange={(v) => draft && draft.type === 'fire' && (draft.times = v)}
						/>
						<RangeField
							label="Interval"
							unit=" ms"
							min={10}
							max={255}
							value={draft.intervalMs}
							disabled={!hasData}
							onchange={(v) => draft && draft.type === 'fire' && (draft.intervalMs = v)}
						/>
					{:else if draft.type === 'keystroke'}
						<div class="field">
							<span class="field-label">Modifiers</span>
							<div class="modifier-grid">
								{#each MODIFIER_OPTIONS as mod (mod.value)}
									{@const pressed = keystrokeDraft.modifiers.includes(mod.value)}
									<button
										type="button"
										class="chip mod-chip"
										class:chip-active={pressed}
										aria-pressed={pressed}
										disabled={!hasData}
										onclick={() => toggleModifier(mod.value, !pressed)}
									>
										{mod.label}
									</button>
								{/each}
							</div>
						</div>
						<SelectField
							label="Key"
							value={keystrokeDraft.key ?? -1}
							options={[{ value: -1, label: 'None' }, ...KEY_OPTIONS]}
							disabled={!hasData}
							onchange={(v) => (keystrokeDraft = { ...keystrokeDraft, key: v === -1 ? null : v })}
						/>
						<p class="field-hint">
							{keystrokeLoaded
								? "Apply saves this chord to the button's keystroke slot."
								: 'Reading the chord currently saved on the device...'}
						</p>
					{:else if draft.type === 'macro'}
						<RangeField
							label="Macro slot"
							min={0}
							max={31}
							value={draft.slot}
							disabled={!hasData}
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
							disabled={!hasData}
							onchange={(v) => setCyclesKind(v)}
						/>
						{#if draft.cycles.cycles === 'times'}
							<RangeField
								label="Cycles"
								min={1}
								max={250}
								value={draft.cycles.n}
								disabled={!hasData}
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
							disabled={!hasData}
							onchange={(v) => draft && draft.type === 'media' && (draft.usage = v)}
						/>
					{/if}
				</div>

				<div class="actions-row">
					<button class="btn btn-primary" disabled={!hasData || saving} onclick={apply}>
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
		gap: var(--space-md);
		padding: 0 var(--space-lg) var(--space-lg);
		align-items: flex-start;
	}

	.art-column {
		width: 360px;
		flex-shrink: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-sm);
	}

	.art-caption {
		text-align: center;
	}

	.unsupported-note {
		text-align: center;
		color: var(--color-warning);
	}

	.content-column {
		flex: 1;
		min-width: 360px;
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

	.button-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
	}

	/* Rows separated by hairlines, not individually boxed (design.md: depth comes from the paper
	   steps and the hairlines, never a border-per-card). */
	.button-list > li + li {
		border-top: 1px solid var(--color-rule);
	}

	.list-row {
		width: 100%;
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-sm);
		background: transparent;
		border: none;
		padding: var(--space-xs) var(--space-2xs);
		text-align: left;
		font-size: var(--text-sm);
	}

	.list-row:hover:not(:disabled) {
		background: var(--color-paper-3);
	}

	.list-row:disabled {
		opacity: 0.45;
	}

	.row-label {
		font-weight: 500;
		color: var(--color-ink);
	}

	.actions-row {
		display: flex;
		gap: var(--space-2xs);
		margin-top: var(--space-md);
	}

	.error-text {
		color: var(--color-danger);
		margin-top: var(--space-sm);
	}

	.modifier-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: var(--space-2xs);
	}

	/* The chord being built is a live value, so its pressed modifiers earn the accent
	   (design.md "Accent appears only on live values..."); unpressed ones stay in the ordinary
	   .chip voice. */
	.mod-chip {
		width: 100%;
		justify-content: center;
	}

	.mod-chip:not(:disabled):hover {
		border-color: var(--color-faint);
	}

	.mod-chip:disabled {
		opacity: 0.45;
	}

	.mod-chip.chip-active {
		background: var(--color-accent-soft);
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	/* Below the window's minimum width, stack the mouse art above the button list/editor instead
	   of squeezing both into one row. */
	@media (max-width: 900px) {
		.layout {
			flex-direction: column;
		}

		.art-column {
			width: 100%;
		}
	}
</style>
