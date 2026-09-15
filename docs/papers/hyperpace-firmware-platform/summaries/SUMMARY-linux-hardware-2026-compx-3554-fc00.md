# SUMMARY: linux-hardware.org USB 3554:fc00 = "Compx 2.4G Receiver"

**Claim.** A community hardware-probe database lists USB 3554:fc00 as vendor "Compx", device "2.4G Receiver", HID class, driven by generic hid-core/usbkbd. A second Compx receiver appears at 25a7:fa03.

**Method.** Fetched the probe page (text extraction).

**Result.** Third-party confirmation that 0x3554 enumerates as "Compx" and that Compx also uses other VIDs (0x25a7).

**Evidence tier.** 3 (crowd-sourced probe data).

**Performance.** N/A.

**Correctness.** Proven: the vendor string "Compx" is what devices report for 0x3554. Consistent with the HYPACE reference and OpenMouse.

**Relevance to hyperpace.** Minor corroboration of vendor identity and that CompX ships receivers under multiple VIDs, which a device-matching layer should keep in mind.
