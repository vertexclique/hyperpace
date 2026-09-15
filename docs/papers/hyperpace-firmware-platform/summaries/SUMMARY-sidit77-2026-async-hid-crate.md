# SUMMARY: async-hid 0.5.3

## Claim
async-hid is a pure-Rust async HID library meant to replace hidapi-rs:
- backends: hidraw on Linux, Win32 or WinRT on Windows, IOHIDManager on a dispatch queue on macOS;
- a `watch()` stream of connect and disconnect events on every backend;
- no C code and no libudev.

## Method
Read the source (git clone at 1d34435), README, Cargo.toml, crates.io JSON and the issue/PR list.

## Result
- **Releases:** 0.5.0 (2026-02-09) through 0.5.3 (2026-06-11). 51,297 downloads, 30 stars, 9 open issues. One main maintainer, currently on vacation per a 2026-09-06 comment.
- **Linux:**
  - enumerates `/sys/class/hidraw`;
  - `watch` binds a netlink kobject-uevent socket (the udev group if /run/udev/control exists, else the kernel group), filtered to subsystem hidraw;
  - async I/O through async-io (default) or tokio.
- **Windows (win32):**
  - `watch` uses CM_Register_Notification with a DEVICEINTERFACE filter on the HID class GUID;
  - DeviceInfo carries the TLC's UsagePage and Usage from HidP_GetCaps;
  - writes pad to OutputReportByteLength;
  - opens set 64 input buffers.
- **macOS:**
  - one serial dispatch queue;
  - manager matching and removal callbacks feed `watch`;
  - `unsafe impl Send + Sync` justified by queue confinement;
  - no run loop or main-thread requirement.
- **API:** DeviceInfo includes usage_page and usage_id. Readers and writers are split. Maintainer comments say the report ID is the first byte on all platforms.
- **README:** its example is stale (per PR 48).

## Evidence tier
1 (source code, registry). The maturity assessment draws on tier 2 and 3 issue content.

## Performance
- Async reads and writes on all backends; enumeration is synchronous on Win32, Linux and macOS.
- A contributor's macOS measurement showed a stall in 0.5.3; the patched version ran at 479 exchanges/s (tier 3, issue 45 / PR 46).

## Correctness
- Proven by source: the backend mechanisms.
- **Open defects:**
  - macOS stops delivering input reports after 30 to 444 exchanges (issue 45, fix PR 46 unmerged).
  - Win32 write pending-state is not cleared on device loss, leading to an "I/O operation already pending" assertion (PR 48 unmerged, untested on hardware).
  - Win32 rejects concurrent writes (issue 19; wrap writers in a mutex).
- Small maintainer base.

## Relevance to hyperpace
- Best-fitting hotplug layer: it covers all three OSes with no C dependency and reports usage pairs.
- As the I/O layer it is currently risky on macOS: at a ~5 s battery poll, one exchange every 5 s would reach the reported stall window in about 2.5 to 37 minutes (arithmetic from 30 to 444 exchanges, not measured).
- Options:
  - (a) async-hid for watch plus hidapi-rs for I/O;
  - (b) async-hid with PR 46 and PR 48 vendored or patched;
  - (c) wait for a release.

  This is an operator decision.
