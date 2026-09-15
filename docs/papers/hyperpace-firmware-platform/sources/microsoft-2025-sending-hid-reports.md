URL: https://learn.microsoft.com/en-us/windows-hardware/drivers/hid/sending-hid-reports
Retrieved: 2026-09-15
Source type: Official conceptual documentation (Microsoft Learn, HID drivers). Page metadata: ms.date 2024-09-17, updated_at 2025-03-25.
Retrieval method: WebFetch (page converted to markdown; text below is the returned page text).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

Section "Send HID reports by user-mode applications":
> User-mode applications use WriteFile to continuously send output reports to a HID collection. You can also use **HidD_SetXxx** routines to send output reports and feature reports to a collection. However, only use **HidD_SetXxx** routines to set the current state of a collection. Some devices might not support HidD_SetOutputReport and become unresponsive if this routine is used.

Section "WriteFile routine":
> Use write requests to send output reports to a HID collection. After a user-mode application creates an output report, it can send an output report to a collection using WriteFile.

Section "IRP_MJ_WRITE requests" (kernel-mode, describes the length the class driver expects for a write):
> - **Parameters.Write.Length**: Set to the length, in bytes, of an output report. Set this member to the length of a HID collection's output reports, as specified by the *OutputReportByteLength* member of a collection's **HIDP_CAPS** structure.
