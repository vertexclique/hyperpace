<script lang="ts">
	import { device } from '../device.svelte';

	let percent = $derived(device.battery?.percent ?? null);
	let charging = $derived(device.battery?.charging ?? false);
	let tone = $derived(percent === null ? 'neutral' : percent <= 15 ? 'danger' : 'success');
</script>

{#if percent === null}
	<span class="tag tag-neutral">Battery: no data</span>
{:else}
	<span class="tag tag-{tone}" title="Estimated from the device's voltage curve">
		<svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
			<rect x="1" y="3" width="8.5" height="6" rx="1.2" stroke="currentColor" stroke-width="1" />
			<rect x="10" y="4.5" width="1.2" height="3" rx="0.4" fill="currentColor" />
			<rect
				x="2"
				y="4"
				width={Math.max(0.5, (percent / 100) * 6.5)}
				height="4"
				rx="0.6"
				fill="currentColor"
			/>
		</svg>
		{percent}%{charging ? ' - charging' : ''}
	</span>
{/if}
