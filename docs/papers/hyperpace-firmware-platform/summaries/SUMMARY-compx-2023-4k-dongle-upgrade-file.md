# SUMMARY: Compx 4K dongle upgrade file V1.27 (primary artifact)

**Claim.** Compx's own published 4K-dongle upgrade .bin is a two-image container ("ComUsbUpgradeFile"). Image 1 targets IC "CH32V305" with boot endpoint vid_3554&pid_f401 and normal endpoint vid_3554&pid_f510; image 2 targets IC "NRF52810" with boot endpoint vid_3554&pid_f510&mi_01&col07. So a single Compx "4K dongle" carries BOTH a WCH CH32V305 and a Nordic nRF52810, each with its own firmware image, boot PID, and reset/prepare/download command. Each image's reset frame (08 0d 00 00 00 03 0X 01 d3 ... ) is command 0x0d (EnterUsbUpdateMode) and its bytes sum to 0x55 mod 256, the same checksum rule as the HYPACE report-8 channel.

**Method.** Downloaded only the .bin (not the .exe installer; sha256 and size recorded) from the Compx news page. Analyzed with file/strings and a Python byte reader; parsed the header with the OpenMouse layout; tested CRC/checksum hypotheses. No execution, no device.

**Result.** Directly confirms the OpenMouse-documented package format on a genuine Compx file, and reveals the dual-MCU 4K receiver architecture.

**Evidence tier.** 1 (primary vendor artifact, measured directly).

**Performance.** Images are ~29 KB and ~19 KB; payloads have ~7.99 bits/byte entropy (encrypted or compressed, not plain code), so the on-wire transfer is opaque.

**Correctness.** Proven by measurement: two ComUsbUpgradeFile entries, IC names, boot/normal PIDs, version 0x127, checksum-to-0x55 reset frames, header layout. Unproven: which CRC covers the payload (prepare tail bytes matched none of zlib/raw CRC-32 or sums; OpenMouse's declared payload-CRC slot is zero here, so either this file predates that field or uses a different location); and the payload cipher (high entropy blocks it). The dual-MCU finding is for a Compx reference 4K dongle, NOT confirmed to be the Lofree HYPACE 4K receiver, though HYPACE's 4K receiver is also stated to be CH32V305.

**Relevance to hyperpace.** High. (1) It shows the Compx upgrade path is USB-HID via boot PIDs (f401 for the CH32V305, f510 for the nRF52810), NOT the WCH factory 4348:55e0 ISP, meaning hyperpace must implement Compx's own boot protocol, not wchisp. (2) A 4K receiver may contain two MCUs to flash, doubling brick surface and making atomic/ordered updates essential. (3) For downgrade: the file has a version field (0x127) and CRCs but no visible signature, so integrity is a checksum, not authentication; nothing in the container structure prevents flashing an older version. (4) The payload cipher and exact flash handshake still require capturing the official updater to implement safely.
