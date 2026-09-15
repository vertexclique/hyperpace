# SUMMARY: Linux drivers/hid/hidraw.c

## Claim
The hidraw source defines the Linux behavior:
- `write` accepts 2 to HID_MAX_BUFFER_SIZE bytes, copies them as-is and hands them to the output-report path, falling back to SET_REPORT if there is no interrupt OUT.
- `read` fails with -EIO if the device disappears while waiting.
- `poll` returns EPOLLERR | EPOLLHUP once the device no longer exists or access was revoked.

## Method
Read mainline source via the GitHub API.

## Result
- No padding to the report length is done in hidraw.c. The buffer and its count pass straight to `__hid_hw_output_report`.
- Removal is visible through poll (EPOLLERR/EPOLLHUP), read (-EIO, or -ENODEV at entry) and write (-ENODEV).
- `hidraw_is_revoked` shows the HIDIOCREVOKE path is present.

## Evidence tier
1 (source code).

## Performance
Writes allocate and copy per call; reads come from a per-open queue. No numbers.

## Correctness
- Proven for hidraw.c.
- Not read: the lower usbhid layer. Whether it pads a short report before the interrupt transfer is unverified.
- The Lamzu capture of a same-vendor device (17 bytes on the wire for report 8) is Windows-side, so it gives no Linux confirmation.

## Relevance to hyperpace
- hidapi-rs linux-native `read_timeout` maps POLLERR/POLLHUP to "device disconnected". The reader loop should treat that error as removal, close the fd, and hand off to the hotplug watcher.
- Writing exactly 17 bytes for report 8 on Linux is correct.
