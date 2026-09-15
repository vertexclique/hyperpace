# SUMMARY: Endgame Gear "Is my mouse bricked?"

**Claim.** For a CompX-based wireless mouse that is no longer detected, the vendor's recovery is: close all software, switch the mouse OFF, plug into a rear USB port; if still undetected, use a second working mouse to run the updater; last resort, plug in while holding LMB+RMB (or LMB+MB+RMB) to "reset bootloader mode and flash the firmware".

**Method.** Curl-fetched the support article (403 to the fetch tool) and stripped to text.

**Result.** A real, vendor-blessed recovery path for a CompX mouse that fails to enumerate after a bad flash.

**Evidence tier.** 1 (vendor support article) for the procedure; chip attribution (CX52850/CX52650) is from retail listings (tier 4).

**Performance.** N/A.

**Correctness.** Proven: a button-combo bootloader entry exists on these CompX mice, and a bricked mouse can be recovered by re-flashing from the bootloader. This is exactly the kind of forced-bootloader entry hyperpace would need for recovery. Not proven that HYPACE uses the identical combo.

**Relevance to hyperpace.** Directly answers brick-recovery for the mouse: there is a physical button-held bootloader entry independent of the application, so an interrupted mouse flash is recoverable without hardware disassembly (unlike a sealed receiver). hyperpace's revert/flash UI should document and support this recovery gesture.
