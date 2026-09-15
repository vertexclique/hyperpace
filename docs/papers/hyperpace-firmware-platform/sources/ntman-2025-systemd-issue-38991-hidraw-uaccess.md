URL: https://github.com/systemd/systemd/issues/38991
Retrieved: 2026-09-15
Source type: GitHub issue closed by a systemd maintainer (Lennart Poettering); CLOSED COMPLETED 2025-09-18.
Retrieval method: `gh issue view 38991 -R systemd/systemd --json ...` (verbatim, truncated).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

`RFE: Add udev rule to grant uaccess for all hidraw devices (to support WebHID out of the box) | NTMan | 2025-09-18T09:30:05Z | CLOSED COMPLETED closed=2025-09-18T11:36:16Z`

> Currently `/dev/hidraw*` devices are created as:
>
> ```
> crw------- root root
> ```
>
> No ACLs are applied for the logged-in user unless the device matches
> specific categories in `70-uaccess.rules` (joysticks, smartcards,
> AV controllers, etc.).

> Adding a local rule fixes the issue immediately:
>
> ```udev
> # /etc/udev/rules.d/50-hidraw-uaccess.rules
> KERNEL=="hidraw*", TAG+="uaccess"
> ```

poettering, 2025-09-18:
> no, we cannot allow that wholesale for security reasons, because we isolate users of different users from each others.
>
> since a while logind has been doing hidraw arbitration, switching between sessions correctly. i.e. via the TakeDevice(). It's already used in various apps, and is really the way to go.

NTMan, 2025-09-24:
> We tried to follow your suggestion and integrate WebHID with `org.freedesktop.login1.Session.TakeDevice(major,minor)` instead of opening `/dev/hidraw*` directly. Unfortunately, the approach does not seem to work in the browser context.

Related, systemd issue 22681 (RFE: uaccess on game controllers' raw HID interfaces, closed 2024-10-09), poettering 2025-09-23:
> as long as gamepads are input devices that can be used to enter passwords and such, i am very reluctant to add udev rules that just blanket allowlist access to them without any chance to take away control. It's not the security model we so far follow. mouse/keyboards/gamepads are in the same territory here: input device access goes through logind, so that we can take bg users off them.

> (or to say this differently: if you want to blanket allow direct access to hidraw devices to unpriv apps, this would have to be a downstream change, I don't think we are going to change the security model to make such a scheme the upstream default. Sorry.)
