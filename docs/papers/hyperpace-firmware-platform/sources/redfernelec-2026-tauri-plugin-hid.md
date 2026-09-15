URL: https://github.com/RedfernElec/tauri-plugin-hid ; https://crates.io/crates/tauri-plugin-hid
Retrieved: 2026-09-15
Source type: Third-party Tauri plugin README and crates.io metadata.
Retrieval method: `gh api repos/RedfernElec/tauri-plugin-hid/readme` (verbatim) and `curl https://crates.io/api/v1/crates/tauri-plugin-hid`.
Character note: none needed.

# Excerpts

crates.io: `max 0.2.3 updated 2026-01-07T14:14:49.345159Z dl 6227 repo https://github.com/RedfernElec/tauri-plugin-hid`

README:
> Tauri plugin to provide access to USB HID devices.
>
> Uses hidapi-rs on MacOS, Windows and Linux.
>
> Uses Android UsbManager on Android.

> **Features:**
>
> *   Enumerate devices
> *   Open multiple devices simultaneously
> *   Read and write input and output reports
>
> **Limitations:**
>
> *   Feature reports not supported yet
> *   Currently only tested on macOS, Windows and Android.

Not evaluated: how the plugin schedules hidapi calls across threads on macOS (source not read).
