<script lang="ts">
	import { device } from '../device.svelte';
	import { BLANK_SETTINGS } from '../types';
	import type { DpiStage, Lod, RgbColor, Settings, SleepTime, WriteSettingRequest } from '../types';
	import Toggle from '../components/Toggle.svelte';
	import ColorSwatchPicker from '../components/ColorSwatchPicker.svelte';
	import HelpTip from '../components/HelpTip.svelte';
	import SegmentDial from '../controls/SegmentDial.svelte';
	import SegmentedControl from '../controls/SegmentedControl.svelte';
	import RangeSlider from '../controls/RangeSlider.svelte';

	// Seven segments leave little width each, and repeating the unit in every one of them truncated
	// the labels ("250 Hz" -> "250 ...", the operator's own screenshot, 2026-09-16). The unit is
	// stated once, on the field's label, and the segments carry the numbers alone.
	const POLLING_OPTIONS_ALL = [125, 250, 500, 1000, 2000, 4000, 8000].map((hz) => ({
		value: hz,
		label: hz >= 1000 ? `${hz / 1000}k` : `${hz}`
	}));

	const LOD_OPTIONS: { value: Exclude<Lod['value'], 'other'>; label: string }[] = [
		{ value: 'oneMillimeter', label: '1 mm' },
		{ value: 'twoMillimeters', label: '2 mm' },
		{ value: 'pointSevenMillimeters', label: '.7 mm' }
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

	// "High performance" truncated in a two-way segmented control at this column's width
	// (operator's own screenshot, 2026-09-16); "High perf" keeps it to one line at the same length
	// as "Low power" rather than a random mid-word cut.
	const SENSOR_MODE_OPTIONS = [
		{ value: 0, label: 'Low power' },
		{ value: 1, label: 'High perf' }
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
	// A doubling sequence, not an even split of the raw number range: a stage's real value is
	// almost always a four-figure number (800-6400 is the whole first page of most DPI tables),
	// a sliver of a linear 50-40000 scale, so the slider itself is logarithmic (`scale="log"`
	// below) and these stops read sensibly against that scale (operator feedback, 2026-09-16).
	const DPI_TICKS = [DPI_MIN, 100, 200, 400, 800, 1600, 3200, 6400, 12800, 25600, DPI_MAX];

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
	// disconnected) instead of null, and what `hasData` gates. There is no separate "dirty"/"apply"
	// state any more: every control below writes the instant it is committed (design.md
	// "Microinteractions stance"), so `draft` is always either the device's last confirmed reading
	// or a control's own optimistic edit while that edit's write is in flight.
	let draft = $state<Settings>(structuredClone(BLANK_SETTINGS));
	let hasData = $derived(device.settings !== null);

	$effect(() => {
		// `device.settings` is a reactive proxy, which `structuredClone` refuses to copy (it throws);
		// `$state.snapshot` is the copy made for it.
		draft = device.settings ? $state.snapshot(device.settings) : structuredClone(BLANK_SETTINGS);
	});

	// The rates this link actually allows (`identity.maxPollingHz`), so a link capped below 8000 Hz
	// never offers a rate it cannot carry. Unfiltered before an identity has been read, since the
	// slider is disabled by `unknown` until then regardless.
	let pollingOptions = $derived(
		device.identity
			? POLLING_OPTIONS_ALL.filter((option) => option.value <= device.identity!.maxPollingHz)
			: POLLING_OPTIONS_ALL
	);

	// One message per control that just failed to write, keyed by a short id local to this screen
	// (`sensorMode`, `dpiStage:2`, ...); see `commit` below.
	let fieldErrors = $state<Record<string, string>>({});
	function setFieldError(id: string, message: string) {
		fieldErrors = { ...fieldErrors, [id]: message };
	}
	function clearFieldError(id: string) {
		if (!(id in fieldErrors)) return;
		const next = { ...fieldErrors };
		delete next[id];
		fieldErrors = next;
	}

	/**
	 * Instant apply, once, for every control on this screen: shows `value` immediately via `write`,
	 * sends `request` through `device.commitSetting`, and on failure puts the control back to what
	 * `read` reported before the edit, with the failure message attached to `id` for that one
	 * control to show. On success the field's own error (if any was still showing) is cleared and
	 * nothing else happens, since a landed write is silent (design.md "Microinteractions stance").
	 */
	async function commit<T>(
		id: string,
		read: () => T,
		write: (value: T) => void,
		value: T,
		request: WriteSettingRequest | WriteSettingRequest[]
	) {
		const previous = read();
		write(value);
		const error = await device.commitSetting(request);
		if (error) {
			write(previous);
			setFieldError(id, error);
		} else {
			clearFieldError(id);
		}
	}

	function addStage() {
		if (!hasData || draft.dpiStages.length >= MAX_DPI_STAGES) return;
		const last = draft.dpiStages.at(-1);
		const stage: DpiStage = {
			dpi: last ? last.dpi : 800,
			color: last ? [...last.color] : [255, 255, 255]
		};
		const index = draft.dpiStages.length;
		commit(
			'dpiStages',
			() => draft.dpiStages,
			(value) => {
				draft.dpiStages = value;
			},
			[...draft.dpiStages, stage],
			[
				{ key: 'dpiStageCount', count: index + 1 },
				{ key: 'dpiStage', index, dpi: stage.dpi, color: stage.color }
			]
		);
	}

	function removeStage(index: number) {
		if (!hasData || draft.dpiStages.length <= 1) return;
		const previousStages = draft.dpiStages;
		const previousCurrent = draft.currentStage;
		const nextStages = previousStages.filter((_, i) => i !== index);
		const nextCurrent = previousCurrent >= nextStages.length ? nextStages.length - 1 : previousCurrent;
		const requests: WriteSettingRequest[] = [{ key: 'dpiStageCount', count: nextStages.length }];
		if (nextCurrent !== previousCurrent) requests.push({ key: 'currentDpiStage', index: nextCurrent });
		commit(
			'dpiStages',
			() => ({ stages: previousStages, current: previousCurrent }),
			(value) => {
				draft.dpiStages = value.stages;
				draft.currentStage = value.current;
			},
			{ stages: nextStages, current: nextCurrent },
			requests
		);
	}

	function selectStage(index: number) {
		if (!hasData || draft.currentStage === index) return;
		commit(
			'currentStage',
			() => draft.currentStage,
			(value) => {
				draft.currentStage = value;
			},
			index,
			{ key: 'currentDpiStage', index }
		);
	}

	function commitStageDpi(index: number, dpi: number) {
		commit(
			`dpiStage:${index}`,
			() => draft.dpiStages[index].dpi,
			(value) => {
				draft.dpiStages[index] = { ...draft.dpiStages[index], dpi: value };
			},
			dpi,
			{ key: 'dpiStage', index, dpi, color: draft.dpiStages[index].color }
		);
	}

	function commitStageColor(index: number, color: RgbColor) {
		commit(
			`dpiStage:${index}`,
			() => draft.dpiStages[index].color,
			(value) => {
				draft.dpiStages[index] = { ...draft.dpiStages[index], color: value };
			},
			color,
			{ key: 'dpiStage', index, dpi: draft.dpiStages[index].dpi, color }
		);
	}

	// The log slider's own click-to-edit affordance is keyboard-only once its own value bubble is
	// hidden (this row already shows the reading via `.stage-value-num`); a visible, typeable
	// field per row is its own small state here instead; it commits through the same
	// `commitStageDpi` -> `commit` path as the slider, so a typed value is still snapped to
	// `DPI_STEP` and still reverts on a failed write like every other control on this screen.
	let editingStage = $state<number | null>(null);
	let editStageText = $state('');

	function startEditStage(index: number) {
		if (!hasData) return;
		editingStage = index;
		editStageText = String(draft.dpiStages[index].dpi);
	}

	function commitEditStage() {
		if (editingStage === null) return;
		const index = editingStage;
		editingStage = null;
		const parsed = Number(editStageText);
		if (!Number.isFinite(parsed)) return;
		const snapped = Math.min(DPI_MAX, Math.max(DPI_MIN, Math.round(parsed / DPI_STEP) * DPI_STEP));
		commitStageDpi(index, snapped);
	}
</script>

{#if !device.connected}
	<p class="page-pad caption-note">No mouse connected. Plug in the receiver or the cable.</p>
{/if}
<div class="perf-grid">
	<section class="plate plate-dpi">
		<div class="plate-head">
			<h2><span class="plate-stub"></span>DPI stages<HelpTip topic="dpiStages" /></h2>
			<span class="field-hint">Up to {MAX_DPI_STAGES} &middot; active stage drives the DPI switch</span>
		</div>
		<div class="stage-row stage-header">
			<span class="stage-header-spacer" aria-hidden="true"></span>
			<span class="stage-header-label">DPI<HelpTip topic="dpiValue" /></span>
			<span class="stage-slider" aria-hidden="true"></span>
			<span class="stage-header-label">Colour<HelpTip topic="dpiColor" /></span>
			<span class="stage-header-spacer" aria-hidden="true"></span>
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
						onclick={() => selectStage(i)}
					>
						{i + 1}
					</button>
					<div class="stage-value">
						{#if editingStage === i}
							<input
								class="stage-value-input mono"
								type="number"
								min={DPI_MIN}
								max={DPI_MAX}
								step={DPI_STEP}
								bind:value={editStageText}
								onblur={commitEditStage}
								onkeydown={(e) => {
									if (e.key === 'Enter') {
										commitEditStage();
									} else if (e.key === 'Escape') {
										editingStage = null;
									}
									e.stopPropagation();
								}}
							/>
						{:else}
							<button
								type="button"
								class="stage-value-num mono"
								disabled={!hasData}
								title="Type an exact DPI value"
								onclick={() => startEditStage(i)}
							>{hasData ? stage.dpi : '-'}</button>
						{/if}
						<span class="stage-value-unit">DPI</span>
					</div>
					<div class="stage-slider">
						<RangeSlider
							label={`DPI stage ${i + 1}`}
							min={DPI_MIN}
							max={DPI_MAX}
							step={DPI_STEP}
							scale="log"
							ticks={DPI_TICKS}
							value={stage.dpi}
							unknown={!hasData}
							accentColor={`rgb(${stage.color.join(',')})`}
							onchange={(v) => commitStageDpi(i, v)}
						/>
					</div>
					<div class="stage-color">
						<ColorSwatchPicker color={stage.color} unknown={!hasData} onchange={(c) => commitStageColor(i, c)} />
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
				{#if fieldErrors[`dpiStage:${i}`]}
					<p class="field-hint error-text stage-error">{fieldErrors[`dpiStage:${i}`]}</p>
				{/if}
			{/each}
		</div>
		{#if fieldErrors['dpiStages']}<p class="field-hint error-text">{fieldErrors['dpiStages']}</p>{/if}
		<div class="add-stage-row">
			<button class="btn" disabled={!hasData || draft.dpiStages.length >= MAX_DPI_STAGES} onclick={addStage}>
				Add stage
			</button>
			<HelpTip topic="dpiAddStage" />
		</div>
	</section>

	<section class="plate plate-tracking">
		<div class="plate-head">
			<h2>Tracking</h2>
			<span class="field-hint">Polling, range, lift-off, sensor mode</span>
		</div>
		<div class="choice-row">
			<div class="field">
				<span class="field-label">Sensor mode<HelpTip topic="sensorMode" /></span>
				<SegmentedControl
					values={SENSOR_MODE_OPTIONS}
					value={draft.sensorMode}
					ariaLabel="Sensor mode"
					unknown={!hasData}
					onchange={(v) =>
						commit(
							'sensorMode',
							() => draft.sensorMode,
							(value) => {
								draft.sensorMode = value;
							},
							v,
							{ key: 'sensorMode', mode: v }
						)}
				/>
				{#if fieldErrors['sensorMode']}<span class="field-hint error-text">{fieldErrors['sensorMode']}</span>{/if}
			</div>
			<div class="field">
				<span class="field-label">Lift-off distance<HelpTip topic="liftOff" /></span>
				<SegmentedControl
					values={LOD_OPTIONS}
					value={lodName(draft.lod.value)}
					ariaLabel="Lift-off distance"
					unknown={!hasData}
					onchange={(v) =>
						commit(
							'lod',
							() => lodName(draft.lod.value),
							(value) => {
								draft.lod = { value };
							},
							v,
							{ key: 'lod', value: { value: v } }
						)}
				/>
				{#if fieldErrors['lod']}<span class="field-hint error-text">{fieldErrors['lod']}</span>{/if}
			</div>
		</div>
		<div class="field">
			<span class="field-label"
				>Polling rate <span class="field-unit mono">Hz</span><HelpTip topic="pollingRate" /></span
			>
			<SegmentedControl
				values={pollingOptions}
				value={draft.pollingHz}
				ariaLabel="Polling rate"
				unknown={!hasData}
				onchange={(v) =>
					commit(
						'polling',
						() => draft.pollingHz,
						(value) => {
							draft.pollingHz = value;
						},
						v,
						{ key: 'polling', hz: v }
					)}
			/>
			{#if fieldErrors['polling']}<span class="field-hint error-text">{fieldErrors['polling']}</span>{/if}
		</div>
		<div class="field">
			{#if device.identity?.capabilities?.longRange}
				{#if hasData && draft.longRange === 'unsupported'}
					<p class="field-hint">Long range not supported on this device.</p>
				{:else}
					<Toggle
						label="Long range mode"
						helpTopic="longRange"
						hint="Trades a little latency for more range."
						checked={draft.longRange === 'on'}
						disabled={!hasData}
						onchange={(v) =>
							commit(
								'longRange',
								() => draft.longRange === 'on',
								(value) => {
									draft.longRange = value ? 'on' : 'off';
								},
								v,
								{ key: 'longRange', on: v }
							)}
					/>
				{/if}
			{/if}
			{#if fieldErrors['longRange']}<span class="field-hint error-text">{fieldErrors['longRange']}</span>{/if}
		</div>
		<div class="linear-row">
			<RangeSlider
				label="Debounce"
				helpTopic="debounce"
				unit=" ms"
				min={0}
				max={25}
				ticks={[0, 5, 10, 15, 20, 25]}
				value={draft.debounceMs}
				unknown={!hasData}
				onchange={(v) =>
					commit(
						'debounce',
						() => draft.debounceMs,
						(value) => {
							draft.debounceMs = value;
						},
						v,
						{ key: 'debounce', ms: v }
					)}
			/>
		</div>
		{#if fieldErrors['debounce']}<p class="field-hint error-text">{fieldErrors['debounce']}</p>{/if}
	</section>

	<section class="plate plate-motion">
		<div class="plate-head">
			<h2>Motion</h2>
		</div>
		<div class="field-stack">
			<Toggle
				label="Motion sync"
				helpTopic="motionSync"
				hint="Times each report to the polling clock."
				checked={draft.motionSync}
				disabled={!hasData}
				onchange={(v) =>
					commit(
						'motionSync',
						() => draft.motionSync,
						(value) => {
							draft.motionSync = value;
						},
						v,
						{ key: 'motionSync', on: v }
					)}
			/>
			<Toggle
				label="Angle snap"
				helpTopic="angleSnap"
				hint="Snaps near-straight strokes into a line."
				checked={draft.angleSnap}
				disabled={!hasData}
				onchange={(v) =>
					commit(
						'angleSnap',
						() => draft.angleSnap,
						(value) => {
							draft.angleSnap = value;
						},
						v,
						{ key: 'angleSnap', on: v }
					)}
			/>
			<Toggle
				label="Ripple control"
				helpTopic="ripple"
				hint="Smooths jitter in slow, precise movement."
				checked={draft.ripple}
				disabled={!hasData}
				onchange={(v) =>
					commit(
						'ripple',
						() => draft.ripple,
						(value) => {
							draft.ripple = value;
						},
						v,
						{ key: 'ripple', on: v }
					)}
			/>
		</div>
		{#if fieldErrors['motionSync']}<p class="field-hint error-text">{fieldErrors['motionSync']}</p>{/if}
		{#if fieldErrors['angleSnap']}<p class="field-hint error-text">{fieldErrors['angleSnap']}</p>{/if}
		{#if fieldErrors['ripple']}<p class="field-hint error-text">{fieldErrors['ripple']}</p>{/if}
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
				onclick={() =>
					commit(
						'performanceOn',
						() => draft.performance.on,
						(value) => {
							draft.performance = { ...draft.performance, on: value };
						},
						!draft.performance.on,
						{ key: 'performanceOn', on: !draft.performance.on }
					)}
			>
				<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
					<path d="M12 3v8" />
					<path d="M6.5 6.5a8 8 0 1 0 11 0" />
				</svg>
			</button>
		{/snippet}
		<div class="dial-row dial-row-hero">
			<div class="dial-field">
				<span class="field-label">Sleep after<HelpTip topic="sleepTime" /></span>
				<SegmentDial
					values={TIMEOUT_OPTIONS}
					value={sleepTimeName(draft.sleep.value)}
					ariaLabel="Sleep after"
					unknown={!hasData}
					size="large"
					centerLabel={timeoutLabel(draft.sleep.value)}
					onchange={(v) =>
						commit(
							'sleep',
							() => sleepTimeName(draft.sleep.value),
							(value) => {
								draft.sleep = { value };
							},
							v,
							{ key: 'sleep', value: { value: v } }
						)}
				/>
				{#if fieldErrors['sleep']}<span class="field-hint error-text">{fieldErrors['sleep']}</span>{/if}
			</div>
			<div class="dial-field">
				<span class="field-label">Highest performance<HelpTip topic="highestPerformance" /></span>
				<SegmentDial
					values={TIMEOUT_OPTIONS}
					value={sleepTimeName(draft.performance.timeout.value)}
					ariaLabel="Highest performance timeout"
					unknown={!hasData}
					disabled={!draft.performance.on}
					size="large"
					centerLabel={timeoutLabel(draft.performance.timeout.value)}
					centerAction={performanceToggle}
					onchange={(v) =>
						commit(
							'performanceTimeout',
							() => sleepTimeName(draft.performance.timeout.value),
							(value) => {
								draft.performance = { ...draft.performance, timeout: { value } };
							},
							v,
							{ key: 'performanceTimeout', value: { value: v } }
						)}
				/>
				{#if fieldErrors['performanceOn']}<span class="field-hint error-text">{fieldErrors['performanceOn']}</span>{/if}
				{#if fieldErrors['performanceTimeout']}<span class="field-hint error-text">{fieldErrors['performanceTimeout']}</span>{/if}
			</div>
		</div>
	</section>
</div>

<style>
	/* The unit for a control whose own options carry only numbers (the polling rate segments). */
	.field-unit {
		color: var(--color-muted);
		font-weight: 400;
	}

	.page-pad {
		padding: 0 var(--space-xl) var(--space-xl);
	}

	/* A real grid, not independent flex rows: one 12-column track shared by every panel, so DPI
	   stages, Tracking, Motion and Power all line up on the same underlying gutters (the
	   operator's "not properly aligned panels") even though the bottom row splits its two panels
	   unevenly (4:8, Motion narrow beside Power wide) while the top row splits its two evenly
	   (6:6): every edge still lands on a shared column line. Grid's default stretch also
	   equalizes each row's panel heights for free, so a shorter panel's own plate extends to
	   match its neighbour instead of leaving a ragged bottom edge or a gap of bare page
	   background under it. Power gets the wider share because its two dials need real width to
	   read as sized to the panel rather than small rings adrift in it (operator feedback,
	   2026-09-16); Motion's few toggles need much less. */
	.perf-grid {
		display: grid;
		grid-template-columns: repeat(12, 1fr);
		/* DPI/Tracking size to their own content; Motion/Power take whatever is left, so the grid's
		   own bottom edge reaches the window's rather than stopping short of it and leaving a bare
		   gap below the last row (operator feedback, 2026-09-16: "about 120px of empty space").
		   120px is TopBar's own rendered height (title, subtitle, its padding): this screen cannot
		   read that from TopBar.svelte, so it is restated here rather than shared. */
		grid-template-rows: auto 1fr;
		gap: var(--space-sm);
		min-height: calc(100vh - 120px);
		padding: 0 var(--space-xl) var(--space-lg);
	}

	.perf-grid > .plate {
		min-width: 0;
		padding: var(--space-sm) var(--space-md);
	}

	.plate-dpi,
	.plate-tracking {
		grid-column: span 6;
	}

	/* Motion is a list of short rows and needs little width; Power holds two dials whose labels sit
	   outside their rings, so it takes the wider share or the pair wraps and the second dial falls
	   below the window's edge (seen in a capture). */
	.plate-motion {
		grid-column: span 3;
	}

	.plate-power {
		grid-column: span 9;
	}

	@media (max-width: 900px) {
		.perf-grid {
			grid-template-columns: 1fr;
		}

		.plate-dpi,
		.plate-tracking,
		.plate-motion,
		.plate-power {
			grid-column: auto;
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

	/* Every toggle's row plus its own one-line muted description (Toggle's `hint`), starting right
	   under the panel header rather than floating in whatever height the row stretch gives this
	   panel (operator feedback, 2026-09-16). */
	.field-stack {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	.plate-motion,
	.plate-power {
		display: flex;
		flex-direction: column;
	}

	/* Sensor mode and lift-off side by side, each a full-width segmented control rather than a
	   dial: two and three choices read faster as a row of labelled buttons than as a ring
	   (operator feedback, 2026-09-16). */
	.choice-row {
		display: flex;
		gap: var(--space-md);
		margin-bottom: var(--space-sm);
	}

	.choice-row > .field {
		flex: 1;
		min-width: 0;
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

	/* Column labels for the value and colour a stage row carries, shown once above the list
	   (design.md legibility; a "?" per row would repeat itself five times over). Reuses
	   `.stage-row`'s own flex layout so its columns land under the real rows without hand-tuned
	   widths, with the row's own background and border stripped back to a plain caption line. */
	.stage-header {
		min-height: 0;
		padding: 0 var(--space-2xs) var(--space-3xs);
		background: none;
		border: none;
	}

	.stage-header-spacer {
		flex-shrink: 0;
		width: 24px;
	}

	.stage-header-label {
		display: flex;
		align-items: center;
		min-width: 4.2em;
		font-size: var(--text-xs);
		color: var(--color-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.add-stage-row {
		display: flex;
		align-items: center;
		gap: var(--space-2xs);
	}

	.stage-error {
		margin: 0 0 var(--space-3xs);
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

	/* A real button (click, or Tab to it and press Enter/Space) that opens the exact-value field
	   below, not just a display span: reset to look identical to the plain reading it replaces. */
	.stage-value-num {
		background: none;
		border: none;
		padding: 0;
		font-family: var(--font-mono);
		font-size: var(--text-lg);
		font-weight: 600;
		color: var(--color-ink);
		text-align: right;
		cursor: text;
	}

	.stage-value-num:disabled {
		cursor: not-allowed;
	}

	.stage-value-input {
		width: 4.4em;
		background: var(--color-paper-2);
		border: 1px solid var(--color-accent);
		padding: 1px var(--space-3xs);
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

	/* A dial's labels sit outside its ring, so two dials need a gap wider than the labels or the
	   right label of one lands on the left label of the next ("1 min" over "10 min", seen in a
	   capture). --space-2xl keeps them apart at every dial tier. */
	.dial-row {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-evenly;
		gap: var(--space-xl);
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
	   (docs/architecture/ui-controls.md section 5): the Power panel's pair is sized "large" below,
	   since Power's own grid column (span 8 of 12) is wide enough for it, and steps down through
	   default to compact only if its own container gets unusually narrow. */
	@container (max-width: 620px) {
		.dial-row-hero {
			--dial-size: var(--dial-default);
		}
	}

	@container (max-width: 380px) {
		.dial-row-hero {
			--dial-size: var(--dial-compact);
		}
	}

	/* The highest-performance on/off button, drawn inside the hero dial's own centre: a plain
	   cut-chip square, never the circle it used to be (design.md "Shape: nothing is round" excepts
	   only the dial face bitmaps themselves, not the chrome sitting on top of one). It stays fully
	   legible and clickable regardless of the ring's own on/off dimming (SegmentDial's
	   `.center-action` is excluded from that rule): this is the one control that turns the ring
	   back on. */
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

	.error-text {
		color: var(--color-danger);
	}

	.caption-note {
		margin: 0 0 var(--space-3xs);
		color: var(--color-faint);
	}
</style>
