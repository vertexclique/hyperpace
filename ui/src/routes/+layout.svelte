<script lang="ts">
	import { onMount } from 'svelte';
	import '../app.css';
	// Self-hosted, never fetched at runtime (design.md Typography): the app must render with no
	// network. Chakra Petch (600 only, the weight titles use) for display, Saira for body,
	// JetBrains Mono for every device value.
	import '@fontsource/chakra-petch/600.css';
	import '@fontsource-variable/saira';
	import '@fontsource-variable/jetbrains-mono';
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
