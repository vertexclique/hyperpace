URL: https://github.com/OpenMouse-Project/mouse-protocol , issue #97
Retrieval date: 2026-09-15
Source type: community open-source re-implementation + community bug report (tier 3)

## Claim
The Compx report-8 config protocol and the "ComUsbUpgradeFile" firmware format are shared across many brands under VID 0x3554 (VXE/ATK, Lamzu Atlantis, VGN, Teevolution, Pulsar receivers). An independent parser confirms the header layout Hyperpace relies on.

## Evidence
- `src/atk/index.ts` (saved as research-bin/vendor/openmouse-ref/atk-index.ts): `ATK_COMPX_COMMAND.enterUsbUpgradeMode = 0x0d`; `ATK_COMPX_FIRMWARE_PAYLOAD_OFFSET = 8192`; `ATK_COMPX_FIRMWARE_HEADER_LENGTH = 720`; `atkCompxPayloadCrc` = CRC-32 init 0xFFFFFFFF, no final XOR; `atkParseCompxFirmware` reads headCRC/headLength/fwLength/nextFileAddress/version/DeviceType/Cid/Mid and the ASCII endpoint + command fields exactly as our images.
- `src/drivers/vendors.ts`: `teevolution: 0x3554`, `vgn: 0x3554`; `lamzu/atlantis.ts`: "CompX vendor id 0x3554 ... CompX is the ODM for all of them", verified on "Lamzu Atlantis Mini 4K, firmware 1.24".
- Issue #97 (edavidxx, 2026-09-12), verbatim: "both files share the exact same bootloader identity (0x3554:0xf408), and the COMPX command table includes enterUsbUpgradeMode (0x0d) ... forcing the device into bootloader mode (where it's anonymous) should let a raw flash of the correct file go through regardless of what's currently installed." He also reports the payload CRC "doesn't validate against the payload" for the newer CH32V305 F1 header revision, matching our finding that the CRC-32 scheme verifies for CX dongles but not for the NRF/CH32 mouse payloads.

## Relevance to Hyperpace
Independent third-party confirmation of the header layout and enter-bootloader command (0x0d), and a documented failure mode: normal-mode PID mismatch blocks the app-layer "Local Firmware" path, but bootloader mode is anonymous (model check is app-layer only). Useful for Hyperpace's recovery design. Tier 3 (community), so treat the CRC caveat as corroborating, not authoritative.
