# SUMMARY: systemd issue 38991, request for uaccess on all hidraw devices (refused)

## Claim
- /dev/hidraw* is created root:root 0600, and only listed device categories get user ACLs.
- A request to add `KERNEL=="hidraw*", TAG+="uaccess"` upstream was closed within about two hours by the systemd maintainer "for security reasons", pointing to logind TakeDevice instead.
- In the related issue 22681, the maintainer says mouse, keyboard and gamepad raw access stays behind logind, and blanket hidraw access "would have to be a downstream change".

## Method
GitHub issue threads with maintainer comments.

## Result
- Upstream systemd will not grant hidraw access to vendor HID tools by default.
- Each application or distribution must ship a device-specific rule.
- The requester reports TakeDevice "does not seem to work in the browser context".

## Evidence tier
2 (maintainer decision in issue comments).

## Performance
Not applicable.

## Correctness
- Proven: the upstream position as of 2025-09.
- Distributions may differ downstream. Not surveyed.

## Relevance to hyperpace
- On Linux, hyperpace cannot open the vendor hidraw node without a rule installed by root. The operator's own dev machine shows root:root 0600, consistent with this.
- Recommended rule shape: `SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="fb16", TAG+="uaccess"`, plus fb14, in a file sorting before 73.
- The AppImage has to detect the permission failure and show the exact command, or offer a pkexec-based installer. That is an operator decision.
