# SUMMARY: TechPowerUp Fantech Aria II disassembly (CX52850 = Beken BK3633)

**Claim.** In a teardown, the mouse MCU "CompX CX52850" is identified as "a rebranded Beken BK3633". Chinese retail/teardown copy independently equates 匠盟 CX52850 with 上海博通 (Beken). This establishes that the CompX CX52xxx mouse/dongle parts are Beken silicon relabeled by Compx.

**Method.** Search-engine snippets of the TechPowerUp page (403 to the fetch tool) plus corroborating Chinese retail snippets and the 博通=Beken company result.

**Result.** Strong convergent evidence that CX52850 (mouse) is Beken BK3633; by extension the CX52650 dongle family is Beken silicon too.

**Evidence tier.** 3 (review teardown via snippet + community retail copy); the rebrand identity is repeated across independent sources, raising confidence.

**Performance.** BK3633 is a BT5.2 dual-mode + 2.4G SoC; the CX52650/N dongle variant is the receiver counterpart.

**Correctness.** Proven that CX52850 = BK3633 by teardown. Assumed by extension (not directly torn down here): CX52650N is the matching Beken dongle SoC. The specific "N" suffix (Lofree's 1K/2K receiver) is not attested in any teardown; it is a Compx marketing suffix. Beken keeps datasheets confidential, so no register-level bootloader spec is public.

**Relevance to hyperpace.** Identifies the CX52650N (1K/2K receiver) as a Beken 2.4G SoC with no public datasheet or open flashing tool. Its update method is therefore whatever Compx's proprietary USB-HID upgrade does (command 0x0d + a Compx boot PID), NOT any documented ISP. This is the least-understood of the three receiver chips and the highest research-risk for a revert feature.
