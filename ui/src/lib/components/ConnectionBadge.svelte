<script lang="ts">
	import { device, isTauriShell } from '../device.svelte';

	let tone = $derived(
		!isTauriShell
			? 'neutral'
			: !device.connected
				? 'neutral'
				: device.online
					? 'success'
					: 'warning'
	);

	let label = $derived(
		!isTauriShell
			? 'Preview mode'
			: !device.connected
				? 'No device'
				: device.online
					? 'Connected'
					: 'Asleep'
	);
</script>

<span class="tag tag-{tone}">
	<span class="dot"></span>
	{label}
</span>

<style>
	.dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: currentColor;
	}
</style>
