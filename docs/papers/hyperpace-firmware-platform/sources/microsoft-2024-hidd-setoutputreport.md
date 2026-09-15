URL: https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/hidsdi/nf-hidsdi-hidd_setoutputreport
Retrieved: 2026-09-15
Source type: Official API reference (Microsoft Learn, WDK DDI reference). Page metadata: ms.date 2022-04-28, updated_at 2024-02-22.
Retrieval method: WebFetch (page converted to markdown; text below is the returned page text).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

Description:
> The **HidD_SetOutputReport** routine sends an output report to a top-level collection.
>
> Only use this routine to set the current state of a collection. Some devices might not support this routine and will become unresponsive if this routine is used.

Parameter `ReportBuffer`:
> Pointer to a caller-allocated output report buffer that the caller uses to specify a report ID.

Parameter `ReportBufferLength`:
> The size of the report buffer in bytes. The report buffer must be large enough to hold the output report plus one additional byte that specifies a nonzero report ID. If report ID is not used, the ID value is zero.

Remarks:
> The correct *ReportBufferLength* is specified by the *OutputReportByteLength* member of a top-level collection's HIDP_CAPS structure returned from HidP_GetCaps call.

> - If the top-level collection includes report IDs, the caller must set the first byte of the *ReportBuffer* parameter to a nonzero report ID.
> - If the top-level collection does not include report IDs, the caller must set the first byte of the *ReportBuffer* parameter to zero.

> Only user-mode applications can call **HidD_SetOutputReport**. Kernel-mode drivers can use an IOCTL_HID_SET_OUTPUT_REPORT request.

Requirements:
> Minimum supported client: Available in Windows XP and later versions of Windows.
