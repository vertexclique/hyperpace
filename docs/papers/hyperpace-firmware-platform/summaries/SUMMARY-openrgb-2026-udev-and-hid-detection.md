# SUMMARY: OpenRGB, udev packaging and HID detector filtering

## Claim
- Distribution packages of OpenRGB install udev rules automatically. AppImage and Flatpak users must install them manually, with the AppImage itself generating the file (`sudo ./OpenRGB.AppImage --generate-udev-rules /etc/udev/rules.d/60-openrgb.rules`), then reload and trigger udev.
- The generator writes `TAG+="uaccess"` rules.
- HID detectors match on VID, PID, usage page, usage, interface number and bus type.
- Hotplug callbacks are compiled in when the linked hidapi provides them.

## Method
Official project documentation page plus source code.

## Result
- The dominant real-world pattern for a portable Linux binary: the app can emit its own rules, and the user runs one sudo command.
- Selecting devices by the usage-page/usage tuple is standard practice in a large cross-platform HID tool.

## Evidence tier
1 (primary for OpenRGB's own behavior), and 3 as evidence of general best practice.

## Performance
Not applicable.

## Correctness
- Proven for OpenRGB.
- Its support burden with AppImage and Flatpak (GitLab issues 3593, 3675) was seen only in search titles, not read.

## Relevance to hyperpace
- Adopt a `--print-udev-rules` / `--install-udev-rules` style command (the latter via pkexec, an operator decision), plus an in-app message with the exact command when hidraw open fails with EACCES.
- Select devices by (VID, PID, usage_page, usage), with interface number as a tiebreaker.
