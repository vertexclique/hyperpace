# SUMMARY: Apple IOKit HID public headers (IOKitUser, IOHIDFamily)

## Claim
- kIOHIDOptionsTypeSeizeDevice opens "exclusive communication with the device" and "will prevent the system and other clients from receiving events from the device".
- IOHIDDeviceOpen establishes a link; the seize option makes it exclusive.
- IOHIDDeviceSetDispatchQueue (macOS 10.15+) must not be combined with run-loop scheduling, must be set once, and requires Activate/Cancel.
- IOHIDDeviceSetReport is synchronous, and the report ID also goes in the first byte when the device has multiple reports.

## Method
Read Apple's open-source header doc comments. A DTS forum explanation (thread 683940) covered the scheduling assertion.

## Result
- Non-exclusive open (kIOHIDOptionsTypeNone) is the correct mode for a configurator.
- Two valid async models: run loop (C hidapi) or dispatch queue (async-hid). Mixing both on one object triggers an os_assert (DTS, thread 683940).
- Synchronous SetReport blocks the caller until the report is issued.
- The IOHIDRequestAccess declaration was not found in the downloaded headers, and Apple's JSON doc endpoints returned 404.

## Evidence tier
1 (Apple header documentation), plus 2 for the DTS forum explanation.

## Performance
SetReport blocks per call; tier-3 measurements show about 480 synchronous exchanges/s possible (PR 46).

## Correctness
- Proven: the documented semantics.
- The open-source drop may lag the shipping macOS; assumed current.

## Relevance to hyperpace
- Open non-exclusively.
- Run SetReport on the device owner thread, and never from the Tauri main thread (it blocks).
- If building a native macOS backend, use a dispatch queue per device (10.15+ covers every supported macOS) and never also schedule on a run loop.
