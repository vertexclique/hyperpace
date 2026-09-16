<script lang="ts">
	// A small "?" beside a control that explains what the control does.
	//
	// The explanation is rendered at the end of <body>, not inside the control's panel: panels are
	// cut plates drawn with `clip-path`, which clips every descendant, positioned or not, so a
	// tooltip left in place would be sliced off at the panel's edge. It is placed from the icon's
	// own position when it opens, and flipped to stay inside the window.
	//
	// Every text comes from `help.ts`, one catalogue for the whole app, so the same setting is never
	// explained two different ways on two screens.
	import { HELP, type HelpEntry, type HelpKey } from '../help';

	interface Props {
		/** Which explanation to show. */
		topic: HelpKey;
	}

	let { topic }: Props = $props();

	const entry: HelpEntry = $derived(HELP[topic]);
	const id = $props.id();

	/** How long the pointer rests on the icon before the explanation opens; focus opens it at once. */
	const HOVER_DELAY_MS = 300;
	/** Gap between the icon and the explanation, and the minimum margin kept from the window edge. */
	const GAP = 8;

	let open = $state(false);
	let pinned = $state(false);
	let button: HTMLButtonElement | undefined = $state();
	let panel: HTMLDivElement | undefined = $state();
	let left = $state(0);
	let top = $state(0);
	let timer: ReturnType<typeof setTimeout> | undefined;

	function place() {
		if (!button || !panel) return;
		const anchor = button.getBoundingClientRect();
		const box = panel.getBoundingClientRect();
		let x = anchor.left;
		let y = anchor.bottom + GAP;
		if (x + box.width > window.innerWidth - GAP) x = window.innerWidth - GAP - box.width;
		if (y + box.height > window.innerHeight - GAP) y = anchor.top - GAP - box.height;
		left = Math.max(GAP, x);
		top = Math.max(GAP, y);
	}

	function show() {
		clearTimeout(timer);
		open = true;
	}

	function hide() {
		clearTimeout(timer);
		if (!pinned) open = false;
	}

	function hoverIn() {
		clearTimeout(timer);
		timer = setTimeout(show, HOVER_DELAY_MS);
	}

	function toggle(event: MouseEvent) {
		// A click must not reach the control the tip sits in (a toggle row, a dial, a tile).
		event.preventDefault();
		event.stopPropagation();
		pinned = !pinned;
		open = pinned;
	}

	function onKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape' && open) {
			event.stopPropagation();
			pinned = false;
			open = false;
		}
	}

	/** Move the element to the end of <body> for as long as it exists. */
	function portal(node: HTMLElement) {
		document.body.appendChild(node);
		return {
			destroy() {
				node.remove();
			}
		};
	}

	$effect(() => {
		if (!open) return;
		place();
		const reposition = () => place();
		window.addEventListener('resize', reposition);
		window.addEventListener('scroll', reposition, true);
		return () => {
			window.removeEventListener('resize', reposition);
			window.removeEventListener('scroll', reposition, true);
		};
	});

	$effect(() => () => clearTimeout(timer));
</script>

<button
	bind:this={button}
	type="button"
	class="help"
	class:open
	aria-label="What {entry.title} does"
	aria-describedby={open ? id : undefined}
	aria-expanded={open}
	onmouseenter={hoverIn}
	onmouseleave={hide}
	onfocus={show}
	onblur={() => {
		pinned = false;
		hide();
	}}
	onclick={toggle}
	onkeydown={onKeydown}
	onpointerdown={(event) => event.stopPropagation()}>?</button
>

{#if open}
	<div bind:this={panel} use:portal {id} role="tooltip" class="help-panel" style:left="{left}px" style:top="{top}px">
		<p class="help-title">{entry.title}</p>
		<p class="help-body">{entry.body}</p>
		{#if entry.note}<p class="help-note">{entry.note}</p>{/if}
	</div>
{/if}

<style>
	.help {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 16px;
		height: 16px;
		flex-shrink: 0;
		padding: 0;
		margin-left: var(--space-3xs);
		vertical-align: middle;
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		line-height: 1;
		color: var(--color-muted);
		background: var(--color-paper-3);
		border: 1px solid var(--color-rule);
		clip-path: polygon(0 0, calc(100% - 4px) 0, 100% 4px, 100% 100%, 0 100%);
		cursor: help;
		transition:
			color var(--dur-settle) var(--ease-out),
			border-color var(--dur-settle) var(--ease-out);
	}

	.help:hover,
	.help.open {
		color: var(--color-accent);
		border-color: var(--color-accent);
	}

	.help:focus-visible {
		outline: 2px solid var(--color-focus);
		outline-offset: 2px;
	}

	/* Rendered at the end of <body>, so these rules must not depend on any screen's own styles. */
	:global(.help-panel) {
		position: fixed;
		z-index: 1000;
		max-width: 300px;
		padding: var(--space-xs) var(--space-sm);
		background: var(--color-paper-3);
		border: 1px solid var(--color-rule);
		border-left: 2px solid var(--color-accent);
		clip-path: polygon(0 0, calc(100% - 8px) 0, 100% 8px, 100% 100%, 0 100%);
		pointer-events: none;
		text-align: left;
	}

	:global(.help-panel .help-title) {
		margin: 0 0 var(--space-3xs);
		font-family: var(--font-body);
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--color-ink);
	}

	:global(.help-panel .help-body) {
		margin: 0;
		font-family: var(--font-body);
		font-size: var(--text-sm);
		line-height: 1.45;
		color: var(--color-ink-2);
	}

	:global(.help-panel .help-note) {
		margin: var(--space-2xs) 0 0;
		font-family: var(--font-body);
		font-size: var(--text-xs);
		line-height: 1.4;
		color: var(--color-muted);
	}
</style>
