<script lang="ts" generics="T extends string | number">
	import HelpTip from './HelpTip.svelte';
	import type { HelpKey } from '../help';

	interface Option {
		value: T;
		label: string;
	}

	interface Props {
		value: T;
		label: string;
		hint?: string;
		options: Option[];
		disabled?: boolean;
		/** No real reading exists yet (no device connected, or its settings have not been read).
		 * Renders a disabled select offering only an em dash, never one of `options` pre-selected
		 * as if it were a real reading. */
		unknown?: boolean;
		/** Keep the label for screen readers, but don't render it visibly (e.g. inside a table with column headers). */
		hideLabel?: boolean;
		/** Shows a "?" beside the label explaining what this field does (see help.ts). Omitted
		 * fields show no tip. */
		helpTopic?: HelpKey;
		onchange?: (value: T) => void;
	}

	let {
		value = $bindable(),
		label,
		hint,
		options,
		disabled = false,
		unknown = false,
		hideLabel = false,
		helpTopic,
		onchange
	}: Props = $props();

	// See RangeField: a per-instance id avoids collisions between fields
	// that share the same label text on one screen.
	const uid = $props.id();

	function handleChange(e: Event) {
		const raw = (e.target as HTMLSelectElement).value;
		const match = options.find((o) => String(o.value) === raw);
		if (match) {
			value = match.value;
			onchange?.(match.value);
		}
	}
</script>

<div class="field">
	<label class="field-label" class:sr-only={hideLabel} for={uid}>{label}{#if helpTopic}<HelpTip topic={helpTopic} />{/if}</label>
	{#if hint}<div class="field-hint">{hint}</div>{/if}
	<!-- appearance: none (app.css) strips the native dropdown arrow, so without one drawn here a
	     select reads as a plain text box, not a control with more choices behind it. A plain
	     angular chevron, never a rounded caret (design.md "Shape: nothing is round"). -->
	<div class="select-shell">
		{#if unknown}
			<select id={uid} disabled value="unknown">
				<option value="unknown">{'-'}</option>
			</select>
		{:else}
			<select id={uid} {disabled} value={String(value)} onchange={handleChange}>
				{#each options as option (option.value)}
					<option value={String(option.value)}>{option.label}</option>
				{/each}
			</select>
		{/if}
		<svg class="chevron" width="10" height="6" viewBox="0 0 10 6" aria-hidden="true">
			<path
				d="M1 1l4 4 4-4"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="square"
				stroke-linejoin="miter"
			/>
		</svg>
	</div>
</div>

<style>
	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}

	.select-shell {
		position: relative;
		display: flex;
	}

	.select-shell select {
		width: 100%;
		padding-right: calc(var(--space-lg) + 2px);
	}

	.chevron {
		position: absolute;
		right: var(--space-xs);
		top: 50%;
		transform: translateY(-50%);
		color: var(--color-muted);
		pointer-events: none;
	}

	.select-shell select:disabled ~ .chevron {
		opacity: 0.45;
	}
</style>
