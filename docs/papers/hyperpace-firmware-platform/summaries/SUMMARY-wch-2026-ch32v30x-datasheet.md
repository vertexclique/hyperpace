# SUMMARY: WCH CH32V303/305/307/317 datasheet

**Claim.** CH32V305 is a "connected" (USB, no Ethernet) QingKe RISC-V V4F MCU with a 28 KB factory bootloader in system flash; program flash is reprogrammable via USART1 and USB. The manufacturer config info area is factory-cured and user-immutable.

**Method.** Downloaded the datasheet PDF (sha256 recorded, Title CH32V307DS0-EN) from a community mirror; extracted text; read section 2.5.2.

**Result.** Confirms CH32V305 has native USB and a USB-reprogrammable factory bootloader.

**Evidence tier.** 1 (vendor datasheet, community mirror).

**Performance.** CH32V30x runs up to 144 MHz with USB2.0 HS PHY on the V307; the V305 is the USB "connected" variant. Ample for a 4K receiver.

**Correctness.** Proven for chip class and boot capability. No ISP protocol details or version checks described (none exist at that layer).

**Relevance to hyperpace.** Confirms the 4K receiver MCU identity and that it is USB-flashable. With the reference manual, fixes that field-flash entry must be software-driven on a sealed dongle.
