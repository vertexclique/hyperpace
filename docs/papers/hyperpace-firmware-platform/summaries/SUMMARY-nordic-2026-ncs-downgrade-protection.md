# SUMMARY: nRF Connect SDK downgrade protection (NSIB and MCUboot)

## Claim
In Nordic's nRF Connect SDK, downgrade protection is an opt-in build feature. MCUboot rejects an image whose version is lower than the booted image when CONFIG_MCUBOOT_DOWNGRADE_PREVENTION is set together with overwrite-only mode. Hardware (monotonic counter) protection is available to MCUboot or NSIB, not both, and is also optional.

## Method
Official NCS documentation pages and the NSIB sample bootloader source, read from a sparse clone of sdk-nrf at commit 2700156c. Nothing was built.

## Result
- Software downgrade protection: MCUboot only, version stored in the active image; enabling it requires overwrite-only mode, which removes fallback recovery to the previous image.
- Hardware downgrade protection: version kept in a provisioned counter area with a limited number of slots, usable by NSIB or MCUboot.
- NSIB validates ECDSA-P256 signatures against provisioned key hashes; with two slots it boots the higher version first and falls back to the other.
- Build metadata after "+" is ignored when comparing versions.

## Evidence tier
1 (official documentation and source code).

## Performance
Not applicable. Counter-based protection limits the total number of version updates to the provisioned slot count.

## Correctness
- Proven: these are configuration options, off unless enabled; overwrite-only mode trades rollback safety for downgrade prevention.
- Not read: bl_validate_firmware_local in full, so whether NSIB compares against the counter at validation is not quoted.
- Applicability to the target mouse is unverified: its update path is the Compx ComUsbUpgradeFile bootloader (see SUMMARY-compx-2025-comusb-upgrade-protocol), not evidence of NCS, MCUboot or NSIB, so these options may not describe the mouse's bootloader at all.

## Relevance to hyperpace
- If a Nordic stock bootloader were in play, rollback would depend on how the vendor built it; nothing here proves the target mouse refuses downgrades.
- For the actual Compx path, rollback feasibility must come from the Compx updater code and bootloader behavior, not from NCS defaults.
