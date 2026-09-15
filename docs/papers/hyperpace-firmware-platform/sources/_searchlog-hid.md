# Search log: stream "hid"

Retrieved: 2026-09-15. Scope: HID transport for hyperpace (hidapi crate and alternatives, per-OS report length, collection selection, permissions, hotplug, concurrency, real-world tools).
Constraints honored: no device node opened, no sysfs hidraw metadata read, no tools installed, no downloaded binaries executed, no JS runtime invoked.

## Direct fetch and clone angles (not web searches)

| Angle | Command or URL | Yield |
|---|---|---|
| Crate versions | `curl https://crates.io/api/v1/crates/{hidapi,async-hid,nusb,hidapi-rusb,hidra,keyroost-hid,basic-udev,udev,libudev-sys,tauri-plugin-hid}` | Exact versions and dates (hidapi 2.6.7 2026-08-27, async-hid 0.5.3 2026-06-11, nusb 0.2.7 2026-08-03, hidapi-rusb 1.3.3 2023-11-24, hidra 0.0.4 2026-09-03) |
| Source clones | hidapi-rs, libusb/hidapi, async-hid, nusb, OpenRGB (gitlab, depth 1), rivalcfg, OpenMouse mouse-protocol, apple-oss-distributions/IOHIDFamily | Load-bearing code excerpts |
| hidapi-rs bundled C version | `git ls-tree HEAD etc/hidapi`, `git describe --tags ff67c77`, `git merge-base --is-ancestor 95e6b98 ff67c77` | Pin is hidapi-0.14.0-41 (2024-10-30), predates hid_read_error and 0.15.0 |
| hidapi-rs Send/Sync | docs.rs 2.6.7 HidDevice and HidApi pages | HidDevice Send + !Sync; HidApi Send + Sync |
| libudev linkage | udev crate Cargo.toml, libudev-sys build.rs, basic-udev README | pkg-config libudev for default and linux-native; basic-udev is pure Rust |
| GitHub issues and PRs | `gh issue view` / `gh pr view` / `gh issue list` / `gh pr list` on hidapi-rs, libusb/hidapi, signal11/hidapi, async-hid, systemd, xdg-desktop-portal, deskhopplus | Verbatim thread text |
| Microsoft docs | HidD_SetOutputReport, HIDP_CAPS, Sending HID Reports, Top-Level Collections, TLCs opened for system use, Obtaining HID Reports, Troubleshooting HID Reports, HidD_SetNumInputBuffers, CM_Register_Notification, device interface arrival and removal, Opening HID Collections | Per-TLC length rule, system-exclusive TLC table, hotplug API |
| Apple headers | `gh api` on apple-oss-distributions/IOKitUser hid.subproj headers and IOHIDFamily IOHIDKeys.h | Seize option semantics, SetDispatchQueue (10.15+), synchronous SetReport |
| Apple TCC gating | grep in IOHIDFamily clone for `kTCCServiceListenEvent`, `ListenEvent`, `protectedDeviceAccess`, `conformsTo` | Input Monitoring required only when the node conforms to GD Keyboard, GD Mouse or Digitizer TouchPad |
| Apple forums | threads 804793, 724608, 774149, 696673, 683940, 124444, 742144, 817009 | DTS statements on Input Monitoring and run-loop threading; 742144 and 124444 and 817009 read but not written up (device-specific or beta-only) |
| Kernel | docs.kernel.org hidraw page; `gh api` drivers/hid/hidraw.c | Write passes the buffer as-is (2..HID_MAX_BUFFER_SIZE); poll reports EPOLLERR/EPOLLHUP on removal; read returns -EIO |
| systemd | commit 305272a; issues 38991, 20533, 22681 | No blanket hidraw uaccess upstream; logind TakeDevice for hidraw |
| Device prior art | lofree-hypace-reverse-engineer README and PROTOCOL.md; OpenMouse README; mouse-protocol docs and drivers | Report 8, 16 B payload; CompX VID 0x3554 sibling with config TLC 0xFF02/0x0002 on Windows |

## Web searches (WebSearch), in order

| # | Query | Yield |
|---|---|---|
| 1 | hidapi macOS "Input Monitoring" vendor usage page IOHIDDeviceOpen kIOReturnNotPermitted | go-macos iokit/hid docs, OpenSnek PR 112, nachtimwald blog, forum threads 804793/744577/735999 |
| 2 | github libusb hidapi issue windows hid_write "OutputReportByteLength" multiple report IDs different lengths | hidapi 589, 513, 274 |
| 3 | hidapi macOS hid_enumerate hid_open non-main thread run loop IOHIDManager crash issue | hidapi wiki Multi-threading Notes; old signal11 crash issues |
| 4 | Apple IOHIDCheckAccess kIOHIDRequestTypeListenEvent Input Monitoring which HID devices require TCC keyboard only | forum 696673, 724608 |
| 5 | systemd udev hidraw uaccess default permissions 0600 AppImage udev rules cannot install vendor HID device | systemd 20533, 38991, 22681; HIDIOCREVOKE patches |
| 6 | hidapi windows hid_read_timeout after device unplugged ERROR_DEVICE_NOT_CONNECTED GetOverlappedResult returns -1 issue | libusb/hidapi 128 (closed 2020), signal11 88/320; not written up (superseded by current code) |
| 7 | OpenRGB udev rules AppImage Flatpak "60-openrgb.rules" install instructions hidraw permissions | openrgb.org/udev.html |
| 8 | tauri macOS hidapi HidApi::new crash main thread run loop | azahar 1543; Tauri main-thread issues (unrelated to HID) |
| 9 | rivalcfg udev rules --update-udev hidraw Windows macOS README flozz | rivalcfg docs (then cloned) |
| 10 | Solaar macOS hidapi "Input Monitoring" permission installation udev rules 42-logitech-unify-permissions | Dead end: Solaar is Linux-only |
| 11 | AppImage excludelist libudev.so.1 bundle or not | Dead end: excludelist excerpts did not mention libudev; unverified |
| 12 | OpenRGB REGISTER_HID_DETECTOR_IPU interface usage page usage Windows hidapi detector macro | OpenRGB IPU detector macro |
| 13 | WCH ISP bootloader USB 4348:55e0 Windows driver WinUSB wchisp CH375 driver required | wchisp README |
| 14 | WriteFile HID output report buffer smaller than OutputReportByteLength fails ERROR_INVALID_USER_BUFFER OR ERROR_INVALID_PARAMETER | signal11 218 |
| 15 | developer.apple.com IOHIDDeviceOpen kIOHIDOptionsTypeSeizeDevice "exclusive access" documentation IOHIDDeviceSetDispatchQueue macOS 10.15 | Pointer to IOKitUser headers; forum 774149 |
| 16 | macOS vendor-defined usage page HID device IOHIDDeviceOpen requires Input Monitoring kIOReturnNotPermitted composite mouse vendor interface hidapi | Nothing new |
| 17 | Windows HID WriteFile report ID not in top-level collection error multiple top-level collections vendor usage page separate device path hidapi | Microsoft troubleshooting page |
| 18 | CoreHID HIDDeviceClient macOS 15 entitlement Input Monitoring vendor-defined device access requirements | CoreHID index; no entitlement text found |
| 19 | Rust crate HID hotplug cross-platform 2026 alternative to hidapi async-hid nusb hidraw watch devices | hidra, keyroost-hid |
| 20 | AppImage app install udev rule pkexec first run hidraw permission prompt Linux desktop configurator | Cura 5069 AppRun + pkexec pattern (snippet only, not written up) |
| 21 | IOHIDManagerScheduleWithRunLoop background thread CFRunLoopRun Rust tao winit main thread conflict HID callbacks | forum 683940, 124444 |
| 22 | hidapi Windows padded output report zeros device rejects exact length report hid_write 65 bytes vendor mouse protocol | signal11 352 |
| 23 | linux hidraw write shorter than report size numbered report usbhid pads output report hid_output_report length | Kernel doc only; led to reading hidraw.c |
| 24 | IOHIDDeviceSetReport report length must match descriptor size macOS kIOReturnBadArgument shorter output report | hidapi 503; forum 742144 |
| 25 | Tauri 2 app HID device configurator Rust hidapi gaming mouse keyboard open source | tauri-plugin-hid, HIDConsole |
| 26 | Lofree HYPACE mouse Linux configure hidraw VID 3554 open source tool | lofree-hypace-reverse-engineer, OpenMouse (major) |
| 27 | Windows HID vendor collection Col05 WriteFile "Incorrect function" report ID wrong collection ERROR_INVALID_FUNCTION hid write | Nothing new |
| 28 | macOS IOHIDDeviceOpen consumer control system control usage Input Monitoring required kIOReturnNotPermitted gaming mouse vendor interface | Nothing new |
| 29 | udev uaccess rule must sort before 73-seat-late.rules 70-uaccess hidraw rule file name number ordering systemd | Confirmation of the hidapi rule comment (ArchWiki) |
| 30 | Rust HID device dedicated reader thread request response matching crossbeam channel read_timeout hidapi HidDevice not Sync pattern | Nothing new |
| 31 | logind TakeDevice hidraw HIDIOCREVOKE kernel 6.12 requires session controller TakeControl ordinary application | who-t blog: fds only to the session leader (new) |
| 32 | hidapi-rs linux-native-basic-udev static musl binary AppImage no libudev dependency hid enumeration sysfs | Nothing new |
| 33 | Tauri macOS spawn thread CFRunLoopRun IOKit notification Rust background run loop alongside tao event loop device hotplug | Tier-4 blog on nusb watch in a thread; nothing new |
| 34 | nusb detach_and_claim_interface HID interface Linux usbhid mouse stops working reattach kernel driver risk vendor HID | Nothing new |
| 35 | Windows HID overlapped ReadFile device removed GetOverlappedResult ERROR_DEVICE_NOT_CONNECTED 1167 handle invalid after surprise removal | Nothing new |
| 36 | xdg-desktop-portal USB portal hidraw access sandboxed application 2025 2026 device portal HID | xdg-desktop-portal issue 611 (still open), USB portal in 1.19.1 (new) |
| 37 | /sys/class/hidraw report_descriptor sysfs file readable without root permissions parse usage page before opening hidraw | kernel ABI sysfs-driver-hid (report_descriptor attribute); minor, not written up |
| 38 | IOHIDManagerOpen triggers Input Monitoring "Keystroke Receiving" prompt avoid enumerate vendor device matching dictionary only | deskhopplus issue 42 (new); led to IOHIDFamily source verification |
| 39 | hardened runtime notarized macOS app IOHIDDeviceOpen entitlement com.apple.security.device.usb required non-sandboxed HID access | Dead end: no authoritative statement found |
| 40 | Chromium WebHID Windows sendReport pads output report OutputReportByteLength hid_connection_win implementation | Dead end |
| 41 | hidapi macOS hid_close slow read thread CFRunLoopStop join delay close device latency issue | Old signal11 114; nothing current |
| 42 | wireless mouse receiver asleep hidraw command no response timeout dongle reports device offline vendor protocol Linux configurator | openrazer_rust PR 14 "Model an idle wireless receiver as a state, not three errors" (minor, not written up) |
| 43 | Windows HID device path changes after replug same device container ID instance path hidapi reconnect identify same device | Container ID docs; OpenRGB issue 2499 "Don't use hidraw device paths for unique IDs" (minor, not written up) |
| 44 | macOS IOHIDDevice kIOHIDLocationIDKey registry entry ID stable across replug identify same USB receiver | Dead end |
| 45 | Rust tokio-udev OR udev crate MonitorBuilder AsyncFd hidraw add remove event example 2026 | tokio-udev exists; minor |
| 46 | hidapi-rs 2.6.7 regression OR bug linux-native enumeration HID_UNIQ issue 2026 | Adversarial check: no regression report found |
| 47 | Windows 11 24H2 25H2 HID vendor defined collection CreateFile access denied change hid.dll regression 2025 2026 | Adversarial check: nothing HID-specific found |
| 48 | macOS 26 Tahoe IOHIDDeviceOpen Input Monitoring change HID access regression hidapi 2026 | forum 817009: macOS 26.4 beta regression limited to built-in SPI keyboard, resolved April 2026 (minor) |
| 49 | Linux kernel HID quirk vendor 0x3554 CompX mouse receiver hid-generic driver | Nothing for 0x3554 |
| 50 | Tauri AppImage install udev rules pkexec on first launch hidraw permission github rust app | Nothing new (repeat of Cura 5069) |
| 51 | hidapi-rs HidDevice Arc Mutex blocking read holds lock write starves deadlock pattern separate reader writer | hidapi 133 (fetched, supports existing conclusion) |

## Dead ends and unverifiable items

- Apple developer JSON endpoints for IOHIDRequestAccess and kIOHIDOptionsTypeSeizeDevice returned HTTP 404. The IOHIDRequestAccess declaration was not found in the downloaded IOKitUser or IOHIDFamily headers.
- xdg-desktop-portal `doc/usb-portal.md` returned HTTP 404 via the GitHub API (moved).
- Whether libudev.so.1 is on the AppImage excludelist: not found.
- The Windows wire length of a padded report-8 write when report 9 shares the TLC: no document or issue found. Only a hardware capture settles it.
- The HYPACE bootloader VID/PID: not present in PROTOCOL.md.
- Hotplug in hidra: not found in docs or code search.
- WebFetch extractions of GitHub pages for hidapi 589 and signal11 352/218 were paraphrases, so they were re-fetched verbatim with gh.

## Stopping rule

The last five searches (47-51) produced no finding that changes a conclusion: two were empty, one confirmed an already-known issue (hidapi 133), one surfaced a macOS 26.4 beta regression limited to the built-in SPI keyboard and resolved in April 2026, and one found nothing for VID 0x3554. The stricter reading, five consecutive searches adding nothing at all, was not fully met because searches 48 and 51 returned minor corroborating items. Search stopped here and that is recorded honestly.
