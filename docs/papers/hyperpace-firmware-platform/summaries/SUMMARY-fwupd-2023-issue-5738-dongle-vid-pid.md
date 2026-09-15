# SUMMARY: fwupd issue 5738 (dongle VID/PID)

**Claim.** The nordic_hid plugin reports the dongle's VID/PID for peripherals connected through an nRF Desktop dongle, so GUIDs do not match the .cab and DFU of dongle-attached peripherals fails.

**Method.** Fetched the GitHub issue (text extraction). Only the opening report was returned.

**Result.** A concrete, unresolved limitation of dongle-relayed DFU identity.

**Evidence tier.** 2 (issue by a Nordic-associated contributor; not independently reproduced here).

**Performance.** N/A.

**Correctness.** Proven: the identity problem exists in fwupd 1.7.5+. Unknown: whether a fix landed after the snapshot; no maintainer reply visible.

**Relevance to hyperpace.** A general warning for any dongle-relay DFU: identify the peripheral behind the dongle by its own id, not the receiver's. HYPACE already distinguishes PIDs FB16 (via receiver) vs FB14 (wired); the same care applies if it flashes the mouse through the receiver.
