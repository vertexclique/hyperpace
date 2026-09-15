URL: https://github.com/libusb/hidapi/issues/589
Retrieved: 2026-09-15
Source type: GitHub issue opened by a HIDAPI collaborator (mcuee); OPEN at retrieval.
Retrieval method: `gh issue view 589 -R libusb/hidapi --json ...` (verbatim, body truncated at 1800 chars).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

`hidapi documentation about number of bytes written and read | mcuee | 2023-06-16T12:01:40Z | OPEN`

> Discusison here:
> * https://github.com/libusb/hidapi/issues/478#issuecomment-1592131366
> * https://github.com/libusb/hidapi/issues/478#issuecomment-1592887588
>
> As @todbot mentioned, this has been an issue bugging him. Same for me.
>
> Testing device: Circuit Python rawhid example, no report IDs.

(The test descriptor in the body declares a single vendor collection `Usage Page (Vendor Defined 0xFF00)` with a 64-byte input and a 64-byte output report and no report IDs.)

mcuee, 2023-06-16:
> Relevant discussion before, which fixed the reported Input Report number of reading bytes
> * https://github.com/libusb/hidapi/issues/229

Note: an earlier WebFetch extraction of this page stated that Windows reports 65 bytes written versus 64 on Linux/macOS; that detail lives in the linked issue 478 comments and was not re-read verbatim here, so it is treated as unverified.
