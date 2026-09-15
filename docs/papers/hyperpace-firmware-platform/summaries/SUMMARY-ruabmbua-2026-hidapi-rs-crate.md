# SUMMARY: hidapi crate (hidapi-rs) 2.6.7

## Claim
hidapi 2.6.7 (released 2026-08-27) is an actively maintained Rust wrapper around the C HIDAPI library. It also has optional pure-Rust Linux and Windows backends.

## Method
Read the source (git clone at 8729463), crates.io registry JSON, docs.rs auto-trait sections, GitHub API metadata, and the linked udev, libudev-sys and basic-udev crates.

## Result
- **Activity:** releases 2.6.4 (2025-12-20), 2.6.5 (2026-02-25), 2.6.6 (2026-05-15), 2.6.7 (2026-08-27). 19 open issues. The owner (ruabmbua) merges community PRs.
- **Linux backends (exactly one must be selected):**
  - Default `linux-static-hidraw`: compiles the C hidraw code and probes `libudev` through pkg-config. It needs libudev at build time and, assuming the pkg-config default of dynamic linking, libudev.so.1 at runtime.
  - `linux-static-libusb` / `linux-shared-*`: libusb-based. Usage page is not available on the Linux libusb backends.
  - `linux-native`: pure-Rust hidraw, but uses the `udev` crate, which links libudev through libudev-sys and pkg-config.
  - `linux-native-basic-udev`: pure-Rust hidraw plus the pure-Rust `basic-udev` crate, no libudev.
- **Windows:** the default compiles C `windows/hid.c` and links setupapi. `windows-native` is pure Rust on windows-sys 0.61.
- **macOS:** always the C `mac/hid.c`, linked against IOKit.
- **Bundled C version:** hidapi-0.14.0-41-gff67c77 (2024-10-30). That predates `hid_read_error` (2025-03-11) and the 0.15.0 release (2025-05-18).
- **Thread traits:** HidApi is Send + Sync. HidDevice is Send and !Sync; the backends declare `unsafe impl Send`.
- **Report IDs:** the report ID is the leading byte, so a 16-byte report is written as 17 bytes. `read_timeout` takes milliseconds (-1 blocks) and returns Ok(0) on timeout.
- **windows-native details:**
  - Writes are padded with zeros, or truncated, to OutputReportByteLength.
  - `write` returns Ok(0) when WriteFile completes synchronously, otherwise the GetOverlappedResult byte count (the padded length). It waits at most 1000 ms.
  - A read timeout leaves the overlapped read pending.
  - `send_output_report` borrows the feature buffer, which is sized to FeatureReportByteLength.
  - Open falls back to a no-access open for system devices and sets 64 input buffers.
- **linux-native details:** opens O_RDWR | O_NONBLOCK | O_CLOEXEC and writes the buffer as-is. `read_timeout` uses poll(), and POLLERR/POLLHUP/POLLNVAL returns "unexpected poll error (device disconnected)".
- **macOS:** exclusive (seize) open by default. `set_open_exclusive(false)` or the `macos-shared-device` feature switches it off.
- **Hotplug:** no API.

## Evidence tier
1 (source code, registry data, generated docs).

## Performance
- No benchmarks in the crate.
- Every call is a synchronous FFI or syscall per report; the Windows padding copy is at most 49 bytes, negligible.
- On Linux, a poll() plus read() per report.
- A tier-3 measurement on macOS through the C backend (async-hid PR 46) got 482 exchanges/s on a test board.

## Correctness
- Proven by source: the feature matrix, pads, return values, traits, and the bundled version.
- Not exercised at runtime: `send_output_report` on windows-native sizes its buffer by FeatureReportByteLength. For a TLC with no feature reports (length 0), that would send an empty buffer; with a smaller feature length it would truncate. This is a probable bug, found by reading code only.
- "Dynamic libudev" assumes the pkg-config crate's default. No binary's link table was inspected.
- The docs.rs auto-trait lists came through an extraction tool.

## Relevance to hyperpace
- A viable primary backend. Recommended settings:
  - Linux: `linux-native-basic-udev`, so an AppImage carries no libudev dependency; otherwise accept libudev.so.1.
  - Windows: `windows-native`.
  - macOS: non-exclusive open.
- Use `write` (WriteFile), never `send_output_report`, and ignore the returned byte count.
- Because HidDevice is !Sync, own each device on a single thread. That thread interleaves writes and short `read_timeout` calls and talks to the rest of the app over channels.
- The 0.15.0 read-thread guarantee does not apply to the bundled 0.14.0-41 code.
- Hotplug must come from elsewhere (async-hid, nusb, or native OS APIs).
