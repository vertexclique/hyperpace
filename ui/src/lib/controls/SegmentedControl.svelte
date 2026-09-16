<script lang="ts" generics="T extends string | number">
	// A row of choices, not a circle: for two or three named values (sensor mode, lift-off) or a
	// short list of fixed rates (polling), a dial's ring adds a radius and a pointer with nothing
	// to show for it, and at few-enough segments a wide label crowds the ring or lands under the
	// pointer marker (operator feedback, 2026-09-16). This is the same choice contract as
	// SegmentDial (role, aria-*, arrow/Home/End keys, one commit per choice) in a plain cut-chip
	// bar instead, styled like app.css's `.instrument`/`.segment` readout but interactive.
	interface Option<V> {
		value: V;
		label: string;
	}

	interface Props<V> {
		values: Option<V>[];
		value: V;
		/** Accessible name for the whole control. */
		ariaLabel: string;
		disabled?: boolean;
		/** No real reading exists yet; see SegmentDial's own doc comment for the exact contract:
		 * hides the active highlight and forces the control disabled, never guesses a selection. */
		unknown?: boolean;
		onchange?: (value: V) => void;
	}

	let { values, value = $bindable(), ariaLabel, disabled = false, unknown = false, onchange }: Props<T> = $props();

	let effectiveDisabled = $derived(disabled || unknown);
	let selectedIndex = $derived(values.findIndex((option) => option.value === value));
	let focusIndex = $derived(selectedIndex === -1 ? 0 : selectedIndex);

	function commit(index: number) {
		if (effectiveDisabled || index === selectedIndex) return;
		const option = values[index];
		if (option) onchange?.(option.value);
	}

	function handleKeyDown(event: KeyboardEvent, index: number) {
		if (effectiveDisabled) return;
		let next = index;
		switch (event.key) {
			case 'ArrowRight':
			case 'ArrowDown':
				next = Math.min(values.length - 1, index + 1);
				break;
			case 'ArrowLeft':
			case 'ArrowUp':
				next = Math.max(0, index - 1);
				break;
			case 'Home':
				next = 0;
				break;
			case 'End':
				next = values.length - 1;
				break;
			default:
				return;
		}
		event.preventDefault();
		commit(next);
	}
</script>

<div class="segmented" class:disabled={effectiveDisabled} role="radiogroup" aria-label={ariaLabel}>
	{#each values as option, i (String(option.value))}
		<button
			type="button"
			role="radio"
			aria-checked={!unknown && i === selectedIndex}
			class="segmented-option"
			class:active={!unknown && i === selectedIndex}
			disabled={effectiveDisabled}
			tabindex={effectiveDisabled ? -1 : i === focusIndex ? 0 : -1}
			onclick={() => commit(i)}
			onkeydown={(e) => handleKeyDown(e, i)}
		>{option.label}</button>
	{/each}
</div>

<style>
	.segmented {
		display: flex;
		width: 100%;
		background: var(--color-paper-3);
		border: 1px solid var(--color-rule);
		clip-path: polygon(0 0, calc(100% - var(--cut-chip)) 0, 100% var(--cut-chip), 100% 100%, 0 100%);
	}

	.segmented.disabled {
		opacity: 0.45;
	}

	.segmented-option {
		flex: 1;
		min-width: 0;
		background: transparent;
		border: none;
		color: var(--color-ink-2);
		font-family: var(--font-body);
		font-size: var(--text-sm);
		font-weight: 500;
		padding: var(--space-2xs) var(--space-3xs);
		text-align: center;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		transition:
			background var(--dur-settle) var(--ease-out),
			color var(--dur-settle) var(--ease-out);
	}

	.segmented-option:disabled {
		cursor: not-allowed;
	}

	.segmented-option:not(:first-child) {
		border-left: 1px solid var(--color-rule);
	}

	.segmented-option:hover:not(:disabled):not(.active) {
		background: var(--color-paper-2);
		color: var(--color-ink);
	}

	.segmented-option.active {
		background: var(--color-accent);
		color: var(--color-accent-ink);
		font-weight: 600;
	}
</style>
