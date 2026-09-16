<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		message: string;
		actions?: Snippet;
	}

	let { title, message, actions }: Props = $props();
</script>

<div class="empty-state">
	<strong>{title}</strong>
	<p>{message}</p>
	{#if actions}
		<div class="empty-actions">
			{@render actions()}
		</div>
	{/if}
</div>

<style>
	.empty-actions {
		display: flex;
		gap: 8px;
		margin-top: 8px;
	}
</style>
