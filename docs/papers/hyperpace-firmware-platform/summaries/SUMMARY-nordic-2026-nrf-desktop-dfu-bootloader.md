# SUMMARY: Nordic nRF Desktop DFU module and bootloader choices

**Claim.** nRF Desktop supports two bootloaders (B0/NSIB and MCUboot) and background DFU over the config channel. The device does NOT validate the image during transfer ("does not check if the update image contains any valid data ... does not check if it is correctly signed"); the bootloader validates on the next reboot. Interrupted transfer resumes from the stored offset until restart, then restarts from zero. Bricking is bounded by bank strategy: dual-bank/swap keeps the old image and reverts a bad one; single-bank / MCUboot overwrite-only / serial recovery overwrites the running app and cannot fall back. Downgrade: B0 boots the highest-version valid slot and blocks older images only if the monotonic counter is compiled in; MCUboot blocks older images only if MCUBOOT_DOWNGRADE_PREVENTION (SW) or MCUBOOT_HW_ROLLBACK_PROT (HW) is enabled.

**Method.** Read dfu.rst, bootloader_dfu.rst, samples/bootloader/src/main.c, subsys/bootloader/bl_validation/bl_validation.c from the pinned sdk-nrf checkout; NCS downgrade-protection and image-version rst; MCUboot design.md; nRF5 SDK validation C (mirror). Corroborated the downgrade text with a search snippet of the NCS DFU page.

**Result.** Consistent across nRF Desktop docs, B0 source, MCUboot design, and nRF5 SDK source.

**Evidence tier.** 1 (vendor docs + vendor source; nRF5 SDK C via third-party mirror is 1-to-2 for that file only).

**Performance.** Background DFU runs during use; writes gated on "no HID reports transmitted", deliberately throttled. Not a latency path.

**Correctness.** Proven from source: fw_version_ok version compare (nRF5 SDK), MCUBOOT_DOWNGRADE_PREVENTION requiring OVERWRITE_ONLY, HW security counter allowing same-counter downgrades, B0 highest-version slot selection, monotonic-counter opt-in. Key uncertainty: which knobs a shipped Compx/Lofree nRF build sets is invisible from outside; a vendor can build with downgrade prevention OFF. Also unverified that the HYPACE mouse runs an nRF-SDK DFU stack rather than a Compx-proprietary one.

**Relevance to hyperpace.** Answers the downgrade scope-extension for the nRF52833 mouse IF it uses a Nordic DFU stack: (1) refusal is build-time-optional and often off; (2) images are ECDSA-P256 signed and key-bound (NSIB revokes lower-index keys), so hyperpace cannot forge an image, only replay a vendor-signed one; (3) rejected/interrupted update keeps the old image on dual-bank/swap, bricks on single-bank/serial-recovery/overwrite-only; recovery on the single-slot case is USB serial recovery only. DECISION for the operator: revert safety on the mouse depends on the vendor's bootloader build, unreadable from the binary; confirm by capturing the official updater or testing on a sacrificial unit.
