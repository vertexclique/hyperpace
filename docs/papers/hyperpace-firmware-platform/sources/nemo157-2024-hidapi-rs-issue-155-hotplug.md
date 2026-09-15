URL: https://github.com/ruabmbua/hidapi-rs/issues/155
Retrieved: 2026-09-15
Source type: GitHub issue (state OPEN at retrieval; last update 2024-05-19). Community contributors, no maintainer reply in thread.
Retrieval method: `gh issue view 155 -R ruabmbua/hidapi-rs --json ...` (verbatim).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

Title: `Support for monitoring of new devices | Nemo157 | 2024-05-08T21:20:35Z | OPEN`

Body:
> I have a usecase where I need to detect when new matching devices are inserted on the system. Currently I just periodically `.refresh_devices()` and re-iterate the list looking if there are any I haven't seen before, but this means there is a delay between when the device is inserted and the periodic refresh happens. It would be better if `hidapi` could monitor for changes in the device list and emit events as new devices appear (and events as they are removed could be useful too, though I already notice these when the `Device::read` returns an error).

alvaro-cuesta, 2024-05-09:
> `hidapi` itself does not have hotplug support yet (see libusb/hidapi#238) so I guess any monitoring in native backend should be modeled after it, but I'm not sure if they have a stable API defined yet.

> ... it's because `hidapi` device list goes through a full bus enumeration

Nemo157, 2024-05-09:
> I just pushed a branch with a quick implementation on the `linux-native` backend only: https://github.com/ruabmbua/hidapi-rs/compare/main...Nemo157:hidapi-rs:monitor-linux-native
>
> Testing it in my app it works perfectly, although as mentioned in the libusb/hidapi#238 there are race conditions between enumerating and setting up the monitor, afaict these are unavoidable.

Nemo157, 2024-05-19: a second branch for the `hidraw` C backend built against hidapi's `connection-callback` branch.

Status check 2026-09-15: hidapi-rs 2.6.7 `src/lib.rs` public API has no monitor, watch or hotplug function (grep of `pub fn` list in the clone at 8729463).
