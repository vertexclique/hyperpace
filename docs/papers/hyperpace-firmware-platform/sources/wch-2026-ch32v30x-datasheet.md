URL: https://ch32-riscv-ug.github.io/CH32V307/datasheet_en/CH32V20x_30xDS0.PDF (community GitHub Pages mirror of the WCH datasheet "CH32V303/305/307/317 Datasheet")
Retrieval date: 2026-09-15
Source type: official vendor datasheet (WCH), obtained from a community mirror

# WCH CH32V303/305/307/317 datasheet: on-chip memory and boot mode

## Acquisition

- curl to `research-bin/chips/pdf/CH32V20x_30xDS0.PDF`: 1,707,866 bytes, sha256 `9747c52e34d1f48cfb437cd7fbd65837e5c0f975a73f784f33634f539ed41250`.
- pdfinfo: Title `CH32V307DS0-EN`, 99 pages. First page reads `V3.9`, later pages `V3.8`. Text extracted with `pdftotext -layout`.

## Excerpt (verbatim, Overview, lines 5 to 14 of the extracted text)

> CH32V series is an industrial-grade general-purpose microcontroller based on QingKe RISC-V core, including
> CH32V305 connected MCU, CH32V307/317 interconnected MCU, CH32V208 wireless MCU, etc. The
> CH32V30x and CH32V31x series is based on QingKe V4F microprocessor design, supports single-precision
> floating-point instruction and fast interrupt response, supports 144MHz main frequency zero-wait operation,
> provides 8 groups of U(S)ART, 4 groups of motor PWM advanced timers, SDIO, DVP, 4 groups of analog OPA,
> dual ADC units, dual DAC units, built-in USB2.0 high-speed PHY transceivers (480Mbps), Gigabit Ethernet
> MAC controller and 10M PHY transceivers, 10/100M PHY transceiver (CH32V317 only).

## Excerpts (verbatim, section 2.5.2)

> Built-in 28K bytes System FLASH, i.e., BOOT area, is used for system boot program storage (manufacturer's
> cured bootloader).

> 128 bytes for system non-volatile configuration information storage area, for the manufacturer's configuration
> word storage, factory-cured, the user can not be modified.

> At startup, one of 3 boot modes can be selected through the boot pins (BOOT0 and BOOT1):
>  Boot from program flash
>  Boot from system memory
>  Boot from internal SRAM
> The bootloader is stored in the system memory, and the contents of the program Flash memory storage can be
> reprogrammed through the USART1 and USB interface.

Feature list line 31: `- 28KB BootLoader`.
