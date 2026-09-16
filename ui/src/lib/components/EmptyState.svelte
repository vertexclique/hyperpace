<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		message: string;
		actions?: Snippet;
	}

	let { title, message, actions }: Props = $props();
</script>

<!-- A compact, on-brand placeholder: no dashed border, no oversized padding. It sits inline in
     whatever plate hosts it (design.md's cut plate already carries the frame), so this only owns
     its own text rhythm, never a second box drawn around itself. -->
<div class="empty">
	<p class="empty-title">{title}</p>
	<p class="empty-message">{message}</p>
	{#if actions}
		<div class="empty-actions">
			{@render actions()}
		</div>
	{/if}
</div>

<style>
	.empty {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: var(--space-3xs);
		padding: var(--space-sm) 0;
	}

	.empty-title {
		margin: 0;
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--color-ink);
	}

	.empty-message {
		margin: 0;
		font-size: var(--text-xs);
		color: var(--color-muted);
		max-width: 52ch;
	}

	.empty-actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2xs);
		margin-top: var(--space-2xs);
	}
</style>
