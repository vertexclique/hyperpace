# SUMMARY: HIDAPI issue 238 and the hotplug pull requests

## Claim
HIDAPI hotplug support has been pending since 2021. In September 2026 it is still being built in open PRs, with per-backend fixes, tests and documentation, and known deadlock and data-completeness issues. Downstream OpenRGB already compiles against it conditionally.

## Method
GitHub issue thread (maintainer and contributor comments), PR list metadata, and a grep of the OpenRGB source.

## Result
- **Open at 2026-09-15:** RFC PR 674 (since 2024-04-06), combined RFC PR 836 (2026-09-09), per-backend async ENUMERATE PRs 822 to 825, test PRs 826 and 830.
- **Merged:** docs PR 790 and a libusb CPU fix, 783.
- **Open problems:** 794 (macOS self-join deadlock), 795 (hid_exit inside callback), 828 (Linux monitor recovery).
- **Reported data issue:** Linux hotplug events arrived with empty usage values. It was later traced to dlopen linking.
- **OpenRGB:** wraps `hid_hotplug_register_callback` under `HID_HOTPLUG_ENABLED`.

## Evidence tier
2 (maintainer and contributor comments), plus 1 for the PR state metadata.

## Performance
A libusb-backend callback thread CPU issue was fixed (PR 783). No other numbers.

## Correctness
- Proven: the hotplug API is not in a release and not in master.
- Unknown: when it will merge, and whether hidapi-rs will bundle it.

## Relevance to hyperpace
- Do not plan on HIDAPI hotplug. hyperpace needs its own watcher: async-hid `watch`, nusb `watch_devices`, or direct udev netlink / CM_Register_Notification / IOHIDManager matching callbacks.
- Revisit if hidapi 0.16 ships with hotplug and hidapi-rs picks it up.
