# SUMMARY: nRF5 SDK dual-bank vs single-bank updates

**Claim.** In nRF5 SDK Secure DFU, dual-bank keeps the running app in bank 0 and receives into bank 1, so a failed or interrupted update leaves the old app bootable; single-bank overwrites the running app during transfer, so an interruption can leave no bootable app.

**Method.** Search-engine snippets of the Nordic InfoCenter page (docs host 403s the fetch tool; a curl was blocked by the session classifier). Mechanism corroborated by the SDK C source (separate summary).

**Result.** Consistent with the source.

**Evidence tier.** 1 for the mechanism (vendor doc + source); the wording here is a snippet, not a verbatim page read.

**Performance.** Dual-bank needs 2x app space; single-bank fits memory-limited parts at the cost of safety.

**Correctness.** Proven by source (use_single_bank, NRF_DFU_FORCE_DUAL_BANK_APP_UPDATES). The single-bank brick risk is real and is why serial recovery exists as a fallback.

**Relevance to hyperpace.** Bank strategy is THE brick-risk lever for the nRF52833 mouse. hyperpace cannot tell from outside which the vendor built; a revert feature must assume single-bank (worst case) unless proven otherwise, and must warn that an interrupted flash on a single-bank device needs a recovery path.
