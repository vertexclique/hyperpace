# SUMMARY: Azahar issue 1543, macOS crash on HID disconnect in SDL's HIDAPI

## Claim
Disconnecting a Bluetooth controller crashes Azahar on macOS in IOHIDDeviceUnscheduleFromRunLoop / CFRetain (pointer authentication trap) inside SDL's bundled HIDAPI mac backend. The analysis: the IOHIDDeviceRef is freed while still in use, and the CFRunLoopRef becomes invalid after the read thread exits. The proposed fix adds a mutex, CFRetain of the device, and NULL checks.

## Method
Community crash analysis in an issue (via an extraction tool).

## Result
- Disconnect during active I/O is a crash-prone path in macOS HIDAPI derivatives.

## Evidence tier
3 (community analysis, not a HIDAPI maintainer).

## Performance
Not applicable.

## Correctness
- The crash is in SDL's fork, not libusb/hidapi master.
- Applicability to hidapi-rs's bundled 0.14.0-41 code is unverified.
- The issue was open with no confirmed fix.

## Relevance to hyperpace
- Include "unplug the receiver during an active exchange on macOS" in hardware tests.
- The owner loop should stop reading and close the device on the first error, never reuse a handle after a removal error, and never close from another thread while a read is in flight (HIDAPI issue 133).
