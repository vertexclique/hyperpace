<script lang="ts">
	interface Props {
		value: number;
		label: string;
		hint?: string;
		min: number;
		max: number;
		step?: number;
		unit?: string;
		disabled?: boolean;
		onchange?: (value: number) => void;
	}

	let {
		value = $bindable(0),
		label,
		hint,
		min,
		max,
		step = 1,
		unit = '',
		disabled = false,
		onchange
	}: Props = $props();

	// A DOM id derived from the label would collide whenever two fields on
	// the same screen share a label (e.g. "Speed" in two lighting panels);
	// this generates one unique per component instance instead.
	const uid = $props.id();

	function commit(next: number) {
		value = Math.min(max, Math.max(min, next));
		onchange?.(value);
	}
</script>

<div class="field">
	<div class="field-row">
		<label class="field-label" for={uid}>{label}</label>
		<span class="range-value">{value}{unit}</span>
	</div>
	{#if hint}<div class="field-hint">{hint}</div>{/if}
	<input
		id={uid}
		type="range"
		{min}
		{max}
		{step}
		{disabled}
		value={String(value)}
		oninput={(e) => commit(Number((e.target as HTMLInputElement).value))}
	/>
</div>

<style>
	.range-value {
		font-size: 12px;
		color: var(--text-muted);
		font-variant-numeric: tabular-nums;
	}

	input[type='range'] {
		width: 100%;
		accent-color: var(--accent);
		background: transparent;
	}
</style>
