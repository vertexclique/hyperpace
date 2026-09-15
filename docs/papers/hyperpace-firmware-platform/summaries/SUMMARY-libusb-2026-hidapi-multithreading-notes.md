# SUMMARY: HIDAPI wiki, Multi-threading Notes

## Claim
- HIDAPI 0.x is not thread-safe.
- init, exit, enumerate, open, open_path, close and error(NULL) must not run concurrently.
- Device functions may run on different threads for different devices, never concurrently on the same device.
- The exception, since v0.15.0: one dedicated thread doing only hid_read, hid_read_timeout and hid_read_error.
- On macOS, init and exit must run on the same thread, and that thread must stay alive until everything is closed.

## Method
Maintainer-authored project wiki. The same rules appear verbatim in the repository's AGENTS.md.

## Result
- Pattern A: one owner thread per device does all I/O.
- Pattern B (0.15.0+ only): reader thread plus writer thread.
- Global calls (enumerate, open, close) must be serialized application-wide.

## Evidence tier
1 (maintainer-authored project documentation, mirrored in the repository).

## Performance
- Pattern A makes a write wait for the current `read_timeout` slice. The latency added is at most the slice length; a 10 to 50 ms slice is negligible against the ~200 ms response budget.
- That figure is design arithmetic, not a measurement.

## Correctness
- Proven: the contract, as stated by the maintainers.
- The 0.15.0 read-thread exception does not cover hidapi-rs 2.6.7, which bundles 0.14.0-41. Rust also forbids it anyway, because HidDevice is !Sync.

## Relevance to hyperpace
With hidapi-rs, use Pattern A:
- one global HID thread (or a mutex) for enumerate, open and close, which on macOS must be the long-lived init thread;
- one owner loop per open device that interleaves queued writes with short read_timeout calls and routes responses and notifications to channels.
