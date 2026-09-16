<script lang="ts">
	// Original top-view mouse illustration, drawn for Hyperpace. Not a trace
	// or reference of any vendor's product: an abstract capsule silhouette
	// with five numbered button hotspots (left, right, wheel, back, forward).

	interface Props {
		selected: number | null;
		onselect: (index: number) => void;
		labels?: string[];
		/** How many of the five hotspots correspond to a real button on the connected device. */
		activeCount?: number;
	}

	let {
		selected,
		onselect,
		labels = ['Left click', 'Right click', 'Wheel click', 'Back', 'Forward'],
		activeCount = 5
	}: Props = $props();

	const hotspots = [
		{ x: 78, y: 68, r: 20 },
		{ x: 162, y: 68, r: 20 },
		{ x: 120, y: 108, r: 17 },
		{ x: 33, y: 168, r: 16 },
		{ x: 33, y: 212, r: 16 }
	];

	function handleKey(e: KeyboardEvent, index: number) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			onselect(index);
		}
	}
</script>

<svg viewBox="0 0 240 380" class="mouse-art" role="img" aria-label="Mouse, top view, five buttons">
	<!-- body -->
	<path
		class="body"
		d="M120,20 C60,20 30,72 26,152 C22,234 42,322 82,357 C97,370 143,370 158,357
			C198,322 218,234 214,152 C210,72 180,20 120,20 Z"
	/>

	<!-- scroll wheel -->
	<rect class="wheel" x="107" y="88" width="26" height="42" rx="11" />

	<!-- seams -->
	<path class="seam" d="M120,24 L120,86" />
	<path class="seam" d="M107,128 C88,146 68,158 46,164" />
	<path class="seam" d="M133,128 C152,146 172,158 194,164" />

	<!-- side ridge suggesting the thumb buttons -->
	<path class="ridge" d="M18,150 C10,168 10,212 18,230" />

	{#each hotspots as spot, i (i)}
		{@const isActive = i < activeCount}
		<g
			class="hotspot"
			class:selected={selected === i}
			class:inactive={!isActive}
			role="button"
			tabindex={isActive ? 0 : -1}
			aria-label={labels[i]}
			aria-disabled={!isActive}
			onclick={() => isActive && onselect(i)}
			onkeydown={(e) => isActive && handleKey(e, i)}
		>
			<circle cx={spot.x} cy={spot.y} r={spot.r} />
			<text x={spot.x} y={spot.y + 4.5} text-anchor="middle">{i + 1}</text>
		</g>
	{/each}
</svg>

<style>
	.mouse-art {
		width: 100%;
		max-width: 220px;
		height: auto;
	}

	.body {
		fill: var(--panel-raised);
		stroke: var(--border);
		stroke-width: 1.5;
	}

	.wheel {
		fill: var(--bg-raised);
		stroke: var(--border);
		stroke-width: 1.2;
	}

	.seam,
	.ridge {
		fill: none;
		stroke: var(--border);
		stroke-width: 1.2;
	}

	.hotspot circle {
		fill: var(--bg-raised);
		stroke: var(--text-faint);
		stroke-width: 1.3;
		transition:
			fill 0.12s ease,
			stroke 0.12s ease;
	}

	.hotspot text {
		fill: var(--text-muted);
		font-size: 13px;
		font-weight: 600;
		pointer-events: none;
	}

	.hotspot {
		cursor: pointer;
	}

	.hotspot:hover circle {
		stroke: var(--accent-strong);
	}

	.hotspot.selected circle {
		fill: var(--accent-soft);
		stroke: var(--accent);
	}

	.hotspot.selected text {
		fill: var(--accent-strong);
	}

	.hotspot.inactive {
		cursor: default;
		opacity: 0.35;
	}
</style>
