<script lang="ts">
	import HelpTip from './HelpTip.svelte';
	import type { HelpKey } from '../help';

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
		/** Shows a "?" beside the label explaining what this field does (see help.ts). Omitted
		 * fields show no tip. */
		helpTopic?: HelpKey;
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
		helpTopic,
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
		<label class="field-label" for={uid}>{label}{#if helpTopic}<HelpTip topic={helpTopic} />{/if}</label>
		<span class="range-value mono" class:unknown>{unknown ? '-' : `${value}${unit}`}</span>
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
	/* The device's current value, so it carries the accent (design.md "Theme": accent appears
	   only on a live value, focus, and one primary action per screen). An unread field forces
	   `unknown`, which shows the em dash in the ordinary label color instead, never accented. */
	.range-value {
		font-size: var(--text-xs);
		font-weight: 600;
		color: var(--color-accent);
	}

	/* No real reading: the em dash stays in the ordinary muted tone, never the accent, so an
	   unread field is never mistaken for a live value (design.md "Per-screen allowances"). */
	.range-value.unknown {
		font-weight: 500;
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
		height: 4px;
		background: var(--color-rule);
	}

	input[type='range']::-webkit-slider-thumb {
		appearance: none;
		width: 14px;
		height: 14px;
		margin-top: -5px;
		background: var(--color-accent);
		transition: background var(--dur-settle) var(--ease-out);
	}

	input[type='range']:hover:not(:disabled)::-webkit-slider-thumb {
		background: var(--color-focus);
	}

	input[type='range']:disabled::-webkit-slider-thumb {
		background: var(--color-faint);
	}

	input[type='range']::-moz-range-track {
		height: 4px;
		background: var(--color-rule);
	}

	input[type='range']::-moz-range-thumb {
		width: 14px;
		height: 14px;
		background: var(--color-accent);
		border: none;
		transition: background var(--dur-settle) var(--ease-out);
	}

	input[type='range']:hover:not(:disabled)::-moz-range-thumb {
		background: var(--color-focus);
	}

	input[type='range']:disabled::-moz-range-thumb {
		background: var(--color-faint);
	}
</style>
