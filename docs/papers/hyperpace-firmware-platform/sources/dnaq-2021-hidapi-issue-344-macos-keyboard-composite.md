URL: https://github.com/libusb/hidapi/issues/344
Retrieved: 2026-09-15
Source type: GitHub issue with maintainer (Youw) comments; CLOSED 2022-04-23.
Retrieval method: `gh issue view 344 -R libusb/hidapi --json ...` (verbatim).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

`Composite hid device that's also a keyboard on Mac OS X | dnaq | 2021-10-13T18:22:58Z | CLOSED closed=2022-04-23T11:16:42Z`

> I have an issue with implementing a HID-based protocol for a BLE device that is also a keyboard an getting it to work under Mac OS X. My guess is that the operating system refuses to give us access to the hid device due to the fact of it also being a keyboard.

Youw (maintainer), 2021-10-13:
> That is true. No workaround is possible on SW/HIDAPI side. The way HID driver is implemented on macOS, a device with multiple usage_pages/usages is represented as a single undividable device.

dnaq, 2021-10-14:
> Looking into the issue a bit more it seems like we can grab the device if we run the application under sudo and if we allow "Input Monitoring" for the application under Privacy/Input Monitoring in the security & privacy settings.
>
> The next issue is that this causes the keypresses of the device to not be handled by the operating system. I think this might be because of the `kIOHIDOptionsTypeSeizeDevice` flag when opening the device.

dnaq, 2021-10-14:
> And this seems to work beautifully and not need root permissions anymore. So just changing that flag to zero makes us able to open those devices from normal user applications if the Input Monitoring setting is enabled for our application.

Youw, 2021-10-14:
> HIDAPI has been opening the device on macOS exclusively since the very first version. And even if it is beeter not to do this way by default, by changing the default, we break the backward compativility of the programs that are _already using HIDAPI on macOS_ and _rely on such behavior_ explicitly or implicitly.
