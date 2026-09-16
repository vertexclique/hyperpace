// Shared trig for the two radial dial primitives (SegmentDial, ArcDial): both need to convert a
// pointer position to a compass-style bearing around a centre point, and a bearing back to a
// point on a circle, using the same convention (0 degrees at the top, increasing clockwise, which
// reads naturally against SVG's y-down coordinate system). Kept here once rather than duplicated
// in both components, per the third-occurrence rule (a third dial needing the same math reuses
// this instead of a third copy).

/** A point in client (viewport) pixel coordinates, as delivered by a PointerEvent. */
export interface ClientPoint {
	clientX: number;
	clientY: number;
}

/**
 * The bearing, in degrees, from the centre of `rect` to `point`: 0 at the top, 90 at the right,
 * 180 at the bottom, 270 at the left, increasing clockwise. Always in `[0, 360)`.
 */
export function bearingFromPoint(rect: DOMRect, point: ClientPoint): number {
	const cx = rect.left + rect.width / 2;
	const cy = rect.top + rect.height / 2;
	const dx = point.clientX - cx;
	const dy = point.clientY - cy;
	let bearing = (Math.atan2(dx, -dy) * 180) / Math.PI;
	if (bearing < 0) bearing += 360;
	return bearing;
}

/** The point at `radius` from `(cx, cy)` at compass `bearingDeg` (see {@link bearingFromPoint}). */
export function pointOnCircle(
	cx: number,
	cy: number,
	radius: number,
	bearingDeg: number
): { x: number; y: number } {
	const rad = (bearingDeg * Math.PI) / 180;
	return { x: cx + radius * Math.sin(rad), y: cy - radius * Math.cos(rad) };
}

/** An SVG arc `d` path, stroked (not filled), sweeping clockwise from `startBearing` to
 * `endBearing` (degrees, `end` > `start`) at `radius` around `(cx, cy)`. */
export function arcPath(
	cx: number,
	cy: number,
	radius: number,
	startBearing: number,
	endBearing: number
): string {
	const start = pointOnCircle(cx, cy, radius, startBearing);
	const end = pointOnCircle(cx, cy, radius, endBearing);
	const largeArc = endBearing - startBearing > 180 ? 1 : 0;
	return `M ${start.x} ${start.y} A ${radius} ${radius} 0 ${largeArc} 1 ${end.x} ${end.y}`;
}

/**
 * Native pixel geometry of one manufacturer dial face and its matching pointer sprite, both
 * shipped in `ui/static/device/` (see PROVENANCE.md). Face and pointer are measured at the same
 * native scale by the vendor, so a caller that scales both by the same factor keeps them
 * registered (docs/architecture/ui-controls.md section 1). The pointer sprite points up (bearing
 * 0) at its native orientation, tapering to a point at its top edge; its bottom edge is the pivot.
 */
export interface DialFace {
	readonly src: string;
	readonly width: number;
	readonly height: number;
	readonly pointerSrc: string;
	readonly pointerWidth: number;
	readonly pointerHeight: number;
}

/** SegmentDial's three full-circle faces and ArcDial's two half-ring faces, one registry because
 * both composite the same way (see the two components' own `face` prop). Kept here, once, per the
 * "two places must agree" rule: the pixel sizes below must match the shipped bitmaps exactly, and
 * a mismatch is a single edit away from being caught in both components at once. */
export const DIAL_FACES = {
	sensor: {
		src: '/device/sensor/sensor_mode.png',
		width: 170,
		height: 170,
		pointerSrc: '/device/sensor/sensor_mode_point.png',
		pointerWidth: 20,
		pointerHeight: 85
	},
	lod: {
		src: '/device/sensor/lod.png',
		width: 186,
		height: 186,
		pointerSrc: '/device/sensor/lod_point.png',
		pointerWidth: 15,
		pointerHeight: 65
	},
	time: {
		src: '/device/sensor/sleep_time.png',
		width: 262,
		height: 262,
		pointerSrc: '/device/sensor/sleep_time_point.png',
		pointerWidth: 21,
		pointerHeight: 133
	},
	brightness: {
		src: '/device/key/brightness.png',
		width: 151,
		height: 84,
		pointerSrc: '/device/key/brightness_point.png',
		pointerWidth: 19,
		pointerHeight: 62
	},
	speed: {
		src: '/device/key/speed.png',
		width: 160,
		height: 89,
		pointerSrc: '/device/key/speed_point.png',
		pointerWidth: 19,
		pointerHeight: 70
	}
} as const satisfies Record<string, DialFace>;

export type SegmentDialFace = 'sensor' | 'lod' | 'time';
export type ArcDialFace = 'brightness' | 'speed';
