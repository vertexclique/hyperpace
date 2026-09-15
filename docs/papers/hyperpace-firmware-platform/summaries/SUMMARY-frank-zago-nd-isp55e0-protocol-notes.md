# SUMMARY: isp55e0 WCH ISP capture notes

**Claim.** Independent USB-capture notes of the official WCHISPTool confirm the wchisp protocol: XOR-key obfuscation whose last byte is (first byte + chip type) so an all-zero key is impossible to forge blindly; config bits gate reset/bootloader/ROM-read; write-protected flash needs a 0xa5 unprotect; a final empty write is required or the firmware check fails; UART variant uses 0x57AB/0x55AA framing with an 8-bit sum CRC.

**Method.** Downloaded protocol.txt (sha256 recorded) and read the README via fetch. No execution.

**Result.** Corroborates wchisp from a second, capture-based source.

**Evidence tier.** 2 (community reverse-engineering from captures).

**Performance.** N/A.

**Correctness.** Proven consistent with wchisp. Reinforces: no version/authentication in the ISP; "encryption" is obfuscation. Captures are for CH579/CH32F103, not CH32V305, so exact CH32V305 config-bit layout is by analogy.

**Relevance to hyperpace.** Second-source confirmation that a downgrade is not blocked by the WCH ISP itself, and that recovery from an interrupted flash is re-running the tool while the part sits in its bootloader.
