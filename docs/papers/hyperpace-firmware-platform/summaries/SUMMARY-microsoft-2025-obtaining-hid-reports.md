# SUMMARY: Microsoft, Obtaining HID Reports

## Claim
- Continuous input comes from ReadFile on the TLC handle, backed by the class driver's ring buffer.
- HidD_GetInputReport is for occasional state reads; used continuously it can lose reports or hang devices.
- A read buffer must be at least InputReportByteLength, and reports arrive in the order received.

## Method
Official conceptual documentation.

## Result
- Report-8 responses and device-pushed report-8 notifications arrive through ReadFile, in order, from the ring buffer.
- The read buffer must be at least the report-8 TLC's InputReportByteLength: 17, or 49 if report 9 shares the TLC. hidapi allocates this internally.

## Evidence tier
1 (official documentation).

## Performance
Order is preserved. Buffer depth: default 32 reports; C hidapi, hidapi-rs windows-native and async-hid win32 all set 64 (see SUMMARY-microsoft-2025-troubleshooting-hid-reports).

## Correctness
- Proven: the ReadFile semantics, the ordering, and the GetInputReport warning.
- The buffer-length rule is stated for the kernel IRP path. That user-mode ReadFile maps to it is an inference.

## Relevance to hyperpace
- Do not implement battery polling with HidD_GetInputReport (hidapi `get_input_report`). Send a report-8 request and wait for the matching input report on the reader loop.
- Responses and notifications share report 8 and arrive in order. The matcher must tell a response from a push notification by content (for example the command byte), not by timing.
