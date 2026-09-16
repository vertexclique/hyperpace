<script lang="ts">
	import type { ScreenId } from '$lib/types';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import TopBar from '$lib/components/TopBar.svelte';
	import ButtonsScreen from '$lib/screens/ButtonsScreen.svelte';
	import PerformanceScreen from '$lib/screens/PerformanceScreen.svelte';
	import MacrosScreen from '$lib/screens/MacrosScreen.svelte';
	import LightingScreen from '$lib/screens/LightingScreen.svelte';
	import FirmwareScreen from '$lib/screens/FirmwareScreen.svelte';
	import SettingsScreen from '$lib/screens/SettingsScreen.svelte';

	let active = $state<ScreenId>('performance');

	const titles: Record<ScreenId, { title: string; subtitle: string }> = {
		buttons: { title: 'Buttons', subtitle: 'Assign an action to each button.' },
		performance: { title: 'Performance', subtitle: 'DPI, polling, tracking and power.' },
		macros: { title: 'Macros', subtitle: 'Record, edit and bind macros to buttons.' },
		lighting: {
			title: 'Lighting and receiver',
			subtitle: 'Body lighting, the DPI indicator, and the wireless receiver.'
		},
		firmware: { title: 'Firmware', subtitle: 'Install, roll back and import firmware packages.' },
		settings: { title: 'Settings', subtitle: 'Connection, profiles, backups and app preferences.' }
	};
</script>

<div class="shell">
	<Sidebar {active} onselect={(s) => (active = s)} />
	<main class="content">
		<TopBar title={titles[active].title} subtitle={titles[active].subtitle} />
		{#if active === 'buttons'}
			<ButtonsScreen />
		{:else if active === 'performance'}
			<PerformanceScreen />
		{:else if active === 'macros'}
			<MacrosScreen />
		{:else if active === 'lighting'}
			<LightingScreen />
		{:else if active === 'firmware'}
			<FirmwareScreen />
		{:else if active === 'settings'}
			<SettingsScreen />
		{/if}
	</main>
</div>

<style>
	/* Fixed viewport height with the sidebar pinned; only `.content` scrolls, so a tall screen
	   never drags the sidebar out of view or grows the window past its own bounds (the window
	   never resizes to fit content; content fits, or scrolls, inside it). */
	.shell {
		display: flex;
		height: 100vh;
		overflow: hidden;
	}

	.content {
		flex: 1;
		min-width: 0;
		height: 100vh;
		overflow-y: auto;
		overflow-x: hidden;
	}
</style>
