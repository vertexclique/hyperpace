<script lang="ts">
	// The manufacturer's own render (ui/static/device/receiver.png, 169x95; see
	// ui/static/device/PROVENANCE.md). The render carries no LED overlay in the source art, so
	// pairing state is read from a status chip below it rather than a tint on the photo itself
	// (design.md Assets: "the white faces are kept white", no tint on the render).
	//
	// `status` never claims "not paired" from missing data: `unknown` is the honest state when
	// this app has no positive proof either way (see LightingScreen's `receiverArtStatus`, which
	// derives it from the live wireless link, not from an idle pairing session).

	interface Props {
		status?: 'unknown' | 'pairing' | 'paired';
	}

	let { status = 'unknown' }: Props = $props();

	const STATUS_LABEL: Record<'unknown' | 'pairing' | 'paired', string> = {
		unknown: 'Pairing unknown',
		pairing: 'Pairing',
		paired: 'Paired'
	};

	const STATUS_TONE: Record<'unknown' | 'pairing' | 'paired', 'neutral' | 'warning' | 'ok'> = {
		unknown: 'neutral',
		pairing: 'warning',
		paired: 'ok'
	};
</script>

<!-- Root keeps the class name `receiver-art`: LightingScreen.svelte sizes this component from
     the outside with `.receiver-layout :global(.receiver-art)`. -->
<div class="receiver-art">
	<img src="/device/receiver.png" alt="Receiver" class="render" />
	<span class="chip chip-{STATUS_TONE[status]}">
		<span class="dot"></span>
		{STATUS_LABEL[status]}
	</span>
</div>

<style>
	.receiver-art {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-2xs);
	}

	/* No drop shadow, no rounded frame, no tint (design.md Assets). */
	.render {
		width: 100%;
		height: auto;
		display: block;
	}

	.dot {
		width: 5px;
		height: 5px;
		background: currentColor;
		flex-shrink: 0;
	}
</style>
