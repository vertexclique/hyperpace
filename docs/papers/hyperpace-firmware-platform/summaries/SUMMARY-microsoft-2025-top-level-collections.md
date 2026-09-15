# SUMMARY: Microsoft, Top-Level Collections

## Claim
For every top-level collection in a report descriptor, Windows HIDClass creates one physical device object (PDO), which is one openable device path. An application that opens a TLC's PDO talks to that collection only.

## Method
Official conceptual documentation.

## Result
- On Windows, the HYPACE vendor interface (usage pages 0x01, 0x0C, 0xFF02..0xFF06 in one descriptor) shows up as several HID device paths, one per TLC.
- Each path has its own HIDP_CAPS: usage page, usage, and report byte lengths.
- The report-8 channel is exactly one of those paths.

## Evidence tier
1 (official documentation).

## Performance
Not applicable. Enumeration cost grows with the number of paths, which is small here.

## Correctness
- Proven: one PDO per TLC.
- Not covered by this page: what happens when a report ID is written to a TLC that does not declare it. A community capture of a same-vendor device reports that other vendor collections reject WriteFile with "Incorrect function" (tier 3, SUMMARY-openmouse-2026-lamzu-atlantis-testing).
- Linux and macOS do not split per TLC: hidraw has one node per USB interface, and macOS one IOHIDDevice per interface.
- hidapi's enumeration emits one entry per top-level usage pair on all three OSes. On Linux and macOS those entries share one path.

## Relevance to hyperpace
- On Windows, choose the path by (VID 0x3554, PID 0xFB16 or 0xFB14, usage_page, usage) from enumeration. Opening the wrong TLC fails or rejects writes.
- On Linux and macOS, dedupe the enumeration entries by path.
- The exact usage pair for HYPACE report 8 must come from a descriptor dump. The same-vendor sibling suggests 0xFF02/0x0002, which is unverified for HYPACE.
