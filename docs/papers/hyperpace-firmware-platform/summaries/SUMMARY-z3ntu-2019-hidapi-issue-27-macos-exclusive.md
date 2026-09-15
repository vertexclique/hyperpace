# SUMMARY: HIDAPI issue 27, macOS exclusive open

## Claim
HIDAPI on macOS opens devices with kIOHIDOptionsTypeSeizeDevice, which keeps the device's input from reaching the OS (for a mouse, the cursor stops). Windows opens are non-exclusive. The issue was resolved by adding `hid_darwin_set_open_exclusive` while keeping exclusive as the default. Issue 212 adds a second point: on macOS no two clients can share a device if either one opens it exclusively, and that includes vendor drivers.

## Method
GitHub issue threads with maintainer (Youw) comments, cross-checked against current source.

## Result
- The current source keeps exclusive as the default "for backward compatibility".
- hidapi-rs exposes `set_open_exclusive` and the `macos-shared-device` feature.
- Issue 212: a vendor driver holding a device exclusively blocks HIDAPI even when HIDAPI opens non-exclusively.

## Evidence tier
2 (maintainer comments), plus 1 for the source.

## Performance
Not applicable.

## Correctness
- Proven: the default and the setter.
- The effect on HYPACE is inferred. HYPACE's vendor interface contains consumer-control and a page-0x01 usage, so seizing it would plausibly swallow those events (media keys, system control) while the app holds the device. Not tested.

## Relevance to hyperpace
- Always open non-exclusively on macOS.
- If Lofree's own software or a web configurator holds the device exclusively, opens fail. The error message should tell the user to close other configurators.
