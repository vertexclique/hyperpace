<script lang="ts">
	import { applyButtonLabel, device, diffSettings } from '../device.svelte';
	import { BLANK_SETTINGS, DPI_INDICATOR_MODES, LIGHTING_MODES, RECEIVER_LIGHT_MODES } from '../types';
	import type {
		DpiIndicatorMode,
		DpiIndicatorModeName,
		LightMode,
		LightModeName,
		ReceiverLight,
		ReceiverLightState,
		RgbColor,
		Settings
	} from '../types';
	import SelectField from '../components/SelectField.svelte';
	import RangeField from '../components/RangeField.svelte';
	import Toggle from '../components/Toggle.svelte';
	import ColorSwatchPicker from '../components/ColorSwatchPicker.svelte';
	import ReceiverArt from '../art/ReceiverArt.svelte';
	import ArcDial from '../controls/ArcDial.svelte';
	import RangeSlider from '../controls/RangeSlider.svelte';

	// docs/research/mouse-protocol-v2.md 7.7: which controls a mode leaves
	// meaningful. Off disables everything, Fixed color has no animation
	// speed, the three cycling modes have no fixed color.
	function lightingControls(mode: LightModeName | 'other') {
		if (mode === 'off') return { color: false, speed: false };
		if (mode === 'fixed') return { color: true, speed: false };
		if (mode === 'rainbow' || mode === 'neon' || mode === 'rainbowBreath') {
			return { color: false, speed: true };
		}
		if (mode === 'other') return { color: false, speed: false };
		return { color: true, speed: true }; // singleColorBreath
	}

	// The select only offers the named variants (never `other`, which needs a `byte` field it
	// never supplies); narrowing here keeps SelectField's generic inferred as the named-only
	// union instead of widening, which would make `{ mode: v }` fail to satisfy `LightMode`.
	function lightModeName(mode: LightMode['mode']): LightModeName {
		return mode === 'other' ? 'off' : mode;
	}

	// Same narrowing as lightModeName above, for the DPI indicator's own mode enum.
	function dpiIndicatorModeName(mode: DpiIndicatorMode['mode']): DpiIndicatorModeName {
		return mode === 'other' ? 'off' : mode;
	}

	function modeLabel(mode: LightModeName): string {
		return LIGHTING_MODES.find((m) => m.value === mode)?.label ?? 'This effect';
	}

	/** The one-line reason a color or speed field is disabled for the active effect (design.md
	 * "no invented default": the field still shows the real reading, only editing is blocked). */
	function controlReason(kind: 'color' | 'speed', mode: LightModeName): string {
		return `${modeLabel(mode)} has no ${kind} control.`;
	}

	function toHex(c: RgbColor): string {
		return `#${c.map((n) => n.toString(16).padStart(2, '0')).join('')}`;
	}

	// Neither field's device-reported unit is documented by the vendor; shared so the two spots
	// that say so (receiver timeout, DPI indicator speed) can't drift apart in wording.
	const RAW_UNIT_HINT = 'Unit not confirmed by the vendor.';

	// `draft` is always a structurally complete Settings object, never null: when no device is
	// connected (or its settings have not been read yet) it holds BLANK_SETTINGS instead, so
	// every control below stays mounted. `hasData` gates each control's `disabled`/`unknown`
	// prop, which is what actually keeps a placeholder value from ever being shown as if it were
	// a real reading (see BLANK_SETTINGS's own doc comment).
	let draft = $state<Settings>(structuredClone(BLANK_SETTINGS));
	let saving = $state(false);
	let dirty = $state(false);
	let hasData = $derived(device.settings !== null);

	$effect(() => {
		// `device.settings` is a reactive proxy, which `structuredClone` refuses to copy (it throws,
		// and the draft silently stayed on the blank defaults); `$state.snapshot` is the copy made for it.
		draft = device.settings ? $state.snapshot(device.settings) : structuredClone(BLANK_SETTINGS);
		dirty = false;
	});

	let controls = $derived(lightingControls(draft.lighting.mode.mode));
	let activeModeName = $derived(lightModeName(draft.lighting.mode.mode));
	let activeEffectIndex = $derived(LIGHTING_MODES.findIndex((m) => m.value === activeModeName));
	let effectTabIndex = $derived(activeEffectIndex === -1 ? 0 : activeEffectIndex);

	function touch() {
		dirty = true;
	}

	function selectEffect(mode: LightModeName) {
		if (!hasData) return;
		draft.lighting = { ...draft.lighting, mode: { mode } };
		touch();
	}

	let effectTileRefs: (HTMLButtonElement | undefined)[] = $state([]);

	/** Roving-tabindex arrow-key navigation across the effect tile grid, the same keyboard
	 * contract a native radiogroup gives (Home/End to the ends, arrows to the neighbour). */
	function handleEffectKeydown(event: KeyboardEvent, index: number) {
		let next = index;
		switch (event.key) {
			case 'ArrowRight':
			case 'ArrowDown':
				next = (index + 1) % LIGHTING_MODES.length;
				break;
			case 'ArrowLeft':
			case 'ArrowUp':
				next = (index - 1 + LIGHTING_MODES.length) % LIGHTING_MODES.length;
				break;
			case 'Home':
				next = 0;
				break;
			case 'End':
				next = LIGHTING_MODES.length - 1;
				break;
			default:
				return;
		}
		event.preventDefault();
		selectEffect(LIGHTING_MODES[next].value);
		effectTileRefs[next]?.focus();
	}

	// The live preview strip near the mouse lighting controls: an honest read of the effect,
	// color, speed and brightness this screen is about to apply, not a decorative loop. Speed
	// 0..9 maps to a faster-cycling animation, brightness 0..9 to the strip's own opacity, and it
	// goes flat and still the moment lighting is off, matching the mouse rather than pretending.
	let previewDuration = $derived(Math.max(0.4, 3.6 - draft.lighting.speed * 0.32).toFixed(2));
	let previewOpacity = $derived((0.3 + (draft.lighting.brightness / 9) * 0.7).toFixed(2));
	let previewColorHex = $derived(toHex(draft.lighting.color));

	/** Sends one `write_setting` call per field that actually changed since the last read (see
	 * `diffSettings`); in practice just the lighting field, since this screen only edits that. */
	async function apply() {
		if (!device.settings) return;
		saving = true;
		try {
			await device.writeSettings(diffSettings(device.settings, draft));
			dirty = false;
		} finally {
			saving = false;
		}
	}

	// Receiver light (SetReceiverLight/GetReceiverLight, command 24/25).
	// Distinct from the mouse body lighting struct above and not part of
	// Settings, so it is edited and sent independently.
	const BLANK_RECEIVER_LIGHT: ReceiverLight = {
		mode: 0,
		color: [255, 255, 255],
		speed: 5,
		brightness: 5,
		time: 0
	};

	let receiverDraft = $state<ReceiverLight>(structuredClone(BLANK_RECEIVER_LIGHT));
	let receiverDirty = $state(false);
	let receiverSaving = $state(false);

	let receiverReportedLight = $derived.by((): ReceiverLight | null => {
		const state: ReceiverLightState | null = device.receiverLight;
		return state && state.state === 'reported' ? state.light : null;
	});
	let receiverUnsupported = $derived(device.receiverLight?.state === 'unsupported');
	let receiverHasData = $derived(receiverReportedLight !== null);
	/** No real reading exists yet, whether because the read has not completed or because the
	 * receiver reported it has no light to read; both show blank, never a guessed value. */
	let receiverUnknown = $derived(!receiverHasData);

	$effect(() => {
		// Same rule as the settings draft above: snapshot the reactive reading, never structuredClone it.
		receiverDraft = receiverReportedLight
			? $state.snapshot(receiverReportedLight)
			: structuredClone(BLANK_RECEIVER_LIGHT);
		receiverDirty = false;
	});

	function touchReceiver() {
		receiverDirty = true;
	}

	async function applyReceiverLight() {
		receiverSaving = true;
		try {
			await device.setReceiverLight(receiverDraft);
			receiverDirty = false;
		} finally {
			receiverSaving = false;
		}
	}

	let pairing = $state(false);

	async function startPairing() {
		pairing = true;
		try {
			await device.pairReceiver();
		} finally {
			pairing = false;
		}
	}

	// device.pairState streams every GetPairState poll live (phase and seconds remaining), but it
	// reports an in-progress pairing SESSION, not whether the receiver is bound to a mouse
	// (docs/research/mouse-protocol-v2.md 10.1). An idle/no-session reply means "no pairing is
	// running", never "not paired", so it must never render as that. A live pairing session (this
	// call, or one already running) takes priority; failing that, a wireless link is itself the
	// proof of pairing, since the mouse's packets are arriving through this receiver right now. A
	// wired connection, or no connection, proves nothing about the receiver's binding, so neither
	// ever renders a paired/not-paired claim.
	let receiverArtStatus = $derived.by((): 'unknown' | 'pairing' | 'paired' => {
		if (pairing) return 'pairing';
		const phase = device.pairState?.state.phase;
		if (phase === 'pairing') return 'pairing';
		if (phase === 'succeeded') return 'paired';
		if (device.connected && device.identity && !device.identity.wired) return 'paired';
		return 'unknown';
	});

	let pairingStatusText = $derived.by((): string | null => {
		const state = device.pairState;
		if (!state) return null;
		switch (state.state.phase) {
			case 'pairing':
				return `Pairing... ${state.secondsLeft}s left`;
			case 'succeeded':
				return 'Paired';
			case 'failed':
				return 'Pairing failed';
			case 'other':
				return `Unexpected pairing state (${state.state.byte})`;
		}
	});

	let confirmingReset = $state(false);

	async function doFactoryReset() {
		await device.factoryReset();
		confirmingReset = false;
	}
</script>

<div class="page-pad lighting-stack">
	<!-- Lead panel: the receiver's light, pairing and reset. Full width and first in reading
	     order, carrying the plate-stub, per this screen's brief ("Lighting leads with the
	     receiver light controls"). -->
	<section class="plate">
		<div class="plate-head">
			<h2><span class="plate-stub"></span>Receiver</h2>
		</div>
		<p class="plate-desc">
			The 2.4 GHz dongle: pairing, factory reset and its own indicator light.
			{#if !device.connected}<span class="caption-note">No mouse connected.</span>{/if}
		</p>
		<div class="receiver-layout">
			<div class="receiver-side">
				<ReceiverArt status={receiverArtStatus} />

				<div class="side-block">
					<div class="field-label">Pairing</div>
					<div class="field-hint">Hold left, right and wheel together for 3s after starting.</div>
					{#if pairingStatusText}
						<div class="field-hint">{pairingStatusText}</div>
					{/if}
					<button class="btn" disabled={!hasData || pairing} onclick={startPairing}>
						{pairing ? 'Starting...' : 'Start pairing'}
					</button>
				</div>

				<div class="side-block">
					<div class="field-label">Factory reset</div>
					<div class="field-hint">Erases all on-device settings back to their defaults.</div>
					{#if confirmingReset}
						<div class="confirm-row">
							<button class="btn btn-danger" onclick={doFactoryReset}>Confirm reset</button>
							<button class="btn" onclick={() => (confirmingReset = false)}>Cancel</button>
						</div>
					{:else}
						<button
							class="btn btn-danger"
							disabled={!hasData}
							onclick={() => (confirmingReset = true)}
						>
							Factory reset
						</button>
					{/if}
				</div>
			</div>

			<div class="field-stack receiver-fields">
				{#if device.connected && receiverUnsupported}
					<p class="field-hint caption-note">
						This receiver reports it has no light to read or set.
					</p>
				{/if}
				<div class="field-pair receiver-mode-row">
					<SelectField
						label="Receiver light effect"
						value={receiverDraft.mode}
						options={RECEIVER_LIGHT_MODES}
						unknown={receiverUnknown}
						onchange={(v) => {
							receiverDraft = { ...receiverDraft, mode: v };
							touchReceiver();
						}}
					/>
					<ColorSwatchPicker
						label="Color"
						color={receiverDraft.color}
						unknown={receiverUnknown}
						onchange={(c) => {
							receiverDraft = { ...receiverDraft, color: c };
							touchReceiver();
						}}
					/>
				</div>
				<div class="dial-row">
					<ArcDial
						label="Speed"
						ariaLabel="Receiver light speed"
						min={0}
						max={9}
						value={receiverDraft.speed}
						unknown={receiverUnknown}
						size="default"
						onchange={(v) => {
							receiverDraft = { ...receiverDraft, speed: v };
							touchReceiver();
						}}
					/>
					<ArcDial
						label="Brightness"
						ariaLabel="Receiver light brightness"
						min={0}
						max={9}
						value={receiverDraft.brightness}
						unknown={receiverUnknown}
						size="default"
						onchange={(v) => {
							receiverDraft = { ...receiverDraft, brightness: v };
							touchReceiver();
						}}
					/>
				</div>
				<RangeSlider
					label="Timeout (raw)"
					hint={RAW_UNIT_HINT}
					min={0}
					max={255}
					ticks={[0, 64, 128, 192, 255]}
					value={receiverDraft.time}
					unknown={receiverUnknown}
					onchange={(v) => {
						receiverDraft = { ...receiverDraft, time: v };
						touchReceiver();
					}}
				/>
				<button
					class="btn btn-primary apply-btn"
					disabled={!receiverHasData || !receiverDirty || receiverSaving}
					onclick={applyReceiverLight}
				>
					{applyButtonLabel(receiverHasData, receiverDirty, receiverSaving)}
				</button>
			</div>
		</div>
		{#if device.lastError}
			<p class="field-hint error-text">{device.lastError}</p>
		{/if}
	</section>

	<div class="light-row">
		<section class="plate light-col">
			<div class="plate-head">
				<h2><span class="plate-stub"></span>Mouse lighting</h2>
			</div>
			<p class="plate-desc">
				The body light strip: its effect, color, speed and brightness.
				{#if !device.connected}<span class="caption-note">No mouse connected.</span>{/if}
			</p>
			<div class="field-stack">
				<Toggle
					label="Lighting on"
					checked={draft.lighting.on}
					disabled={!hasData}
					onchange={(v) => {
						draft.lighting = { ...draft.lighting, on: v };
						touch();
					}}
				/>

				<div class="preview-row">
					<div
						class="preview-surface live-preview"
						data-effect={hasData && draft.lighting.on ? activeModeName : undefined}
						style="--tile-color:{previewColorHex}; --preview-duration:{previewDuration}s; --preview-opacity:{previewOpacity};"
						aria-hidden="true"
					></div>
					<span class="field-hint">
						{#if !hasData}No reading yet.
						{:else if !draft.lighting.on}Lighting is off.
						{:else}Live preview at the current speed and brightness.
						{/if}
					</span>
				</div>

				<div class="field">
					<span class="field-label">Effect</span>
					<div class="effect-grid" role="radiogroup" aria-label="Mouse lighting effect">
						{#each LIGHTING_MODES as option, i (option.value)}
							<button
								type="button"
								role="radio"
								aria-checked={activeModeName === option.value}
								class="effect-tile"
								class:active={activeModeName === option.value}
								disabled={!hasData}
								tabindex={!hasData ? -1 : i === effectTabIndex ? 0 : -1}
								bind:this={effectTileRefs[i]}
								onclick={() => selectEffect(option.value)}
								onkeydown={(e) => handleEffectKeydown(e, i)}
							>
								<span
									class="preview-surface effect-preview"
									data-effect={option.value}
									style="--tile-color:{previewColorHex}"
									aria-hidden="true"
								></span>
								<span class="effect-name">{option.label}</span>
							</button>
						{/each}
					</div>
				</div>

				<div class="field-pair">
					<div class="field-with-reason">
						<ColorSwatchPicker
							label="Color"
							color={draft.lighting.color}
							unknown={!hasData}
							disabled={!controls.color}
							onchange={(c) => {
								draft.lighting = { ...draft.lighting, color: c };
								touch();
							}}
						/>
						{#if hasData && !controls.color}
							<span class="caption-note">{controlReason('color', activeModeName)}</span>
						{/if}
					</div>
					<div class="field-with-reason">
						<RangeField
							label="Speed"
							min={0}
							max={9}
							value={draft.lighting.speed}
							unknown={!hasData}
							disabled={!controls.speed}
							onchange={(v) => {
								draft.lighting = { ...draft.lighting, speed: v };
								touch();
							}}
						/>
						{#if hasData && !controls.speed}
							<span class="caption-note">{controlReason('speed', activeModeName)}</span>
						{/if}
					</div>
				</div>
				<RangeField
					label="Brightness"
					min={0}
					max={9}
					value={draft.lighting.brightness}
					unknown={!hasData}
					onchange={(v) => {
						draft.lighting = { ...draft.lighting, brightness: v };
						touch();
					}}
				/>

				<button class="btn btn-primary apply-btn" disabled={!hasData || !dirty || saving} onclick={apply}>
					{applyButtonLabel(hasData, dirty, saving)}
				</button>
				{#if device.lastError}
					<p class="field-hint error-text">{device.lastError}</p>
				{/if}
			</div>
		</section>

		<section class="plate light-col">
			<div class="plate-head">
				<h2><span class="plate-stub"></span>DPI indicator</h2>
			</div>
			<p class="plate-desc">
				The per-stage indicator light: mode, brightness, speed and on/off.
				{#if !device.connected}<span class="caption-note">No mouse connected.</span>{/if}
			</p>
			<div class="field-stack">
				<Toggle
					label="Indicator on"
					checked={draft.dpiIndicator.on}
					disabled={!hasData}
					onchange={(v) => {
						draft.dpiIndicator = { ...draft.dpiIndicator, on: v };
						touch();
					}}
				/>
				<SelectField
					label="Effect"
					value={dpiIndicatorModeName(draft.dpiIndicator.mode.mode)}
					options={DPI_INDICATOR_MODES}
					unknown={!hasData}
					onchange={(v) => {
						draft.dpiIndicator = { ...draft.dpiIndicator, mode: { mode: v } };
						touch();
					}}
				/>
				<div class="field-pair">
					<RangeField
						label="Brightness"
						min={1}
						max={10}
						value={draft.dpiIndicator.brightness}
						unknown={!hasData}
						onchange={(v) => {
							draft.dpiIndicator = { ...draft.dpiIndicator, brightness: v };
							touch();
						}}
					/>
					<RangeField
						label="Speed (raw)"
						hint={RAW_UNIT_HINT}
						min={0}
						max={255}
						value={draft.dpiIndicator.speed}
						unknown={!hasData}
						onchange={(v) => {
							draft.dpiIndicator = { ...draft.dpiIndicator, speed: v };
							touch();
						}}
					/>
				</div>

				<button class="btn btn-primary apply-btn" disabled={!hasData || !dirty || saving} onclick={apply}>
					{applyButtonLabel(hasData, dirty, saving)}
				</button>
				{#if device.lastError}
					<p class="field-hint error-text">{device.lastError}</p>
				{/if}
			</div>
		</section>
	</div>
</div>

<style>
	.page-pad {
		padding: 0 var(--space-xl) var(--space-xl);
	}

	.lighting-stack {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.light-row {
		display: flex;
		gap: var(--space-sm);
		align-items: stretch;
	}

	.light-col {
		flex: 1 1 0;
		min-width: 0;
	}

	@media (max-width: 900px) {
		.light-row {
			flex-direction: column;
		}
	}

	/* Title row above its own hairline (.plate-head, app.css); the stub marks each panel as a
	   live, functioning surface rather than a static label. */
	.plate-head h2 {
		display: flex;
		align-items: center;
		gap: var(--space-2xs);
		margin: 0;
		font-family: var(--font-body);
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--color-ink);
	}

	.plate-desc {
		margin: 0 0 var(--space-sm);
		font-size: var(--text-xs);
		color: var(--color-muted);
	}

	.field-stack {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	/* Two fields sharing one row where both are short enough to (Color+Speed, the two receiver
	   dials' companion select+swatch): wraps to a single column below its own min-width rather
	   than compressing a slider or a select illegibly narrow. */
	.field-pair {
		display: flex;
		gap: var(--space-sm);
		flex-wrap: wrap;
	}

	.field-pair :global(.field) {
		flex: 1;
		min-width: 140px;
	}

	/* Wraps a field plus its one-line "not used by this effect" note (design.md's control-language
	   contract: a disabled control stays visible with a stated reason, never hidden outright). */
	.field-with-reason {
		flex: 1;
		min-width: 140px;
		display: flex;
		flex-direction: column;
		gap: var(--space-3xs);
	}

	/* The receiver's effect+color row: a select field genuinely wants the row's width (its option
	   labels), but the color swatch does not, and stretching it to match left a bare strip of
	   panel ground past the swatch. Only the swatch field drops the shared flex:1 here; the select
	   still grows to fill what the swatch does not want. */
	.receiver-mode-row :global(.field:last-child) {
		flex: 0 1 180px;
	}

	.receiver-layout {
		display: flex;
		gap: var(--space-md);
		align-items: flex-start;
		flex-wrap: wrap;
	}

	/* The art plus the pairing and reset actions share this narrow column, instead of running the
	   full width below the compose fields: keeps the lead panel's total height down to what the
	   default 1180x940 window can show without scrolling (docs/architecture/ui-controls.md
	   section 5), since this panel now carries everything the old three-panel grid split apart. */
	.receiver-side {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		width: 150px;
		flex-shrink: 0;
	}

	.receiver-side :global(.receiver-art) {
		width: 100%;
	}

	.side-block {
		display: flex;
		flex-direction: column;
		gap: var(--space-2xs);
		padding-top: var(--space-sm);
		border-top: 1px solid var(--color-rule-2);
	}

	.confirm-row {
		display: flex;
		flex-direction: column;
		gap: var(--space-2xs);
	}

	.receiver-fields {
		flex: 1;
		min-width: 260px;
	}

	/* .field-stack's column stretches every child to its own width by default (flex-direction:
	   column defaults align-items to stretch); a primary action is a button sized to its label,
	   not a bar (design.md CTA voice), so every Apply button on this screen opts back out. */
	.apply-btn {
		align-self: flex-start;
	}

	.dial-row {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		gap: var(--space-md);
		container-type: inline-size;
	}

	/* Base size is `default` (set on the ArcDial elements themselves): at `compact` the half-ring's
	   own caption and value text, drawn in the dial's fixed SVG viewBox, scaled down to a few
	   pixels and read as unlabeled stumps. Dials shrink a tier before the panel needs to scroll
	   (docs/architecture/ui-controls.md section 5), same rule as PerformanceScreen's own dial rows. */
	@container (max-width: 280px) {
		.dial-row {
			--dial-size: var(--dial-compact);
		}
	}

	.error-text {
		color: var(--color-danger);
	}

	/* A short caption noting why a panel's controls are disabled, not a placeholder replacing
	   them: the controls themselves stay rendered (see BLANK_SETTINGS in ../types.ts). */
	.caption-note {
		display: block;
		margin-top: var(--space-3xs);
		color: var(--color-faint);
	}

	/* --- Effect previews -----------------------------------------------------
	   One drawn surface, reused at tile size (the effect picker) and strip size (the live
	   preview): a per-effect [data-effect] attribute selects the background and animation, so
	   the two call sites can never draw a different picture of the same effect. No preview image
	   depicts these; the vendor's colors/color*.png files are flat swatches, not effect
	   animations (see PROVENANCE.md), so the motion is drawn here instead. */
	.preview-surface {
		background: var(--color-paper-3);
		border: 1px solid var(--color-rule-2);
	}

	.preview-surface[data-effect='rainbow'],
	.preview-surface[data-effect='rainbowBreath'] {
		background: linear-gradient(90deg, #ff3b30, #ff9500, #ffd60a, #34c759, #32ade6, #5e5ce6, #ff3b30);
		background-size: 200% 100%;
	}

	.preview-surface[data-effect='singleColorBreath'],
	.preview-surface[data-effect='fixed'] {
		background: var(--tile-color, var(--color-accent));
	}

	.preview-surface[data-effect='neon'] {
		background: linear-gradient(90deg, #ff2d95, #32ade6, #ff2d95, #32ade6);
		background-size: 300% 100%;
	}

	@keyframes preview-scroll {
		to {
			background-position: -200% 0;
		}
	}

	@keyframes preview-breathe {
		0%,
		100% {
			opacity: 0.35;
		}
		50% {
			opacity: 1;
		}
	}

	@keyframes preview-strobe {
		to {
			background-position: -300% 0;
		}
	}

	/* Looping animation is opt-in under reduced motion, same convention as ArcDial/RangeSlider/
	   SegmentDial's own settle transitions: nothing here animates unless the viewer allows it. */
	@media (prefers-reduced-motion: no-preference) {
		.preview-surface[data-effect='rainbow'],
		.preview-surface[data-effect='rainbowBreath'] {
			animation: preview-scroll 2.2s linear infinite;
		}

		.preview-surface[data-effect='rainbowBreath'] {
			animation:
				preview-scroll 2.2s linear infinite,
				preview-breathe 2s ease-in-out infinite;
		}

		.preview-surface[data-effect='singleColorBreath'] {
			animation: preview-breathe 1.8s ease-in-out infinite;
		}

		.preview-surface[data-effect='neon'] {
			animation: preview-strobe 0.8s steps(6) infinite;
		}

		.live-preview[data-effect] {
			animation-duration: var(--preview-duration, 2s);
		}
	}

	.preview-row {
		display: flex;
		flex-direction: column;
		gap: var(--space-3xs);
	}

	.live-preview {
		height: 22px;
		width: 100%;
	}

	/* The current brightness scales the strip's own intensity (the matching animation-duration
	   override lives in the reduced-motion media query above); only meaningful once it is
	   actually drawing an effect (data-effect is only set while lighting is on and a reading
	   exists, see the markup), so an off/unread strip stays the flat, still base above rather
	   than a bright loop pretending the mouse is lit. */
	.live-preview[data-effect] {
		opacity: var(--preview-opacity, 1);
	}

	/* --- Effect tile grid ------------------------------------------------------
	   The mode picker: every effect the device supports as its own tile (design.md "no screen may
	   hide a choice inside a control the operator has to open first"), not one option folded into
	   a <select>. Built as an ARIA radiogroup of plain buttons with roving tabindex, so arrow keys
	   move between tiles and Home/End jump to the ends, the same contract a native radio group
	   gives. */
	.effect-grid {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2xs);
	}

	.effect-tile {
		display: flex;
		flex-direction: column;
		gap: var(--space-3xs);
		flex: 1 1 30%;
		min-width: 100px;
		padding: var(--space-2xs);
		background: var(--color-paper);
		border: 1px solid var(--color-rule);
		color: var(--color-ink-2);
		font-family: var(--font-body);
		text-align: left;
		clip-path: polygon(0 0, calc(100% - var(--cut-chip)) 0, 100% var(--cut-chip), 100% 100%, 0 100%);
		transition:
			border-color var(--dur-settle) var(--ease-out),
			background var(--dur-settle) var(--ease-out);
	}

	.effect-tile:hover:not(:disabled) {
		border-color: var(--color-faint);
	}

	.effect-tile.active {
		border-color: var(--color-accent);
		background: var(--color-accent-soft);
		color: var(--color-ink);
	}

	.effect-tile:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}

	.effect-preview {
		height: 16px;
		width: 100%;
	}

	.effect-name {
		font-size: var(--text-xs);
		font-weight: 500;
	}
</style>
