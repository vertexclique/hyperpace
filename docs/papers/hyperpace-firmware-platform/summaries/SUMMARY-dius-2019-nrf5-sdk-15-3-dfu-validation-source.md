# SUMMARY: nRF5 SDK 15.3 DFU validation source

**Claim.** The nRF5 SDK Secure DFU bootloader enforces, in fw_version_ok: if NRF_DFU_APP_DOWNGRADE_PREVENTION is false it accepts ANY version; else requires a strictly higher version, accepting equal only if NRF_DFU_APP_ACCEPT_SAME_VERSION. It also checks hardware version and SoftDevice requirements. The init packet must be ECDSA-P256-SHA256 signed; a missing signature is rejected. A debug init packet (NRF_DFU_DEBUG + is_debug) skips recording the version. Single vs dual bank chosen by use_single_bank / NRF_DFU_FORCE_DUAL_BANK_APP_UPDATES.

**Method.** Fetched two C files from a third-party GitHub mirror (DiUS) of nRF5 SDK 15.3.0; quoted the returned code.

**Result.** Directly answers downgrade question 1 and signing question 2 for the nRF5-SDK path.

**Evidence tier.** 1 for being vendor code, 2 in practice: third-party mirror not diffed against Nordic's official zip, and only the functions the fetch tool returned are quoted (surrounding call sites unverified).

**Performance.** N/A (boot-time validation).

**Correctness.** Proven: downgrade prevention is a compile-time flag and can be disabled by a vendor. Signature is mandatory and binds the image to the vendor's private key (P-256). Same-version acceptance is a second flag. Not proven: exact defaults in a Compx/Lofree build, and that HYPACE uses nRF5 SDK (vs nRF Connect SDK MCUboot/B0, vs Compx-proprietary).

**Relevance to hyperpace.** For revert on the mouse: if it runs nRF5 Secure DFU, an older vendor-signed image is accepted only if the vendor did not enable downgrade prevention. hyperpace cannot re-sign, so it can only replay official images. This is the concrete code behind the operator's downgrade question.
