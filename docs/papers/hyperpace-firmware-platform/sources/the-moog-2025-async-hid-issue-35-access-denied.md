URL: https://github.com/sidit77/async-hid/issues/35
Retrieved: 2026-09-15
Source type: GitHub issue with maintainer (sidit77) comment; OPEN at retrieval.
Retrieval method: `gh issue view 35 -R sidit77/async-hid --json ...` (verbatim).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

`Access is denied | the-moog | 2025-12-17T09:40:48Z | OPEN`

> I'm trying to run your example 'open_device.rs' with a USB HID barcode scanner under win11 (effectively a keyboard).

> ```text
> Backend error: Access is denied. (0x80070005) at C:\Users\the-moog\work\async-hid\src\backend\win32\device.rs:42:20
> ```

> ```text
>   Handheld Barcode Scanner
>     id: UncPath("\\?\HID#VID_05F9&PID_220D#6&67cec54&0&0000#{4d1e55b2-f16f-11cf-88cb-001111000030}\KBD")
>     vid/pid/usage/page: 0x5F9 0x220D 0x6 0x1
> ```

sidit77, 2025-12-18:
> This is likely the issue. If you look at this list you'll see that Windows will open keyboards in exclusive mode for system use. In practice this means that you can't open keyboards with read or write permissions.

the-moog, 2025-12-19:
> The odd thing is that I've used the same code to capture my mouse and that works fine, despite the mouse being 'exclusive' in the list.
