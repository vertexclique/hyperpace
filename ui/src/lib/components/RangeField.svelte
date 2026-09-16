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
		/** No real reading exists yet (no device connected, or its settings have not been read).
		 * Shows an em dash instead of `value`/`unit`, never a plausible-looking number, and forces
		 * `disabled`. The slider itself still needs a numeric `value` to render (an HTML
		 * requirement); it is never shown, since the label text is what the user reads. */
		unknown?: boolean;
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
		unknown = false,
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
		<span class="range-value mono">{unknown ? '-' : `${value}${unit}`}</span>
	</div>
	{#if hint}<div class="field-hint">{hint}</div>{/if}
	<input
		id={uid}
		type="range"
		{min}
		{max}
		{step}
		disabled={disabled || unknown}
		value={String(unknown ? min : value)}
		oninput={(e) => commit(Number((e.target as HTMLInputElement).value))}
	/>
</div>

<style>
	.range-value {
		font-size: var(--text-xs);
		color: var(--color-muted);
	}

	/* accent-color's themed thumb is round in every engine, and zeroing a radius property to
	   cancel that is exactly what the hard rule forbids (design.md "Shape: nothing is round"
	   says delete the declaration, never write a zero one). A fully custom track and thumb
	   sidesteps the theming instead: appearance: none draws a plain box with no radius of its
	   own, so it is square because nothing ever asks it to be round. */
	input[type='range'] {
		width: 100%;
		appearance: none;
		background: transparent;
	}

	input[type='range']::-webkit-slider-runnable-track {
		height: 3px;
		background: var(--color-rule);
	}

	input[type='range']::-webkit-slider-thumb {
		appearance: none;
		width: 12px;
		height: 12px;
		margin-top: -4.5px;
		background: var(--color-accent);
	}

	input[type='range']:disabled::-webkit-slider-thumb {
		background: var(--color-faint);
	}

	input[type='range']::-moz-range-track {
		height: 3px;
		background: var(--color-rule);
	}

	input[type='range']::-moz-range-thumb {
		width: 12px;
		height: 12px;
		background: var(--color-accent);
		border: none;
	}

	input[type='range']:disabled::-moz-range-thumb {
		background: var(--color-faint);
	}
</style>
