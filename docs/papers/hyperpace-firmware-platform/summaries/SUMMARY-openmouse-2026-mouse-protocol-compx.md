# SUMMARY: OpenMouse mouse-protocol (Compx / 0x3554)

**Claim.** An active open-source project reverse-engineers the exact Compx report-8 protocol the HYPACE uses (VID 0x3554, usage page 0xff02, report id 0x08, 16-byte frames, checksum-to-0x55). It names command 0x0d "enterUsbUpgradeMode" and 0x5a/0x5b as mouse-upgrade error/status reports, and it fully parses the Compx firmware package: an 8192-byte header area with a 720-byte logical header (headerCrc, headerLength, firmwareLength, nextFileAddress, version, deviceType, cid, mid), ten 64-byte string fields (fileId "ComUsbUpgradeFile", IC name, boot/normal USB endpoints as vid_XXXX&pid_XXXX, reset/prepare/download command frames, sensor, product), then the payload with a CRC-32 (init 0xffffffff, poly 0xedb88320, no final XOR) stored big-endian in the prepare command. It documents that Lamzu mice/dongles take separate "DFU bootloader" PIDs while flashing (mouse 0x000a, receiver 0x0002 on Lamzu's own VID). Crucially, OpenMouse deliberately does NOT implement entering boot mode or the chunk transfers: firmware handling is read-only.

**Method.** Cloned mouse-protocol (commit a915bb3) and openmouse; read the ATK/Compx codec, the Compx firmware parser, atk-testing.md and lamzu-inca-testing.md hardware notes, the vendors table. Nothing built or run.

**Result.** This is the single best public description of the exact HYPACE-family upgrade protocol and package format.

**Evidence tier.** 2 (community reverse-engineering, but with hardware-verified test notes for config reads; the firmware-flash path is documented, not exercised).

**Performance.** Report-8 is 16 bytes/transaction; the package payload is transferred in vendor-defined chunks (not implemented by OpenMouse), analogous to the 4K upgrade file's structure.

**Correctness.** Proven: protocol framing and package layout (independently reproduced against the Compx 4K upgrade .bin in this research). Assumed/unverified: the meaning of the reset/prepare/download command bytes and the actual chunk-transfer handshake, because no one has captured or run the boot-mode flash. "enterUsbUpgradeMode" = command 0x0d confirms the HYPACE reference's "13 EnterUsbUpdateMode" is the command that puts the device into its USB upgrade bootloader.

**Relevance to hyperpace.** The most directly actionable source: it gives hyperpace the package format to parse and validate, the command ids, and a strong warning that the flash handshake itself is unproven and dangerous. It also confirms mouse and dongle re-enumerate under distinct boot PIDs during flashing, which the driver must expect. EnterMTKMode (HYPACE cmd 17) and SetDeviceVidPid (11)/SetDeviceDescriptorString (12) have NO match anywhere in OpenMouse, so they remain undocumented Compx factory commands.
