<script lang="ts">
	// Original receiver illustration for Hyperpace: a small USB dongle body
	// with a status LED. No vendor artwork or branding.

	interface Props {
		status?: 'idle' | 'pairing' | 'paired';
	}

	let { status = 'idle' }: Props = $props();
</script>

<svg viewBox="0 0 140 100" class="receiver-art" role="img" aria-label="Receiver">
	<rect class="plug" x="6" y="40" width="26" height="20" rx="3" />
	<rect class="body" x="30" y="18" width="86" height="64" rx="12" />
	<rect class="seam" x="30" y="46" width="86" height="1.4" />
	<circle
		class="led"
		class:pairing={status === 'pairing'}
		class:paired={status === 'paired'}
		cx="96"
		cy="32"
		r="4.2"
	/>
	<path class="wave" d="M104,58 q6,6 0,12" />
	<path class="wave wave-2" d="M110,54 q12,10 0,20" />
</svg>

<style>
	.receiver-art {
		width: 100%;
		max-width: 150px;
		height: auto;
	}

	.plug {
		fill: var(--bg-raised);
		stroke: var(--border);
		stroke-width: 1.2;
	}

	.body {
		fill: var(--panel-raised);
		stroke: var(--border);
		stroke-width: 1.5;
	}

	.seam {
		fill: var(--border);
	}

	.led {
		fill: var(--text-faint);
	}

	.led.pairing {
		fill: var(--warning);
		animation: pulse 1s ease-in-out infinite;
	}

	.led.paired {
		fill: var(--success);
	}

	.wave {
		fill: none;
		stroke: var(--text-faint);
		stroke-width: 1.4;
		stroke-linecap: round;
	}

	.wave-2 {
		opacity: 0.6;
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.35;
		}
	}
</style>
