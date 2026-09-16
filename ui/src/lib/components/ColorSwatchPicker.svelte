<script lang="ts">
	import type { RgbColor } from '../types';

	interface Props {
		color: RgbColor;
		label?: string;
		disabled?: boolean;
		/** No real reading exists yet (no device connected, or its settings have not been read).
		 * Shows an em dash and a neutral gray swatch instead of `color`, never a plausible-looking
		 * hex value, and forces `disabled`. */
		unknown?: boolean;
		onchange?: (color: RgbColor) => void;
	}

	let {
		color = $bindable([255, 255, 255]),
		label,
		disabled = false,
		unknown = false,
		onchange
	}: Props = $props();

	function toHex(c: RgbColor): string {
		const part = (n: number) => n.toString(16).padStart(2, '0');
		return `#${part(c[0])}${part(c[1])}${part(c[2])}`;
	}

	function fromHex(hex: string): RgbColor {
		return [parseInt(hex.slice(1, 3), 16), parseInt(hex.slice(3, 5), 16), parseInt(hex.slice(5, 7), 16)];
	}

	function handleInput(e: Event) {
		color = fromHex((e.target as HTMLInputElement).value);
		onchange?.(color);
	}
</script>

<div class="field">
	{#if label}<span class="field-label">{label}</span>{/if}
	<div class="swatch-row">
		<!-- The colour is always a value this app knows (a device reading or the draft in hand),
		     never one of the vendor's 14 preset preview bitmaps (ui/static/device/colors/): those
		     are flat swatches of the same RGB this control already holds, so the swatch is drawn
		     from the value itself instead of an extra image request. -->
		<span class="swatch" class:unknown style:background={unknown ? null : toHex(color)}></span>
		<input
			type="color"
			disabled={disabled || unknown}
			value={unknown ? '#808080' : toHex(color)}
			oninput={handleInput}
			aria-label={label ?? 'Color'}
		/>
		<span class="field-hint mono">{unknown ? '-' : toHex(color)}</span>
	</div>
</div>

<style>
	.swatch-row {
		display: flex;
		align-items: center;
		gap: var(--space-2xs);
	}

	/* Cut at --cut-chip, same as every other swatch, tag and badge (design.md "Shape"). */
	.swatch {
		width: 18px;
		height: 18px;
		flex-shrink: 0;
		border: 1px solid var(--color-rule);
		clip-path: polygon(0 0, calc(100% - var(--cut-chip)) 0, 100% var(--cut-chip), 100% 100%, 0 100%);
	}

	/* No reading yet: a plain neutral fill stands in for the colour, never a guessed one. */
	.swatch.unknown {
		background: var(--color-paper-3);
	}

	input[type='color'] {
		width: 34px;
		height: 26px;
		padding: 0;
		border: 1px solid var(--color-rule);
		background: transparent;
	}
</style>
