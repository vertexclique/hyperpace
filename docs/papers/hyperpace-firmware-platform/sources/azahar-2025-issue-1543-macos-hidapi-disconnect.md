URL: https://github.com/azahar-emu/azahar/issues/1543
Retrieved: 2026-09-15
Source type: GitHub issue in a third-party emulator (community crash analysis), opened December 29, 2025, OPEN at retrieval.
Retrieval method: WebFetch (extraction model; short fragments in quotation marks, rest is a tool summary).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

Title: "[macOS] Fix for crash when Bluetooth controller disconnects #1543"

Tool summary with quoted fragments:
- Crash: "EXC_BREAKPOINT (SIGTRAP)" pointer authentication trap in `__CFCheckCFInfoPACSignature` during `IOHIDDeviceUnscheduleFromRunLoop`; thread crashed in `CFRetain` called from `__IOHIDDeviceRegisterInputReportCallback`.
- Location: SDL's bundled HIDAPI macOS backend, `externals/sdl2/SDL/src/hidapi/mac/hid.c`.
- Analysis: when devices disconnect, "the system deallocates the IOHIDDeviceRef while code still attempts to use it" and "CFRunLoopRef becomes invalid after the read thread exits."
- Proposed fix (tool summary): mutex synchronization, `CFRetain()` of device handles, NULL checks with matching `CFRelease()` in cleanup.

Caveat: the crash is in SDL's fork of HIDAPI, not in libusb/hidapi master; applicability to hidapi-rs's bundled C code is unverified.
