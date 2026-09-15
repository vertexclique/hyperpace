# SUMMARY: async-hid PR 48, recovering from interrupted HID writes on Win32

## Claim
When a device disappears during a pending overlapped write, GetOverlappedResult returns ERROR_NO_SUCH_DEVICE (0x800701B1). A failing receiver access can also return ERROR_GEN_FAILURE (0x8007001F). async-hid 0.5.3 did not clear its pending flag, swallowed the error, and hit an "I/O operation already pending" assertion on the next write. The PR fixes the state machine and maps ERROR_NO_SUCH_DEVICE to Disconnected.

## Method
Code change with unit tests and cross-compiled lint, motivated by OpenLogi hardware reports. The author states it was not validated on Windows hardware.

## Result
- Unplugging the receiver during a write is a real, reported failure path on Windows.
- The fix was unmerged on 2026-09-15; the maintainer acknowledged it.

## Evidence tier
3 (contributor PR, unvalidated on hardware, with linked user reports).

## Performance
Not applicable.

## Correctness
- The error codes come from real reports.
- The fix is untested on hardware.

## Relevance to hyperpace
- The Windows removal path must handle ERROR_NO_SUCH_DEVICE, ERROR_DEVICE_NOT_CONNECTED and ERROR_GEN_FAILURE as terminal: close the handle, mark the device offline, and let the watcher reopen it.
- Include an "unplug the receiver during a write" case in the test plan.
