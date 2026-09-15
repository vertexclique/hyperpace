# SUMMARY: OpenMouse independent Compx parser and a documented failure mode

- Claim: An independent open-source project confirms the Compx header layout and enter-bootloader command, and documents that model checks are app-layer only (bootloader is anonymous).
- Method: Read src/atk/index.ts (atkParseCompxFirmware, atkCompxPayloadCrc), vendors.ts, lamzu/atlantis.ts; read issue #97 (full body via gh API) and its comment.
- Result: enterUsbUpgradeMode=0x0d, payload offset 8192, header length 720, CRC-32 init 0xFFFFFFFF no final xor; VID 0x3554 shared by VXE/ATK, Lamzu, VGN, Teevolution. Issue #97 confirms the payload CRC does not validate for the newer CH32V305 header revision, matching our NRF/CH32 finding, and that forcing bootloader mode bypasses the normal-mode PID gate.
- Evidence tier: 3 (community code + community bug report).
- Performance: n/a.
- Correctness: Corroborates tier-1 findings; the CRC caveat and the "bootloader anonymous" claim are community-derived, treat as strong corroboration not proof.
- Relevance: Confirms header layout Hyperpace parses, and informs recovery design (bootloader-mode raw flash bypasses the model check).
