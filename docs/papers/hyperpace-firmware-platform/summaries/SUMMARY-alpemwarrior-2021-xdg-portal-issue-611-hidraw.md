# SUMMARY: xdg-desktop-portal issue 611, no portal for hidraw nodes

## Claim
Configuration tools (mice, keyboards, RGB) need hidraw access from sandboxes. A hidraw portal was requested in 2021 and was still open on 2026-09-15. Progress went to kernel HIDIOCREVOKE (6.12) and logind support instead. The general USB portal (added in xdg-desktop-portal 1.19.1) works through libusb, so HIDAPI users would have to go through libhidapi-usb's `hid_libusb_wrap_sys_device` (search snippet, unverified).

## Method
GitHub issue thread plus release metadata. The USB portal doc path returned 404.

## Result
- No hidraw portal exists.
- The USB portal is a libusb-level route, which on Linux would mean hidapi's libusb backend or nusb with detach of the kernel HID driver: unsuitable for a mouse that must keep working.
- Latest xdg-desktop-portal release: 1.22.1 (2026-06-17).

## Evidence tier
2 (freedesktop developer comments), plus 3 for the USB portal snippet.

## Performance
Not applicable.

## Correctness
- Proven: the issue is open.
- USB portal details unverified: its doc moved and was not re-read. Whether the portal serves non-Flatpak apps was not checked.

## Relevance to hyperpace
- A Flatpak distribution would not solve hidraw access either.
- AppImage or native packages plus a udev rule remain the Linux path.
