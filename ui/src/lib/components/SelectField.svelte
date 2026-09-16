<script lang="ts" generics="T extends string | number">
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
		/** Keep the label for screen readers, but don't render it visibly (e.g. inside a table with column headers). */
		hideLabel?: boolean;
		onchange?: (value: T) => void;
	}

	let {
		value = $bindable(),
		label,
		hint,
		options,
		disabled = false,
		hideLabel = false,
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
	<label class="field-label" class:sr-only={hideLabel} for={uid}>{label}</label>
	{#if hint}<div class="field-hint">{hint}</div>{/if}
	<select id={uid} {disabled} value={String(value)} onchange={handleChange}>
		{#each options as option (option.value)}
			<option value={String(option.value)}>{option.label}</option>
		{/each}
	</select>
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
</style>
