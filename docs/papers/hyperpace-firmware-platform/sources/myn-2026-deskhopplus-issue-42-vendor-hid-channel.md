URL: https://github.com/myn/deskhopplus/issues/42
Retrieved: 2026-09-15
Source type: Design spec in a GitHub issue of a community hardware project (states some findings as "measured"); author and date fields not captured in the retrieved excerpt.
Retrieval method: `gh issue view 42 -R myn/deskhopplus --json title,author,createdAt,state,body` filtered with grep (verbatim lines).
Character note: em-dash characters in quoted text were replaced with ASCII hyphens; nothing else changed.

# Excerpts

Line 41:
> The channel is driverless on both operating systems, needs **no administrator rights**, and needs **no permission grant on either platform** - measured, not assumed: a vendor-defined usage page is outside the three usage pairs macOS gates behind Input Monitoring, and HID needs no driver installation on Windows at all.

Line 117:
> **Each channel is its own USB HID interface** with an interrupt IN *and* an interrupt OUT endpoint (`TUD_HID_INOUT_DESCRIPTOR`; the plain variant is IN-only and would force host->device traffic onto the shared control pipe).

Line 119:
> **The vendor collection must never share an interface with keyboard or mouse.** macOS `IOHIDDevice::conformsTo` iterates a device's complete usage-pair list, not its primary usage, so one keyboard collection anywhere in a node flags the whole node as requiring Input Monitoring - including any vendor collection sharing it. Both existing DeskHop HID interfaces are flagged today. This is the single constraint that, if violated, silently costs the permission-free property the transport was chosen for.

Line 125:
> Consequences: **helpers locate the device by USB identifier, serial, usage page and usage - never by a device path**, because Windows device interface paths embed a collection index and are not stable across reconnects. And the interface disappearing and returning is **normal operation**.

The spec does not name the "three usage pairs" in the retrieved lines.
