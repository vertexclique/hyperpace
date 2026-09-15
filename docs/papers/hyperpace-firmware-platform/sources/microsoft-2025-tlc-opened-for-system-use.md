URL: https://learn.microsoft.com/en-us/windows-hardware/drivers/hid/top-level-collections-opened-by-windows-for-system-use
Retrieved: 2026-09-15
Source type: Official reference documentation (Microsoft Learn, HID drivers). Page metadata: ms.date 2024-01-11, updated_at 2025-07-18.
Retrieval method: WebFetch (page converted to markdown; text below is the returned page text).
Character note: the en-dash in "0x00 - 0xFF" was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

> Windows opens the following top-level collections for system use:

| Usage page | Usage ID | Device type | Access mode |
| --- | --- | --- | --- |
| 0x0001 | 0x0001 - 0x0002 | Mouse | Exclusive |
| 0x0001 | 0x0004 - 0x0005 | Game controllers | Shared |
| 0x0001 | 0x0006 - 0x0007 | Keyboard | Exclusive |
| 0x0001 | 0x000C | Flight mode switch | Shared |
| 0x0001 | 0x0080 | System controls (power) | Shared |
| 0x000C | 0x0001 | Consumer controls | Shared |
| 0x000D | 0x0001 | External pen device | Exclusive |
| 0x000D | 0x0002 | Integrated pen device | Exclusive |
| 0x000D | 0x0004 | Touchscreen | Exclusive |
| 0x000D | 0x0005 | Precision touchpad (PTP) | Exclusive |
| 0x0020 | *Multiple | Sensors | Shared |
| 0x0084 | 0x0004 | HID UPS battery | Shared |
| 0x008C | 0x0002 | Barcode scanner | Shared |

> *Multiple: Sensors usages from 0x00 - 0xFF are segmented for different purposes.
