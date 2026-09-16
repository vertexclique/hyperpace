<script lang="ts">
	// The manufacturer's own top-down render (ui/static/device/mouse-top.png, 710x380; see
	// ui/static/device/PROVENANCE.md), with five hotspots over it.
	//
	// The render is rotated: the mouse's nose points LEFT, its palm end right. Three things fix
	// that orientation, and they agree: the scroll wheel sits in the left third of the centre gap,
	// the two side buttons sit just behind it, and the wordmark is printed on the palm end. Read
	// as a top view with the nose turned up, the image's top half is therefore the mouse's right
	// side and its bottom half the mouse's left, which puts the two orange side buttons on the
	// left flank where a right-handed thumb reaches them.
	//
	//   - Right click: the large surface above the centre gap.
	//   - Left click:  the large surface below it.
	//   - Wheel click: the ridged wheel in the gap.
	//   - Forward / back: the two orange tabs on the lower edge, forward being the one nearer the
	//     nose.
	//
	// Every rectangle below was measured off the shipped PNG, not estimated: the orange tabs come
	// from a colour-mask scan of the file (runs at x 215-310 and x 320-414 of 710, y 324-361), and
	// the wheel from a 4x crop of the gap.
	//
	// Numbered callouts are the shipped badges (key/Keys/<n>.png, <n>-click.png), not hand-drawn
	// circles, so they carry the same idle and pressed art as the vendor's own configurator.

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

	/** Percent rectangles against the 710x380 render's own geometry. */
	const hotspots: { x: number; y: number; w: number; h: number }[] = [
		{ x: 14.0, y: 60.0, w: 30.0, h: 20.0 }, // left click: surface below the gap
		{ x: 14.0, y: 18.0, w: 30.0, h: 20.0 }, // right click: surface above the gap
		{ x: 11.6, y: 45.0, w: 21.5, h: 9.6 }, // wheel, in the centre gap
		{ x: 45.1, y: 85.3, w: 13.2, h: 9.7 }, // back: rear orange tab
		{ x: 30.3, y: 85.3, w: 13.4, h: 9.7 } // forward: front orange tab
	];

	// A shallow parallax tilt, the app's one piece of depth. Held to a few degrees and driven
	// straight off the pointer, so it reads as the object sitting in the panel rather than as an
	// animation playing at the operator. Dropped entirely under prefers-reduced-motion by the
	// media query below, which is why the transform lives in CSS rather than in these handlers.
	const MAX_TILT_DEG = 5;

	let tiltX = $state(0);
	let tiltY = $state(0);

	function handlePointerMove(event: PointerEvent) {
		const box = (event.currentTarget as HTMLElement).getBoundingClientRect();
		const fromCentreX = (event.clientX - box.left) / box.width - 0.5;
		const fromCentreY = (event.clientY - box.top) / box.height - 0.5;
		tiltY = fromCentreX * 2 * MAX_TILT_DEG;
		tiltX = -fromCentreY * 2 * MAX_TILT_DEG;
	}

	function resetTilt() {
		tiltX = 0;
		tiltY = 0;
	}
</script>

<!-- The tilt follows the pointer over the artwork; it carries no information and no action, so it
     needs no keyboard or pointer role of its own. The hotspots inside remain the interactive
     elements. -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="mouse-art"
	style:--tilt-x="{tiltX}deg"
	style:--tilt-y="{tiltY}deg"
	onpointermove={handlePointerMove}
	onpointerleave={resetTilt}
>
	<div class="stage">
		<img src="/device/mouse-top.png" alt="Mouse, top view" class="render" />
		{#each hotspots as spot, i (i)}
			{@const isActive = i < activeCount}
			{@const isSelected = selected === i}
			<button
				type="button"
				class="hotspot"
				class:selected={isSelected}
				style:left="{spot.x}%"
				style:top="{spot.y}%"
				style:width="{spot.w}%"
				style:height="{spot.h}%"
				disabled={!isActive}
				aria-label={labels[i]}
				aria-pressed={isSelected}
				onclick={() => onselect(i)}
			>
				<img
					src={isSelected
						? `/device/key/Keys/${i + 1}-click.png`
						: `/device/key/Keys/${i + 1}.png`}
					alt=""
					class="badge"
				/>
			</button>
		{/each}
	</div>
</div>

<style>
	.mouse-art {
		position: relative;
		width: 100%;
		max-width: 100%;
		aspect-ratio: 710 / 380;
		perspective: 900px;
	}

	.stage {
		position: absolute;
		inset: 0;
		transform-style: preserve-3d;
		transform: rotateX(var(--tilt-x, 0deg)) rotateY(var(--tilt-y, 0deg));
		transition: transform var(--dur-settle) var(--ease-out);
	}

	/* The vendor's own render: no drop shadow, no rounded frame, no tint (design.md Assets). */
	.render {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: contain;
		pointer-events: none;
		user-select: none;
	}

	.hotspot {
		position: absolute;
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: visible;
		background: transparent;
		border: none;
		padding: 0;
		/* Lifted off the render so the badges keep their own place as the stage tilts. */
		transform: translateZ(18px);
	}

	.hotspot:disabled {
		cursor: default;
	}

	.badge {
		width: 22px;
		height: 22px;
		display: block;
		transition: opacity var(--dur-settle) var(--ease-out);
	}

	.hotspot:disabled .badge {
		opacity: 0.35;
	}

	/* Hover, focus and selection read as a square accent outline on the hotspot itself, never as a
	   tint or shadow on the render underneath (design.md Assets: "the white faces are kept
	   white"). */
	.hotspot:not(:disabled):hover .badge,
	.hotspot.selected .badge {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
	}

	.hotspot:focus-visible .badge {
		outline: 2px solid var(--color-focus);
		outline-offset: 2px;
	}

	@media (prefers-reduced-motion: reduce) {
		.stage {
			transform: none;
			transition: none;
		}

		.hotspot {
			transform: none;
		}
	}
</style>
