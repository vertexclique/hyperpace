URL: https://github.com/libusb/hidapi/issues/503
Retrieved: 2026-09-15
Source type: GitHub issue with maintainer (Youw) comments; CLOSED 2023-02-10.
Retrieval method: `gh issue view 503 -R libusb/hidapi --json ...` (verbatim, truncated).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

`macOS: IOHIDDeviceSetReport failed: (0xE00002BC) (iokit/common) general error | benma | 2023-02-06T23:54:15Z | CLOSED closed=2023-02-10T16:44:12Z`

> I communicate to a HID device and get this error every once in a while, in around 5% of the messages sent to the device. The messages are all the same and work most of the time.

> When I simply override this to ignore this specific error, everything works normally. The device does receive the data and responds to it normally.

> The length of the data sent is 64 bytes, but the same issue appears for smaller sizes.

> ```
> IOHIDLibUserClient:0x100002065 setReport failed: 0xe00002bc
> ```

Youw, 2023-02-08:
> it sounds like there is no actual error, since the device responds the command just fine
>
> One more thing to note: can you confirm there is no multithreading involved in the code related to HIDAPI?

benma, 2023-02-08:
> There is no multithreading involved.

(The retrieved comments do not contain a root cause; the issue was closed without one in the retrieved portion.)
