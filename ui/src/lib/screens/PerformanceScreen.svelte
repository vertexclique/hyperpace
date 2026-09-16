<script lang="ts">
	import { applyButtonLabel, device, diffSettings } from '../device.svelte';
	import { BLANK_SETTINGS } from '../types';
	import type { DpiStage, Lod, Settings, SleepTime } from '../types';
	import Toggle from '../components/Toggle.svelte';
	import ColorSwatchPicker from '../components/ColorSwatchPicker.svelte';
	import SegmentDial from '../controls/SegmentDial.svelte';
	import RangeSlider from '../controls/RangeSlider.svelte';

	const POLLING_OPTIONS = [125, 250, 500, 1000, 2000, 4000, 8000].map((hz) => ({
		value: hz,
		label: `${hz} Hz`
	}));

	const LOD_OPTIONS: { value: Exclude<Lod['value'], 'other'>; label: string }[] = [
		{ value: 'oneMillimeter', label: '1 mm' },
		{ value: 'twoMillimeters', label: '2 mm' },
		{ value: 'pointSevenMillimeters', label: '0.7 mm' }
	];

	// mouse-protocol-v2.md 7.8: the same named codes shared by sleep timing and the highest
	// performance timeout (SleepTimeDto).
	const TIMEOUT_OPTIONS: { value: Exclude<SleepTime['value'], 'other'>; label: string }[] = [
		{ value: 'tenSeconds', label: '10 s' },
		{ value: 'thirtySeconds', label: '30 s' },
		{ value: 'oneMinute', label: '1 min' },
		{ value: 'twoMinutes', label: '2 min' },
		{ value: 'fiveMinutes', label: '5 min' },
		{ value: 'tenMinutes', label: '10 min' },
		{ value: 'fifteenMinutes', label: '15 min' }
	];

	const SENSOR_MODE_OPTIONS = [
		{ value: 0, label: 'Low power' },
		{ value: 1, label: 'High performance' }
	];

	const MAX_DPI_STAGES = 8;

	// DPI per stage stays a linear control (docs/architecture/ui-controls.md section 2.3); these
	// bounds cover both model tables' documented range (`maxDpi` 40000 OLD / 32000 NEW, step 50 for
	// the common range: docs/research/mouse-protocol-v2.md), since the frontend does not know which
	// table the connected device uses. A value the protocol codec cannot represent still surfaces
	// as a write error the same way the free-typed number field it replaces already could.
	const DPI_MIN = 50;
	const DPI_MAX = 40000;
	const DPI_STEP = 50;
	// Starts at the real floor (DPI_MIN), not at whatever a stage's current value happens to be.
	const DPI_TICKS = [DPI_MIN, 8000, 16000, 24000, 32000, DPI_MAX];

	// The dial only offers the named variants (never `other`), so reading its bound value needs a
	// narrowed read: this also keeps SegmentDial's generic inferred as the named-only union
	// (excluding `other`, which needs a `byte` field the dial never supplies) instead of widening
	// to the full wire type, which would make `{ value: v }` fail to satisfy `Lod` / `SleepTime`
	// below.
	function lodName(value: Lod['value']): Exclude<Lod['value'], 'other'> {
		return value === 'other' ? 'oneMillimeter' : value;
	}
	function sleepTimeName(value: SleepTime['value']): Exclude<SleepTime['value'], 'other'> {
		return value === 'other' ? 'tenSeconds' : value;
	}
	function timeoutLabel(value: SleepTime['value']): string {
		return TIMEOUT_OPTIONS.find((option) => option.value === sleepTimeName(value))?.label ?? '';
	}

	// See LightingScreen for why `draft` is always a full Settings object (BLANK_SETTINGS when
	// disconnected) instead of null, and what `hasData` gates.
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

	function touch() {
		dirty = true;
	}

	function addStage() {
		if (!hasData || draft.dpiStages.length >= MAX_DPI_STAGES) return;
		const last = draft.dpiStages.at(-1);
		draft.dpiStages.push({
			dpi: last ? last.dpi : 800,
			color: last ? [...last.color] : [255, 255, 255]
		});
		touch();
	}

	function removeStage(index: number) {
		if (!hasData || draft.dpiStages.length <= 1) return;
		draft.dpiStages.splice(index, 1);
		if (draft.currentStage >= draft.dpiStages.length) {
			draft.currentStage = draft.dpiStages.length - 1;
		}
		touch();
	}

	function updateStage(index: number, patch: Partial<DpiStage>) {
		if (!hasData) return;
		draft.dpiStages[index] = { ...draft.dpiStages[index], ...patch };
		touch();
	}

	/** Sends one `write_setting` call per field that actually changed since the last read, since
	 * the command has no batch variant (see `diffSettings`). */
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
</script>

{#if !device.connected}
	<p class="page-pad caption-note">No mouse connected. Plug in the receiver or the cable.</p>
{/if}
<div class="perf-stack">
	<div class="perf-row">
		<section class="plate plate-dpi">
			<div class="plate-head">
				<h2><span class="plate-stub"></span>DPI stages</h2>
				<span class="field-hint">Up to {MAX_DPI_STAGES} &middot; active stage drives the DPI switch</span>
			</div>
			<div class="stage-list">
				{#each draft.dpiStages as stage, i (i)}
					<div class="stage-row" class:current={draft.currentStage === i}>
						<button
							type="button"
							class="stage-index"
							aria-pressed={draft.currentStage === i}
							title="Set as active stage"
							disabled={!hasData}
							onclick={() => {
								draft.currentStage = i;
								touch();
							}}
						>
							{i + 1}
						</button>
						<div class="stage-value">
							<span class="stage-value-num mono">{hasData ? stage.dpi : '-'}</span>
							<span class="stage-value-unit">DPI</span>
						</div>
						<div class="stage-slider">
							<RangeSlider
								label={`DPI stage ${i + 1}`}
								min={DPI_MIN}
								max={DPI_MAX}
								step={DPI_STEP}
								ticks={DPI_TICKS}
								value={stage.dpi}
								unknown={!hasData}
								accentColor={`rgb(${stage.color.join(',')})`}
								onchange={(v) => updateStage(i, { dpi: v })}
							/>
						</div>
						<div class="stage-color">
							<ColorSwatchPicker
								color={stage.color}
								unknown={!hasData}
								onchange={(c) => updateStage(i, { color: c })}
							/>
						</div>
						<button
							type="button"
							class="stage-remove"
							disabled={!hasData || draft.dpiStages.length <= 1}
							aria-label={`Remove DPI stage ${i + 1}`}
							title="Remove stage"
							onclick={() => removeStage(i)}
						>
							<svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
								<path d="M4 4l8 8M12 4l-8 8" />
							</svg>
						</button>
					</div>
				{/each}
			</div>
			<button class="btn" disabled={!hasData || draft.dpiStages.length >= MAX_DPI_STAGES} onclick={addStage}>
				Add stage
			</button>
		</section>

		<section class="plate plate-tracking">
			<div class="plate-head">
				<h2>Tracking</h2>
				<span class="field-hint">Polling, lift-off, sensor mode</span>
			</div>
			<div class="dial-row">
				<div class="dial-field">
					<span class="field-label">Sensor mode</span>
					<SegmentDial
						values={SENSOR_MODE_OPTIONS}
						value={draft.sensorMode}
						ariaLabel="Sensor mode"
						unknown={!hasData}
						size="compact"
						onchange={(v) => {
							draft.sensorMode = v;
							touch();
						}}
					/>
				</div>
				<div class="dial-field">
					<span class="field-label">Lift-off distance</span>
					<SegmentDial
						values={LOD_OPTIONS}
						value={lodName(draft.lod.value)}
						ariaLabel="Lift-off distance"
						unknown={!hasData}
						size="compact"
						onchange={(v) => {
							draft.lod = { value: v };
							touch();
						}}
					/>
				</div>
			</div>
			<div class="linear-row">
				<RangeSlider
					label="Polling rate"
					stops={POLLING_OPTIONS}
					value={draft.pollingHz}
					unknown={!hasData}
					onchange={(v) => {
						draft.pollingHz = v;
						touch();
					}}
				/>
				<RangeSlider
					label="Debounce"
					unit=" ms"
					min={0}
					max={25}
					ticks={[0, 5, 10, 15, 20, 25]}
					value={draft.debounceMs}
					unknown={!hasData}
					onchange={(v) => {
						draft.debounceMs = v;
						touch();
					}}
				/>
			</div>
		</section>
	</div>

	<div class="perf-row">
		<section class="plate plate-motion">
			<div class="plate-head">
				<h2>Motion</h2>
			</div>
			<div class="field-stack">
				<Toggle
					label="Motion sync"
					checked={draft.motionSync}
					disabled={!hasData}
					onchange={(v) => {
						draft.motionSync = v;
						touch();
					}}
				/>
				<Toggle
					label="Angle snap"
					checked={draft.angleSnap}
					disabled={!hasData}
					onchange={(v) => {
						draft.angleSnap = v;
						touch();
					}}
				/>
				<Toggle
					label="Ripple control"
					checked={draft.ripple}
					disabled={!hasData}
					onchange={(v) => {
						draft.ripple = v;
						touch();
					}}
				/>
			</div>
		</section>

		<section class="plate plate-longrange">
			<div class="plate-head">
				<h2>Long range</h2>
			</div>
			{#if hasData && draft.longRange === 'unsupported'}
				<p class="field-hint">Not supported on this device.</p>
			{:else}
				<Toggle
					label="Long range mode"
					checked={draft.longRange === 'on'}
					disabled={!hasData}
					onchange={(v) => {
						draft.longRange = v ? 'on' : 'off';
						touch();
					}}
				/>
			{/if}
		</section>

		<section class="plate plate-power">
			<div class="plate-head">
				<h2><span class="plate-stub"></span>Power</h2>
			</div>
			{#snippet performanceToggle()}
				<button
					type="button"
					class="dial-power"
					class:on={hasData && draft.performance.on}
					disabled={!hasData}
					aria-pressed={hasData && draft.performance.on}
					aria-label="Highest performance mode: keeps the sensor at full power instead of stepping down"
					title="Highest performance mode: keeps the sensor at full power instead of stepping down"
					onclick={() => {
						draft.performance = { ...draft.performance, on: !draft.performance.on };
						touch();
					}}
				>
					<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
						<path d="M12 3v8" />
						<path d="M6.5 6.5a8 8 0 1 0 11 0" />
					</svg>
				</button>
			{/snippet}
			<div class="dial-row dial-row-hero">
				<div class="dial-field">
					<span class="field-label">Sleep after</span>
					<SegmentDial
						values={TIMEOUT_OPTIONS}
						value={sleepTimeName(draft.sleep.value)}
						ariaLabel="Sleep after"
						unknown={!hasData}
						size="default"
						centerLabel={timeoutLabel(draft.sleep.value)}
						onchange={(v) => {
							draft.sleep = { value: v };
							touch();
						}}
					/>
				</div>
				<div class="dial-field">
					<span class="field-label">Highest performance</span>
					<SegmentDial
						values={TIMEOUT_OPTIONS}
						value={sleepTimeName(draft.performance.timeout.value)}
						ariaLabel="Highest performance timeout"
						unknown={!hasData}
						size="default"
						centerLabel={timeoutLabel(draft.performance.timeout.value)}
						centerAction={performanceToggle}
						onchange={(v) => {
							draft.performance = { ...draft.performance, timeout: { value: v } };
							touch();
						}}
					/>
				</div>
			</div>
		</section>
	</div>
</div>

<div class="apply-bar">
	<button class="btn btn-primary" disabled={!hasData || !dirty || saving} onclick={apply}>
		{applyButtonLabel(hasData, dirty, saving)}
	</button>
	{#if device.lastError}
		<span class="field-hint error-text">{device.lastError}</span>
	{/if}
</div>

<style>
	.page-pad {
		padding: 0 var(--space-xl) var(--space-xl);
	}

	/* Explicit rows, not an auto-flowing grid: each row needs real, predictable width to hold its
	   panels side by side, so DPI/Tracking share one row and Motion/Long range/Power share the
	   next. Rows align to the top, not stretched to match height, so a shorter panel never carries
	   a slab of empty space just to match a taller sibling. Padding is tightened from the shared
	   .plate default (space-md all round) to space-sm vertically so this screen's six panels clear
	   the default window without scrolling (docs/architecture/ui-controls.md section 5); the
	   horizontal space-md stays, so panel content keeps its breathing room side to side. */
	.perf-stack {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		padding: 0 var(--space-xl) var(--space-md);
	}

	.perf-stack .plate {
		padding: var(--space-sm) var(--space-md);
	}

	.perf-row {
		display: flex;
		gap: var(--space-sm);
		align-items: flex-start;
	}

	.perf-row > .plate-dpi {
		flex: 1 1 0;
		min-width: 0;
	}

	.perf-row > .plate-tracking {
		flex: 1.1 1 0;
		min-width: 0;
	}

	.perf-row > .plate-motion,
	.perf-row > .plate-longrange {
		flex: 1 1 0;
		min-width: 0;
	}

	.perf-row > .plate-power {
		flex: 1.6 1 0;
		min-width: 0;
	}

	@media (max-width: 900px) {
		.perf-row {
			flex-direction: column;
		}
	}

	/* Title row above its own hairline (.plate-head, app.css); the stub marks DPI stages and
	   Power as this screen's two lead panels. The inline hint on the right keeps a panel's one
	   line of context without a whole extra paragraph row eating vertical space. */
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

	.plate-head .field-hint {
		white-space: nowrap;
	}

	.field-stack {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	.linear-row {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-md);
	}

	.linear-row > :global(*) {
		flex: 1;
		min-width: 150px;
	}

	.stage-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-3xs);
		margin-bottom: var(--space-xs);
	}

	/* One line per stage, about 40px tall: an index, the reading, an inline slider spanning the
	   row, the stored colour and a small remove control. Up to MAX_DPI_STAGES (8) of these stack
	   without pushing anything below the fold. */
	.stage-row {
		display: flex;
		align-items: center;
		gap: var(--space-2xs);
		min-height: 40px;
		padding: var(--space-3xs) var(--space-2xs);
		background: var(--color-paper-2);
		border: 1px solid var(--color-rule);
	}

	.stage-row.current {
		border-color: var(--color-accent);
	}

	/* A numbered index badge, not a device reading: cut at --cut-chip like every other swatch and
	   badge in the app (design.md "Shape"), never the circle it used to be. */
	.stage-index {
		width: 24px;
		height: 24px;
		background: var(--color-paper-3);
		border: 1px solid var(--color-rule);
		color: var(--color-ink-2);
		font-size: var(--text-xs);
		font-weight: 700;
		flex-shrink: 0;
		clip-path: polygon(0 0, calc(100% - var(--cut-chip)) 0, 100% var(--cut-chip), 100% 100%, 0 100%);
	}

	.stage-row.current .stage-index {
		background: var(--color-accent);
		color: var(--color-accent-ink);
		border-color: var(--color-accent);
	}

	/* The stage's actual DPI reading: a real instrument readout, large and in mono, the one number
	   in this row a device actually reports (design.md Typography). */
	.stage-value {
		display: flex;
		align-items: baseline;
		gap: var(--space-3xs);
		flex-shrink: 0;
		min-width: 4.2em;
	}

	.stage-value-num {
		font-size: var(--text-lg);
		font-weight: 600;
		color: var(--color-ink);
		text-align: right;
	}

	.stage-value-unit {
		font-size: var(--text-xs);
		color: var(--color-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	/* The row's own DPI reading and swatch already carry the value and colour; RangeSlider's own
	   value bubble and tick-row text would just repeat them at this row height, so both are
	   dropped, leaving the bare track (with its tick marks) spanning the row. Its accessible name
	   still comes from the `aria-label` on the slider element itself, not this hidden label.
	   `.value-row` is collapsed to zero height rather than `display: none`: its label and its
	   edit-on-Enter input are both absolutely positioned inside it (so the zero height never clips
	   either one), and a `display: none` ancestor would make the edit input unfocusable, silently
	   breaking the "press Enter to type an exact value" path this row must keep. */
	.stage-slider {
		flex: 1;
		min-width: 90px;
	}

	.stage-slider :global(.field-label),
	.stage-slider :global(.value-label),
	.stage-slider :global(.tick-row) {
		display: none;
	}

	.stage-slider :global(.value-row) {
		height: 0;
	}

	.stage-slider :global(.field) {
		gap: 0;
	}

	/* ColorSwatchPicker renders its own preview chip alongside a native colour input and a hex
	   caption, which at this row's width read as two blank white squares (the operator's words)
	   instead of one swatch. The native input stays fully functional and in the tab order, just
	   stacked invisibly over the one visible cut-chip swatch so it still opens the native picker
	   on click; the hex caption is dropped since the swatch already shows the colour. Overridden
	   from outside since ColorSwatchPicker.svelte is not this row's file to change; !important
	   guards against a tie with that component's own same-specificity scoped rules. */
	.stage-color :global(.swatch-row) {
		position: relative;
	}

	.stage-color :global(input[type='color']) {
		position: absolute;
		inset: 0;
		width: 100% !important;
		height: 100% !important;
		padding: 0;
		border: 0 !important;
		opacity: 0;
		cursor: pointer;
	}

	.stage-color :global(.field-hint) {
		display: none;
	}

	.stage-color :global(.swatch) {
		width: 22px !important;
		height: 22px !important;
	}

	/* A plain icon button, danger only on hover/focus, never a persistent red-outlined box
	   (the operator's "DPI Stages looks shit" was in part five of these per screen). */
	.stage-remove {
		flex-shrink: 0;
		width: 26px;
		height: 26px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: transparent;
		border: 1px solid transparent;
		color: var(--color-muted);
		clip-path: polygon(0 0, calc(100% - var(--cut-chip)) 0, 100% var(--cut-chip), 100% 100%, 0 100%);
	}

	.stage-remove:hover:not(:disabled) {
		border-color: var(--color-danger);
		color: var(--color-danger);
		background: var(--color-danger-soft);
	}

	.stage-remove:disabled {
		opacity: 0.35;
	}

	.dial-row {
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: var(--space-lg);
		margin-bottom: var(--space-sm);
		container-type: inline-size;
	}

	.dial-field {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-2xs);
		text-align: center;
	}

	/* Dials drop a tier before their panel ever needs to scroll
	   (docs/architecture/ui-controls.md section 5): the Power panel's pair starts at the default
	   tier already (chosen explicitly below, not "large") and steps down to compact only if its
	   own container gets unusually narrow. */
	@container (max-width: 380px) {
		.dial-row-hero {
			--dial-size: var(--dial-compact);
		}
	}

	/* The highest-performance on/off button, drawn inside the hero dial's own centre: a plain
	   cut-chip square, never the circle it used to be (design.md "Shape: nothing is round" excepts
	   only the dial face bitmaps themselves, not the chrome sitting on top of one). */
	.dial-power {
		width: 100%;
		height: 100%;
		background: var(--color-paper-3);
		border: 1px solid var(--color-rule);
		color: var(--color-muted);
		display: flex;
		align-items: center;
		justify-content: center;
		clip-path: polygon(0 0, calc(100% - var(--cut-chip)) 0, 100% var(--cut-chip), 100% 100%, 0 100%);
	}

	.dial-power.on {
		background: var(--color-accent);
		border-color: var(--color-accent);
		color: var(--color-accent-ink);
	}

	.dial-power:disabled {
		opacity: 0.45;
	}

	/* A solid ground, not a fade to transparent: the bar sits below the panels rather than
	   bleeding into whatever the last row happened to draw beneath it. */
	.apply-bar {
		position: sticky;
		bottom: 0;
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		padding: var(--space-sm) var(--space-xl);
		border-top: 1px solid var(--color-rule);
		background: var(--color-paper);
	}

	.error-text {
		color: var(--color-danger);
	}

	.caption-note {
		margin: 0 0 var(--space-3xs);
		color: var(--color-faint);
	}
</style>
