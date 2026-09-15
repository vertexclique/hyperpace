# SUMMARY: WCH CH32F/V20x_V30x_V31x reference manual

**Claim.** CH32V30x (incl. CH32V305) has a 28 KB factory bootloader in system memory that reprograms program flash over USART1 and USB. Boot source is selected by BOOT0/BOOT1 pins: BOOT0=1,BOOT1=0 boots the system-memory bootloader. The 128-byte manufacturer config word is factory-locked; a separate 128-byte user option-byte area exists.

**Method.** Downloaded the manual PDF (sha256 recorded, 617 pages, 2024-07-04) from a distributor mirror; extracted text; read section 1.2.2/1.3; grepped for a software boot-mode flag.

**Result.** Boot into the factory bootloader is documented via pins only for this family.

**Evidence tier.** 1 (vendor reference manual, via a distributor mirror not byte-compared to wch-ic.com).

**Performance.** N/A.

**Correctness.** Proven: pin-based boot selection and USB/USART1 reprogramming. Negative finding: no documented software (application-triggered) jump into the factory bootloader for CH32V30x (unlike CH32V00x's FLASH_STATR.MODE path). A sealed 4K dongle has no user-accessible BOOT0, so field-flashing must be via a Compx application command jumping to a (possibly custom) bootloader, not the pin.

**Relevance to hyperpace.** Field-flashing the CH32V305 receiver cannot rely on the WCH pin method; it depends on a Compx software entry. Downgrade at the WCH-ISP layer is version-unconstrained. Recovery of a truly bricked receiver may require physical BOOT0 access.
