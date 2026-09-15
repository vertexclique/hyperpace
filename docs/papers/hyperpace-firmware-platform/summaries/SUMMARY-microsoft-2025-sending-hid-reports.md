# SUMMARY: Microsoft, Sending HID Reports

## Claim
Applications should send output reports continuously with WriteFile. HidD_SetOutputReport is only for setting state and can hang some devices. A write to a collection is sized to that collection's OutputReportByteLength.

## Method
Official conceptual documentation.

## Result
- Continuous command traffic belongs on WriteFile, which uses the interrupt OUT path.
- Write length = the collection's OutputReportByteLength. The page states this explicitly only for the kernel IRP_MJ_WRITE path.
- A user-mode WriteFile on a HID handle becomes that same kernel write request. That link is an inference, not something the page says.

## Evidence tier
1 (official documentation).

## Performance
No numbers given. WriteFile supports overlapped I/O, so a dedicated device thread can bound each write with a timeout. C hidapi and hidapi-rs windows-native both wait up to 1000 ms.

## Correctness
- Proven: the guidance (WriteFile for continuous traffic, HidD_Set routines for state), and the length rule on the kernel path.
- Inferred, not stated for user mode: that a WriteFile buffer shorter than OutputReportByteLength fails. The support for this is outside Microsoft docs: hidapi source comments, and signal11/hidapi issue 218 ("Removing the above code ... results in an error from WriteFile()").

## Relevance to hyperpace
The report-8 command channel carries sporadic requests plus a battery poll about every 5 s. It should use hidapi `write`, which calls WriteFile and pads to the opened TLC's OutputReportByteLength.
