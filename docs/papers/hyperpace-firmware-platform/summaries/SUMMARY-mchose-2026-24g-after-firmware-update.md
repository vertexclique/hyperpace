# SUMMARY: MCHOSE "2.4G stops working after firmware update"

**Claim.** 2.4G failing after an update is "usually caused by an incorrect or incomplete firmware update between the mouse and receiver"; both must be updated, some models require flashing mouse and receiver simultaneously via two cables, and re-pairing is needed afterward.

**Method.** Search-engine snippets (article and the Corsair analog both 403 the fetch tool). MCHOSE uses its own protocol (different brand), so this is analog evidence.

**Result.** Independent confirmation that mouse/receiver firmware are coupled and a mismatch breaks 2.4G.

**Evidence tier.** 3 (vendor FAQ via snippet, different protocol family).

**Performance.** N/A.

**Correctness.** Supports (does not prove for CompX) that version mismatch between mouse and receiver breaks the RF link and must be fixed by matching versions and re-pairing. Cross-brand generalization, so weight it as suggestive.

**Relevance to hyperpace.** Reinforces operator question 4: revert must be done as a matched mouse+receiver pair; leaving one side reverted and the other current risks a dead 2.4G link. A safe revert UI should refuse or warn on a mismatched pair.
