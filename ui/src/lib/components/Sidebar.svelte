<script lang="ts">
	import type { ScreenId } from '../types';
	import Icon from './Icon.svelte';

	interface Props {
		active: ScreenId;
		onselect: (screen: ScreenId) => void;
	}

	let { active, onselect }: Props = $props();

	const items: { id: ScreenId; label: string }[] = [
		{ id: 'buttons', label: 'Buttons' },
		{ id: 'performance', label: 'Performance' },
		{ id: 'macros', label: 'Macros' },
		{ id: 'lighting', label: 'Lighting and receiver' },
		{ id: 'firmware', label: 'Firmware' },
		{ id: 'data', label: 'Data management' },
		{ id: 'settings', label: 'Settings' }
	];
</script>

<nav class="sidebar" aria-label="Screens">
	<div class="brand">
		<!-- The Hyperpace mark, inlined from art/hyperpace.svg (also served at /hyperpace.svg) at
		     rail scale (design.md "The mark", the ~46px "rail mark" tier): the three pieces that
		     still read clearly at this size, the angular shell, its red head, and the two neon
		     corner brackets. The background grid and the fine data-node traces are dropped; both
		     turn to a grey smear once the artwork is this small. Stroke widths are drawn heavier
		     than the source file's so a hairline in a 512-unit box still reads as a solid line once
		     scaled down to ~46px on screen. -->
		<svg width="46" height="46" viewBox="0 0 512 512" aria-hidden="true" class="mark">
			<path d="M 24 120 L 24 64 L 80 24" fill="none" stroke="var(--color-accent)" stroke-width="22" stroke-linecap="square" />
			<path d="M 488 392 L 488 448 L 432 488" fill="none" stroke="var(--color-accent)" stroke-width="22" stroke-linecap="square" />
			<polygon
				points="256,80 350,170 340,390 256,450 172,390 162,170"
				fill="var(--color-paper-3)"
				stroke="var(--color-accent)"
				stroke-width="22"
				stroke-linejoin="miter"
			/>
			<polygon points="256,110 310,180 256,220 202,180" fill="none" stroke="#FF003C" stroke-width="18" stroke-linejoin="miter" />
		</svg>
		<span class="brand-name">Hyperpace</span>
	</div>

	<ul class="nav-list">
		{#each items as item (item.id)}
			<li>
				<button
					type="button"
					class="nav-item"
					class:active={active === item.id}
					onclick={() => onselect(item.id)}
					aria-current={active === item.id ? 'page' : undefined}
					title={item.label}
				>
					<Icon name={item.id} size={20} />
					<span>{item.label}</span>
				</button>
			</li>
		{/each}
	</ul>

</nav>

<style>
	.sidebar {
		display: flex;
		flex-direction: column;
		width: 232px;
		flex-shrink: 0;
		background: var(--color-paper-2);
		border-right: 1px solid var(--color-rule);
		padding: 16px 14px;
		height: 100%;
	}

	.brand {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 10px 6px 22px;
	}

	.mark {
		flex-shrink: 0;
	}

	/* The wordmark, in the display face, sized to read as the product name (design.md "The mark"),
	   not a menu item: nearly double the tallest nav label beside it. */
	.brand-name {
		font-family: var(--font-display);
		font-size: 1.5rem;
		font-weight: 600;
		line-height: 1;
		letter-spacing: -0.02em;
		color: var(--color-ink);
	}

	.nav-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
		flex: 1;
	}

	.nav-item {
		position: relative;
		width: 100%;
		display: flex;
		align-items: center;
		gap: 12px;
		background: transparent;
		border: none;
		color: var(--color-muted);
		padding: 12px 12px 12px 18px;
		font-size: 14px;
		font-weight: 500;
		text-align: left;
		transition:
			background var(--dur-settle) var(--ease-out),
			color var(--dur-settle) var(--ease-out);
	}

	.nav-item :global(svg) {
		flex-shrink: 0;
	}

	.nav-item:hover {
		background: var(--color-paper-3);
		color: var(--color-ink);
	}

	.nav-item.active {
		background: var(--color-paper-3);
		color: var(--color-ink);
	}

	/* The active screen is marked by the accent stub (design.md "Shape"), never a filled pill.
	   An absolutely positioned pseudo-element, not the shared .plate-stub child, so the icon and
	   label never shift left when a row becomes active; it mirrors .plate-stub's 2px / 24px. */
	.nav-item.active::before {
		content: '';
		position: absolute;
		left: 0;
		top: 50%;
		transform: translateY(-50%);
		width: 2px;
		height: 24px;
		background: var(--color-accent);
	}

	/* Below the window's minimum width, the sidebar collapses to icons instead of eating into
	   the content area: labels disappear, the nav column narrows and its buttons center their
	   icon. */
	@media (max-width: 900px) {
		.sidebar {
			width: 68px;
			padding: 16px 10px;
		}

		.brand {
			justify-content: center;
			padding: 10px 0 22px;
		}

		.brand-name {
			display: none;
		}

		.nav-item {
			justify-content: center;
			padding: 12px;
		}

		.nav-item span:last-child {
			display: none;
		}
	}
</style>
