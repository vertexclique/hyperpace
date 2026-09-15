# SUMMARY: Microsoft, Troubleshooting HID Reports

## Claim
- A collection can return any of its reports in any order.
- The class driver keeps a per-handle input ring buffer of 32 reports by default.
- If the application reads more slowly than the device sends, reports are lost to overflow.

## Method
Official troubleshooting documentation.

## Result
- If the report-8 TLC declares other input reports as well (9, or 2/3/5/11/16 if they share the TLC), the reader must dispatch on the report-ID byte.
- A stalled reader thread (UI busy, host asleep) can overflow the buffer and lose device-pushed notifications.

## Evidence tier
1 (official documentation).

## Performance
- Default depth is 32 reports; C hidapi, hidapi-rs windows-native and async-hid win32 set 64 (source code, tier 1).
- The maximum is 512 (SUMMARY-microsoft-2019-hidd-setnuminputbuffers).

## Correctness
- Proven: the default of 32 and loss on overflow.
- Not stated: which report is dropped (oldest or newest).

## Relevance to hyperpace
- Run the reader as a dedicated loop that drains continuously and never waits on UI work.
- After reconnect or a long stall, stale queued reports may be present. Matching must discard unmatched responses and not assume the next report answers the last request.
