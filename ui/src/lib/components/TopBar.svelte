<script lang="ts">
	import BatteryIndicator from './BatteryIndicator.svelte';
	import ConnectionBadge from './ConnectionBadge.svelte';

	interface Props {
		title: string;
		subtitle?: string;
	}

	let { title, subtitle }: Props = $props();
</script>

<header class="top-bar">
	<div class="titles">
		<h1>{title}</h1>
		{#if subtitle}<p class="subtitle">{subtitle}</p>{/if}
	</div>

	<!-- The window's one device readout: a single composed instrument, not three loose chips
	     (app.css .instrument/.segment). The tray carries the same facts for when the window is
	     closed; both read them from the same connection state, never from a default. -->
	<div class="instrument" aria-live="polite">
		<ConnectionBadge />
		<BatteryIndicator />
	</div>
</header>

<style>
	.top-bar {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--space-md);
		padding: var(--space-lg) var(--space-xl) var(--space-2xs);
	}

	/* font-family/weight/tracking come from app.css's global h1 rule (design.md Typography:
	   Chakra Petch 600 for screen titles); only the size is set here. */
	h1 {
		font-size: var(--text-display);
	}

	.subtitle {
		margin-top: var(--space-3xs);
		font-size: var(--text-sm);
		color: var(--color-muted);
	}

	.instrument {
		flex-shrink: 0;
		/* Sits on the display line's optical centre rather than its box top. */
		margin-top: var(--space-3xs);
	}

	@media (max-width: 900px) {
		.top-bar {
			flex-direction: column;
			gap: var(--space-2xs);
		}

		.instrument {
			margin-top: 0;
		}
	}
</style>
