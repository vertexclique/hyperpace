URL: https://github.com/flatpak/xdg-desktop-portal/issues/611
Retrieved: 2026-09-15
Source type: GitHub issue in xdg-desktop-portal with comments from GNOME/freedesktop developers (hadess, whot); OPEN at retrieval.
Retrieval method: `gh issue view 611 -R flatpak/xdg-desktop-portal --json ...` (verbatim, last 5 comments) and `gh api repos/flatpak/xdg-desktop-portal/releases`.
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

`Access to hidraw nodes | alpemwarrior | 2021-08-08T10:53:55Z | OPEN closed=null`

> HID is often used as a general purpose protocol, beyond what Human Interface Device might suggest. In USB and bluetooth peripherals, for example, HID is often used to configure sensitivity, macro keys, lighting and other options. The software that is used to tweak these options is often found in user space.

> 2. Opening a hidraw node and writing to that. Hidraw nodes are created by the kernel and include bluetooth HID devices in addition to USB HID devices.

hadess, 2021-09-15:
> I didn't mean to discourage you from working on the Wayland protocol, just that it's not going to be enough to not require some other implementation for Flatpak/portals.

whot, 2022-04-26:
> ftr, a draft for hidraw revoke support in logind is here: ~~https://github.com/systemd/systemd/pull/23140~~ https://github.com/systemd/systemd/pull/33970

whot, 2024-09-04:
> the `HIDIOCREVOKE` ioctl is now scheduled for kernel 6.12

Latest xdg-desktop-portal releases: `1.22.1 2026-06-17T16:51:25Z`, `1.22.0 2026-06-08T13:38:19Z`.

Search-result context (not re-read verbatim; the doc path `doc/usb-portal.md` returned HTTP 404 via the GitHub API at retrieval, so it has moved): Phoronix reported "XDG-Desktop-Portal 1.19.1 Brings USB Portal"; a search snippet of the USB portal documentation said "If you use libhidapi, you need to use the function hid_libusb_wrap_sys_device() provided by libhidapi-usb."
