<script lang="ts">
	import type { RgbColor } from '../types';

	interface Props {
		color: RgbColor;
		label?: string;
		disabled?: boolean;
		onchange?: (color: RgbColor) => void;
	}

	let { color = $bindable([255, 255, 255]), label, disabled = false, onchange }: Props = $props();

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
		<input
			type="color"
			{disabled}
			value={toHex(color)}
			oninput={handleInput}
			aria-label={label ?? 'Color'}
		/>
		<span class="field-hint">{toHex(color)}</span>
	</div>
</div>

<style>
	.swatch-row {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	input[type='color'] {
		width: 34px;
		height: 26px;
		padding: 0;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: transparent;
	}
</style>
