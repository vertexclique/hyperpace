# SUMMARY: MCUboot design (swap/revert, direct-xip, downgrade prevention)

**Claim.** MCUboot supports swap-with-test-and-revert (bad image auto-reverts to the old one on next reset, anti-brick), direct-xip (boots the highest-version valid slot), and downgrade prevention that is OFF unless enabled: software prevention (MCUBOOT_DOWNGRADE_PREVENTION, requires overwrite-only) compares versions; hardware prevention (MCUBOOT_HW_ROLLBACK_PROT) compares a security counter in a signed TLV and permits same-counter downgrades.

**Method.** Read docs/design.md from a pinned clone (commit 184e436).

**Result.** Authoritative for the MCUboot path that nRF Connect SDK can use.

**Evidence tier.** 1 (project source docs).

**Performance.** Swap increases boot time after an update; direct-xip avoids the swap. Not a runtime latency concern.

**Correctness.** Proven and precise. Nuance: overwrite-only (needed for SW downgrade prevention) removes the revert safety net, so enabling downgrade prevention and keeping anti-brick revert are in tension. HW rollback via security counter still allows equal-counter downgrades.

**Relevance to hyperpace.** If the nRF52833 mouse uses nRF Connect SDK MCUboot: revert feasibility hinges on the built mode. Swap mode = downgrade possible unless SW downgrade prevention is on (which needs overwrite-only). A rejected downgrade keeps the current image (swap) or fails validation and boots the other slot (direct-xip); brick only on overwrite-only with an interrupted flash. Answers operator questions 1 and 3 for the MCUboot path.
