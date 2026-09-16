<script lang="ts" generics="T extends string | number">
	// A full circle divided into N equal segments, one per allowed value: sensor mode and
	// lift-off distance (3 segments each), sleep time and highest performance (7 segments, the
	// number of named timeout codes this protocol actually defines; see
	// docs/architecture/ui-controls.md and the DECS row explaining the deviation from that doc's
	// vendor-measured "8"). The ring, pointer dot and labels are drawn as inline SVG, themed from
	// CSS variables; when `face` is given, the manufacturer's own dial-face bitmap and pointer
	// sprite (ui/static/device/, see PROVENANCE.md) composite underneath and in place of that dot,
	// with the drawn ring standing in for the live arc and empty track over the face. Highest
	// performance has no bitmap face (its neutral face has the vendor's orange and its segment
	// labels baked into the pixels, see docs/architecture/ui-controls.md), so it always renders
	// the plain vector ring.
	import type { Snippet } from 'svelte';
	import { arcPath, bearingFromPoint, pointOnCircle, DIAL_FACES } from './geometry';
	import type { SegmentDialFace } from './geometry';

	interface SegmentOption<V> {
		value: V;
		label: string;
	}

	interface Props<V> {
		values: SegmentOption<V>[];
		value: V;
		/** Accessible name for the whole control (the per-segment labels are `aria-valuetext`,
		 * not a substitute for this). */
		ariaLabel: string;
		disabled?: boolean;
		/** No real reading exists yet (no device connected, or its settings have not been read).
		 * Hides the pointer and any centre value text and forces the control disabled, per the
		 * "No device" row of the control-language state table; never highlights a segment as
		 * though it were a real reading. */
		unknown?: boolean;
		size?: 'compact' | 'default' | 'large';
		/** Selects the manufacturer's dial-face bitmap and pointer sprite to composite under the
		 * drawn ring (see docs/architecture/ui-controls.md section 1). Omitted for a dial with no
		 * usable neutral face (highest performance), which stays the plain vector ring. */
		face?: SegmentDialFace;
		/** Large text in the middle, e.g. "30 s" or "2 min". */
		centerLabel?: string;
		/** A button rendered in the middle (highest performance's on/off). Its own `disabled`
		 * state is the caller's responsibility, same as every other control on these screens. */
		centerAction?: Snippet;
		onchange?: (value: V) => void;
	}

	let {
		values,
		value = $bindable(),
		ariaLabel,
		disabled = false,
		unknown = false,
		size = 'default',
		face,
		centerLabel,
		centerAction,
		onchange
	}: Props<T> = $props();

	const uid = $props.id();

	const CX = 120;
	const CY = 120;
	const RING_RADIUS = 82;
	const RING_STROKE = 26;
	const LABEL_RADIUS = 112;
	const POINTER_RADIUS = 5.5;
	// The face bitmap is rendered at this radius, matching the ring's own outer edge, so the
	// drawn segments sit right over the face's rim rather than floating past or short of it.
	const FACE_RADIUS = RING_RADIUS + RING_STROKE / 2;

	let svgEl: SVGSVGElement | undefined = $state();
	let dragIndex = $state<number | null>(null);
	let renderedSize = $state(0);

	// The dial's own viewBox (240 square) is scaled down to whichever --dial-size tier is actually
	// rendered, including a tier a container query can force independently of the `size` prop; text
	// drawn at a fixed font-size in that same coordinate space shrank well below legible size at the
	// `compact` tier (exactly the "Sensor mode" / "Lift-off distance" defect: labels a few pixels
	// tall). Measuring the rendered size and inflating the font-size by the inverse of the current
	// scale keeps every label a fixed, legible size in real device pixels no matter which tier is on
	// screen (see ArcDial's identical fix).
	let renderScale = $derived(renderedSize > 0 ? renderedSize / 240 : 1);
	function fixedFontSize(targetPx: number): number {
		return targetPx / renderScale;
	}

	let effectiveDisabled = $derived(disabled || unknown);
	let segmentAngle = $derived(360 / values.length);
	let valueIndex = $derived(values.findIndex((option) => option.value === value));
	let displayIndex = $derived(dragIndex ?? (valueIndex === -1 ? 0 : valueIndex));

	// Face and pointer are native-scale to each other (see geometry.ts); scaling both by the same
	// factor onto FACE_RADIUS keeps them registered, per docs/architecture/ui-controls.md.
	let faceInfo = $derived(face ? DIAL_FACES[face] : null);
	let faceScale = $derived(faceInfo ? (FACE_RADIUS * 2) / faceInfo.width : 1);
	let pointerWidth = $derived(faceInfo ? faceInfo.pointerWidth * faceScale : 0);
	let pointerHeight = $derived(faceInfo ? faceInfo.pointerHeight * faceScale : 0);
	let pointerBearing = $derived(displayIndex * segmentAngle);

	function indexFromPoint(point: { clientX: number; clientY: number }): number {
		if (!svgEl) return 0;
		const bearing = bearingFromPoint(svgEl.getBoundingClientRect(), point);
		return Math.round(bearing / segmentAngle) % values.length;
	}

	function commit(index: number) {
		if (index === valueIndex) return;
		onchange?.(values[index].value);
	}

	function handlePointerDown(event: PointerEvent) {
		if (effectiveDisabled) return;
		svgEl?.setPointerCapture(event.pointerId);
		dragIndex = indexFromPoint(event);
		event.preventDefault();
	}

	function handlePointerMove(event: PointerEvent) {
		if (effectiveDisabled || dragIndex === null) return;
		dragIndex = indexFromPoint(event);
	}

	function handlePointerUp(event: PointerEvent) {
		if (effectiveDisabled || dragIndex === null) return;
		svgEl?.releasePointerCapture(event.pointerId);
		const finalIndex = dragIndex;
		dragIndex = null;
		commit(finalIndex);
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (effectiveDisabled) return;
		const current = valueIndex === -1 ? 0 : valueIndex;
		let next = current;
		switch (event.key) {
			case 'ArrowRight':
			case 'ArrowUp':
				next = Math.min(values.length - 1, current + 1);
				break;
			case 'ArrowLeft':
			case 'ArrowDown':
				next = Math.max(0, current - 1);
				break;
			case 'Home':
				next = 0;
				break;
			case 'End':
				next = values.length - 1;
				break;
			default:
				return;
		}
		event.preventDefault();
		commit(next);
	}

	function handleWheel(event: WheelEvent) {
		if (effectiveDisabled || svgEl !== document.activeElement || event.deltaY === 0) return;
		event.preventDefault();
		const current = valueIndex === -1 ? 0 : valueIndex;
		const next = Math.min(
			values.length - 1,
			Math.max(0, current + (event.deltaY > 0 ? 1 : -1))
		);
		commit(next);
	}
</script>

<svg
	bind:this={svgEl}
	bind:clientWidth={renderedSize}
	class="segment-dial"
	class:disabled={effectiveDisabled}
	style="width:var(--dial-size, var(--dial-{size})); height:var(--dial-size, var(--dial-{size}));"
	viewBox="0 0 240 240"
	role="slider"
	tabindex={effectiveDisabled ? -1 : 0}
	aria-label={ariaLabel}
	aria-valuemin={0}
	aria-valuemax={values.length - 1}
	aria-valuenow={valueIndex === -1 ? undefined : valueIndex}
	aria-valuetext={unknown || valueIndex === -1 ? undefined : values[valueIndex].label}
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
			<!-- Recolors the pointer sprite's orange to --color-ink by its own alpha, the same way
			     the plain vector pointer dot is inked: accent stays reserved for the live ring,
			     per design.md's accent budget ("the live arc of each dial and nowhere else"). -->
			<filter id="{uid}-ink" color-interpolation-filters="sRGB">
				<feFlood style="flood-color: var(--color-ink)" result="flood" />
				<feComposite in="flood" in2="SourceAlpha" operator="in" />
			</filter>
		</defs>
		<image
			class="face-image"
			href={faceInfo.src}
			x={CX - FACE_RADIUS}
			y={CY - FACE_RADIUS}
			width={FACE_RADIUS * 2}
			height={FACE_RADIUS * 2}
			aria-hidden="true"
		/>
	{/if}

	{#each values as option, i (String(option.value))}
		{@const start = i * segmentAngle - segmentAngle / 2 + Math.min(6, segmentAngle * 0.15) / 2}
		{@const end = i * segmentAngle + segmentAngle / 2 - Math.min(6, segmentAngle * 0.15) / 2}
		<path
			class="segment"
			class:active={!unknown && i === displayIndex}
			d={arcPath(CX, CY, RING_RADIUS, start, end)}
			stroke-width={RING_STROKE}
		/>
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
			{@const p = pointOnCircle(CX, CY, RING_RADIUS, displayIndex * segmentAngle)}
			<rect
				class="pointer"
				x={p.x - POINTER_RADIUS}
				y={p.y - POINTER_RADIUS}
				width={POINTER_RADIUS * 2}
				height={POINTER_RADIUS * 2}
			/>
		{/if}
	{/if}

	{#each values as option, i (String(option.value))}
		{@const p = pointOnCircle(CX, CY, LABEL_RADIUS, i * segmentAngle)}
		<text
			class="segment-label"
			class:active={!unknown && i === displayIndex}
			x={p.x}
			y={p.y}
			style="font-size: {fixedFontSize(13)}px"
		>{option.label}</text>
	{/each}

	{#if centerAction}
		<foreignObject x={CX - 30} y={CY - 42} width="60" height="60">
			<div class="center-action">
				{@render centerAction()}
			</div>
		</foreignObject>
		{#if centerLabel !== undefined}
			<text class="center-label sub" x={CX} y={CY + 46} style="font-size: {fixedFontSize(13)}px">{unknown ? '-' : centerLabel}</text>
		{/if}
	{:else if centerLabel !== undefined}
		<text class="center-label" x={CX} y={CY} style="font-size: {fixedFontSize(26)}px">{unknown ? '-' : centerLabel}</text>
	{/if}
</svg>

<style>
	.segment-dial {
		display: block;
		touch-action: none;
		cursor: pointer;
		overflow: visible;
	}

	.segment-dial.disabled {
		cursor: not-allowed;
	}

	/* Focus ring: the global [tabindex]:focus-visible rule in app.css already gives every
	   focusable control the square --color-focus outline design.md specifies; no local override. */

	.face-image,
	.pointer-sprite {
		pointer-events: none;
	}

	.segment {
		fill: none;
		stroke: var(--color-rule);
		stroke-linecap: butt;
	}

	.segment.active {
		stroke: var(--color-accent);
	}

	.pointer {
		fill: var(--color-ink);
	}

	/* font-size on .segment-label, .center-label and .center-label.sub is set inline per element
	   (fixedFontSize above), not here: it must stay a fixed real pixel size regardless of which
	   --dial-size tier is actually rendered, so it is computed as the inverse of the dial's own
	   measured scale rather than a static value that would shrink with it. */
	.segment-label {
		fill: var(--color-muted);
		font-weight: 500;
		text-anchor: middle;
		dominant-baseline: middle;
	}

	.segment-label.active {
		fill: var(--color-accent);
		font-weight: 700;
	}

	.center-label {
		fill: var(--color-ink);
		font-family: var(--font-display);
		font-weight: 600;
		text-anchor: middle;
		dominant-baseline: middle;
	}

	.center-label.sub {
		font-weight: 500;
		fill: var(--color-muted);
	}

	.center-action {
		width: 100%;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	@media (prefers-reduced-motion: no-preference) {
		.segment,
		.pointer,
		.segment-label {
			transition:
				stroke var(--dur-settle) var(--ease-out),
				fill var(--dur-settle) var(--ease-out),
				x var(--dur-settle) var(--ease-out),
				y var(--dur-settle) var(--ease-out);
		}

		.pointer-sprite {
			transition: transform var(--dur-settle) var(--ease-out);
		}
	}
</style>
