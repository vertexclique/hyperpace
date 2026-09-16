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
		class="segment battery tone-{tone}"
		title="Estimated from the device's voltage curve"
		role="img"
		aria-label="Battery {percent} percent{charging ? ', charging' : ''}"
	>
		<div class="meter" aria-hidden="true">
			{#each segmentIndices as i (i)}
				<span class="cell" class:filled={i < filled}></span>
			{/each}
		</div>
		<span class="percent mono">{percent}%</span>
		<!-- Unmistakable, not just an icon: the operator's own complaint was that charging never
		     showed at all. Always the ok colour, regardless of the percent's own tone, because an
		     actively charging mouse is never the thing to flag as low. -->
		{#if charging}
			<span class="charging">
				<svg class="bolt" width="9" height="12" viewBox="0 0 8 11" aria-hidden="true">
					<polygon points="4.5,0 0,6.2 3,6.2 2,11 7.5,4.2 4.3,4.2" fill="currentColor" />
				</svg>
				Charging
			</span>
		{/if}
	</div>
{/if}

<style>
	.battery {
		font-size: var(--text-xs);
	}

	/* Square-ended cells with tick divisions, never a rounded capsule (design.md "Shape"). A
	   bitmap dial face (key/battery.png) exists but is built for the ~130px dial tier; shrunk to
	   this cluster's scale it reads as a muddy grey blob, so the meter is drawn instead. */
	.meter {
		display: flex;
		gap: 1px;
		height: 9px;
	}

	.cell {
		width: 3px;
		background: var(--color-rule);
	}

	.tone-ok .cell.filled {
		background: var(--color-ok);
	}

	.tone-danger .cell.filled {
		background: var(--color-danger);
	}

	.percent {
		font-weight: 600;
	}

	.tone-ok .percent {
		color: var(--color-ok);
	}

	.tone-danger .percent {
		color: var(--color-danger);
	}

	.charging {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		color: var(--color-ok);
		font-weight: 700;
		letter-spacing: 0.04em;
		text-transform: uppercase;
	}

	.bolt {
		flex-shrink: 0;
	}
</style>
