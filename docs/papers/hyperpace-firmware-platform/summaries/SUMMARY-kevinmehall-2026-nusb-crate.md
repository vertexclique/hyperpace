# SUMMARY: nusb 0.2.7

## Claim
nusb is a pure-Rust, async-first USB library with device listing and `watch_devices()` hotplug on Linux, Windows, macOS and WebUSB:
- Linux: needs write access to /dev/bus/usb nodes.
- Windows: only works on interfaces bound to WinUSB.
- macOS: cannot access devices that have a kernel driver.
- It is intended for vendor-class devices; for standard classes like HID, the docs point to the OS class driver.

## Method
Read the source and crate docs (clone at 58b056f), crates.io JSON, GitHub metadata, and issues searched for HID.

## Result
- **Activity:** 0.2.2 (2026-02-17) through 0.2.7 (2026-08-03), MSRV 1.85, 1.43 M downloads, 20 open issues, a Win7 hotplug PR merged 2026-09-10.
- **Hotplug implementations:**
  - Linux: netlink uevent on the udev multicast group, without libudev.
  - Windows: CM_Register_Notification on GUID_DEVINTERFACE_USB_DEVICE (plus a win7 path).
  - macOS: IOServiceAddMatchingNotification on nusb's own event-thread run loop.
- The HotplugWatch type is Send + Sync.
- Windows note: composite device interfaces may not be ready at Connected, so retry after a delay.
- No HID-related issues exist.

## Evidence tier
1 (source code and crate documentation).

## Performance
- Async transfers on a global event thread; no numbers for hotplug.

## Correctness
- Proven: platform constraints and mechanisms.
- nusb cannot send HID reports to HYPACE's HID interfaces:
  - Windows binds them to hidclass;
  - macOS has a kernel HID driver attached;
  - Linux would require detaching usbhid, which breaks the mouse.
- That conclusion is inferred from the documented constraints; no experiment was run.

## Relevance to hyperpace
- **Hotplug watcher (good fit).** It sees USB device arrival and removal of 0x3554:FB16/FB14 on all three OSes. After a Connected event, re-enumerate HID paths, retrying on Windows.
- **Bootloader transport (candidate).** If the firmware update mode re-enumerates as a vendor-class bootloader, nusb fits. On Windows that needs WinUSB bound (WCID descriptor, or Zadig, which is a user burden). On Linux it needs a usb-subsystem udev rule.
- Not a fit for the report-8 channel.
