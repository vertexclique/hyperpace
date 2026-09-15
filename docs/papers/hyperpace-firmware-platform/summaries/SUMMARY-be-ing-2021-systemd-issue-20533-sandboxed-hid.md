# SUMMARY: systemd issue 20533, HID access for sandboxed applications

## Claim
Apps work around missing hidraw access by shipping udev rules with the uaccess tag, but a sandboxed app (Flatpak) cannot install a rule. Mixxx had many support requests before shipping one. The systemd maintainer rejects allowlisting whole vendors; hwdb entries per device are "driver support" that someone must review.

## Method
GitHub issue thread with systemd maintainer and GNOME developer comments.

## Result
- udev rules are the accepted mechanism.
- No vendor-wide allowlisting upstream.
- One suggestion (not adopted in the retrieved portion): tag hidraw nodes of devices that have no other input driver.

## Evidence tier
2 (maintainer comments).

## Performance
Not applicable.

## Correctness
- Proven: the positions as stated.
- The "COMPLETED" close state suggests some resolution, likely hwdb or FIDO-related, which was not investigated.

## Relevance to hyperpace
- The support-burden lesson (Mixxx) argues for an in-app, first-run permission check with a copy-paste fix.
- HYPACE has boot-keyboard and mouse interfaces, so it would not qualify for an "only hidraw, no input device" exemption even if one existed.
