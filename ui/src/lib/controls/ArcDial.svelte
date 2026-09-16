<script lang="ts">
	// A half ring covering the top 180 degrees, filled from the left end (min) to the current
	// step, used by receiver brightness and receiver speed (and any future one-to-ten setting).
	// The track, fill and tick marks are drawn as inline SVG, themed from CSS variables; when
	// `face` is given, the manufacturer's own half-ring bitmap and pointer sprite (ui/static/device/,
	// see PROVENANCE.md) composite underneath and in place of the plain pointer dot, the same
	// convention SegmentDial uses.
	import { arcPath, bearingFromPoint, pointOnCircle, DIAL_FACES } from './geometry';
	import type { ArcDialFace } from './geometry';

	interface Props {
		min: number;
		max: number;
		step?: number;
		value: number;
		ariaLabel: string;
		disabled?: boolean;
		/** No real reading exists yet; see SegmentDial's own doc comment for the exact contract. */
		unknown?: boolean;
		size?: 'compact' | 'default' | 'large';
		label: string;
		format?: (value: number) => string;
		/** Selects the manufacturer's half-ring bitmap and pointer sprite to composite under the
		 * drawn arc (see docs/architecture/ui-controls.md section 1). */
		face?: ArcDialFace;
		onchange?: (value: number) => void;
	}

	let {
		min,
		max,
		step = 1,
		value = $bindable(),
		ariaLabel,
		disabled = false,
		unknown = false,
		size = 'default',
		label,
		format = (v) => String(v),
		face,
		onchange
	}: Props = $props();

	const uid = $props.id();

	const CX = 120;
	const CY = 118;
	const RADIUS = 86;
	const STROKE = 20;
	const TICK_INNER = RADIUS + STROKE / 2 + 3;
	const TICK_OUTER = RADIUS + STROKE / 2 + 9;
	const POINTER_RADIUS = 5;
	// The face bitmap is rendered at this radius, matching the arc's own outer edge, so the
	// drawn track/fill sit right over the face's rim rather than floating past or short of it.
	const FACE_RADIUS = RADIUS + STROKE / 2;

	let svgEl: SVGSVGElement | undefined = $state();
	let dragFraction = $state<number | null>(null);
	let renderedWidth = $state(0);

	// The dial's own viewBox (240 wide) is scaled down to whichever --dial-size tier is actually
	// rendered, including tiers a container query can force independently of the `size` prop; text
	// drawn at a fixed font-size in that same coordinate space shrank below legible size at every
	// tier but `large` (LightingScreen's own container-query comment already names the symptom: "a
	// few pixels, read as unlabeled stumps"). Measuring the rendered width and inflating the
	// font-size by the inverse of the current scale keeps every label a fixed, legible size in real
	// device pixels no matter which tier is on screen.
	let renderScale = $derived(renderedWidth > 0 ? renderedWidth / 240 : 1);
	function fixedFontSize(targetPx: number): number {
		return targetPx / renderScale;
	}

	let effectiveDisabled = $derived(disabled || unknown);
	let stepCount = $derived(Math.max(1, Math.round((max - min) / step)));
	let tickIndices = $derived(Array.from({ length: stepCount + 1 }, (_, i) => i));
	let clampedValue = $derived(Math.min(max, Math.max(min, value)));
	let committedFraction = $derived((clampedValue - min) / (max - min || 1));
	let displayFraction = $derived(dragFraction ?? committedFraction);

	// The half-ring face's pivot is its bottom-centre pixel (where the pointer's mechanical mount
	// sits in the render), radius = the bitmap's own native height; scaling face and pointer by
	// the same factor onto FACE_RADIUS keeps them registered (see geometry.ts).
	let faceInfo = $derived(face ? DIAL_FACES[face] : null);
	let faceScale = $derived(faceInfo ? FACE_RADIUS / faceInfo.height : 1);
	let faceWidth = $derived(faceInfo ? faceInfo.width * faceScale : 0);
	let pointerWidth = $derived(faceInfo ? faceInfo.pointerWidth * faceScale : 0);
	let pointerHeight = $derived(faceInfo ? faceInfo.pointerHeight * faceScale : 0);
	let pointerBearing = $derived(-90 + displayFraction * 180);

	function fractionToValue(fraction: number): number {
		const raw = min + fraction * (max - min);
		const snapped = Math.round(raw / step) * step;
		return Math.min(max, Math.max(min, snapped));
	}

	function fractionFromBearing(bearing: number): number {
		if (bearing <= 90) return (bearing + 90) / 180;
		if (bearing >= 270) return (bearing - 270) / 180;
		return bearing <= 180 ? 1 : 0;
	}

	function fractionFromPoint(point: { clientX: number; clientY: number }): number {
		if (!svgEl) return committedFraction;
		const bearing = bearingFromPoint(svgEl.getBoundingClientRect(), point);
		return fractionFromBearing(bearing);
	}

	function commit(fraction: number) {
		const next = fractionToValue(fraction);
		if (next !== clampedValue) onchange?.(next);
	}

	function handlePointerDown(event: PointerEvent) {
		if (effectiveDisabled) return;
		svgEl?.setPointerCapture(event.pointerId);
		dragFraction = fractionFromPoint(event);
		event.preventDefault();
	}

	function handlePointerMove(event: PointerEvent) {
		if (effectiveDisabled || dragFraction === null) return;
		dragFraction = fractionFromPoint(event);
	}

	function handlePointerUp(event: PointerEvent) {
		if (effectiveDisabled || dragFraction === null) return;
		svgEl?.releasePointerCapture(event.pointerId);
		const finalFraction = dragFraction;
		dragFraction = null;
		commit(finalFraction);
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (effectiveDisabled) return;
		let next = clampedValue;
		switch (event.key) {
			case 'ArrowRight':
			case 'ArrowUp':
				next = Math.min(max, clampedValue + step);
				break;
			case 'ArrowLeft':
			case 'ArrowDown':
				next = Math.max(min, clampedValue - step);
				break;
			case 'Home':
				next = min;
				break;
			case 'End':
				next = max;
				break;
			default:
				return;
		}
		event.preventDefault();
		if (next !== clampedValue) onchange?.(next);
	}

	function handleWheel(event: WheelEvent) {
		if (effectiveDisabled || svgEl !== document.activeElement || event.deltaY === 0) return;
		event.preventDefault();
		const next = Math.min(
			max,
			Math.max(min, clampedValue + (event.deltaY > 0 ? step : -step))
		);
		if (next !== clampedValue) onchange?.(next);
	}
</script>

<svg
	bind:this={svgEl}
	bind:clientWidth={renderedWidth}
	class="arc-dial"
	class:disabled={effectiveDisabled}
	style="width:var(--dial-size, var(--dial-{size})); height:auto; aspect-ratio: 240 / 186;"
	viewBox="0 0 240 186"
	role="slider"
	tabindex={effectiveDisabled ? -1 : 0}
	aria-label={ariaLabel}
	aria-valuemin={min}
	aria-valuemax={max}
	aria-valuenow={unknown ? undefined : clampedValue}
	aria-valuetext={unknown ? undefined : format(clampedValue)}
	aria-disabled={effectiveDisabled}
	onpointerdown={handlePointerDown}
	onpointermove={handlePointerMove}
	onpointerup={handlePointerUp}
	onpointercancel={handlePointerUp}
	onkeydown={handleKeyDown}
	onwheel={handleWheel}
>
	{#if faceInfo}
		<defs>
			<!-- Recolors the pointer sprite's orange to --color-ink by its own alpha; see
			     SegmentDial's identical filter for the accent-budget reasoning. -->
			<filter id="{uid}-ink" color-interpolation-filters="sRGB">
				<feFlood style="flood-color: var(--color-ink)" result="flood" />
				<feComposite in="flood" in2="SourceAlpha" operator="in" />
			</filter>
		</defs>
		<image
			class="face-image"
			href={faceInfo.src}
			x={CX - faceWidth / 2}
			y={CY - FACE_RADIUS}
			width={faceWidth}
			height={FACE_RADIUS}
			aria-hidden="true"
		/>
	{/if}

	<path class="track" d={arcPath(CX, CY, RADIUS, -90, 90)} stroke-width={STROKE} />
	{#if !unknown && displayFraction > 0}
		<path
			class="fill"
			d={arcPath(CX, CY, RADIUS, -90, -90 + displayFraction * 180)}
			stroke-width={STROKE}
		/>
	{/if}

	{#each tickIndices as i (i)}
		{@const bearing = -90 + (i / stepCount) * 180}
		{@const inner = pointOnCircle(CX, CY, TICK_INNER, bearing)}
		{@const outer = pointOnCircle(CX, CY, TICK_OUTER, bearing)}
		<line class="tick" x1={inner.x} y1={inner.y} x2={outer.x} y2={outer.y} />
	{/each}

	{#if !unknown}
		{#if faceInfo}
			<image
				class="pointer-sprite"
				href={faceInfo.pointerSrc}
				x={CX - pointerWidth / 2}
				y={CY - pointerHeight}
				width={pointerWidth}
				height={pointerHeight}
				transform="rotate({pointerBearing} {CX} {CY})"
				filter="url(#{uid}-ink)"
				aria-hidden="true"
			/>
		{:else}
			{@const p = pointOnCircle(CX, CY, RADIUS, -90 + displayFraction * 180)}
			<rect
				class="pointer"
				x={p.x - POINTER_RADIUS}
				y={p.y - POINTER_RADIUS}
				width={POINTER_RADIUS * 2}
				height={POINTER_RADIUS * 2}
			/>
		{/if}
	{/if}

	<text class="value-label mono" x={CX} y={CY + 34} style="font-size: {fixedFontSize(22)}px">{unknown ? '-' : format(clampedValue)}</text>
	<text class="caption-label" x={CX} y={CY + 54} style="font-size: {fixedFontSize(12)}px">{label}</text>
</svg>

<style>
	.arc-dial {
		display: block;
		touch-action: none;
		cursor: pointer;
		overflow: visible;
	}

	.arc-dial.disabled {
		cursor: not-allowed;
	}

	/* Focus ring: the global [tabindex]:focus-visible rule in app.css already gives every
	   focusable control the square --color-focus outline design.md specifies; no local override. */

	.face-image,
	.pointer-sprite {
		pointer-events: none;
	}

	.track {
		fill: none;
		stroke: var(--color-rule);
		stroke-linecap: butt;
	}

	.fill {
		fill: none;
		stroke: var(--color-accent);
		stroke-linecap: butt;
	}

	.tick {
		stroke: var(--color-rule);
		stroke-width: 2;
	}

	.pointer {
		fill: var(--color-ink);
	}

	/* font-size is set inline per element (fixedFontSize above), not here: it must stay a fixed
	   real pixel size regardless of which --dial-size tier is actually rendered, so it is computed
	   as the inverse of the dial's own measured scale rather than a static value that would shrink
	   with it. */
	.value-label {
		fill: var(--color-ink);
		font-weight: 700;
		text-anchor: middle;
		dominant-baseline: middle;
	}

	.caption-label {
		fill: var(--color-ink-2);
		font-weight: 500;
		text-anchor: middle;
		dominant-baseline: middle;
	}

	@media (prefers-reduced-motion: no-preference) {
		.fill,
		.pointer {
			transition:
				d var(--dur-settle) var(--ease-out),
				x var(--dur-settle) var(--ease-out),
				y var(--dur-settle) var(--ease-out);
		}

		.pointer-sprite {
			transition: transform var(--dur-settle) var(--ease-out);
		}
	}
</style>
