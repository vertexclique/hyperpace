URL: https://github.com/sidit77/async-hid (HEAD 1d34435, 2026-06-11) ; https://crates.io/crates/async-hid
Retrieved: 2026-09-15
Source type: Source code (git clone), README, crates.io registry API (raw JSON via curl), GitHub REST API.
Retrieval method: `git clone https://github.com/sidit77/async-hid`; `curl https://crates.io/api/v1/crates/async-hid`; `gh api repos/sidit77/async-hid`; `gh issue list` / `gh pr list`.
Character note: emoji check marks in the README table are rendered as "yes"/"no" below; any em-dash or en-dash was replaced with an ASCII hyphen.

# Registry and activity

crates.io API (raw):
```
max_version 0.5.3 newest 0.5.3 updated_at 2026-06-11T08:02:20.952875Z downloads 51297 repo https://github.com/sidit77/async-hid
  0.5.3 2026-06-11T08:02:20.952875Z
  0.5.2 2026-06-10T09:41:41.280821Z
  0.5.1 2026-03-16T10:32:43.868134Z
  0.5.0 2026-02-09T11:32:25.826060Z
  0.4.4 2025-06-28T12:42:57.577369Z
  0.4.3 2025-06-16T16:53:11.767510Z
```
GitHub API: `open_issues_count=9 pushed_at=2026-06-11T08:02:07Z stars=30`. Single primary maintainer (sidit77); recent merged PRs from outside contributors (#42, #43 on 2026-06-10). Maintainer comment on PR 48, 2026-09-06: "Thanks for the PR. I am going to look at it once I am back from vacation. Sorry for the delay."

Open issues (selected): `45 macOS: SetReportWithCallback stops input report delivery after a few hundred writes` (2026-08-17); `44 Would you accept a FreeBSD backend?`; `35 Access is denied`; `19 Concurrent writes on windows`; `17 Expose raw report descriptor`. Open PRs: `48 fix(win32): recover from interrupted HID writes` (2026-09-05); `46 Do not use SetReportWithCallback on macOS` (2026-08-18).

# README

> This crate aims to be a replacement for hidapi-rs without the baggage that comes from being a wrapper around a C library.

| Operating System | Underlying API |
|---|---|
| Windows | Win32 (`Windows.Win32.Devices`) |
| Windows | WinRT (`Windows.Devices.HumanInterfaceDevice`) |
| Linux | hidraw |
| MacOs | IOHIDManager |

Async table (README): enumerate is async only on WinRT; open is async on WinRT and macOS; read_input_report and write_output_report are async on all backends.

> Under Linux this crate uses either `async-io` (default) or `tokio` feature for the async functionality.

Note: PR 48 reports "the existing top-level documentation example ... still imports the removed `AccessMode` API and calls the removed `DeviceInfo::enumerate`" (README example is stale relative to 0.5.x).

# Cargo.toml (0.5.3)

```toml
[features]
default = ["async-io", "win32"]
tokio = ["dep:tokio"]
async-io = ["dep:async-io"]
win32 = ["dep:atomic-waker", "dep:crossbeam-queue"]
winrt = ["dep:windows-future", "dep:once_cell", "dep:async-channel"]

[target."cfg(target_os = \"linux\")".dependencies]
tokio = { version = "1", features = ["net"], optional = true }
async-io = { version = "2", optional = true }
nix = { version = "0.29.0", features = ["fs", "ioctl", "socket"] }

[target."cfg(target_os = \"macos\")".dependencies]
objc2-io-kit = "0.3.2"
objc2-core-foundation = "0.3.2"
dispatch2 = "0.3.0"
```
(No libudev, no C compiler dependency.)

# src/device_info.rs (public API)

```rust
pub enum DeviceEvent {
    Connected(DeviceId),
    Disconnected(DeviceId)
}
```
```rust
/// Listen for device connect/disconnect events
///
/// For "connect" events the returned id can be turned into a list of new devices using [self.query_devices]
pub fn watch(&self) -> HidResult<impl Stream<Item = DeviceEvent> + Send + Unpin> {
```
```rust
pub struct DeviceInfo {
    pub id: DeviceId,
    pub name: String,
    pub manufacturer: Option<String>,
    pub product_id: u16,
    pub vendor_id: u16,
    pub usage_id: u16,
    pub usage_page: u16,
    pub serial_number: Option<String>
}
assert_impl_all!(DeviceInfo: Send, Sync, Unpin);
```
```rust
/// Open the device in read and write mode
pub async fn open(&self) -> HidResult<DeviceReaderWriter> {
```

# Linux backend (src/backend/hidraw/mod.rs)

```rust
async fn enumerate(&self) -> HidResult<DeviceInfoStream> {
    let devices = read_dir("/sys/class/hidraw/")?
```
```rust
fn watch(&self) -> HidResult<Boxed<DeviceEvent>> {
    const MONITOR_GROUP_KERNEL: u32 = 1;
    const MONITOR_GROUP_UDEV: u32 = 2;

    let socket = socket(
        AddressFamily::Netlink,
        SockType::Datagram,
        SockFlag::SOCK_CLOEXEC | SockFlag::SOCK_NONBLOCK,
        SockProtocol::NetlinkKObjectUEvent
    )?;
    let group = match access("/run/udev/control", AccessFlags::F_OK) {
        Ok(_) => {
            trace!("Udev deamon seems to be running, binding to udev monitor group");
            MONITOR_GROUP_UDEV
        }
```
```rust
if event.subsystem != "hidraw" {
    continue;
}
```

# Windows backend (src/backend/win32)

`mod.rs`:
```rust
if read {
    check_error(unsafe { HidD_SetNumInputBuffers(device.handle(), 64) })?;
}
let caps = device.preparsed_data()?.caps()?;
...
true => Some(IoBuffer::<Writable>::new(device.clone(), caps.OutputReportByteLength as usize)?),
```
```rust
usage_id: caps.Usage,
usage_page: caps.UsagePage,
```
`interface.rs`:
```rust
let filter = CM_NOTIFY_FILTER {
    cbSize: size_of::<CM_NOTIFY_FILTER>() as u32,
    Flags: 0,
    FilterType: CM_NOTIFY_FILTER_TYPE_DEVICEINTERFACE,
    ...
            ClassGuid: *Interface::guid()
```
```rust
CM_NOTIFY_ACTION_DEVICEINTERFACEARRIVAL => Some(DeviceEvent::Connected(DeviceId::UncPath(device_id))),
CM_NOTIFY_ACTION_DEVICEINTERFACEREMOVAL => Some(DeviceEvent::Disconnected(DeviceId::UncPath(device_id))),
```
`buffer.rs` write: `self.buffer[data_size..].fill(0);` / `self.buffer[..data_size].copy_from_slice(&data[..data_size]);` (pads to the OutputReportByteLength-sized buffer).

# macOS backend (src/backend/iohidmanager/mod.rs)

```rust
static DISPATCH_QUEUE: LazyLock<DispatchRetained<DispatchQueue>> = LazyLock::new(|| DispatchQueue::new("async-hid", DispatchQueueAttr::SERIAL));
```
```rust
//SAFETY: IOHIDManager is immediately connected to a dispatch queue, and
// all functions are called on that queue to the best of my knowledge
unsafe impl Send for IoHidManagerBackendInner {}
unsafe impl Sync for IoHidManagerBackendInner {}
```
```rust
let manager = IOHIDManager::new(None, IOHIDManagerOptions::None.bits());
manager.set_dispatch_queue(&DISPATCH_QUEUE);
...
manager.register_device_matching_callback(Some(ManagerCallbackContext::added_callback), context as *mut c_void);
manager.register_device_removal_callback(Some(ManagerCallbackContext::removed_callback), context as *mut c_void);
trace!("Scheduling manager with run loop");
manager.activate();
```
