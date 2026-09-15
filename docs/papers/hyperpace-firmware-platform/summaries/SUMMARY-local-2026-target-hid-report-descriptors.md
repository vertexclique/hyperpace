# SUMMARY: target hardware HID report descriptors (sysfs)

## Claim
The mouse's vendor USB interface (interface 1 on both PID 0xFB16 receiver and PID 0xFB14 wired) carries report 8 alone in top-level collection 0xFF02:0x0002 and report 9 alone in collection 0xFF06:0x0002. The receiver's vendor interface additionally declares a Generic Desktop Mouse collection (0x0001:0x0002, input report 11); the wired vendor interface does not.

## Method
Read the kernel's cached report descriptors from /sys/class/hidraw/hidraw{8..13}/device/report_descriptor (no traffic to the device) and decoded them with a short HID item parser that tracks global, local and main items and top-level collections.

## Result
- Receiver vendor interface: 266-byte descriptor, 8 top-level collections: 0xff05:0x0000 (input 16), 0xff03:0x0000 (input 2), 0x000c:0x0001 (input 5), 0x0001:0x0080 (input 3), 0xff02:0x0002 (input and output 8, 16 bytes), 0xff04:0x0002 (feature 6, 7 bytes), 0xff06:0x0002 (input and output 9, 48 bytes), 0x0001:0x0002 (input 11).
- Wired vendor interface: 177-byte descriptor, the first 7 collections identical, no mouse collection.
- Derived Windows lengths (HIDP_CAPS rule): report 8 collection OutputReportByteLength 17, InputReportByteLength 17; report 9 collection 49 and 49.

## Evidence tier
1 (primary data from the target hardware as enumerated by the kernel).

## Performance
Not applicable.

## Correctness
- Proven: descriptor bytes and collection layout for this unit's current firmware (receiver bcdDevice 0x0216, wired 0x0300). A firmware change could change the descriptor.
- The Windows byte lengths are derived from the descriptor with Microsoft's documented rule, not measured on Windows.
- The macOS consequence is an inference from Apple's IOHIDFamily source (see SUMMARY-apple-2026-iohidfamily-tcc-gating-source): a node conforming to GD Mouse anywhere in its usage pairs requires Input Monitoring. It has not been run on macOS.

## Relevance to hyperpace
- Resolves the HID stream's open questions without touching the device: on Windows, open the collection with usage page 0xFF02 and usage 0x0002 and expect 17-byte report-8 writes (hidapi pads); report 9 is on a different collection path.
- macOS: opening the receiver's vendor interface likely triggers the Input Monitoring requirement because of its GD Mouse collection; the wired vendor interface likely does not. Hyperpace needs an Input Monitoring first-run flow with relaunch for receiver use on macOS, to be confirmed on a Mac.
- Receiver and wired vendor interfaces share collections 0 to 6, so one protocol path serves both links.
