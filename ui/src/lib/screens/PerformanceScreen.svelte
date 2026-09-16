<script lang="ts">
	import { device } from '../device.svelte';
	import type { DpiStage, Lod, Settings } from '../types';
	import EmptyState from '../components/EmptyState.svelte';
	import RangeField from '../components/RangeField.svelte';
	import SelectField from '../components/SelectField.svelte';
	import Toggle from '../components/Toggle.svelte';
	import ColorSwatchPicker from '../components/ColorSwatchPicker.svelte';

	const POLLING_OPTIONS = [125, 250, 500, 1000, 2000, 4000, 8000].map((hz) => ({
		value: hz,
		label: `${hz} Hz`
	}));

	const LOD_OPTIONS: { value: Lod; label: string }[] = [
		{ value: 'OneMm', label: '1 mm' },
		{ value: 'TwoMm', label: '2 mm' },
		{ value: 'PointSevenMm', label: '0.7 mm' }
	];

	// mouse-protocol-v2.md 7.8: same tens-of-seconds codes shared by sleep
	// timing and the highest-performance timeout.
	const TIMEOUT_OPTIONS = [
		{ value: 10, label: '10 s' },
		{ value: 30, label: '30 s' },
		{ value: 60, label: '1 min' },
		{ value: 120, label: '2 min' },
		{ value: 300, label: '5 min' },
		{ value: 600, label: '10 min' },
		{ value: 900, label: '15 min' }
	];

	const SENSOR_MODE_OPTIONS = [
		{ value: 0, label: 'Low power' },
		{ value: 1, label: 'High performance' }
	];

	const MAX_DPI_STAGES = 8;

	let draft = $state<Settings | null>(null);
	let saving = $state(false);
	let dirty = $state(false);

	$effect(() => {
		draft = device.settings ? structuredClone(device.settings) : null;
		dirty = false;
	});

	function touch() {
		dirty = true;
	}

	function addStage() {
		if (!draft || draft.dpi_stages.length >= MAX_DPI_STAGES) return;
		const last = draft.dpi_stages.at(-1);
		draft.dpi_stages.push({
			dpi: last ? last.dpi : 800,
			color: last ? { ...last.color } : { r: 255, g: 255, b: 255 }
		});
		touch();
	}

	function removeStage(index: number) {
		if (!draft || draft.dpi_stages.length <= 1) return;
		draft.dpi_stages.splice(index, 1);
		if (draft.current_stage >= draft.dpi_stages.length) {
			draft.current_stage = draft.dpi_stages.length - 1;
		}
		touch();
	}

	function updateStage(index: number, patch: Partial<DpiStage>) {
		if (!draft) return;
		draft.dpi_stages[index] = { ...draft.dpi_stages[index], ...patch };
		touch();
	}

	async function apply() {
		if (!draft) return;
		saving = true;
		try {
			await device.writeSetting({
				polling_hz: draft.polling_hz,
				dpi_stages: draft.dpi_stages,
				current_stage: draft.current_stage,
				lod: draft.lod,
				debounce_ms: draft.debounce_ms,
				motion_sync: draft.motion_sync,
				angle_snap: draft.angle_snap,
				ripple: draft.ripple,
				performance: draft.performance,
				sleep: draft.sleep,
				sensor_mode: draft.sensor_mode,
				long_range: draft.long_range
			});
			dirty = false;
		} finally {
			saving = false;
		}
	}
</script>

{#if !device.connected}
	<div class="page-pad">
		<EmptyState
			title="No device connected"
			message="Connect a Hyperpace mouse from Settings to read and tune its performance settings."
		/>
	</div>
{:else if device.settingsLoading || !draft}
	<div class="page-pad">
		<EmptyState title="Reading settings" message="Fetching the current settings shadow from the device." />
	</div>
{:else}
	<div class="page-pad grid" style="grid-template-columns: repeat(auto-fit, minmax(280px, 1fr))">
		<section class="panel">
			<div class="panel-title">DPI stages</div>
			<div class="panel-subtitle">Up to {MAX_DPI_STAGES} stages. The active stage is bound to the DPI switch action.</div>
			<div class="stage-list">
				{#each draft.dpi_stages as stage, i (i)}
					<div class="stage-row" class:current={draft.current_stage === i}>
						<button
							type="button"
							class="stage-radio"
							aria-pressed={draft.current_stage === i}
							title="Set as active stage"
							onclick={() => {
								if (!draft) return;
								draft.current_stage = i;
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
							value={stage.dpi}
							oninput={(e) => updateStage(i, { dpi: Number((e.target as HTMLInputElement).value) })}
						/>
						<span class="field-hint">DPI</span>
						<ColorSwatchPicker
							color={stage.color}
							onchange={(c) => updateStage(i, { color: c })}
						/>
						<button
							type="button"
							class="btn btn-danger stage-remove"
							disabled={draft.dpi_stages.length <= 1}
							onclick={() => removeStage(i)}
						>
							Remove
						</button>
					</div>
				{/each}
			</div>
			<button class="btn" disabled={draft.dpi_stages.length >= MAX_DPI_STAGES} onclick={addStage}>
				Add stage
			</button>
		</section>

		<section class="panel">
			<div class="panel-title">Tracking</div>
			<div class="panel-subtitle">Polling rate, lift-off distance and sensor mode.</div>
			<div class="field-stack">
				<SelectField
					label="Polling rate"
					value={draft.polling_hz}
					options={POLLING_OPTIONS}
					onchange={(v) => {
						if (draft) draft.polling_hz = v;
						touch();
					}}
				/>
				<SelectField
					label="Lift-off distance"
					value={draft.lod}
					options={LOD_OPTIONS}
					onchange={(v) => {
						if (draft) draft.lod = v;
						touch();
					}}
				/>
				<SelectField
					label="Sensor mode"
					value={draft.sensor_mode}
					options={SENSOR_MODE_OPTIONS}
					onchange={(v) => {
						if (draft) draft.sensor_mode = v;
						touch();
					}}
				/>
				<RangeField
					label="Debounce"
					unit=" ms"
					min={0}
					max={25}
					value={draft.debounce_ms}
					onchange={(v) => {
						if (draft) draft.debounce_ms = v;
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
					checked={draft.motion_sync}
					onchange={(v) => {
						if (draft) draft.motion_sync = v;
						touch();
					}}
				/>
				<Toggle
					label="Angle snap"
					checked={draft.angle_snap}
					onchange={(v) => {
						if (draft) draft.angle_snap = v;
						touch();
					}}
				/>
				<Toggle
					label="Ripple control"
					checked={draft.ripple}
					onchange={(v) => {
						if (draft) draft.ripple = v;
						touch();
					}}
				/>
			</div>
		</section>

		<section class="panel">
			<div class="panel-title">Power</div>
			<div class="panel-subtitle">Sleep timer, highest-performance mode and the wireless link.</div>
			<div class="field-stack">
				<SelectField
					label="Sleep after"
					value={draft.sleep.seconds}
					options={TIMEOUT_OPTIONS}
					onchange={(v) => {
						if (draft) draft.sleep = { seconds: v };
						touch();
					}}
				/>
				<Toggle
					label="Highest performance mode"
					hint="Keeps the sensor at full power instead of stepping down"
					checked={draft.performance.on}
					onchange={(v) => {
						if (draft) draft.performance = { ...draft.performance, on: v };
						touch();
					}}
				/>
				{#if draft.performance.on}
					<SelectField
						label="Highest performance timeout"
						value={draft.performance.timeout_s}
						options={TIMEOUT_OPTIONS}
						onchange={(v) => {
							if (draft) draft.performance = { ...draft.performance, timeout_s: v };
							touch();
						}}
					/>
				{/if}
				{#if draft.long_range !== undefined}
					<Toggle
						label="Long range mode"
						hint="Extends the wireless link at the cost of polling headroom"
						checked={draft.long_range}
						onchange={(v) => {
							if (draft) draft.long_range = v;
							touch();
						}}
					/>
				{:else}
					<p class="field-hint">Long range state is not reported by this read yet.</p>
				{/if}
			</div>
		</section>
	</div>

	<div class="apply-bar">
		<button class="btn btn-primary" disabled={!dirty || saving} onclick={apply}>
			{saving ? 'Applying...' : dirty ? 'Apply changes' : 'Up to date'}
		</button>
		{#if device.lastError}
			<span class="field-hint error-text">{device.lastError}</span>
		{/if}
	</div>
{/if}

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
</style>
