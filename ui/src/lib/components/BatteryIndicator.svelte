<script lang="ts">
	import { device } from '../device.svelte';

	// docs/architecture/ui-controls.md 4 ("no default DPI, brightness or timeout is displayed
	// as though it came from the mouse"): the same honesty rule applies to battery. With no
	// reading, this renders nothing at all rather than a "no data" placeholder or a 0% meter.
	let percent = $derived(device.battery?.percent ?? null);
	let charging = $derived(device.battery?.charging ?? false);
	let tone = $derived(percent !== null && percent <= 15 ? 'danger' : 'ok');

	const SEGMENTS = 5;
	const segmentIndices = Array.from({ length: SEGMENTS }, (_, i) => i);
	let filled = $derived(
		percent === null ? 0 : Math.max(percent > 0 ? 1 : 0, Math.round((percent / 100) * SEGMENTS))
	);
</script>

{#if percent !== null}
	<div
		class="battery chip chip-{tone}"
		title="Estimated from the device's voltage curve"
		role="img"
		aria-label="Battery {percent} percent{charging ? ', charging' : ''}"
	>
		<div class="meter" aria-hidden="true">
			{#each segmentIndices as i (i)}
				<span class="segment" class:filled={i < filled}></span>
			{/each}
		</div>
		<span class="percent mono">{percent}%</span>
		{#if charging}
			<svg class="bolt" width="8" height="11" viewBox="0 0 8 11" aria-hidden="true">
				<polygon points="4.5,0 0,6.2 3,6.2 2,11 7.5,4.2 4.3,4.2" fill="currentColor" />
			</svg>
		{/if}
	</div>
{/if}

<style>
	.battery {
		gap: var(--space-2xs);
	}

	/* Square-ended segments with tick divisions, never a rounded capsule (design.md "Shape"). A
	   bitmap dial face (key/battery.png) exists but is built for the ~130px dial tier; shrunk to
	   this rail's scale it reads as a muddy grey blob, so the meter is drawn instead. */
	.meter {
		display: flex;
		gap: 1px;
		height: 8px;
	}

	.segment {
		width: 3px;
		background: var(--color-rule);
	}

	.segment.filled {
		background: currentColor;
	}

	.bolt {
		flex-shrink: 0;
	}
</style>
