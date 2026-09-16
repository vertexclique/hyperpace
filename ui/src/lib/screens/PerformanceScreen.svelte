<script lang="ts">
	import { device, diffSettings } from '../device.svelte';
	import { BLANK_SETTINGS } from '../types';
	import type { DpiStage, Lod, Settings, SleepTime } from '../types';
	import RangeField from '../components/RangeField.svelte';
	import SelectField from '../components/SelectField.svelte';
	import Toggle from '../components/Toggle.svelte';
	import ColorSwatchPicker from '../components/ColorSwatchPicker.svelte';

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

	// The select only offers the named variants (never `other`), so binding its `value` needs a
	// narrowed read: this also keeps SelectField's generic inferred as the named-only union
	// (excluding `other`, which needs a `byte` field the select never supplies) instead of
	// widening to the full wire type, which would make `{ value: v }` fail to satisfy `Lod` /
	// `SleepTime` below.
	function lodName(value: Lod['value']): Exclude<Lod['value'], 'other'> {
		return value === 'other' ? 'oneMillimeter' : value;
	}
	function sleepTimeName(value: SleepTime['value']): Exclude<SleepTime['value'], 'other'> {
		return value === 'other' ? 'tenSeconds' : value;
	}

	// See LightingScreen for why `draft` is always a full Settings object (BLANK_SETTINGS when
	// disconnected) instead of null, and what `hasData` gates.
	let draft = $state<Settings>(structuredClone(BLANK_SETTINGS));
	let saving = $state(false);
	let dirty = $state(false);
	let hasData = $derived(device.settings !== null);

	$effect(() => {
		draft = device.settings ? structuredClone(device.settings) : structuredClone(BLANK_SETTINGS);
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

{#if !hasData}
	<p class="page-pad caption-note">No device connected: every control below is disabled.</p>
{/if}
<div class="page-pad grid" style="grid-template-columns: repeat(auto-fit, minmax(280px, 1fr))">
	<section class="panel">
		<div class="panel-title">DPI stages</div>
		<div class="panel-subtitle">Up to {MAX_DPI_STAGES} stages. The active stage is bound to the DPI switch action.</div>
		<div class="stage-list">
			{#each draft.dpiStages as stage, i (i)}
				<div class="stage-row" class:current={draft.currentStage === i}>
					<button
						type="button"
						class="stage-radio"
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
					<input
						class="text-input stage-dpi"
						type="number"
						min="1"
						max="999999"
						disabled={!hasData}
						value={hasData ? stage.dpi : ''}
						oninput={(e) => updateStage(i, { dpi: Number((e.target as HTMLInputElement).value) })}
					/>
					<span class="field-hint">DPI</span>
					<ColorSwatchPicker
						color={stage.color}
						unknown={!hasData}
						onchange={(c) => updateStage(i, { color: c })}
					/>
					<button
						type="button"
						class="btn btn-danger stage-remove"
						disabled={!hasData || draft.dpiStages.length <= 1}
						onclick={() => removeStage(i)}
					>
						Remove
					</button>
				</div>
			{/each}
		</div>
		<button class="btn" disabled={!hasData || draft.dpiStages.length >= MAX_DPI_STAGES} onclick={addStage}>
			Add stage
		</button>
	</section>

	<section class="panel">
		<div class="panel-title">Tracking</div>
		<div class="panel-subtitle">Polling rate, lift-off distance and sensor mode.</div>
		<div class="field-stack">
			<SelectField
				label="Polling rate"
				value={draft.pollingHz}
				options={POLLING_OPTIONS}
				unknown={!hasData}
				onchange={(v) => {
					draft.pollingHz = v;
					touch();
				}}
			/>
			<SelectField
				label="Lift-off distance"
				value={lodName(draft.lod.value)}
				options={LOD_OPTIONS}
				unknown={!hasData}
				onchange={(v) => {
					draft.lod = { value: v };
					touch();
				}}
			/>
			<SelectField
				label="Sensor mode"
				value={draft.sensorMode}
				options={SENSOR_MODE_OPTIONS}
				unknown={!hasData}
				onchange={(v) => {
					draft.sensorMode = v;
					touch();
				}}
			/>
			<RangeField
				label="Debounce"
				unit=" ms"
				min={0}
				max={25}
				value={draft.debounceMs}
				unknown={!hasData}
				onchange={(v) => {
					draft.debounceMs = v;
					touch();
				}}
			/>
		</div>
	</section>

	<section class="panel">
		<div class="panel-title">Motion</div>
		<div class="panel-subtitle">Cursor path corrections applied by the sensor firmware.</div>
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

	<section class="panel">
		<div class="panel-title">Power</div>
		<div class="panel-subtitle">Sleep timer and highest-performance mode.</div>
		<div class="field-stack">
			<SelectField
				label="Sleep after"
				value={sleepTimeName(draft.sleep.value)}
				options={TIMEOUT_OPTIONS}
				unknown={!hasData}
				onchange={(v) => {
					draft.sleep = { value: v };
					touch();
				}}
			/>
			<Toggle
				label="Highest performance mode"
				hint="Keeps the sensor at full power instead of stepping down"
				checked={draft.performance.on}
				disabled={!hasData}
				onchange={(v) => {
					draft.performance = { ...draft.performance, on: v };
					touch();
				}}
			/>
			{#if draft.performance.on}
				<SelectField
					label="Highest performance timeout"
					value={sleepTimeName(draft.performance.timeout.value)}
					options={TIMEOUT_OPTIONS}
					unknown={!hasData}
					onchange={(v) => {
						draft.performance = { ...draft.performance, timeout: { value: v } };
						touch();
					}}
				/>
			{/if}
		</div>
	</section>

	<section class="panel">
		<div class="panel-title">Long range</div>
		<div class="panel-subtitle">Extends the wireless link's effective range on receivers that support it.</div>
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
</div>

<div class="apply-bar">
	<button class="btn btn-primary" disabled={!hasData || !dirty || saving} onclick={apply}>
		{saving ? 'Applying...' : dirty ? 'Apply changes' : 'Up to date'}
	</button>
	{#if device.lastError}
		<span class="field-hint error-text">{device.lastError}</span>
	{/if}
</div>

<style>
	.page-pad {
		padding: 0 32px 32px;
	}

	.grid {
		padding: 0 32px 24px;
	}

	.field-stack {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.stage-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-bottom: 12px;
	}

	.stage-row {
		display: flex;
		align-items: center;
		gap: 10px;
		background: var(--bg-raised);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		padding: 8px 10px;
	}

	.stage-row.current {
		border-color: var(--accent);
	}

	.stage-radio {
		width: 26px;
		height: 26px;
		border-radius: 50%;
		background: var(--panel-raised);
		border: 1px solid var(--border);
		font-size: 11px;
		font-weight: 700;
		flex-shrink: 0;
	}

	.stage-row.current .stage-radio {
		background: var(--accent);
		color: #06090f;
		border-color: var(--accent);
	}

	.stage-dpi {
		width: 90px;
	}

	.stage-remove {
		margin-left: auto;
	}

	.apply-bar {
		position: sticky;
		bottom: 0;
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 14px 32px;
		background: linear-gradient(to top, var(--bg) 60%, transparent);
	}

	.error-text {
		color: var(--danger);
	}

	.caption-note {
		margin: 0 0 4px;
		color: var(--text-faint);
	}
</style>
