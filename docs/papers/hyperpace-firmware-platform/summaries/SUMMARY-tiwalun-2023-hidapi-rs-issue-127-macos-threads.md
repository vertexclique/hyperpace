# SUMMARY: hidapi-rs issue 127, macOS crash when HidApi::new runs on different threads

## Claim
On macOS, calling `HidApi::new` from several threads, or from a thread that later exits, aborts the process in `CFRunLoopAddSource` / `_CFAssertMismatchedTypeID`. The C HIDAPI's IOHIDManager is bound to the run loop of the thread that first initialized it.

## Method
User reports with stack traces, a maintainer reply, and a community root-cause explanation that points into IOKitUser source.

## Result
- Crashes were reproduced by several users in 2023, 2024 (macOS Sequoia 15.1, inside egui) and 2025.
- The maintainer admits macOS was untested.
- The working workaround is to create HidApi on one dedicated thread and keep that thread alive for the whole process.
- micolous explains: "IOHIDManager needs to be alive and associated with an active CFRunLoop while any IOHIDDevice instance it created is still open or in use."
- The issue was still open on 2026-09-15.

## Evidence tier
2 (maintainer comment plus technically sourced community analysis), and 3 for the individual crash reports.

## Performance
Not applicable.

## Correctness
- The mechanism matches the C source: `init_hid_manager` schedules on `CFRunLoopGetCurrent()`, and `hid_enumerate` pumps the calling thread's run loop.
- The HIDAPI wiki independently requires `hid_init`/`hid_exit` on the same thread, kept alive.
- Not proven: that every crash path is covered by the one-thread workaround. The repeated new/drop crash in unit tests (splix) is not explained.

## Relevance to hyperpace
- With hidapi-rs on macOS, create the HidApi context once, on a dedicated long-lived HID thread, never dropped until exit.
- Run all enumeration, open and close calls on that thread.
- Tauri's event loop owns the main thread, so this must be an explicit background thread, never a transient tokio task.
