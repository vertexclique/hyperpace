URL: https://github.com/frank-zago/isp55e0 (README) and https://raw.githubusercontent.com/frank-zago/isp55e0/master/protocol.txt
Retrieval date: 2026-09-15
Source type: community reverse-engineering notes (USB captures of the vendor WCHISPTool), open-source tool

# isp55e0: captured WCH ISP protocol notes

## Acquisition

- `protocol.txt` downloaded with curl to `research-bin/chips/pdf/isp55e0-protocol.txt`: 6,753 bytes, sha256 `28f0a69a9a1562d0ee26bd20e4804140b1ae39b124bf25413b91fce9c854c171`.
- README read with the fetch tool (text extraction).

## Excerpts

### protocol.txt (verbatim)

> Protocol used by WCHISPTool to flash a CH579 with a 2.8.0 bootloader.

> The host computes a random 8-byte XOR key and sends it obfuscated to
> the host. The request length and content varies each time.

> The data in the request starts at byte 8, following the address in the
> flash. It is encrypted with the same repeating 8 bytes key.

> After changing the key with the A3 command, the memory is checked by
> resending the data. This works similarly as the A5 command. This is
> done that way because the flash can't be read when it's protected, so
> asking the device to do the compare is another way to check.

> 47-52-00-50 seems to be register 0x00040010.
> byte 1, bit 3: CFG_RESET_EN   (1=reset pin/button enabled)
>             6: CFG_BOOT_EN    (1=bootloader enabled)
>             7: CFG_ROM_READ   (0=disabled, 1=read allowed)

(Those config bits are for the CH579 capture.)

> Protocol used by WCHISPTool to flash a CH32F103C8T6 with a 2.3.1 bootloader.

> - If the first data bytes of the config are 0xff-0x00, the firmware is
>   write protected. It must be unprotected by writing 0xa5.

> - After flashing the last 56-byte block (or less than), the bootloader
>   must be sent a write with no data. Failing to do so results in a
>   failed firmware check.

> WCH chips expose similar protocol via UART1.
> The only difference is prefix and crc:
> 0x57 0xAB for request
> 0x55 0xAA for response

### README (fetch tool extraction)

> "the firmware has to be sent encrypted to the device" using "a simple fixed size xor key."

> "the last byte of the key is the sum of the first byte plus the chip type, it is not possible to create a fake key full of zeroes."

> "The program will make no attempt to recover from an error and will just exit."

Supported list (tool extraction) includes CH32V203, CH32V307 and CH32X035; CH32V305 is not named.
