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
	   "nothing is round" rule forbids most on sight, so it gets no exception. Off and on are
	   deliberately far apart in weight, an empty dark track with a dim knob versus a solid accent
	   fill with a dark knob, the same fill/ink pairing .btn-primary uses for "this is the active
	   choice", so the state reads at a glance and never as a plain white square. */
	.switch {
		width: 40px;
		height: 22px;
		background: var(--color-paper);
		border: 1px solid var(--color-rule);
		padding: 3px;
		display: flex;
		align-items: center;
		justify-content: flex-start;
		transition:
			background var(--dur-settle) var(--ease-out),
			border-color var(--dur-settle) var(--ease-out);
		flex-shrink: 0;
	}

	.switch:hover:not(:disabled):not(.on) {
		border-color: var(--color-faint);
	}

	.switch.on {
		background: var(--color-accent);
		border-color: var(--color-accent);
		justify-content: flex-end;
	}

	.switch:disabled {
		opacity: 0.45;
	}

	.knob {
		width: 16px;
		height: 16px;
		background: var(--color-muted);
		transition: background var(--dur-settle) var(--ease-out);
	}

	.switch.on .knob {
		background: var(--color-accent-ink);
	}
</style>
