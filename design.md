# Design - Hyperpace

A locked design system for this app. Every screen reads this file before emitting code. Do not
regenerate per screen: extend or amend this file when the system needs to grow.

Route: Hallmark custom theme (tuned). Confirmed by the operator on 2026-09-16: ion cyan accent,
nothing round, the manufacturer's own device art used in the app.

## Genre

atmospheric (dark-tool school), held to instrument discipline rather than mood lighting.

## Macrostructure family

This product has one page type: app screens. There are no marketing or content pages.

- App screens: **Workbench**. A fixed left rail, a titled work area, and panels that carry one
  concern each. Screens vary only in panel composition and which control primitives they use.
- Variation knobs per screen: number of panels, which control primitive leads (segment dial,
  arc dial, slider, list), and whether the screen has a hero control (one large centre-readout
  dial) or a flat panel grid.

## Theme

Paper is a tinted void, never pure black. Accent appears only on live values, focus and the one
primary action on a screen.

- `--color-paper`     oklch(14% 0.014 265)
- `--color-paper-2`   oklch(17% 0.016 265)
- `--color-paper-3`   oklch(21% 0.016 265)
- `--color-ink`       oklch(93% 0.010 250)
- `--color-ink-2`     oklch(72% 0.010 250)
- `--color-muted`     oklch(56% 0.012 255)
- `--color-rule`      oklch(30% 0.014 265)
- `--color-rule-2`    oklch(25% 0.012 265)
- `--color-accent`    oklch(80% 0.14 200)
- `--color-accent-ink` oklch(14% 0.014 265)
- `--color-focus`     oklch(84% 0.19 200)

Status colours, used only for status and never as decoration:

- `--color-warning`   oklch(80% 0.13 75)
- `--color-danger`    oklch(66% 0.17 25)
- `--color-ok`        oklch(78% 0.12 155)

Accent budget: at most 5 percent of any screen. A screen with three dials shows accent on the
live arc of each dial and nowhere else.

## Shape: nothing is round

This is the system's signature and it is not negotiable. Every radius token is `0`. No rounded
cards, no pill buttons, no circular avatars, no rounded inputs, no rounded badges, no
`border-radius` literal anywhere in the codebase.

In place of radius, the **cut plate**: every panel is a machined plate with its top-right corner
cut at 45 degrees, and the hairline border follows the cut.

- `--cut-panel` 14px, applied as `clip-path: polygon(0 0, calc(100% - 14px) 0, 100% 14px, 100% 100%, 0 100%)`
- `--cut-control` 6px, cut on both right corners of buttons, tabs and badges
- `--cut-chip` 4px, for swatches, tags and the connection badge

Supporting rules:

- Panels are separated by 1px hairlines in `--color-rule`, never by shadow. There are no drop
  shadows anywhere: depth comes from the three paper steps and the hairlines.
- A panel header is a strip above a hairline. The active panel carries a 2px accent stub on its
  left edge, 24px tall, aligned to the header baseline.
- Focus is a 2px square `--color-focus` outline with a 2px offset, never a glow, never rounded.
- Progress and meters are square-ended bars with tick divisions, never rounded capsules.

The one exception, stated so nobody has to guess: the **dial instruments are circular**, because
they are the manufacturer's own hardware faces (see Assets). Their frames, labels, readouts and
every piece of chrome around them follow the cut plate.

## The mark

The Hyperpace logo is `art/hyperpace.svg`, also served to the app at `/hyperpace.svg`: a chamfered
HUD frame with neon corner brackets, an angular shell, and a red head. Its cut corners are the same
idea as the cut plate, which is why the system and the mark agree.

The logo carries one colour the interface palette does not, the head's red
`#FF003C`. That is the mark's own, and it stays inside the mark: it is not an interface accent, and
`--color-danger` remains the only red the screens use.

Three sizes, because the full artwork does not survive being shrunk:

- Full logo, 64px and up: the SVG as drawn, grid and traces and all.
- Rail mark, about 22px: the shell outline plus its head, redrawn inline in `Sidebar.svelte`. The
  grid, the brackets and the traces are dropped because they turn to mush at that size.
- Tray icon, 40px: the chamfered frame and the two neon corner brackets, redrawn as pixels in
  `crates/hyperpace-app/src/icon.rs`, filled with the battery's status colour and carrying the
  percentage. Redrawn rather than rasterized, so the app needs no SVG renderer for one small square.

## Assets: the device's own art

The app ships the manufacturer's product renders and control faces. Authorized by the operator on
2026-09-16, replacing the earlier original-line-art rule. They live in `ui/static/device/` and are
catalogued in `ui/static/device/PROVENANCE.md`.

- The mouse render and the receiver render are the device illustrations on every screen that needs
  one. They sit on the dark ground with no drop shadow and no rounded frame.
- Dials are built exactly the way the hardware configurator builds them: the face bitmap at its
  native diameter (sensor mode 170, lift-off 186, sleep time 262, brightness 151x84, speed
  160x89), the pointer sprite rotated about its bottom edge at the dial centre, and the live arc
  drawn by Hyperpace in `--color-accent` over the face.
- The white faces are kept white. White hardware on a black instrument panel with one cyan live
  value is the look; do not tint the faces.
- Faces with the vendor's orange baked into the pixels are not used. Hyperpace draws those
  segments itself so the live value is always this app's accent.
- The vendor wordmark printed on the physical device is visible in the device renders. It appears
  nowhere else: not in the app's chrome, copy, window title, binary, or package metadata.

## Typography

- Display: Chakra Petch, weight 600, roman. Screen titles and dial centre readouts. Its
  letterforms carry the same chamfered cut corner as the mark and the cut plate, which is why it
  replaced the earlier geometric-sans display face.
- Body: Saira, weight 400 and 500. Labels, descriptions, buttons. A squared grotesque that stays
  legible at small sizes, in the same family as the display face's angularity.
- Numerics: JetBrains Mono, weight 500, `font-variant-numeric: tabular-nums`. Every value a
  device reports: DPI, hertz, milliseconds, percent, version strings, byte counts. A true
  monospace, not Saira's own tabular figures, so a device readout stays visually distinct from a
  label at a glance.
- Display tracking: -0.02em at display sizes, 0 at body sizes.
- Type scale anchor: `--text-display` = clamp(1.75rem, 1.4rem + 1.2vw, 2.25rem). This is an app,
  not a landing page: display type stays modest.

Fonts are self-hosted and never fetched at runtime, because the app must render with no network.
Installed: `@fontsource/chakra-petch` (600 weight only, the only weight the app uses),
`@fontsource-variable/saira`, `@fontsource-variable/jetbrains-mono`, all 5.3.0, imported once in
`ui/src/routes/+layout.svelte`. Chakra Petch ships no variable build, so its package registers the
plain family name `Chakra Petch`; Saira and JetBrains Mono are variable packages and register
their family names with the `Variable` suffix, which is why the tokens below read "Saira Variable"
and "JetBrains Mono Variable" but plain "Chakra Petch". Saira's variable font does carry genuine
tabular figures (its GSUB table exposes `tnum`), verified directly against the upstream binary
rather than assumed; JetBrains Mono is used for numerics anyway; because a true monospace keeps
the instrument-readout look design.md's `.mono` class describes, which Saira's proportional
letterforms would not.

## Spacing

4-point named scale in `tokens.css`. Screens use named tokens (`var(--space-md)`), never raw
values.

## Motion

One primitive, one duration.

- `--ease-out` cubic-bezier(0.16, 1, 0.3, 1)
- `--dur-settle` 120ms: a dial's pointer and arc settling to a new value.
- No reveal animations, no staggered entrances, no hover lifts on panels.
- `prefers-reduced-motion: reduce` collapses the settle to an instant change.

### Depth

The app has exactly one piece of 3D, and it belongs to the hardware: the device renders sit on a
900px perspective and tilt with the pointer, at most 5 degrees on either axis, settling over
`--dur-settle`. The numbered button callouts are lifted 18px toward the viewer so they stay pinned
to the shell as it turns.

That is the whole budget. Panels do not tilt, cards do not lift, nothing rotates on entry, and no
element moves unless the pointer is on the artwork. The effect should read as the object sitting
inside the panel, not as an animation being played at the operator, and it disappears completely
under `prefers-reduced-motion: reduce`.

## Microinteractions stance

- Silent success. A write that lands changes the control and nothing else. No toast.
- A write that fails returns the control to the device's last known value and shows one inline
  message on that control.
- Dials commit once on release, never per frame of a drag.
- Hover tooltips delay 300ms; focus tooltips appear instantly.
- Every adjustable control, and every panel that names a feature, carries a help tip: a 16px cut
  "?" right after its label, using `HelpTip` with an entry from `ui/src/lib/help.ts`. One catalogue
  for the whole app, so a setting is explained the same way everywhere. An explanation states only
  what is established about the hardware and marks anything unconfirmed as unconfirmed. One tip per
  concept: a table gets one on its header, not one per row.
- `:focus-visible` shows the square focus outline immediately, never animated.

## CTA voice

- Primary action: accent fill, `--color-accent-ink` text, `--cut-control`, one per screen.
- Secondary action: 1px `--color-rule` outline, ink text, transparent fill.
- Destructive action: `--color-danger` outline and text, fill only on hover.
- Copy is a verb plus its object: "Apply receiver light", "Start pairing", "Import package".

## Per-screen allowances

- App screens MUST NOT use decorative enrichment. Function carries the screen.
- No screen may show a value it has not read from the device. A control with no reading is
  disabled and blank. No invented default DPI, brightness, colour or timeout is ever displayed as
  though it came from the hardware.

## What screens MUST share

- The rail, its order and its type treatment.
- The cut plate, the hairlines and the zero radius.
- The accent colour and its placement rule (live value, focus, one primary action).
- The three fonts and their roles, numerics in mono without exception.
- The disabled and unsupported states.

## What screens MAY differ on

- Panel count and arrangement within the Workbench shape.
- Which control primitive leads the screen.
- Whether a hero dial is present.

## Exports

### tokens.css

```css
:root {
  --color-paper:       oklch(14% 0.014 265);
  --color-paper-2:     oklch(17% 0.016 265);
  --color-paper-3:     oklch(21% 0.016 265);
  --color-ink:         oklch(93% 0.010 250);
  --color-ink-2:       oklch(72% 0.010 250);
  --color-muted:       oklch(56% 0.012 255);
  --color-rule:        oklch(30% 0.014 265);
  --color-rule-2:      oklch(25% 0.012 265);
  --color-accent:      oklch(80% 0.14 200);
  --color-accent-ink:  oklch(14% 0.014 265);
  --color-focus:       oklch(84% 0.19 200);
  --color-warning:     oklch(80% 0.13 75);
  --color-danger:      oklch(66% 0.17 25);
  --color-ok:          oklch(78% 0.12 155);

  --font-display: "Chakra Petch", sans-serif;
  --font-body:    "Saira Variable", "Saira", system-ui, sans-serif;
  --font-mono:    "JetBrains Mono Variable", ui-monospace, monospace;

  --color-faint:        oklch(44% 0.012 258);
  --color-accent-soft:  oklch(80% 0.14 200 / 0.14);
  --color-ok-soft:      oklch(78% 0.12 155 / 0.14);
  --color-warning-soft: oklch(80% 0.13 75 / 0.14);
  --color-danger-soft:  oklch(66% 0.17 25 / 0.14);

  --space-3xs: 0.25rem; --space-2xs: 0.5rem;  --space-xs: 0.75rem;
  --space-sm:  1rem;    --space-md:  1.5rem;  --space-lg: 2rem;
  --space-xl:  3rem;    --space-2xl: 4.5rem;

  --text-xs: 0.75rem;  --text-sm: 0.8125rem; --text-md: 0.9375rem;
  --text-lg: 1.125rem; --text-xl: 1.5rem;
  --text-display: clamp(1.75rem, 1.4rem + 1.2vw, 2.25rem);

  --ease-out: cubic-bezier(0.16, 1, 0.3, 1);
  --dur-settle: 120ms;

  --radius: 0;
  --cut-panel: 14px; --cut-control: 6px; --cut-chip: 4px;

  --dial-compact: 120px; --dial-default: 168px; --dial-large: 232px;
}
```

### DTCG tokens.json

```json
{
  "color": {
    "paper":  { "$value": "oklch(14% 0.014 265)", "$type": "color" },
    "ink":    { "$value": "oklch(93% 0.010 250)", "$type": "color" },
    "accent": { "$value": "oklch(80% 0.14 200)", "$type": "color" }
  },
  "font": {
    "display": { "$value": "Chakra Petch", "$type": "fontFamily" },
    "body":    { "$value": "Saira", "$type": "fontFamily" },
    "mono":    { "$value": "JetBrains Mono", "$type": "fontFamily" }
  },
  "space":  { "md": { "$value": "1.5rem", "$type": "dimension" } },
  "shape":  { "radius": { "$value": "0", "$type": "dimension" },
              "cutPanel": { "$value": "14px", "$type": "dimension" } }
}
```

## Stamp

Every stylesheet in this project carries:

```css
/* Hallmark · genre: atmospheric · macrostructure: Workbench · design-system: design.md · designed-as-app
 * theme: custom · vibe: "deep space, starlight, instrument-grade, cold vacuum"
 * paper: oklch(14% 0.014 265) · accent: oklch(80% 0.14 200) · shape: cut plate, zero radius
 * display: Chakra Petch · body: Saira · mono: JetBrains Mono
 * axes: dark / geometric-sans / cool · studied: no
 */
```
