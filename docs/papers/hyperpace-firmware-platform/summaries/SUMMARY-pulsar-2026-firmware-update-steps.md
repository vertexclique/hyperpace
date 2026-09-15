# SUMMARY: Pulsar firmware update steps (CompX-family mice)

**Claim.** For CompX-based mice + 1K/4K dongles, the vendor mandates: update the DONGLE firmware FIRST, then the mouse; if you do the mouse first "you may need to re-pair the device"; disconnect all other mice/dongles; keep the mouse wired when flashing it and the dongle connected when flashing it.

**Method.** Curl-fetched the support article (403 to the fetch tool); article dated 2026-06-16.

**Result.** Establishes a required update order and that mouse/dongle firmware are coupled (pairing can break if versions are updated out of order).

**Evidence tier.** 1 (vendor support article).

**Performance.** N/A.

**Correctness.** Proven: order matters and a version step can force re-pairing. Implies mouse and receiver firmware are a coupled pair, but does not explicitly say a version MISMATCH breaks the RF link permanently, only that re-pairing may be needed. Pulsar is a different brand from Lofree but same CompX protocol family.

**Relevance to hyperpace.** Answers operator question 4 (pairing across versions) for the CompX family: updates/reverts should be done as an ordered pair (receiver first), and a re-pair step must be offered after any firmware change. Strong analog evidence that hyperpace must treat mouse+receiver firmware as a matched set for a safe revert.
