URL: https://github.com/dmtrKovalenko/lofree-hypace-reverse-engineer
Retrieval date: 2026-09-15
Source type: community reverse-engineering repo (tier 3)

## Claim
The only public HYPACE-specific reverse engineering. It documents the report-8 CONFIG protocol (16-byte, DPI/keys/macros) and a node-hid remap script, but contains NO firmware flashing code and NO firmware images. Confirms the HYPACE config transport and its VID/PID/bootloader command constants.

## Evidence (repo checked out locally)
- README verbatim: "For some reason their web based HID firmware flasher doesn't support certain features on macos ... the protocol used for flashing the firmware." (README claims a flasher exists but the repo does not implement or include firmware.)
- PROTOCOL.md: report ID 8, 16-byte config packets; checksum `sum(reportId + buf[0..15]) mod 256 == 0x55`; opcodes incl. `EnterUsbUpdateMode` not used; flash map, button/macro layout. This is the CONFIG protocol (matches our lofree-hypace-protocol.md), not the 64-byte DFU protocol.
- macos-remap-sidebuttons-to-desktop-switch.js: `VIDS = [0x373B, 0x3554]`, `PIDS = [..0xFB14, 0xFB16]`, vendor usage pages 0xFF02-0xFF06, report id 8. Config writes only; no bootloader entry.
- Single commit a4700d1 (2026-06-09), no firmware artifacts, no releases.

## Relevance to Hyperpace
Confirms HYPACE config-side constants and that no community HYPACE firmware exists. For flashing, Hyperpace must use the Compx DFU protocol documented from the sibling code (see compx-2025-comusb-upgrade-protocol.md).
