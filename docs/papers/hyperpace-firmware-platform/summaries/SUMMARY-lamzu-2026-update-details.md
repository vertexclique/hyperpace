# SUMMARY: Lamzu update details (4K dongle)

**Claim.** After a 4K-dongle firmware update, "you might need to pair the 4K dongle again", via another mouse in software or by pressing the keyboard space bar.

**Method.** Fetched the page (text extraction).

**Result.** Confirms re-pairing is expected after a receiver firmware change on a CompX-family device.

**Evidence tier.** 1 (vendor page) for the pairing note; no date.

**Performance.** N/A.

**Correctness.** Proven: pairing state does not survive a receiver flash and must be re-established. No statement on downgrade or failure recovery.

**Relevance to hyperpace.** Reinforces that any receiver firmware change (update or revert) invalidates pairing, so hyperpace must drive the pair sequence (HYPACE command 5 DongleEnterPair + the physical L+M+R hold) after flashing the receiver.
