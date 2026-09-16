<script lang="ts">
	// The linear control, restyled to match the two dials: a track, a filled portion, a handle,
	// tick marks at meaningful stops, and the value shown at the handle (editable, so an exact
	// value can still be typed, matching what the plain number field it replaces already let a
	// caller do). Drawn as inline SVG like SegmentDial and ArcDial, not a native <input
	// type="range">, so its look matches theirs exactly.
	//
	// Two value modes: continuous (`min`/`max`/`step`, evenly spaced: DPI per stage, debounce, the
	// raw receiver timeout) or discrete `stops` (unevenly spaced named values laid out in equal
	// slots: polling rate's 125/250/.../8000 Hz).
	import HelpTip from '../components/HelpTip.svelte';
	import type { HelpKey } from '../help';

	interface Stop {
		value: number;
		label: string;
	}

	interface Props {
		value: number;
		label: string;
		hint?: string;
		/** Shows a "?" beside the label explaining what this control does (see help.ts). Omitted
		 * controls show no tip. */
		helpTopic?: HelpKey;
		min?: number;
		max?: number;
		step?: number;
		/** Discrete, possibly unevenly spaced allowed values with their own labels. When given,
		 * overrides `min`/`max`/`step`: the track is divided into `stops.length - 1` equal slots,
		 * not scaled by numeric distance. */
		stops?: Stop[];
		/** Values (continuous mode only) that get a labelled tick mark. Defaults to `[min, max]`. */
		ticks?: number[];
		/** Continuous mode only: `'log'` spaces the track logarithmically instead of linearly, so a
		 * range whose real values cluster near one end (DPI: most stages sit in the low thousands
		 * of a 50-40000 scale) still spreads them usefully across the track. Committed and typed
		 * values are unaffected: `step` still snaps them the same way either scale. */
		scale?: 'linear' | 'log';
		unit?: string;
		format?: (value: number) => string;
		disabled?: boolean;
		/** No real reading exists yet (no device connected, or its settings have not been read).
		 * Hides the handle and fill and shows a dash instead of a value, forcing the control
		 * disabled; see SegmentDial's own doc comment for the full contract. */
		unknown?: boolean;
		/** Overrides the fill/handle colour (the DPI stage swatch colour is data the mouse
		 * stores, not decoration; see docs/architecture/ui-controls.md section 6). */
		accentColor?: string;
		onchange?: (value: number) => void;
	}

	let {
		value = $bindable(0),
		label,
		hint,
		helpTopic,
		min = 0,
		max = 100,
		step = 1,
		stops,
		ticks,
		scale = 'linear',
		unit = '',
		format,
		disabled = false,
		unknown = false,
		accentColor,
		onchange
	}: Props = $props();

	const uid = $props.id();

	const VB_WIDTH = 1000;
	const VB_HEIGHT = 32;
	const TRACK_Y = 16;
	const HANDLE_R = 10;
	const INSET = 14;

	// A tick's label and the value bubble render as real HTML text (see the markup below), not SVG
	// <text>, because the track's viewBox is stretched independently in x and y
	// (preserveAspectRatio="none", so a fixed-height track can span any panel width) and that same
	// independent stretch used to shear any text drawn inside the SVG's own coordinate space into a
	// few illegible, sheared pixels at most panel widths. These constants estimate a label's
	// rendered width without a DOM measurement pass, to decide how many tick labels can fit before
	// they start to overlap: JetBrains Mono's average glyph advance is close to 0.62em, and
	// TICK_GAP_PX is the minimum clear space wanted between two neighbouring labels.
	const TICK_FONT_PX = 12; // var(--text-xs)
	const TICK_CHAR_WIDTH = TICK_FONT_PX * 0.62;
	const TICK_GAP_PX = 10;

	let svgEl: SVGSVGElement | undefined = $state();
	let inputEl: HTMLInputElement | undefined = $state();
	let dragFraction = $state<number | null>(null);
	let editing = $state(false);
	let editText = $state('');
	let wrapWidth = $state(0);

	let effectiveDisabled = $derived(disabled || unknown);
	let sortedStops = $derived(stops ? [...stops].sort((a, b) => a.value - b.value) : undefined);
	let effectiveMin = $derived(sortedStops ? sortedStops[0].value : min);
	let effectiveMax = $derived(sortedStops ? sortedStops[sortedStops.length - 1].value : max);
	let clampedValue = $derived(Math.min(effectiveMax, Math.max(effectiveMin, value)));
	let tickValues = $derived(
		sortedStops ? sortedStops.map((s) => s.value) : (ticks ?? [effectiveMin, effectiveMax])
	);
	// Log10 of the floor/ceiling, used to map a value to/from track position on a log scale.
	// `effectiveMin` is never <= 0 for any current caller (DPI's real floor is 50); a scale that
	// ever needed to reach 0 would need its own zero-symlog handling, not attempted here.
	let logMin = $derived(scale === 'log' ? Math.log10(Math.max(effectiveMin, 1e-9)) : 0);
	let logMax = $derived(scale === 'log' ? Math.log10(Math.max(effectiveMax, 1e-9)) : 0);

	function nearestStopIndex(v: number): number {
		if (!sortedStops) return -1;
		let bestIndex = 0;
		let bestDiff = Infinity;
		sortedStops.forEach((stop, index) => {
			const diff = Math.abs(stop.value - v);
			if (diff < bestDiff) {
				bestDiff = diff;
				bestIndex = index;
			}
		});
		return bestIndex;
	}

	/** Snaps an arbitrary numeric target (typed, or interpolated from a step) to a value this
	 * control can actually represent: the nearest stop, or the nearest step within range. */
	function normalizeValue(raw: number): number {
		if (sortedStops) {
			const index = nearestStopIndex(raw);
			return sortedStops[index].value;
		}
		const snapped = Math.round(raw / step) * step;
		return Math.min(effectiveMax, Math.max(effectiveMin, snapped));
	}

	function fractionOf(v: number): number {
		if (sortedStops) {
			return sortedStops.length > 1 ? nearestStopIndex(v) / (sortedStops.length - 1) : 0;
		}
		if (scale === 'log') {
			const lv = Math.log10(Math.max(v, effectiveMin, 1e-9));
			return logMax > logMin ? (lv - logMin) / (logMax - logMin) : 0;
		}
		return effectiveMax > effectiveMin ? (v - effectiveMin) / (effectiveMax - effectiveMin) : 0;
	}

	function valueFromFraction(fraction: number): number {
		if (sortedStops) {
			const index = Math.round(fraction * (sortedStops.length - 1));
			return sortedStops[Math.min(sortedStops.length - 1, Math.max(0, index))].value;
		}
		if (scale === 'log') {
			return normalizeValue(10 ** (logMin + fraction * (logMax - logMin)));
		}
		return normalizeValue(effectiveMin + fraction * (effectiveMax - effectiveMin));
	}

	/** One step in `direction`: the adjacent stop, or `value +/- step`. */
	function stepValue(direction: 1 | -1): number {
		if (sortedStops) {
			const index = nearestStopIndex(clampedValue);
			const next = Math.min(sortedStops.length - 1, Math.max(0, index + direction));
			return sortedStops[next].value;
		}
		return Math.min(effectiveMax, Math.max(effectiveMin, clampedValue + direction * step));
	}

	function formatValue(v: number): string {
		if (format) return format(v);
		if (sortedStops) return sortedStops.find((s) => s.value === v)?.label ?? `${v}${unit}`;
		return `${v}${unit}`;
	}

	function estimateLabelWidth(text: string): number {
		return text.length * TICK_CHAR_WIDTH;
	}

	/** A label's natural centre position (as a percent of the track's own width) pushed in from
	 * either edge by half its own estimated width, so a label near an end never overflows its own
	 * track into a neighbouring control (two RangeSliders sitting side by side, each with its
	 * value near an end, used to overlap into each other's space). */
	function clampLabelPct(rawPct: number, text: string): number {
		if (wrapWidth <= 0) return rawPct;
		const halfPct = (estimateLabelWidth(text) / 2 / wrapWidth) * 100;
		if (halfPct * 2 >= 100) return 50;
		return Math.min(100 - halfPct, Math.max(halfPct, rawPct));
	}

	/** Every tick gets a mark; only some get a text label once there is not enough width for all of
	 * them to sit apart without touching. Always keeps the first and last (the scale's own ends)
	 * and spaces the rest as evenly as possible among the indices that remain, rather than
	 * dropping from one side inward, so the scale still reads correctly at any width. */
	let visibleTickValues = $derived.by(() => {
		if (tickValues.length <= 2 || wrapWidth <= 0) return tickValues;
		const widest = Math.max(...tickValues.map((t) => estimateLabelWidth(formatValue(t))));
		const slot = widest + TICK_GAP_PX;
		const maxCount = Math.max(2, Math.min(tickValues.length, Math.floor(wrapWidth / slot)));
		if (maxCount >= tickValues.length) return tickValues;
		if (sortedStops) {
			// Discrete named stops: pick maxCount of the real stops nearest an even spread of
			// indices, since an interpolated in-between value would not be a value this control
			// can actually land on.
			const indices = new Set<number>();
			for (let i = 0; i < maxCount; i++) {
				indices.add(Math.round((i * (tickValues.length - 1)) / (maxCount - 1)));
			}
			return [...indices].sort((a, b) => a - b).map((i) => tickValues[i]);
		}
		// Continuous range: generate maxCount ticks evenly spaced along the control's own scale
		// (respecting `scale`) and snapped to `step`, so the ticks that fit are always evenly
		// spaced on the track, never a lopsided subset of the full candidate list (operator
		// feedback, 2026-09-16: debounce's ticks thinned to "0 ms, 15 ms, 25 ms", a 15/10 ms gap).
		const generated = new Set<number>();
		for (let i = 0; i < maxCount; i++) {
			generated.add(valueFromFraction(i / (maxCount - 1)));
		}
		return [...generated].sort((a, b) => a - b);
	});

	let committedFraction = $derived(fractionOf(clampedValue));
	let displayFraction = $derived(dragFraction ?? committedFraction);
	let displayValue = $derived(valueFromFraction(displayFraction));
	let handleX = $derived(INSET + displayFraction * (VB_WIDTH - 2 * INSET));
	// With no real reading, the handle position itself comes from a blank placeholder value (see
	// BLANK_SETTINGS), so anchoring the dash to it would put it somewhere meaningless-looking
	// instead of legibly centred; the handle circle is already hidden for the same reason.
	let valueLabelX = $derived(unknown ? VB_WIDTH / 2 : handleX);

	function fractionFromPoint(point: { clientX: number }): number {
		if (!svgEl) return committedFraction;
		const rect = svgEl.getBoundingClientRect();
		const usableLeft = rect.left + (INSET / VB_WIDTH) * rect.width;
		const usableWidth = rect.width * ((VB_WIDTH - 2 * INSET) / VB_WIDTH);
		return Math.min(1, Math.max(0, (point.clientX - usableLeft) / usableWidth));
	}

	function commit(next: number) {
		if (next !== clampedValue) onchange?.(next);
	}

	function handlePointerDown(event: PointerEvent) {
		if (effectiveDisabled || editing) return;
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
		const finalValue = valueFromFraction(dragFraction);
		dragFraction = null;
		commit(finalValue);
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (effectiveDisabled || editing) return;
		let next = clampedValue;
		switch (event.key) {
			case 'ArrowRight':
			case 'ArrowUp':
				next = stepValue(1);
				break;
			case 'ArrowLeft':
			case 'ArrowDown':
				next = stepValue(-1);
				break;
			case 'Home':
				next = effectiveMin;
				break;
			case 'End':
				next = effectiveMax;
				break;
			case 'Enter':
			case ' ':
				startEditing();
				event.preventDefault();
				return;
			default:
				return;
		}
		event.preventDefault();
		commit(next);
	}

	function handleWheel(event: WheelEvent) {
		if (effectiveDisabled || editing || svgEl !== document.activeElement || event.deltaY === 0) {
			return;
		}
		event.preventDefault();
		commit(stepValue(event.deltaY > 0 ? -1 : 1));
	}

	function startEditing() {
		if (effectiveDisabled) return;
		editText = String(clampedValue);
		editing = true;
	}

	function finishEditing() {
		if (!editing) return;
		editing = false;
		const parsed = Number(editText);
		if (Number.isFinite(parsed)) commit(normalizeValue(parsed));
	}

	$effect(() => {
		if (editing) inputEl?.focus();
	});
</script>

<div class="field">
	<label class="field-label" for={uid}>{label}{#if helpTopic}<HelpTip topic={helpTopic} />{/if}</label>
	{#if hint}<div class="field-hint">{hint}</div>{/if}

	<div class="range-slider-wrap" bind:clientWidth={wrapWidth}>
		<div class="value-row">
			{#if !editing}
				<!-- A pointer-only shortcut into edit mode: the accessible path is Enter/Space on the
				     slider itself (handleKeyDown), so this stays a plain, non-focusable span rather
				     than duplicating the slider as a second "button" in the accessibility tree. -->
				<span
					class="value-label mono"
					class:clickable={!effectiveDisabled}
					style="left: {clampLabelPct((valueLabelX / VB_WIDTH) * 100, unknown ? '-' : formatValue(displayValue))}%"
					aria-hidden="true"
					onclick={startEditing}
				>{unknown ? '-' : formatValue(displayValue)}</span>
			{:else}
				<input
					bind:this={inputEl}
					class="edit-input mono"
					type="number"
					min={effectiveMin}
					max={effectiveMax}
					{step}
					style="left: {(handleX / VB_WIDTH) * 100}%"
					bind:value={editText}
					onblur={finishEditing}
					onkeydown={(e) => {
						if (e.key === 'Enter') {
							finishEditing();
						} else if (e.key === 'Escape') {
							editing = false;
						}
						e.stopPropagation();
					}}
				/>
			{/if}
		</div>

		<svg
			bind:this={svgEl}
			id={uid}
			class="range-slider"
			class:disabled={effectiveDisabled}
			style={accentColor ? `--slider-accent: ${accentColor};` : undefined}
			viewBox="0 0 {VB_WIDTH} {VB_HEIGHT}"
			preserveAspectRatio="none"
			role="slider"
			tabindex={effectiveDisabled ? -1 : 0}
			aria-label={label}
			aria-valuemin={effectiveMin}
			aria-valuemax={effectiveMax}
			aria-valuenow={unknown ? undefined : clampedValue}
			aria-valuetext={unknown ? undefined : formatValue(clampedValue)}
			aria-disabled={effectiveDisabled}
			onpointerdown={handlePointerDown}
			onpointermove={handlePointerMove}
			onpointerup={handlePointerUp}
			onpointercancel={handlePointerUp}
			onkeydown={handleKeyDown}
			onwheel={handleWheel}
		>
			<line class="track" x1={INSET} y1={TRACK_Y} x2={VB_WIDTH - INSET} y2={TRACK_Y} />
			{#if !unknown}
				<line class="fill" x1={INSET} y1={TRACK_Y} x2={handleX} y2={TRACK_Y} />
			{/if}

			{#each tickValues as tick (tick)}
				{@const x = INSET + fractionOf(tick) * (VB_WIDTH - 2 * INSET)}
				<line class="tick" x1={x} y1={TRACK_Y + 7} x2={x} y2={TRACK_Y + 13} />
			{/each}

			{#if !unknown}
				<rect
					class="handle"
					x={handleX - HANDLE_R}
					y={TRACK_Y - HANDLE_R}
					width={HANDLE_R * 2}
					height={HANDLE_R * 2}
				/>
			{/if}
		</svg>

		<div class="tick-row" aria-hidden="true">
			{#each visibleTickValues as tick (tick)}
				{@const rawPct = ((INSET + fractionOf(tick) * (VB_WIDTH - 2 * INSET)) / VB_WIDTH) * 100}
				<span class="tick-label mono" style="left: {clampLabelPct(rawPct, formatValue(tick))}%">{formatValue(tick)}</span>
			{/each}
		</div>
	</div>
</div>

<style>
	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.range-slider-wrap {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.value-row {
		position: relative;
		height: 18px;
	}

	.range-slider {
		display: block;
		width: 100%;
		height: 32px;
		touch-action: none;
		cursor: pointer;
		overflow: visible;
	}

	.range-slider.disabled {
		cursor: not-allowed;
	}

	/* Focus ring: the global [tabindex]:focus-visible rule in app.css already gives every
	   focusable control the square --color-focus outline design.md specifies; no local override. */

	.track {
		stroke: var(--color-rule);
		stroke-width: 8;
		stroke-linecap: butt;
	}

	.fill {
		stroke: var(--slider-accent, var(--color-accent));
		stroke-width: 8;
		stroke-linecap: butt;
	}

	.tick {
		stroke: var(--color-rule);
		stroke-width: 2;
	}

	.handle {
		fill: var(--slider-accent, var(--color-accent));
		stroke: var(--color-paper);
		stroke-width: 3;
	}

	.tick-row {
		position: relative;
		height: 14px;
	}

	.tick-label {
		position: absolute;
		top: 0;
		transform: translateX(-50%);
		font-size: var(--text-xs);
		color: var(--color-ink-2);
		white-space: nowrap;
		line-height: 1;
		pointer-events: none;
	}

	.value-label {
		position: absolute;
		top: 0;
		transform: translateX(-50%);
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--color-ink);
		white-space: nowrap;
		line-height: 1;
	}

	.value-label.clickable {
		cursor: text;
	}

	.edit-input {
		position: absolute;
		top: -2px;
		transform: translateX(-50%);
		width: 68px;
		box-sizing: border-box;
		background: var(--color-paper-2);
		border: 1px solid var(--color-accent);
		color: var(--color-ink);
		font-size: var(--text-sm);
		text-align: center;
		padding: 2px 4px;
	}

	@media (prefers-reduced-motion: no-preference) {
		.fill,
		.handle {
			transition:
				x2 var(--dur-settle) var(--ease-out),
				x var(--dur-settle) var(--ease-out),
				y var(--dur-settle) var(--ease-out);
		}
	}
</style>
