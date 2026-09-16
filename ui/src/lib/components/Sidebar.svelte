<script lang="ts">
	import type { ScreenId } from '../types';
	import Icon from './Icon.svelte';
	import ConnectionBadge from './ConnectionBadge.svelte';
	import BatteryIndicator from './BatteryIndicator.svelte';

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
		<svg width="20" height="20" viewBox="0 0 24 24" aria-hidden="true">
			<rect x="3" y="3" width="18" height="18" rx="6" fill="var(--accent)" />
			<circle cx="12" cy="9" r="2.1" fill="#06090f" />
			<rect x="10.2" y="12.5" width="3.6" height="6.5" rx="1.8" fill="#06090f" />
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

	<div class="sidebar-footer">
		<ConnectionBadge />
		<BatteryIndicator />
	</div>
</nav>

<style>
	.sidebar {
		display: flex;
		flex-direction: column;
		width: 220px;
		flex-shrink: 0;
		background: var(--bg-raised);
		border-right: 1px solid var(--border);
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
		width: 100%;
		display: flex;
		align-items: center;
		gap: 10px;
		background: transparent;
		border: none;
		color: var(--text-muted);
		padding: 9px 10px;
		border-radius: var(--radius-sm);
		font-size: 13px;
		font-weight: 500;
		text-align: left;
	}

	.nav-item:hover {
		background: var(--panel-raised);
		color: var(--text);
	}

	.nav-item.active {
		background: var(--accent-soft);
		color: var(--accent-strong);
	}

	.sidebar-footer {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding-top: 12px;
		border-top: 1px solid var(--border-soft);
	}

	/* Below the window's minimum width, the sidebar collapses to icons instead of eating into
	   the content area: labels and badge text disappear (via font-size, so a child component's
	   own icon glyphs, sized in px rather than em, stay visible), the nav column narrows and its
	   buttons center their icon. */
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

		.sidebar-footer {
			align-items: center;
		}

		.sidebar-footer :global(.tag) {
			font-size: 0;
			padding: 4px;
			justify-content: center;
			gap: 0;
		}
	}
</style>
