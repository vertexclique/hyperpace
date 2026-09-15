# SUMMARY: go-macos iokit/hid package documentation

## Claim
A third-party Go IOKit HID library (v0.13.0, 2026-09-08) documents three things:
- Vendor-specific devices (usage page 0xFF00+) normally open without consent; devices publishing keyboard or pointer usages are gated by Input Monitoring and fail with kIOReturnNotPermitted.
- IOKit callbacks are delivered on a run loop tied to one OS thread, so the stream pins a thread.
- Releasing a device while it is still scheduled on a run loop crashes with "os_unfair_lock is corrupt" (SIGKILL, no stack). Close order must be cancel, wait, close.

## Method
Library documentation written from its authors' experience.

## Result
- The permission rule: vendor pages exempt, keyboard and pointer usages gated.
- A concrete crash signature and the correct teardown order.

## Evidence tier
3 (third-party library documentation).

## Performance
Not applicable. It notes that a slow callback handler back-pressures the device.

## Correctness
- The permission rule mostly matches Apple source: keyboard, mouse and touchpad pairs. "Pointer" (0x01/0x01) is not in Apple's list; only mouse (0x01/0x02) is.
- The crash-on-release claim is plausible and consistent with hidapi-rs issue 127 analysis, but unverified.

## Relevance to hyperpace
- For any native or async macOS path: stop the read loop and join its thread before releasing or closing the device. Never drop the device from another thread while it is scheduled.
- hidapi's `hid_close` stops the run loop and joins the thread internally, but must not race with reads (HIDAPI issue 133).
