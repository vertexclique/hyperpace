<script lang="ts">
	// Small original line-icon set, hand-authored for this app. No third-party
	// icon set, no vendor artwork.
	import type { ScreenId } from '../types';

	interface Props {
		name: ScreenId;
		size?: number;
	}

	let { name, size = 18 }: Props = $props();
</script>

<svg
	width={size}
	height={size}
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width="1.6"
	stroke-linecap="round"
	stroke-linejoin="round"
	aria-hidden="true"
>
	{#if name === 'buttons'}
		<circle cx="7" cy="7" r="2.1" />
		<circle cx="17" cy="7" r="2.1" />
		<circle cx="12" cy="12" r="2.1" />
		<circle cx="7" cy="17" r="2.1" />
		<circle cx="17" cy="17" r="2.1" />
	{:else if name === 'performance'}
		<path d="M4 15a8 8 0 0 1 16 0" />
		<path d="M12 15l4.2-5.2" />
		<circle cx="12" cy="15" r="1.1" fill="currentColor" stroke="none" />
	{:else if name === 'macros'}
		<rect x="4" y="4" width="16" height="4.5" />
		<rect x="4" y="10.5" width="16" height="4.5" />
		<rect x="4" y="17" width="10" height="4.5" />
	{:else if name === 'lighting'}
		<path d="M9 18h6" />
		<path d="M10 21h4" />
		<path d="M12 3a6 6 0 0 0-3.4 10.9c.6.45.9 1.15.9 1.9V16h5v-.2c0-.75.3-1.45.9-1.9A6 6 0 0 0 12 3Z" />
	{:else if name === 'firmware'}
		<rect x="7" y="7" width="10" height="10" />
		<path d="M12 2.5v3M12 18.5v3M2.5 12h3M18.5 12h3" />
		<path d="M5 5l2 2M19 5l-2 2M5 19l2-2M19 19l-2-2" />
	{:else if name === 'settings'}
		<circle cx="12" cy="12" r="3" />
		<g stroke-width="1.8">
			<path d="M12 3v2.6M12 18.4V21M21 12h-2.6M5.6 12H3" />
			<path d="M18.4 5.6l-1.8 1.8M7.4 16.6l-1.8 1.8M18.4 18.4l-1.8-1.8M7.4 7.4 5.6 5.6" />
		</g>
	{/if}
</svg>
