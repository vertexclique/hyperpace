<script lang="ts">
	import { onMount } from 'svelte';
	import '../app.css';
	import type { Snippet } from 'svelte';
	import { device } from '$lib/device.svelte';

	let { children }: { children: Snippet } = $props();

	// Opens this window's device event channel once, app-wide, via the `device_state` command
	// (the contract's subscribe path). A no-op outside the Tauri shell.
	onMount(() => {
		void device.init();
	});
</script>

{@render children()}
