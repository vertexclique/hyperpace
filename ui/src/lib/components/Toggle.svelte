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
	.switch {
		width: 38px;
		height: 22px;
		border-radius: 999px;
		background: var(--bg-raised);
		border: 1px solid var(--border);
		padding: 2px;
		display: flex;
		align-items: center;
		justify-content: flex-start;
		transition: background 0.15s ease;
		flex-shrink: 0;
	}

	.switch.on {
		background: var(--accent);
		border-color: var(--accent);
		justify-content: flex-end;
	}

	.switch:disabled {
		opacity: 0.45;
	}

	.knob {
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: var(--text);
	}
</style>
