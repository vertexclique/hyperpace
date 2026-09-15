# SUMMARY: ATK firmware update guide (CompX-family)

**Claim.** If a wireless mouse is not recognized, the 2.4G/4K/8K receiver must also be updated to the latest version. A failed update shows "step 1 error code / step 2 error code"; recovery is running separate MCU files sequentially until an "echo 0" confirmation.

**Method.** Fetched the page (text extraction).

**Result.** Confirms a two-step (two-MCU) update and a sequential-file recovery, matching the dual-MCU 4K dongle finding.

**Evidence tier.** 1 (vendor page).

**Performance.** N/A.

**Correctness.** Proven: updates involve multiple MCU files ("step 1"/"step 2"), consistent with the Compx 4K .bin carrying two images (CH32V305 + nRF52810). Recovery is re-running the per-MCU files. No downgrade guidance.

**Relevance to hyperpace.** Corroborates that a CompX 4K receiver update is multi-MCU and staged, and that recovery is re-flashing the individual MCU images. hyperpace's flow must handle per-MCU steps and their independent success/failure, and expose a re-run-from-failure recovery.
