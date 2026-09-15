# SUMMARY: lofree-hypace-reverse-engineer, HYPACE WebHID protocol notes

## Claim
Lofree's "Control HUB WEB" configurator talks to the HYPACE over WebHID:
- report ID 8 with 16 payload bytes, sent as an output report by default (some models use feature reports, flagged per command);
- a checksum making the sum of the report ID plus 16 bytes equal 0x55 mod 256;
- the device filter lists VIDs 0x373B and 0x3554.

The author also states the web flasher lacks certain features on macOS.

## Method
Reverse engineering of the minified web bundle and its JSON config files by a community member.

## Result
- The report-8, 16-byte transport matches the device facts in the brief.
- The WebHID path proves that a user-space HID-only channel suffices for configuration, and, per the README, for the web flasher's firmware flashing.
- PROTOCOL.md's filter table lists PIDs 0x101B, 0xF5F4, 0xF590, 0xF5D5, 0xF53E, 0xF501, 0xF5F6 and 0xFB16. **Correction from the coordinator:** the web driver's cfg.json does list the wired PID (`"wired": ["0xFB14"]` under `pid.mouse`). PROTOCOL.md's table is therefore incomplete, and the earlier finding that "0xFB14 is missing from the filter list" is wrong. This stream did not re-read cfg.json itself.
- PROTOCOL.md contains no bootloader VID/PID or re-enumeration description.

## Evidence tier
3 (community reverse engineering of vendor code). The FB14 correction is coordinator-supplied and unverified by this stream.

## Performance
Not applicable.

## Correctness
- Byte-level facts are inferred from minified code, not from wire captures.
- The feature-report variant and the macOS limitation are unexplained.
- PROTOCOL.md's PID list is known to be incomplete (it omits FB14).

## Relevance to hyperpace
- Confirms the HID transport choice.
- A native backend must allow both report-8 output reports and feature reports if the config blob says so. Feature report 6 (7 bytes) exists on the device.
- Device matching must include both FB16 (receiver) and FB14 (wired).
- VID 0x373B may also need matching; confirm with the protocol stream.
