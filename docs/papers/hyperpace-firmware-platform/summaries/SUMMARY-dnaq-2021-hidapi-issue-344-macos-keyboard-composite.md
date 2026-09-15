# SUMMARY: HIDAPI issue 344, a macOS device that also carries a keyboard collection

## Claim
- On macOS, a HID device node with several usage pages or usages is "a single undividable device".
- A node that includes a keyboard collection needs Input Monitoring.
- Opening it with SeizeDevice also takes its keystrokes away from the OS.
- Opening non-exclusively with Input Monitoring granted works without root.

## Method
User experimentation plus maintainer comments.

## Result
- Access is per node, not per collection.
- Exclusive open plus a keyboard usage means the user's keys stop working.
- Non-exclusive open plus granted Input Monitoring works.

## Evidence tier
2 (maintainer statement), and 3 for the user's experimental report.

## Performance
Not applicable.

## Correctness
- The per-node rule is confirmed by Apple's IOHIDFamily source: `conformsTo` walks every usage pair of the node (SUMMARY-apple-2026-iohidfamily-tcc-gating-source).
- The "root plus Input Monitoring" step reported for exclusive open is anecdotal.

## Relevance to hyperpace
- The HYPACE vendor interface must not conform to keyboard or mouse usages, or the app needs Input Monitoring. Its page-0x01 usage must be checked from the descriptor.
- The boot-keyboard interface must never be the one opened.
- Non-exclusive open is mandatory either way.
