URL: https://github.com/ruabmbua/hidapi-rs (HEAD 8729463, 2026-08-27); https://crates.io/crates/hidapi ; https://docs.rs/hidapi/2.6.7
Retrieved: 2026-09-15
Source type: Source code (git clone), crates.io registry API (raw JSON via curl), docs.rs rendered API page, GitHub REST API (gh).
Retrieval method: `git clone https://github.com/ruabmbua/hidapi-rs`; `curl https://crates.io/api/v1/crates/hidapi`; `gh api repos/ruabmbua/hidapi-rs`; WebFetch of docs.rs auto-trait sections.
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Registry and activity

crates.io API (raw):
```
max_version 2.6.7 newest 2.6.7 updated_at 2026-08-27T18:29:23.182474Z downloads 8685960 repo https://github.com/ruabmbua/hidapi-rs
  2.6.7 2026-08-27T18:29:23.182474Z
  2.6.6 2026-05-15T18:51:38.885281Z
  2.6.5 2026-02-25T18:54:40.963085Z
  2.6.4 2025-12-20T09:17:37.806946Z
  2.6.3 2024-08-07T18:17:17.881249Z
  2.6.2 2024-08-03T19:42:49.430551Z
```
GitHub API: `open_issues_count=19 pushed_at=2026-08-27T18:29:28Z stars=242`

git log (selected, since v2.6.3):
```
8729463 2026-08-27 Bumped crate version
d2c41b3 2026-07-29 Fix empty HID_UNIQ causing devices to be dropped from enum...
9bfb93a 2026-04-24 linux-native: make output ioctls readwrite to match kernel...
722f007 2026-02-17 Get rid of heavyweight `windows` dependency, define blueto...
bf61744 2026-01-16 Upgrade dependencies, in particular windows-sys to 0.61
310ad77 2025-12-19 linux-native: do not panic on invalid strings from the OS
332c007 2025-06-08 Deprecate HidApi::new_without_enumerate().
1a1d1a7 2025-03-09 basic-udev: add no-dependency udev alternative
62e9710 2025-03-07 linux_native: parse udev directly
32a4111 2024-12-16 Implement send_output_report() for windows-native
1c60b2f 2024-12-07 Update hidapi
```

Open issues (gh issue list, selected): `181 How would I target a specific composite device?` (2026-02-19); `157 Linux Native target does not find any devices`; `155 Support for monitoring of new devices`; `127 Assertion / Illegal instruction on macOS when calling HidApi::new in different threads`; `81 Multiple open on same device leads to device lock`; `51 Async interface`; `30 Create pure rust macos backend (using IOHidManager)`.

# Cargo.toml (version 2.6.7)

```toml
[features]
default = ["linux-static-hidraw", "illumos-static-libusb"]
linux-static-libusb = []
linux-static-hidraw = []
linux-shared-libusb = []
linux-shared-hidraw = []
linux-native = ["dep:udev", "dep:nix"]
linux-native-basic-udev = ["dep:basic-udev", "dep:nix"]
illumos-static-libusb = []
illumos-shared-libusb = []
macos-shared-device = []
windows-native = [ "windows-sys/Win32_Devices_DeviceAndDriverInstallation", ... ]

[target.'cfg(target_os = "linux")'.dependencies]
udev = { version = "0.9", optional = true }
basic-udev = { version = "0.1", optional = true }
nix = { version = "0.30", optional = true, features = ["fs", "ioctl", "poll"] }

[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.61", features = ["Win32_Foundation"] }
```

# build.rs (link requirements)

```rust
("LINUX_STATIC_HIDRAW", &|| {
    let mut config = cc::Build::new();
    ...
    config
        .file("etc/hidapi/linux/hid.c")
        .include("etc/hidapi/hidapi");
    pkg_config::probe_library("libudev").expect("Unable to find libudev");
    config.compile("libhidapi.a");
```
```rust
("LINUX_NATIVE", &|| {
    // The udev crate takes care of finding its library
}),
("LINUX_NATIVE_BASIC_UDEV", &|| {
    // Enable `feature="linux-native"` to reuse the existing
    // linux-native code. It is considered an error in
    // basic-udev if this fails to compile.
    println!("cargo:rustc-cfg=feature=\"linux-native\"");
}),
```
```rust
if backends.clone().count() != 1 {
    panic!("Exactly one linux hidapi backend must be selected.");
}
```
```rust
fn compile_windows() {
    #[cfg(not(feature = "windows-native"))]
    {
        ...
        cc.file("etc/hidapi/windows/hid.c")
        ...
        println!("cargo:rustc-link-lib=setupapi");
```
```rust
fn compile_macos() {
    cc::Build::new()
        .file("etc/hidapi/mac/hid.c")
        ...
    println!("cargo:rustc-link-lib=framework=IOKit");
```

Related crates read for the libudev question:
- `udev` 0.9.3 Cargo.toml (Smithay/udev-rs): `libudev-sys = "0.1.4"`.
- `libudev-sys` build.rs (dcuddeback/libudev-sys): `pkg_config::find_library("libudev").unwrap();`
- `basic-udev` 0.1.2 (2025-03-08) README (xobs/basic-udev): "This is a simple implementation of `udev`, written entirely in Rust and without dependencies." and "This library is currently enough to be used as a drop-in replacement for [hidapi-rs](https://github.com/ruabmbua/hidapi-rs). Other projects are untested."
- hidapi-rs `src/linux_native.rs` line 6: `use basic_udev as udev;` (under the basic-udev feature).

# Bundled C hidapi version

`git ls-tree HEAD etc/hidapi` -> `160000 commit ff67c77daddbd8e61ad3873ac16f8edc005f943f etc/hidapi`
In a clone of libusb/hidapi: `git describe --tags ff67c77` -> `hidapi-0.14.0-41-gff67c77`, commit date 2024-10-30.
`hid_read_error introduced by 95e6b98 2025-03-11 Add hid_read_error (#721)`; `git merge-base --is-ancestor 95e6b98 ff67c77` -> `NOT included in hidapi-rs pin ff67c77`. `hidapi-0.15.0 tag commit d6b2a97 2025-05-18`.

# src/lib.rs (API docs)

```rust
/// `hidapi` context.
///
/// The `hidapi` C library is lazily initialized when creating the first instance,
/// and never deinitialized. Therefore, it is allowed to create multiple `HidApi`
/// instances.
///
/// Each instance has its own device list cache.
pub struct HidApi {
    device_list: Vec<DeviceInfo>,
}
```
```rust
#[cfg(all(target_os = "macos", feature = "macos-shared-device"))]
unsafe {
    ffi::macos::hid_darwin_set_open_exclusive(0)
}
```
```rust
trait HidDeviceBackend: HidDeviceBackendBase + Send {}
...
pub struct HidDevice {
    inner: Box<dyn HidDeviceBackend>,
}
```
```rust
/// Write an Output report to a HID device.
///
/// The first byte of `data` must contain the Report ID. For
/// devices which only support a single report, this must be set
/// to 0x0. The remaining bytes contain the report data. Since
/// the Report ID is mandatory, calls to `write()` will always
/// contain one more byte than the report contains. For example,
/// if a hid report is 16 bytes long, 17 bytes must be passed to
/// `write()`, the Report ID (or 0x0, for devices with a
/// single report), followed by the report data (16 bytes). In
/// this example, the length passed in would be 17.
```
```rust
/// Read an Input report from a HID device with timeout.
///
/// Input reports are returned to the host through the 'INTERRUPT IN'
/// endpoint. The first byte will contain the Report number if the device
/// uses numbered reports. Timeout measured in milliseconds, set -1 for
/// blocking wait.
pub fn read_timeout(&self, buf: &mut [u8], timeout: i32) -> HidResult<usize> {
```
```rust
/// Usage page is not available on linux libusb backends
pub fn usage_page(&self) -> u16 {
```
`src/macos.rs`:
```rust
/// Changes the behavior of all further calls that open a new [`HidDevice`]
/// like [`HidApi::open`] or [`HidApi::open_path`]. By default on Darwin
/// platform all devices opened by [`HidApi`] are opened in exclusive mode.
pub fn set_open_exclusive(&self, exclusive: bool) {
```

docs.rs 2.6.7, "Auto Trait Implementations" (WebFetch extraction):
- HidDevice: `impl !RefUnwindSafe`, `impl !Sync for HidDevice`, `impl !UnwindSafe`, `impl Freeze`, `impl Send for HidDevice`, `impl Unpin`.
- HidApi: `Freeze, RefUnwindSafe, Send, Sync, Unpin, UnsafeUnpin, UnwindSafe`.

# src/windows_native/mod.rs

```rust
    read_pending: Cell<bool>,
    blocking: Cell<bool>,
    read_state: RefCell<AsyncState>,
    write_state: RefCell<AsyncState>,
    feature_state: RefCell<AsyncState>,
}

unsafe impl Send for HidDevice {}
```
```rust
fn fill_buffer(&mut self, data: &[u8]) {
    // Make sure the right number of bytes are passed to WriteFile. Windows
    // expects the number of bytes which are in the _longest_ report (plus
    // one for the report number) bytes even if the data is a report
    // which is shorter than that. Windows gives us this value in
    // caps.OutputReportByteLength. If a user passes in fewer bytes than this,
    // use cached temporary buffer which is the proper size.
    let data_size = data.len().min(self.buffer.len());
    self.buffer[..data_size].copy_from_slice(&data[..data_size]);
    if data_size < self.buffer.len() {
        self.buffer[data_size..].fill(0);
    }
}
```
```rust
fn write(&self, data: &[u8]) -> HidResult<usize> {
    ensure!(!data.is_empty(), Err(HidError::InvalidZeroSizeData));
    let mut state = self.write_state.borrow_mut();
    state.fill_buffer(data);
    let res = unsafe {
        WriteFile(
            self.device_handle.as_raw(),
            state.buffer_ptr(),
            state.buffer_len() as u32,
            null_mut(),
            state.overlapped.as_raw(),
        )
    };
    if res != TRUE {
        let err = Win32Error::last();
        ensure!(err == Win32Error::IoPending, Err(err.into()));
        Ok(state
            .overlapped
            .get_result(&self.device_handle, Some(1000))?)
    } else {
        Ok(0)
    }
}
```
```rust
//There was no data this time. Return zero bytes available, but leave the Overlapped I/O running.
```
```rust
fn send_output_report(&self, data: &[u8]) -> HidResult<()> {
    ensure!(!data.is_empty(), Err(HidError::InvalidZeroSizeData));
    let mut state = self.feature_state.borrow_mut();
    state.fill_buffer(data);
    check_boolean(unsafe {
        HidD_SetOutputReport(
            self.device_handle.as_raw(),
            state.buffer_ptr() as _,
            state.buffer_len() as u32,
        )
    })?;
```
```rust
fn open_path(device_path: &CStr) -> HidResult<HidDevice> {
    let device_path = U16String::try_from(device_path).unwrap();
    let handle = open_device(&device_path, true)
        // System devices, such as keyboards and mice, cannot be opened in
        // read-write mode, because the system takes exclusive control over
        // them.  This is to prevent keyloggers.  However, feature reports
        // can still be sent and received.  Retry opening the device, but
        // without read/write access.
        .or_else(|_| open_device(&device_path, false))?;
    check_boolean(unsafe { HidD_SetNumInputBuffers(handle.as_raw(), 64) })?;
    let caps = PreparsedData::load(&handle)?.get_caps()?;
    ...
        read_state: RefCell::new(AsyncState::new(caps.InputReportByteLength as usize)),
        write_state: RefCell::new(AsyncState::new(caps.OutputReportByteLength as usize)),
        feature_state: RefCell::new(AsyncState::new(caps.FeatureReportByteLength as usize)),
```

# src/linux_native.rs

```rust
pub(crate) fn open_path(device_path: &CStr) -> HidResult<HidDevice> {
    // Paths on Linux can be anything but devnode paths are going to be ASCII
    let path = device_path.to_str().expect("path must be utf-8");
    let fd: OwnedFd = match OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(path)
```
```rust
fn write(&self, data: &[u8]) -> HidResult<usize> {
    if data.is_empty() {
        return Err(HidError::InvalidZeroSizeData);
    }
    Ok(write(self.fd.as_fd(), data)?)
}
```
```rust
fn read_timeout(&self, buf: &mut [u8], timeout: i32) -> HidResult<usize> {
    ...
    let mut pollfds = [PollFd::new(self.fd.as_fd(), PollFlags::POLLIN)];
    let res = poll(&mut pollfds, poll_timeout)?;
    if res == 0 {
        return Ok(0);
    }
    let events = pollfds[0]
        .revents()
        .map(|e| e.intersects(PollFlags::POLLERR | PollFlags::POLLHUP | PollFlags::POLLNVAL));
    if events.is_none() || events == Some(true) {
        return Err(HidError::HidApiError {
            message: "unexpected poll error (device disconnected)".into(),
        });
    }
```
Enumeration: `let mut enumerator = match udev::Enumerator::new() {` / `enumerator.match_subsystem("hidraw").unwrap();` and per-usage entries: `// Now we can create DeviceInfo for all the other usages`.
