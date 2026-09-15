URL: https://github.com/OpenMouse-Project/mouse-protocol/blob/main/docs/razer-testing.md
Retrieved: 2026-09-15
Source type: Community hardware test checklist with macOS findings on real devices.
Retrieval method: `git clone --depth 1 https://github.com/OpenMouse-Project/mouse-protocol` (HEAD a915bb3); file read verbatim.
Character note: the Unicode arrow characters in the original were replaced with "->"; any em-dash or en-dash was replaced with an ASCII hyphen.

# Excerpts

Lines 1-11:
> Test in Chrome or Edge over HTTPS. Quit Razer Synapse first - it holds the
> control interface open and reads then time out.
>
> On macOS the browser itself must be granted Input Monitoring permission
> (System Settings -> Privacy & Security -> Input Monitoring) **before** the first
> connection, and then quit and reopened. Razer's control interface is a Generic
> Desktop Mouse collection, which macOS reserves for its own input stack; without
> the permission the browser's `IOHIDDeviceOpen` call is refused and the app
> shows "Failed to open the device." while the device still appears in the
> picker. This is a system grant, not an app setting.

Lines 555-565:
> ## DeathAdder V2 on macOS
>
> Confirmed on macOS: the mouse enumerates as four HID interfaces, the
> configuration channel sits on the Generic Desktop Mouse interface, and the
> browser is refused from opening it unless it holds the Input Monitoring
> permission. The device still shows up in the picker, so the failure looks like
> a driver bug: the sidebar lists the mouse as available, and connecting fails
> with `NotAllowedError: Failed to open the device.` before any feature report is
> exchanged - no Synapse installed. Granting the browser Input Monitoring
> (System Settings -> Privacy & Security -> Input Monitoring) and restarting it is
> the fix; the app now says exactly that when the open is refused on macOS.
