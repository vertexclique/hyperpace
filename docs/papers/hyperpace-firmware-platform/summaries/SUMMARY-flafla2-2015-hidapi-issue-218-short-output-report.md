# SUMMARY: signal11 HIDAPI issue 218, sending less than OutputReportByteLength on Windows

## Claim
- On Windows, WriteFile rejects a HID output buffer shorter than OutputReportByteLength. Removing HIDAPI's padding makes WriteFile fail.
- A device that needs exact-length packets (a Wiimote over the Microsoft Bluetooth stack) worked with HidD_SetOutputReport instead.
- signal11 notes that HidD_SetOutputReport uses the control endpoint, while WriteFile uses the OUT endpoint when one exists.

## Method
Issue thread; user experiment; original author's comment.

## Result
- A shorter WriteFile buffer fails (user-observed).
- HidD_SetOutputReport is a transport-level alternative (control endpoint).

## Evidence tier
2 (original author comment), and 3 for the user experiment.

## Performance
Not measured. Control-endpoint transfers are generally slower than interrupt OUT, but that is an assumption.

## Correctness
- The failure of short buffers is user-reported, not tied to a documented error code.
- The Wiimote case is Bluetooth-stack-specific. Its relevance to a USB-wired or 2.4 GHz receiver device is low.

## Relevance to hyperpace
- Always let the library pad to the TLC's OutputReportByteLength.
- If a HYPACE command is ever rejected because of padding (only possible if report 9 shares the TLC and padding reaches the wire), try HidD_SetOutputReport, which hidapi exposes as `send_output_report`. With hidapi-rs windows-native that path is the probably-buggy one, so use the C backend or a fixed version.
