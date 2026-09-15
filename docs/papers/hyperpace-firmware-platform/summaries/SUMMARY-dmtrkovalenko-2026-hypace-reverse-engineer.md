# SUMMARY: Only HYPACE-specific RE; config protocol only, no firmware

- Claim: The single public HYPACE RE repo documents the 16-byte config protocol and a remap script but contains no firmware flashing code and no images.
- Method: Cloned the repo, read README/PROTOCOL.md/JS and git log.
- Result: Confirms HYPACE config transport (report 8, 16-byte, checksum sum==0x55) and constants (VID 0x373B/0x3554, PID 0xFB14/0xFB16); no bootloader/flash implementation; single commit, no releases.
- Evidence tier: 3 (community).
- Performance: n/a.
- Correctness: Author asserts a Lofree web flasher exists but does not include it; the flashing protocol must be taken from the sibling Compx code.
- Relevance: Establishes HYPACE config constants and that no community HYPACE firmware exists.
