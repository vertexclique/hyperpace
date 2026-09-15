URL: https://learn.microsoft.com/en-us/windows-hardware/drivers/hid/obtaining-hid-reports
Retrieved: 2026-09-15
Source type: Official conceptual documentation (Microsoft Learn, HID drivers). Page metadata: ms.date 2024-09-12, updated_at 2025-03-25.
Retrieval method: WebFetch (page converted to markdown; text below is the returned page text).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

> However, an application should only use the **HidD_Get**Xxx routines to obtain the current state of a device. If an application attempts to use HidD_GetInputReport to continuously obtain input reports, the reports can be lost. In addition, some devices don't support **HidD_GetInputReport** and become unresponsive if this routine is used.

Section "Using ReadFile":
> An application uses the open file handle it obtained by using **CreateFile** to open a file on the collection. When the application calls **ReadFile**, it doesn't have to specify overlapped I/O because the HID Client Drivers buffers reports in a ring buffer. However, an application can use overlapped I/O to have more than one outstanding read request.

Section "Using HidD_GetXxx Routines":
> To retrieve a specific report using HidD_GetXxx routines, the application allocates the report output buffer, zero-initializes the buffer, and sets the first byte in the buffer to the specific report ID.

Kernel-mode read length (IRP_MJ_READ):
> - **Parameters.Read.Length**: Set to the size, in bytes, of the read buffer. The size must be greater than or equal to the value specified by the InputReportByteLength member of a HID collection's HIDP_CAPS structure.

> Consecutive read requests return input reports in the order in which they were received from the collection.
