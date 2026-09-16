<script lang="ts">
	import { applyButtonLabel, device, diffSettings } from '../device.svelte';
	import { BLANK_SETTINGS, DPI_INDICATOR_MODES, LIGHTING_MODES, RECEIVER_LIGHT_MODES } from '../types';
	import type {
		DpiIndicatorMode,
		DpiIndicatorModeName,
		LightMode,
		LightModeName,
		ReceiverLight,
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

	function touch() {
		dirty = true;
	}

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
	let receiverLight = $state<ReceiverLight>({
		mode: 0,
		color: [255, 255, 255],
		speed: 5,
		brightness: 5,
		time: 0
	});
	let receiverSaving = $state(false);

	// Honesty fix: unlike every other control on this screen, the receiver light has no read path
	// at all. The protocol has a GetReceiverLight command (mouse-protocol-v2.md 7.x, command 25),
	// and the device layer can round-trip it (hyperpace-device::owner), but no Tauri command
	// exposes that read to the frontend: crates/hyperpace-app/src/command_list.rs registers only
	// the write-only `receiver_light`. So `receiverLight` above is never a device reading, only a
	// draft the operator composes to send. Gating its controls on `hasData` (whether the *mouse's*
	// Settings loaded) showed these plain JS defaults ("Speed 5", "Brightness 5", "#ffffff",
	// "Timeout 0") the moment any device connected, as though the receiver had reported them.
	// Until a real read command exists, every control bound to it stays `unknown`, the same
	// disabled-and-blank contract every reading-backed control here already honors (see
	// BLANK_SETTINGS's doc comment and SegmentDial's `unknown` prop doc). Always `true`, not a
	// derived value, because nothing in this app can ever make it otherwise today.
	const RECEIVER_LIGHT_UNKNOWN = true;

	async function applyReceiverLight() {
		receiverSaving = true;
		try {
			await device.setReceiverLight(receiverLight);
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
			The 2.4 GHz dongle: its light, pairing and a factory reset. The receiver never reports its
			light settings back, so the controls below stay disabled until that changes.
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
				<div class="field-pair receiver-mode-row">
					<SelectField
						label="Receiver light effect"
						value={receiverLight.mode}
						options={RECEIVER_LIGHT_MODES}
						unknown={RECEIVER_LIGHT_UNKNOWN}
						onchange={(v) => (receiverLight = { ...receiverLight, mode: v })}
					/>
					<ColorSwatchPicker
						label="Color"
						color={receiverLight.color}
						unknown={RECEIVER_LIGHT_UNKNOWN}
						onchange={(c) => (receiverLight = { ...receiverLight, color: c })}
					/>
				</div>
				<div class="dial-row">
					<ArcDial
						label="Speed"
						ariaLabel="Receiver light speed"
						min={0}
						max={9}
						value={receiverLight.speed}
						unknown={RECEIVER_LIGHT_UNKNOWN}
						size="large"
						onchange={(v) => (receiverLight = { ...receiverLight, speed: v })}
					/>
					<ArcDial
						label="Brightness"
						ariaLabel="Receiver light brightness"
						min={0}
						max={9}
						value={receiverLight.brightness}
						unknown={RECEIVER_LIGHT_UNKNOWN}
						size="large"
						onchange={(v) => (receiverLight = { ...receiverLight, brightness: v })}
					/>
				</div>
				<RangeSlider
					label="Timeout (raw)"
					hint="Unit is not specified in the protocol reference"
					min={0}
					max={255}
					ticks={[0, 64, 128, 192, 255]}
					value={receiverLight.time}
					unknown={RECEIVER_LIGHT_UNKNOWN}
					onchange={(v) => (receiverLight = { ...receiverLight, time: v })}
				/>
				<button
					class="btn btn-primary apply-receiver-btn"
					disabled={!hasData || receiverSaving}
					onclick={applyReceiverLight}
				>
					{receiverSaving ? 'Applying...' : 'Apply receiver light'}
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
				<h2>Mouse lighting</h2>
			</div>
			<p class="plate-desc">
				The body light strip and its effect.
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
				<SelectField
					label="Effect"
					value={lightModeName(draft.lighting.mode.mode)}
					options={LIGHTING_MODES}
					unknown={!hasData}
					onchange={(v) => {
						draft.lighting = { ...draft.lighting, mode: { mode: v } };
						touch();
					}}
				/>
				{#if controls.color || controls.speed}
					<div class="field-pair">
						{#if controls.color}
							<ColorSwatchPicker
								label="Color"
								color={draft.lighting.color}
								unknown={!hasData}
								onchange={(c) => {
									draft.lighting = { ...draft.lighting, color: c };
									touch();
								}}
							/>
						{/if}
						{#if controls.speed}
							<RangeField
								label="Speed"
								min={0}
								max={9}
								value={draft.lighting.speed}
								unknown={!hasData}
								onchange={(v) => {
									draft.lighting = { ...draft.lighting, speed: v };
									touch();
								}}
							/>
						{/if}
					</div>
				{/if}
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

				<button class="btn btn-primary" disabled={!hasData || !dirty || saving} onclick={apply}>
					{applyButtonLabel(hasData, dirty, saving)}
				</button>
				{#if device.lastError}
					<p class="field-hint error-text">{device.lastError}</p>
				{/if}
			</div>
		</section>

		<section class="plate light-col">
			<div class="plate-head">
				<h2>DPI indicator</h2>
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
						hint="Unit is not specified in the protocol reference"
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

				<button class="btn btn-primary" disabled={!hasData || !dirty || saving} onclick={apply}>
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

	/* Title row above its own hairline (.plate-head, app.css); the stub marks Receiver as this
	   screen's lead panel. */
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

	/* .field-stack's column stretches every child to its own width (design.md CTA voice: a
	   primary action is a button, not a bar); this is the one primary action on the panel, so it
	   is sized to its label like every other button in the app instead of spanning the column. */
	.apply-receiver-btn {
		align-self: flex-start;
	}

	.dial-row {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		gap: var(--space-md);
		container-type: inline-size;
	}

	/* Base size is `large` (set on the ArcDial elements themselves): at `compact` the half-ring's
	   own caption and value text, drawn in the dial's fixed SVG viewBox, scaled down to a few
	   pixels and read as unlabeled stumps. Dials shrink a tier before the panel needs to scroll
	   (docs/architecture/ui-controls.md section 5), same rule as PerformanceScreen's own dial rows. */
	@container (max-width: 460px) {
		.dial-row {
			--dial-size: var(--dial-default);
		}
	}

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
</style>
