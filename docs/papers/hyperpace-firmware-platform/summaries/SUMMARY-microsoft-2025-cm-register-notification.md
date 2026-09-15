# SUMMARY: Microsoft, CM_Register_Notification

## Claim
- CM_Register_Notification (Windows 8+) delivers PnP arrival and removal callbacks for device interfaces, device handles, or device instances, with no window or message loop.
- It does not report devices that are already present. The documented pattern is to register first, then list, and tolerate duplicates.
- Callbacks must return quickly.

## Method
Official Win32 API reference.

## Result
- Windows hotplug without polling and without a window handle: register a DEVICEINTERFACE filter on the HID interface class GUID, then list the existing interfaces.
- A device arriving between the two calls appears twice, so the device registry must be keyed by path and be idempotent.
- No blocking work in the callback; forward events to a channel.

## Evidence tier
1 (official documentation).

## Performance
Event-driven, so no polling cost. The callback's thread is system-owned. The "start another thread" advice implies that, but the page does not name the thread.

## Correctness
- Proven: the ordering race, duplicate delivery, the quick-return rule, and the Windows 8 minimum.
- async-hid's win32 backend registers a DEVICEINTERFACE filter on the HID interface GUID.
- nusb's Windows hotplug uses the same API with GUID_DEVINTERFACE_USB_DEVICE, and nusb also has a hotplug_win7.rs path (tier 1 source code).
- Neither the hidapi 0.15.0 release nor hidapi-rs 2.6.7 exposes a hotplug API.

## Relevance to hyperpace
- Windows hotplug for "receiver unplugged / cable attached" should use this API, through async-hid `watch()`, nusb `watch_devices()`, or a small windows-sys wrapper.
- The HID interface GUID reports per-TLC HID paths directly. The USB-device GUID reports the composite USB device and needs a HID re-enumeration afterwards.
- Windows 7 support would need RegisterDeviceNotification, which is an operator scope decision.
