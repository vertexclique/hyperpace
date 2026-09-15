URL: https://static.chipdip.ru/lib/820/DOC059820997.pdf (third-party distributor mirror of the WCH document "CH32F/V20x_V30x_V31x Reference Manual V2.2")
Retrieval date: 2026-09-15
Source type: official vendor reference manual (WCH), obtained from a distributor mirror

# WCH CH32F/V20x_V30x_V31x Reference Manual V2.2: memory and boot configuration

## Acquisition

- curl to `research-bin/chips/pdf/CH32FV2x_V3xRM.pdf`: 7,771,502 bytes, sha256 `29575ed31788d4a3576a039a9c8cc3410280a36d73bcbf6509805203449d232d`.
- pdfinfo: Title `CH32FV2x_V3xRM-EN`, CreationDate 2024-07-04, 617 pages. Text extracted with `pdftotext -layout`.
- Mirror not byte-compared against wch-ic.com.

## Excerpts (verbatim, section 1.2.2 and 1.3, page 16)

> Built-in maximum 480 Kbytes of Flash program memory, for storing user applications.
> Built-in 28K bytes of system memory (Bootloader), is selected as the boot space (Manufacturer's solidified
> bootloading program).
> Built-in 128-byte used to store the manufacturer's configuration word, which is solidified before delivery out
> of the factory and cannot be modified by the user.
> Built-in 128-byte space is used for user option byte storage.

> The system can select 3 different boot modes through the BOOT0 and BOOT1 pins.

```
                                            Table 1-1 Boot modes
                BOOT0          BOOT1                            Boot mode
                  0              X                   Boot from program flash memory
                  1              0                      Boot from system memory
                  1              1                      Boot from internal SRAM
```

> The user selects the startup mode after reset by setting the status value of BOOT pins. After the system is reset
> or the power is reset, the value of the BOOT pin will be latched again.

>  When it is started up from the system memory, the system memory address will be mapped to the address
>       area 0x00000000 and can also be accessed in the original address area 0x1FFF8000.

## Local search

`grep -n -E "BOOT_MODE|BOOT_LOCK|BOOT_AVA|MODE +\[|boot area|user area|BOOT_"` over the extracted text: no software boot-mode flag for the V30x family was found (hits were memory-map labels `BOOT_28KB` and unrelated SDIO text). A software path into the factory bootloader for CH32V30x is therefore not documented in this manual as far as this search reached.
