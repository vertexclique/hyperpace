URL: https://github.com/OpenMouse-Project/openmouse
Retrieved: 2026-09-15
Source type: README of a community WebHID mouse configurator (AGPL-3.0; repository created 2026-07-27, pushed 2026-09-15, 1931 stars).
Retrieval method: `gh api repos/OpenMouse-Project/openmouse` and `gh api repos/OpenMouse-Project/openmouse/readme` (verbatim).
Character note: none needed.

# Excerpts

> ### Linux WebHID permissions
>
> If a device appears in Chromium's picker but OpenMouse reports `Failed to open
> the device`, check the permissions on its `/dev/hidraw*` nodes. Linux does not
> grant user access to every HID device by default.

> VXE R1 SE+ hardware ships with more than one receiver implementation. If the
> mouse and 1K receiver appear as `3554:f58f` and `3554:f58e`, respectively,
> install these narrowly scoped rules:
>
> ```udev
> SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f58f", TAG+="uaccess"
> SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f58e", TAG+="uaccess"
> ```
>
> Save it as `/etc/udev/rules.d/70-openmouse-vxe.rules`, reload the rules, then
> unplug and reconnect both devices:

> Grant access to every `hidraw` node for each product. Chromium opens the HID
> device before OpenMouse selects its vendor configuration collection, so access
> to only the `0xff02:0x0002` collection's node is insufficient.
