# SUMMARY: Microsoft, HidD_SetOutputReport reference

## Claim
HidD_SetOutputReport sends one output report to a top-level collection (TLC). Byte 0 of the buffer is the report ID, which must be nonzero when the TLC uses IDs. The correct buffer length is the TLC's `HIDP_CAPS.OutputReportByteLength`. Microsoft says to use it only to set current state, because some devices stop responding when it is used.

## Method
Official WDK API reference page.

## Result
- Length is set per TLC, not per report: buffer length = OutputReportByteLength of the opened TLC.
- The report ID goes in byte 0.
- The call is meant for occasional state setting, not for a continuous command stream.

## Evidence tier
1 (official documentation).

## Performance
The page gives no numbers. A tier-2 comment from the original hidapi author (signal11, in signal11/hidapi issue 218) says it uses the control endpoint, while WriteFile uses the interrupt OUT endpoint. The latency difference has not been measured for this device.

## Correctness
- Proven (documented): the length comes from OutputReportByteLength, and the report ID is byte 0.
- Not stated by Microsoft: what happens with a shorter or longer buffer. The C hidapi source comment says a shorter buffer fails with ERROR_INVALID_PARAMETER and a longer one is silently truncated. That is tier-1 code commentary, not Microsoft documentation.
- "Some devices ... become unresponsive" is a warning, not a device list. Whether HYPACE firmware accepts SET_REPORT(Output) over the control endpoint is unknown.

## Relevance to hyperpace
- Send report 8 with WriteFile (hidapi `write`), not `send_output_report`. HidD_SetOutputReport is only a fallback and must be tested on hardware first.
- A Windows write of report 8 must match the OutputReportByteLength of the TLC that carries report 8. That is 17 bytes if report 8 is the largest output report in that TLC, and 49 bytes if report 9 (48 B) shares it. hidapi pads short buffers automatically.
