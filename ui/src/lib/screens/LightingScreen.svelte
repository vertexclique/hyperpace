<script lang="ts">
	import { device } from '../device.svelte';
	import { LIGHTING_MODES } from '../types';
	import type { ReceiverLight, Settings } from '../types';
	import EmptyState from '../components/EmptyState.svelte';
	import SelectField from '../components/SelectField.svelte';
	import RangeField from '../components/RangeField.svelte';
	import Toggle from '../components/Toggle.svelte';
	import ColorSwatchPicker from '../components/ColorSwatchPicker.svelte';
	import ReceiverArt from '../art/ReceiverArt.svelte';

	// docs/research/mouse-protocol-v2.md 7.7: which controls a mode leaves
	// meaningful. Off disables everything, Fixed color has no animation
	// speed, the three cycling modes have no fixed color, mode 6 has neither.
	function lightingControls(mode: number) {
		if (mode === 0) return { color: false, speed: false };
		if (mode === 3) return { color: true, speed: false };
		if (mode === 1 || mode === 4 || mode === 5) return { color: false, speed: true };
		if (mode === 6) return { color: false, speed: false };
		return { color: true, speed: true }; // mode 2
	}

	let draft = $state<Settings | null>(null);
	let saving = $state(false);
	let dirty = $state(false);

	$effect(() => {
		draft = device.settings ? structuredClone(device.settings) : null;
		dirty = false;
	});

	let controls = $derived(draft ? lightingControls(draft.lighting.mode) : { color: false, speed: false });

	function touch() {
		dirty = true;
	}

	async function apply() {
		if (!draft) return;
		saving = true;
		try {
			await device.writeSetting({ lighting: draft.lighting, dpi_indicator: draft.dpi_indicator });
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
		color: { r: 255, g: 255, b: 255 },
		speed: 5,
		brightness: 5,
		time: 0
	});
	let receiverSaving = $state(false);

	async function applyReceiverLight() {
		receiverSaving = true;
		try {
			await device.setReceiverLight(receiverLight);
		} finally {
			receiverSaving = false;
		}
	}

	let pairing = $state(false);
	let pairingStarted = $state(false);

	async function startPairing() {
		pairing = true;
		try {
			await device.pairReceiver();
			pairingStarted = true;
		} finally {
			pairing = false;
		}
	}

	let confirmingReset = $state(false);

	async function doFactoryReset() {
		await device.factoryReset();
		confirmingReset = false;
	}
</script>

<div class="page-pad grid" style="grid-template-columns: repeat(auto-fit, minmax(300px, 1fr))">
	<section class="panel">
		<div class="panel-title">Mouse lighting</div>
		<div class="panel-subtitle">The body light strip and its effect.</div>
		{#if !device.connected}
			<EmptyState title="No device connected" message="Connect a mouse to edit its lighting." />
		{:else if device.settingsLoading || !draft}
			<EmptyState title="Reading settings" message="Fetching the current lighting state." />
		{:else}
			<div class="field-stack">
				<Toggle
					label="Lighting on"
					checked={draft.lighting.on}
					onchange={(v) => {
						if (draft) draft.lighting = { ...draft.lighting, on: v };
						touch();
					}}
				/>
				<SelectField
					label="Effect"
					value={draft.lighting.mode}
					options={LIGHTING_MODES.map((m) => ({ value: m.value, label: m.label }))}
					onchange={(v) => {
						if (draft) draft.lighting = { ...draft.lighting, mode: v };
						touch();
					}}
				/>
				{#if controls.color}
					<ColorSwatchPicker
						label="Color"
						color={draft.lighting.color}
						onchange={(c) => {
							if (draft) draft.lighting = { ...draft.lighting, color: c };
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
						onchange={(v) => {
							if (draft) draft.lighting = { ...draft.lighting, speed: v };
							touch();
						}}
					/>
				{/if}
				<RangeField
					label="Brightness"
					min={0}
					max={9}
					value={draft.lighting.brightness}
					onchange={(v) => {
						if (draft) draft.lighting = { ...draft.lighting, brightness: v };
						touch();
					}}
				/>

				<div class="divider"></div>
				<div class="panel-title" style="margin:0">DPI indicator</div>
				{#if draft.dpi_indicator}
					<Toggle
						label="Indicator on"
						checked={draft.dpi_indicator.on}
						onchange={(v) => {
							if (draft?.dpi_indicator) draft.dpi_indicator.on = v;
							touch();
						}}
					/>
					<SelectField
						label="Indicator mode"
						value={draft.dpi_indicator.mode}
						options={[
							{ value: 0, label: 'Off' },
							{ value: 1, label: 'Steady' },
							{ value: 2, label: 'Breathing' }
						]}
						onchange={(v) => {
							if (draft?.dpi_indicator) draft.dpi_indicator.mode = v;
							touch();
						}}
					/>
					<RangeField
						label="Indicator brightness"
						min={1}
						max={10}
						value={draft.dpi_indicator.brightness}
						onchange={(v) => {
							if (draft?.dpi_indicator) draft.dpi_indicator.brightness = v;
							touch();
						}}
					/>
				{:else}
					<p class="field-hint">DPI indicator state is not reported by this read yet.</p>
				{/if}

				<button class="btn btn-primary" disabled={!dirty || saving} onclick={apply}>
					{saving ? 'Applying...' : dirty ? 'Apply changes' : 'Up to date'}
				</button>
				{#if device.lastError}
					<p class="field-hint error-text">{device.lastError}</p>
				{/if}
			</div>
		{/if}
	</section>

	<section class="panel">
		<div class="panel-title">Receiver</div>
		<div class="panel-subtitle">The 2.4 GHz dongle: its light, pairing and a factory reset.</div>
		<div class="receiver-layout">
			<ReceiverArt status={pairing ? 'pairing' : pairingStarted ? 'paired' : 'idle'} />
			<div class="field-stack" style="flex:1">
				<SelectField
					label="Receiver light effect"
					value={receiverLight.mode}
					options={LIGHTING_MODES.map((m) => ({ value: m.value, label: m.label }))}
					onchange={(v) => (receiverLight = { ...receiverLight, mode: v })}
				/>
				<ColorSwatchPicker
					label="Color"
					color={receiverLight.color}
					onchange={(c) => (receiverLight = { ...receiverLight, color: c })}
				/>
				<RangeField
					label="Speed"
					min={0}
					max={9}
					value={receiverLight.speed}
					onchange={(v) => (receiverLight = { ...receiverLight, speed: v })}
				/>
				<RangeField
					label="Brightness"
					min={0}
					max={9}
					value={receiverLight.brightness}
					onchange={(v) => (receiverLight = { ...receiverLight, brightness: v })}
				/>
				<RangeField
					label="Timeout (raw)"
					hint="Unit is not specified in the protocol reference"
					min={0}
					max={255}
					value={receiverLight.time}
					onchange={(v) => (receiverLight = { ...receiverLight, time: v })}
				/>
				<button class="btn btn-primary" disabled={receiverSaving} onclick={applyReceiverLight}>
					{receiverSaving ? 'Applying...' : 'Apply receiver light'}
				</button>
			</div>
		</div>

		<div class="divider"></div>

		<div class="field-row">
			<div>
				<div class="field-label">Pairing</div>
				<div class="field-hint">
					After starting, hold the left, right and wheel buttons together for 3 seconds.
				</div>
			</div>
			<button class="btn" disabled={pairing} onclick={startPairing}>
				{pairing ? 'Starting...' : 'Start pairing'}
			</button>
		</div>

		<div class="divider"></div>

		<div class="field-row">
			<div>
				<div class="field-label">Factory reset</div>
				<div class="field-hint">Erases all on-device settings back to their defaults.</div>
			</div>
			{#if confirmingReset}
				<div class="confirm-row">
					<button class="btn btn-danger" onclick={doFactoryReset}>Confirm reset</button>
					<button class="btn" onclick={() => (confirmingReset = false)}>Cancel</button>
				</div>
			{:else}
				<button class="btn btn-danger" onclick={() => (confirmingReset = true)}>Factory reset</button>
			{/if}
		</div>
		{#if device.lastError}
			<p class="field-hint error-text">{device.lastError}</p>
		{/if}
	</section>
</div>

<style>
	.page-pad {
		padding: 0 32px 32px;
	}

	.field-stack {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.divider {
		height: 1px;
		background: var(--border-soft);
		margin: 16px 0;
	}

	.receiver-layout {
		display: flex;
		gap: 18px;
		align-items: flex-start;
		flex-wrap: wrap;
	}

	.receiver-layout :global(.receiver-art) {
		width: 130px;
		flex-shrink: 0;
	}

	.confirm-row {
		display: flex;
		gap: 8px;
	}

	.error-text {
		color: var(--danger);
	}
</style>
