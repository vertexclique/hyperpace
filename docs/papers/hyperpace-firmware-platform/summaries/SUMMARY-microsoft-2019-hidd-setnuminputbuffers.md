# SUMMARY: Microsoft, HidD_SetNumInputBuffers

## Claim
Each TLC's input ring buffer holds 32 reports by default. It can be set anywhere from 2 to 512 on Windows XP and later.

## Method
Official API reference, cross-checked against the C hidapi and hidapi-rs source.

## Result
- Both hidapi Windows backends give every opened TLC handle a 64-report buffer.
- At one notification per second that is over a minute of slack. For bursts of report-8 notifications it is probably enough, but that is not measured.

## Evidence tier
1 (official documentation plus source code).

## Performance
Memory is 64 x InputReportByteLength per handle, under 4 KB even for 49-byte reports.

## Correctness
- Proven: the limits, the default, and the value hidapi sets.
- Unknown: HYPACE's notification rate, which only a hardware capture reveals.

## Relevance to hyperpace
- Nothing to do if hidapi (C or windows-native) or async-hid is used.
- A custom Windows backend must call HidD_SetNumInputBuffers itself, or accept 32.
