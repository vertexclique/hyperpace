<script lang="ts">
	import { device, isTauriShell } from '../device.svelte';
	import { linkDetail, presenceLabel, presenceOf, statusLine, toneOf } from '../status';

	let presence = $derived(presenceOf(device, isTauriShell));
	let tone = $derived(toneOf(presence));
	let detail = $derived(linkDetail(device, isTauriShell));
</script>

<span class="connection" title={statusLine(device, isTauriShell)}>
	<span class="chip chip-{tone}">
		<span class="dot"></span>
		{presenceLabel(presence)}
	</span>
	{#if detail}<span class="detail mono">{detail}</span>{/if}
</span>

<style>
	.connection {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2xs);
	}

	.dot {
		width: 5px;
		height: 5px;
		background: currentColor;
		flex-shrink: 0;
	}

	.detail {
		font-size: var(--text-xs);
		color: var(--color-muted);
		white-space: nowrap;
	}
</style>
