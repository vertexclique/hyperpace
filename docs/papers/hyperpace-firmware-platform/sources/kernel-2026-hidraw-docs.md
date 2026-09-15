URL: https://docs.kernel.org/hid/hidraw.html
Retrieved: 2026-09-15
Source type: Official Linux kernel documentation (rendered page reports kernel version 7.3.0-rc3).
Retrieval method: WebFetch (extraction model returned these sentences in quotation marks; wording matches Documentation/hid/hidraw.rst as commonly published).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

read():
> read() will read a queued report received from the HID device. On USB devices, the reports read using read() are the reports sent from the device on the INTERRUPT IN endpoint.

> By default, read() will block until there is a report available to be read. read() can be made non-blocking, by passing the O_NONBLOCK flag to open().

> On a device which uses numbered reports, the first byte of the returned data will be the report number; the report data follows, beginning in the second byte.

write():
> The first byte of the buffer passed to write() should be set to the report number. If the device does not use numbered reports, the first byte should be set to 0.

ioctls:
> HIDIOCGRDESCSIZE: Get Report Descriptor Size - This ioctl will get the size of the device's report descriptor.

> HIDIOCGRDESC: Get Report Descriptor - This ioctl returns the device's report descriptor using a hidraw_report_descriptor struct.

> HIDIOCSOUTPUT(len): Send an Output Report - This ioctl will send an output report to the device, using the control endpoint.

Secondary confirmation of the USB transport sentence (search-engine snippet of the same document, not re-read verbatim): "if the device has an INTERRUPT OUT endpoint, the report will be sent on that endpoint. If it does not, the report will be sent over the control endpoint, using a SET_REPORT transfer."
