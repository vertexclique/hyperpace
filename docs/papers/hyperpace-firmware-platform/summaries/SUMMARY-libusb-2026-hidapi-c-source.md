# SUMMARY: C HIDAPI (libusb/hidapi) source, master 2026-08-14

## Claim
The C HIDAPI source defines the behavior that hidapi-rs inherits on macOS and on the default Windows and Linux backends.
- Windows: pads writes to OutputReportByteLength, waits up to 1000 ms per write, and sets 64 input buffers.
- macOS: seizes devices exclusively by default, uses a per-device read thread with its own run loop, and sends reports with synchronous IOHIDDeviceSetReport.
- Linux hidraw and macOS emit one enumeration entry per usage pair, all sharing one path.
- Hotplug is not in master.

## Method
git clone and reading of `windows/hid.c`, `mac/hid.c`, `linux/hid.c`, `udev/69-hid.rules` and AGENTS.md; GitHub releases and repository metadata.

## Result
- **Releases:** 0.15.0 (2025-05-18) is the latest release. Master's VERSION reads 0.16.0. 76 open issues; last push 2026-09-09.
- **Windows `hid_write`:** pads short buffers to `output_report_length` and returns `bytes_written`, which is the padded length. Waits at most `write_timeout_ms` (1000 ms by default) before returning an error.
- **Windows `hid_read_timeout`:** on timeout, "leave the Overlapped I/O running" and return 0. Strips a leading 0x00 report byte.
- **Windows `hid_send_output_report`:** pads using `write_buf`, sized to OutputReportByteLength. The comment says a shorter buffer gives ERROR_INVALID_PARAMETER and a longer one is silently truncated.
- **macOS:**
  - `hid_init` sets exclusive open "for backward compatibility", and the manager is scheduled on the calling thread's run loop.
  - `hid_open_path` calls IOHIDDeviceOpen with those options, registers input and removal callbacks, and starts a pthread read thread.
  - `set_report` strips a zero report ID and refuses to send when `dev->disconnected`.
- **Linux:** the enumeration loop creates an extra record for each additional usage pair with the same path.
- **udev sample rule:** `KERNEL=="hidraw*", ATTRS{idVendor}=="04d8", ATTRS{idProduct}=="003f", TAG+="uaccess"`. The file must sort before 73-seat-late.rules; add `GROUP="plugdev", MODE="660"` for remote sessions.
- **AGENTS.md (2026-08-14):**
  - Not thread-safe; one thread per device.
  - Since v0.15.0, a dedicated read thread using only `hid_read`, `hid_read_timeout` and `hid_read_error` is safe.
  - On macOS, init and exit must run on the same long-lived thread.
- **Hotplug:** `hid_hotplug_register_callback` exists only on the hotplug branches.

## Evidence tier
1 (source code and maintainer-authored contract).

## Performance
- Windows write latency is bounded by the 1000 ms wait; reads return within the given timeout.
- macOS runs a read thread per open device.
- No benchmarks in the repository were read.

## Correctness
- Proven: the behaviors above, as of master 852cc68.
- hidapi-rs bundles an older commit (0.14.0-41). Its code may differ in detail, notably the read-thread safety and `hid_read_error`.
- The Windows error codes in comments are maintainer statements, not Microsoft documentation.

## Relevance to hyperpace
- On Windows the report-8 write length is handled by the library, but callers must not trust the returned count.
- On macOS, call `hid_darwin_set_open_exclusive(0)` (hidapi-rs: `set_open_exclusive(false)`) before opening the vendor interface, and keep the init thread alive.
- The udev rule template and its "sort before 73" constraint carry over directly to a HYPACE rule for VID 3554 with PIDs fb16 and fb14.
