# Hyperpace control language: gauges and sliders

Status: specification for the interface rework. The controls below replace the plain selects and
linear sliders currently on the Performance and Lighting screens.

Why gauges: the settings this app edits are small, discrete and physical (three lift-off
distances, eight sleep timeouts, ten brightness steps). A dial shows the whole range and the
current position at a glance, which a select does not, and it reads as an instrument rather than
a form. The vendor's own configurator uses the same model, so the interaction will be familiar to
anyone coming from it.

The dial faces and the device renders are the manufacturer's own art, shipped in
`ui/static/device/` and catalogued in `ui/static/device/PROVENANCE.md`. The operator authorized
their use on 2026-09-16. Hyperpace draws the live arc, the labels, the readouts and every piece of
chrome itself, so the value always carries this app's accent. No vendor name, logo or model string
appears in the app's own chrome, copy, binary or package metadata; the wordmark printed on the
physical device is visible in the device renders because it is part of the hardware.

## 1. Reference geometry

Measured from the vendor bundle, recorded so the replacements are designed from real numbers
rather than an impression. These are the source proportions, not a pixel target: Hyperpace scales
them to fit its own window (see section 5).

| Control | Vendor diameter | Positions | Pointer | Shape |
|---|---|---|---|---|
| Sensor mode | 170 | 3 | 20 | full circle, equal segments |
| Lift-off distance | 186 | 3 | 15 | full circle, equal segments |
| Sleep time | 262 | 8 | 24 | full circle, centre readout at 60 px |
| Highest performance | 262 | 8 | 24 | full circle, centre readout, power button inside |
| Receiver brightness | 151 | 20 | 19 | half ring, top half only |
| Receiver speed | 160 | 20 | 14 | half ring, top half only |
| DPI, polling rate, debounce | n/a | continuous or stepped | n/a | linear slider |

The measurements above are the native pixel dimensions of the shipped face bitmaps, so the
control is built at true size: the face at its native diameter, the pointer sprite rotated about
its bottom edge at the dial centre (`transform-origin: 50% 100%`), and the live arc drawn by
Hyperpace as an SVG path over the face in `--color-accent`. The two faces that carry the vendor's
orange baked into their pixels, `sensor/performance_on.png` and `performance_off.png`, are not
used; Hyperpace draws those segments itself.

## 2. Primitives

Three components carry every setting. Each is a single Svelte component in `ui/src/lib/controls/`.

### 2.1 `SegmentDial`

A full circle divided into N equal segments, one per allowed value. The active segment is filled
with the accent colour, the rest with the track colour, and a pointer points at the active
segment's centre. Labels sit outside the circle at each segment's mid angle.

Used by: sensor mode (3), lift-off distance (3), sleep time (8), highest performance (8).

Props: `values` (label plus value per segment), `value`, `disabled`, `size`, optional
`centerLabel` (large text in the middle, for example `30 s` or `2 min`), optional `centerAction`
(a button rendered in the middle, used by highest performance for its on/off).

### 2.2 `ArcDial`

A half ring covering the top 180 degrees, divided into N steps, filled from the left end to the
current step. Used by receiver brightness (10 steps) and receiver speed (10 steps), and by any
future one-to-ten setting.

Props: `min`, `max`, `step`, `value`, `disabled`, `size`, `label`, `format`.

### 2.3 `RangeSlider`

The linear control, restyled to match: a track, a filled portion, a handle, tick marks at
meaningful stops, and the value shown at the handle. Used by DPI per stage, polling rate,
debounce and the raw receiver timeout.

## 3. Interaction

Every gauge is a real control, not a picture:

- Pointer: click or tap a segment to select it; press and drag anywhere on the dial to sweep.
- Keyboard: arrow keys move one step, Home and End jump to the ends, Tab reaches every control.
- Wheel: one step per notch while the control has focus.
- Semantics: `role="slider"` with `aria-valuenow`, `aria-valuemin`, `aria-valuemax` and
  `aria-valuetext` carrying the human label (for example "2 mm"), so the value is announced as
  text, not as a bare number.
- Motion: the pointer and fill animate over about 120 ms, and the animation is dropped entirely
  under `prefers-reduced-motion`.
- Commit: dragging updates the visual immediately and writes to the device once, on release, so a
  sweep does not send a burst of writes.

## 4. States

| State | Appearance |
|---|---|
| No device | full control drawn, track and labels visible, pointer hidden, no value text, control disabled and not focusable |
| Reading | never shown as a placeholder; the control stays disabled until a value arrives |
| Ready | accent fill, pointer at the value, value text shown |
| Unsupported on this device | control drawn disabled with a short caption naming why, never a dead toggle that looks live |
| Write failed | control returns to the device's last known value and the error is surfaced once |

A control with no value never invents one: no default DPI, brightness or timeout is displayed as
though it came from the mouse.

## 5. Sizing and fit

Sizes are relative, not fixed: each gauge takes `size` from a CSS variable with three tiers
(compact 120, default 168, large 232 CSS px) and the screen picks a tier from the available
width using container queries. A screen must fit the default window without scrolling its primary
content, so dials shrink a tier before anything scrolls, and the large tier is used only for the
one hero control on a screen (sleep time on Performance, for example).

## 6. Colour

From the palette in `design.md`: the face bitmap is left white and untinted, the live arc is
`--color-accent`, the empty track is `--color-rule`, labels are `--color-muted`, and the active
label is `--color-accent`. Nothing around a dial is round: frames, readouts and captions follow
the cut plate. The
DPI stage colours are the exception: each stage's own colour fills its slider handle and its
swatch, because that colour is data the mouse stores, not decoration.
