<script lang="ts">
	import { device, isTauriShell } from '../device.svelte';
	import { linkDetail, presenceLabel, presenceOf, statusLine, toneOf } from '../status';

	let presence = $derived(presenceOf(device, isTauriShell));
	let tone = $derived(toneOf(presence));
	let detail = $derived(linkDetail(device, isTauriShell));
</script>

<!-- Two segments of the top bar's one instrument cluster (app.css .instrument/.segment): presence
     always renders, the link detail only once the identity has actually resolved. Both render as
     plain siblings so TopBar's single .instrument wrapper lays them out with BatteryIndicator's
     own segment. -->
<span class="segment presence tone-{tone}" title={statusLine(device, isTauriShell)}>
	<span class="dot" aria-hidden="true"></span>
	{presenceLabel(presence)}
</span>
{#if detail}
	<span class="segment link mono">{detail}</span>
{/if}

<style>
	.presence {
		font-size: var(--text-xs);
		font-weight: 600;
		letter-spacing: 0.04em;
		text-transform: uppercase;
	}

	.dot {
		width: 6px;
		height: 6px;
		flex-shrink: 0;
		background: currentColor;
	}

	.tone-ok {
		color: var(--color-ok);
	}

	.tone-warning {
		color: var(--color-warning);
	}

	.tone-neutral {
		color: var(--color-muted);
	}

	.link {
		font-size: var(--text-xs);
		font-weight: 500;
		color: var(--color-ink-2);
	}
</style>
