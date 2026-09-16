<script lang="ts">
	interface Props {
		title: string;
		subtitle?: string;
	}

	let { title, subtitle }: Props = $props();
</script>

<header class="top-bar">
	<h1>{title}</h1>
	{#if subtitle}<p class="subtitle">{subtitle}</p>{/if}
</header>

<style>
	.top-bar {
		padding: 26px 32px 6px;
	}

	h1 {
		font-size: 19px;
		font-weight: 650;
	}

	.subtitle {
		margin-top: 4px;
		font-size: 12.5px;
		color: var(--text-muted);
	}
</style>
