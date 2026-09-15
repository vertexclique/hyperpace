URL: https://github.com/kevinmehall/nusb (HEAD 58b056f, 2026-09-10) ; https://crates.io/crates/nusb ; https://docs.rs/nusb
Retrieved: 2026-09-15
Source type: Source code and crate-level documentation (git clone), crates.io registry API (raw JSON), GitHub REST API.
Retrieval method: `git clone https://github.com/kevinmehall/nusb`; `curl https://crates.io/api/v1/crates/nusb`; `gh api repos/kevinmehall/nusb`; `gh search issues --repo kevinmehall/nusb HID`.
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Registry and activity

```
max_version 0.2.7 newest 0.2.7 updated_at 2026-08-03T16:05:05.577806Z downloads 1425644 repo https://github.com/kevinmehall/nusb
  0.2.7 2026-08-03T16:05:05.577806Z  rust_version 1.85
  0.2.6 2026-08-01T17:52:39.970834Z  rust_version 1.85
  0.2.5 2026-07-16T19:16:37.279254Z  rust_version 1.79
  0.2.4 2026-06-21T17:39:26.411922Z  rust_version 1.79
  0.2.3 2026-03-10T18:42:31.436697Z  rust_version 1.79
  0.2.2 2026-02-17T20:46:46.307318Z  rust_version 1.79
```
GitHub API: `nusb open_issues_count=20 pushed_at=2026-09-10T22:06:12Z`.
git log: `58b056f 2026-09-10 Kevin Mehall Merge pull request #231 from whitequark/win7`; `6d6dfac 2026-09-10 windows: rename back to hotplug.rs so as to not imply intended support for Win8`.

# README

> `nusb` supports Windows, macOS, Linux, and WebUSB and provides both async and
> blocking APIs for listing and watching USB devices, reading descriptor
> details, opening and managing devices and interfaces, and performing
> transfers on control, bulk, and interrupt endpoints.

> * Pure Rust, no dependency on libusb or any other C library.
> * Async-first, while not requiring an async runtime.
> * No context object. You just open a device. There is a global event loop thread
>   that is started when opening the first device.

# src/lib.rs crate docs (platform notes)

> class such as Mass Storage, CDC (Serial), HID, Audio, or Video, this is ... existing kernel driver instead. (On some platforms you could detach or replace the kernel driver and program the device from user-space using this

(lines 19-22; the full sentence recommends using the OS class driver API for standard classes such as HID.)

Linux:
> A user must have write access on the `/dev/bus/usb/XXX/YYY` nodes to
> successfully open a device. Use udev rules to configure these permissions.

Windows:
> `nusb` uses WinUSB on Windows.
>
> To use `nusb`, your device or interface must be associated with the `WinUSB`
> driver. If you control the device firmware, the recommended way is to use a
> WCID descriptor to tell Windows to install the WinUSB driver automatically
> when the device is first connected. Alternatively Zadig (GUI) or libwdi
> (CLI / C library) can be used to manually install the WinUSB driver for a
> device.

macOS:
> `nusb` uses IOKit on macOS.
>
> Users have access to USB devices by default, with no permission
> configuration needed. Devices with a kernel driver are not accessible.

`src/device.rs`:
> /// This function can only detach kernel drivers on Linux. Calling on other platforms has

# Hotplug (`watch_devices`)

```rust
/// Events will be returned for devices connected or disconnected beginning at
/// the time this function is called. To maintain a list of connected devices,
/// call [`list_devices`] after creating the watch with this function to avoid
/// potentially missing a newly-attached device:
...
/// ### Platform-specific notes:
///
///   * On Windows, the interfaces of a composite device might not be ready
///     when the `Connected` event is emitted. If you are immediately opening the device
///     and claiming an interface when receiving a `Connected` event,
///     you should retry after a short delay if opening or claiming fails.
pub fn watch_devices() -> Result<hotplug::HotplugWatch, Error> {
```
`src/hotplug.rs`:
```rust
pub enum HotplugEvent {
    /// A device has been connected.
    Connected(DeviceInfo),
    /// A device has been disconnected.
    Disconnected(DeviceId),
}
...
fn assert_send_sync() {
    fn require_send_sync<T: Send + Sync>() {}
    require_send_sync::<HotplugWatch>();
}
```
Linux (`src/platform/linux_usbfs/hotplug.rs`): `const UDEV_MAGIC: &[u8; 12] = b"libudev\0\xfe\xed\xca\xfe";` / `const UDEV_MULTICAST_GROUP: u32 = 1 << 1;` / `socket_with(AddressFamily::NETLINK, SocketType::RAW, SocketFlags::CLOEXEC, Some(netlink::KOBJECT_UEVENT))`.
Windows (`src/platform/windows_winusb/hotplug.rs`): `CM_Register_Notification, ... CM_NOTIFY_FILTER_TYPE_DEVICEINTERFACE ... Usb::GUID_DEVINTERFACE_USB_DEVICE`. A `hotplug_win7.rs` file also exists.
macOS (`src/platform/macos_iokit/hotplug.rs`): `IONotificationPortGetRunLoopSource, IOServiceAddMatchingNotification`; `events.rs` adds the source to its own event thread's `CFRunLoop` in `kCFRunLoopCommonModes`.

Issue search `repo:kevinmehall/nusb HID` returned no issue about opening HID-class interfaces (6 unrelated results, all closed).
