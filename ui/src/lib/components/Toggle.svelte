<script lang="ts">
	interface Props {
		checked: boolean;
		label: string;
		hint?: string;
		disabled?: boolean;
		onchange?: (value: boolean) => void;
	}

	let { checked = $bindable(false), label, hint, disabled = false, onchange }: Props = $props();

	function toggle() {
		if (disabled) return;
		checked = !checked;
		onchange?.(checked);
	}
</script>

<div class="field-row">
	<div>
		<div class="field-label">{label}</div>
		{#if hint}<div class="field-hint">{hint}</div>{/if}
	</div>
	<button
		type="button"
		role="switch"
		aria-checked={checked}
		aria-label={label}
		{disabled}
		class="switch"
		class:on={checked}
		onclick={toggle}
	>
		<span class="knob"></span>
	</button>
</div>

<style>
	/* A square track and a square knob: the pill-shaped switch is the one control design.md's
	   "nothing is round" rule forbids most on sight, so it gets no exception. */
	.switch {
		width: 36px;
		height: 20px;
		background: var(--color-paper);
		border: 1px solid var(--color-rule);
		padding: 2px;
		display: flex;
		align-items: center;
		justify-content: flex-start;
		transition: background var(--dur-settle) var(--ease-out);
		flex-shrink: 0;
	}

	.switch.on {
		background: var(--color-accent-soft);
		border-color: var(--color-accent);
		justify-content: flex-end;
	}

	.switch:disabled {
		opacity: 0.45;
	}

	.knob {
		width: 14px;
		height: 14px;
		background: var(--color-ink);
	}

	.switch.on .knob {
		background: var(--color-accent);
	}
</style>
