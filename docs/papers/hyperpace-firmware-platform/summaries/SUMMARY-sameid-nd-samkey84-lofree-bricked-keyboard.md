# SUMMARY: samkey84 (Lofree bricked a keyboard via its own firmware update)

**Claim.** A user reports that following Lofree's official firmware-update instructions bricked their Lofree Flow 2 keyboard (it stopped entering DFU mode); Lofree support could not recover it and shipped a replacement. The author recovered basic function by tracing board pins.

**Method.** Fetched the README (text extraction). Not the HYPACE mouse; a Lofree keyboard on an unknown platform.

**Result.** A documented case of a Lofree first-party firmware update bricking a device.

**Evidence tier.** 3 (single user report).

**Performance.** N/A.

**Correctness.** Proven only as one anecdote: a Lofree official update can brick a device beyond software recovery. Not the HYPACE, unknown MCU, so not directly transferable, but a real caution about Lofree's updater quality.

**Relevance to hyperpace.** Concrete justification for the operator's insistence that the update path be understood precisely before hyperpace ships flashing: Lofree's own updater has bricked at least one device unrecoverably. Raises the bar for a safe revert feature (dual-bank preference, verified recovery path, abort protection).
