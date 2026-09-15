# SUMMARY: Microsoft, Register for device interface arrival and removal

## Claim
An application that holds a device open should register twice:
- a DEVICEINTERFACE filter, to catch arrivals;
- a DEVICEHANDLE filter on each open handle, to catch query-remove and removal, so it closes the handle and does not block removal.

Unregistering must happen from a deferred routine, not inside the callback.

## Method
Official how-to documentation.

## Result
- Arrival: register the interface filter, list existing interfaces, dedupe.
- Open handle: register a per-handle filter. On QUERYREMOVE, close the handle. On REMOVEPENDING or REMOVECOMPLETE, unregister (deferred) and close.
- An open handle that ignores query-remove can make a removal fail or stall.

## Evidence tier
1 (official documentation).

## Performance
Event-driven, no polling.

## Correctness
- Proven: the documented sequence.
- Observed in the source read: async-hid registers only the DEVICEINTERFACE filter. No DEVICEHANDLE registration was seen in the hidapi-rs windows-native or C hidapi Windows code read (hotplug in C hidapi is still unmerged).
- That is an absence found by reading, so a registration in unread code could have been missed.
- These libraries effectively detect removal through failing I/O on the vanished handle.

## Relevance to hyperpace
- The firmware update may re-enumerate the device under another VID/PID. Close all HID handles before sending the reboot-to-bootloader command, then detect surprise removal by I/O failure, or add a DEVICEHANDLE notification.
- On removal, cancel the reader thread's pending overlapped read (CancelIoEx) before closing the handle.
- hidapi-rs windows-native calls CancelIoEx (src/windows_native/mod.rs lines 184 and 362; the context around line 362 was not read). C hidapi master still calls CancelIo in hid_read_timeout's error path.
