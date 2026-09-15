# SUMMARY: signal11 HIDAPI issue 352, hid_write returns the padded length on Windows

## Claim
On Windows, hid_write returns OutputReportByteLength, not the length passed in. The original author (signal11) says this is how Windows works and only matters for devices with several numbered reports of different sizes.

## Method
Issue thread with a code walk-through and the original author's comment, cross-checked against current libusb/hidapi master.

## Result
- Writing 2 bytes to a 32-byte-report device returns 32.
- A common check like `nbytes != size` falsely reports failure on Windows only.
- Current master still returns `bytes_written` of the padded buffer.

## Evidence tier
2 (original author comment), plus 1 for the current source.

## Performance
Not applicable.

## Correctness
- Proven: the return value semantics (C backend).
- hidapi-rs windows-native behaves differently: Ok(0) when WriteFile completes synchronously, otherwise the overlapped result, which is the padded length.

## Relevance to hyperpace
- HYPACE has numbered reports of different sizes (8: 16 B, 9: 48 B). This is exactly the case signal11 names, if both share a TLC.
- Never compare the write count with the request length. Check for Ok or Err only.
