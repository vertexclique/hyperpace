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
		{ id: 'settings', label: 'Settings' }
	];
</script>

<nav class="sidebar" aria-label="Screens">
	<div class="brand">
		<!-- The Hyperpace mark, reduced to the two shapes that survive at rail size: the angular
		     shell and its head. The full logo (art/hyperpace.svg, also served at /hyperpace.svg)
		     carries a grid, corner brackets and traces that turn to mush below about 64px. -->
		<svg width="22" height="22" viewBox="0 0 512 512" aria-hidden="true" class="mark">
			<polygon
				points="256,80 350,170 340,390 256,450 172,390 162,170"
				fill="var(--color-paper-3)"
				stroke="var(--color-accent)"
				stroke-width="38"
				stroke-linejoin="miter"
			/>
			<polygon points="256,120 312,186 256,232 200,186" fill="var(--color-danger)" />
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
					<Icon name={item.id} />
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
		width: 220px;
		flex-shrink: 0;
		background: var(--color-paper-2);
		border-right: 1px solid var(--color-rule);
		padding: 16px 12px;
		height: 100%;
	}

	.brand {
		display: flex;
		align-items: center;
		gap: 9px;
		padding: 6px 8px 18px;
	}

	.brand-name {
		font-size: 14px;
		font-weight: 700;
		letter-spacing: 0.01em;
	}

	.nav-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1;
	}

	.nav-item {
		position: relative;
		width: 100%;
		display: flex;
		align-items: center;
		gap: 10px;
		background: transparent;
		border: none;
		color: var(--color-muted);
		padding: 9px 10px 9px 16px;
		font-size: 13px;
		font-weight: 500;
		text-align: left;
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
			width: 60px;
			padding: 16px 8px;
		}

		.brand-name {
			display: none;
		}

		.nav-item {
			justify-content: center;
			padding: 9px;
		}

		.nav-item span:last-child {
			display: none;
		}
	}
</style>
