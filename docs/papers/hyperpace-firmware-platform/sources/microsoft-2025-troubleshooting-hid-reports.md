URL: https://learn.microsoft.com/en-us/windows-hardware/drivers/hid/troubleshooting-hid-reports
Retrieved: 2026-09-15
Source type: Official troubleshooting documentation (Microsoft Learn, HID drivers). Page metadata: ms.date 2024-01-11, updated_at 2025-03-25.
Retrieval method: WebFetch (page converted to markdown; text below is the returned page text).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

Section "HID report ID errors":
> When an application or driver receives a HID report from a HID collection, it can be any report that the collection contains (because a collection can return reports in any order).

> HIDP_STATUS_INCOMPATIBLE_REPORT_ID A requested usage is in a report supported by the HID collection, but not in the report that the application or driver specified.
>
> HIDP_STATUS_USAGE_NOT_FOUND A requested usage is not in any report supported by the top-level collection.

Section "Dropped HID reports":
> When the HID Client Drivers obtains input reports from a HID collection, the reports are stored in a ring buffer maintained by the HID class driver. This mechanism reduces the possibility that an application or driver will miss input reports that it requires.

> By default, the HID class driver maintains an input report ring buffer that holds 32 reports. If a collection transmits data to the HID class driver faster than a user-mode application or kernel-mode driver retrieves it from the buffer, input reports are lost because of buffer overflow. To reduce the possibility of buffer overflow, an application or driver can reconfigure the size, in number of reports, of the buffer. Drivers retrieve and change the size of the buffer by using an IOCTL_GET_NUM_DEVICE_INPUT_BUFFERS request and an IOCTL_SET_NUM_DEVICE_INPUT_BUFFERS request. Applications do the same operation by calling HidD_GetNumInputBuffers and HidD_SetNumInputBuffers.
