URL: https://github.com/systemd/systemd/issues/20533
Retrieved: 2026-09-15
Source type: GitHub issue with systemd maintainer comments; CLOSED COMPLETED 2022-03-14.
Retrieval method: `gh issue view 20533 -R systemd/systemd --json ...` (verbatim, last 6 comments truncated at 900 chars).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

`implement some way for unprivileged users to access HID devices with sandboxed applications | Be-ing | 2021-08-24T23:42:16Z | CLOSED COMPLETED closed=2022-03-14T23:40:51Z`

> By default, udev prohibits unprivileged users from directly accessing HID devices, both through hidraw and directly accessing USB devices. Applications that require access to HID devices can work around this by shipping a udev rule file adding the `uaccess` tag to devices they are designed to work with. This works fine for traditional distribution packages, but there is no way for a sandboxed application to install a udev rule file to the system.

> Tell users to manually install a udev rule file if they use a Flatpak. This is unappealing because it would lead to lots of support requests. Before Mixxx shipped a udev rule file, we got many of these requests.

poettering, 2021-08-30:
> Well, as with any driver support (and hwdb is pretty much that: driver support) somebody has to have a look at the respective hw before opening it up via uaccess. Blanket allowlisting whole companies is something we cannot do.

hadess, 2021-08-30:
> - devices that only export a `hidraw` interface, and not any input devices, or anything else: the `hidraw` node should be added the `uaccess` tag
