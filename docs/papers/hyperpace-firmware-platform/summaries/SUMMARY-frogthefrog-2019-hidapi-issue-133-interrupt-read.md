# SUMMARY: HIDAPI issue 133, a blocking hid_read cannot be safely interrupted

## Claim
- `hid_close` cannot safely interrupt a blocking `hid_read` on another thread. On Windows the C code used CancelIo, which only cancels I/O started on the calling thread; on hidraw and macOS there is always a race.
- The maintainer's prescribed pattern: loop on `hid_read_timeout` with a finite timeout, check an interrupt flag, join the reader thread, then close.
- node-hid switched to that pattern.

## Method
Issue thread with the HIDAPI maintainer and the node-hid maintainer.

## Result
- Cancellation must be cooperative: bounded read timeouts plus a stop flag, then join, then close.
- Closing during a read is undefined behavior by design.

## Evidence tier
2 (maintainer statements).

## Performance
- Shutdown latency is bounded by the read timeout slice (1000 ms in the maintainer example).
- A shorter slice trades CPU wakeups for faster shutdown. Not measured.

## Correctness
- Proven as the library contract.
- hidapi-rs windows-native uses CancelIoEx, but the Rust type system already forbids a concurrent close (HidDevice is !Sync, and Drop needs ownership).

## Relevance to hyperpace
- The device owner loop uses `read_timeout` slices (for example 50 ms, which also bounds write latency), checks a cancellation channel each slice, and drops the device only on its own thread.
- Async cancellation of a request drops the waiter, not the device. The loop discards the late response by matching.
