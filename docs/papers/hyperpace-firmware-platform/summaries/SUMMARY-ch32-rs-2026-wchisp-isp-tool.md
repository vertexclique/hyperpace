# SUMMARY: wchisp (WCH USB/UART ISP protocol)

**Claim.** The WCH CH32V30x family (incl. CH32V305) exposes a factory USB ISP bootloader at USB 4348:55e0 (or 1a86:55e0). Protocol: identify (0xa1), send XOR-key seed (0xa3), erase (0xa4), program (0xa5), verify (0xa6), read/write config (0xa7/0xa8), isp_end/reset (0xa2). Program data is XOR-obfuscated with an 8-byte key derived from the chip UID (sum of UID bytes, last byte += chip_id); wchisp uses an all-zero seed so the effective key is fixed. The ISP has NO firmware-version field: the bootloader accepts any image at any address. Code flash can be read-protected (RDPR) and write-protected (WRP); unprotect writes 0xa5 to RDPR (mass-erase-then-unlock). Flash sequence: unprotect -> erase -> program -> verify -> reset, with a final empty-data write required for success.

**Method.** Cloned the repo (commit cefd870); read protocol.rs, constants.rs, flashing.rs, transport/usb.rs, the CH32V30x device yaml, README. Nothing built or run.

**Result.** Complete protocol and key derivation from source; CH32V305 defined (chip_id 0x50, 128K) but not in the tool's "tested" list.

**Evidence tier.** 1 (reverse-engineered but open-source and widely used; corroborated by isp55e0 capture notes).

**Performance.** 56-byte program chunks over USB bulk, 5 s timeout; fast enough for a ~30 KB dongle image.

**Correctness.** Proven: no version check in the ISP protocol, so the WCH bootloader itself never blocks a downgrade. The "encryption" is obfuscation keyed to the UID, not authentication: no signature, so any image (any version) can be flashed. Uncertainty: CH32V305 ISP entry on a sealed dongle (no exposed BOOT0) is unverified; the software-jump-to-bootloader note in wchisp is documented only for CH32V00x, and the V30x manual documents boot selection via BOOT0/BOOT1 pins only.

**Relevance to hyperpace.** For the 4K receiver: (1) the WCH ISP has no version gate, so downgrade is trivially allowed at the bootloader level; the only gate is whatever Compx's upgrade wrapper enforces (the .bin has CRC and endpoint fields but no visible signature). (2) Brick risk: interrupting an ISP flash leaves the chip in the bootloader (4348:55e0), recoverable by re-running the flash; a bad RDPR/WRP write can lock it, read-protect triggers erase. (3) Open question: how the receiver ENTERS ISP: a Compx application command (HYPACE 13 EnterUsbUpdateMode / the .bin reset command 08 0d ...) vs the WCH factory 4348:55e0 bootloader. The Compx 4K upgrade file targets the CH32V305 via pid_f401 (a Compx boot PID), NOT 4348:55e0, suggesting Compx ships its own USB-HID upgrade path on top of or instead of the WCH ISP.
