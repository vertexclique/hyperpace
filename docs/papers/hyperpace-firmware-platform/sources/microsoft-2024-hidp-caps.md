URL: https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/hidpi/ns-hidpi-_hidp_caps
Retrieved: 2026-09-15
Source type: Official API reference (Microsoft Learn, WDK DDI reference). Page metadata: ms.date 2022-04-28, updated_at 2024-02-22.
Retrieval method: WebFetch (page converted to markdown; text below is the returned page text).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

> The HIDP_CAPS structure contains information about a top-level collection's capability.

Member `Usage`:
> Specifies a top-level collection's usage ID.

Member `UsagePage`:
> Specifies the top-level collection's usage page.

Member `InputReportByteLength`:
> Specifies the maximum size, in bytes, of all the input reports. Includes the report ID, which is prepended to the report data. If report ID is not used, the ID value is zero.

Member `OutputReportByteLength`:
> Specifies the maximum size, in bytes, of all the output reports. Includes the report ID, which is prepended to the report data. If report ID is not used, the ID value is zero.

Member `FeatureReportByteLength`:
> Specifies the maximum length, in bytes, of all the feature reports. Includes the report ID, which is prepended to the report data. If report ID is not used, the ID value is zero.

Remarks:
> Callers of the HIDClass support routines use the information provided in this structure when a called routine requires, as input, the size of a report type, the number of link collection nodes, the number of control capabilities, or the number of data indices.
