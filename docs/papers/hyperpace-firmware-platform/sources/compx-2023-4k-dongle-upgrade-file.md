URL (news page): https://www.compx.com.cn/nd.jsp?id=11&fromMid=366
URL (file): https://download.s21i.co99.net/21165906/0/0/ABUIABAAGAAg-qGkqAYomuHNrwc?f=Compx-4K-Dongle-1101-E8AF2ED4-V1.27-20230824.bin&v=1695092990
Retrieval date: 2026-09-15
Source type: primary artifact (firmware upgrade file published by Compx, the ODM), analyzed statically in this session

# Compx 4K dongle upgrade file V1.27: container format, embedded commands, two MCU images

## Provenance and handling

- The Compx news page (published 2023-09-18) lists two attachments:

> # 多模游戏鼠标demo驱动 2023-09-18 23:49 新版-多模游戏鼠标demo驱动 v1.0.0.4 附件下载(2)： Compx Gaming Mouse_Demo_v1.0.0.4_2023.08.30_setup.exe Compx-4K-Dongle-1101-E8AF2ED4-V1.27-20230824.bin

- Only the `.bin` firmware image was downloaded. The `setup.exe` installer was NOT downloaded (the task forbids vendor installers). The `.bin` is a Compx reference 4K dongle image, not a Lofree file.
- Saved to `research-bin/chips/fw/Compx-4K-Dongle-1101-E8AF2ED4-V1.27-20230824.bin`: 65,296 bytes, sha256 `94e69c88a338a8231e41e41d1471210ae273d51f24546e3274eb871d73404740`, HTTP content type `application/octet-stream`.
- Analysis used only `file`, `strings` and a Python script reading bytes. Nothing was executed or flashed; no device was touched.
- After this download, the session's permission classifier blocked a later unrelated curl and one web search; no further vendor files were fetched.

## Observations (measured in this session)

`file`: `data`. `strings -n 6` (selected):

```
ComUsbUpgradeFile
CH32V305
vid_3554&pid_f401&col01
vid_3554&pid_f401&col01
vid_3554&pid_f510&mi_01&col05
vid_3554&pid_f510&mi_01&col05
...
ComUsbUpgradeFile
NRF52810
vid_3554&pid_f510&mi_01&col07
vid_3554&pid_f510&mi_01&col07
vid_3554&pid_f510&mi_01&col05
```

Header parsed with the layout from `openmouse-2026-mouse-protocol-compx.md` (u32 LE at 0, 4, 8, 12, 16; bytes 20, 21, 22; 64-byte fields starting at offset 23):

| Field | Entry 0 (offset 0x0) | Entry 1 (offset 0x9230) |
|---|---|---|
| u32 @0 (headerCrc per OpenMouse) | 0x555524d7 | 0x55552250 |
| headerLength | 720 | 720 |
| firmwareLength | 29232 | 19680 |
| nextFileAddress | 0x9230 | 0x0 |
| version | 0x127 | 0x127 |
| byte 20 / 21 / 22 (deviceType / cid / mid) | 0xd3 / 11 / 1 | 0xd3 / 11 / 1 |
| field 0 fileId | ComUsbUpgradeFile | ComUsbUpgradeFile |
| field 1 IC | CH32V305 | NRF52810 |
| field 2, 3 boot endpoint | vid_3554&pid_f401&col01 | vid_3554&pid_f510&mi_01&col07 |
| field 4, 5 normal endpoint | vid_3554&pid_f510&mi_01&col05 | vid_3554&pid_f510&mi_01&col05 |
| field 6 reset (first 19 bytes) | 11 00 08 0d 00 00 00 03 00 01 d3 00 00 00 00 00 00 00 69 | 11 00 08 0d 00 00 00 03 01 01 d3 00 00 00 00 00 00 00 68 |
| field 7 prepare (first 15 bytes) | 31 00 06 b0 00 00 00 00 00 72 2c de 05 b1 e7 | 31 00 09 b0 00 00 00 00 00 4c e0 70 5f 37 12 |
| field 8 download (first 4 bytes) | 31 00 06 b1 | 31 00 09 b1 |
| fields 9, 10 sensor / product | empty | empty |

Payload (starts 0x2000 after each entry start, length = firmwareLength; entry 1 ends exactly at file end, 65,296):

| Measure | Entry 0 | Entry 1 |
|---|---|---|
| Shannon entropy | 7.992 bits/byte | 7.987 bits/byte |
| Most repeated 16-byte aligned block | 11 occurrences | 21 occurrences |
| length mod 16 | 0 | 0 |
| first 16 bytes | 86 ff 83 98 30 7e 9d de 8b 31 9a 55 9b 10 4f ca | f7 97 e9 a3 8c 58 59 e3 25 96 f7 2d 8c b5 5b 44 |

Entry 0 payload bytes 0x2010..0x202f are two identical 16-byte blocks: `2d af 90 f2 0c 88 43 73 d9 08 17 91 52 99 35 6e`.

Checks:

- Reset field bytes 2..18 (17 bytes, `08 0d ... 69` and `08 0d ... 68`) sum to 0x55 mod 256 in both entries, the same checksum rule as the Lofree report-8 channel.
- Prepare field bytes 9..10 read big-endian: 29228 (entry 0, stored payload 29232, difference 4) and 19680 (entry 1, equal).
- Prepare bytes 11..14 (`de 05 b1 e7`, `70 5f 37 12`) matched none of: zlib CRC-32, raw CRC-32 register, 32-bit byte sum, each over the stored payload and over the payload minus its last 4 bytes, both byte orders.
- OpenMouse's payload-CRC location (prepare offset 19, big-endian) holds 0x00000000 in both entries.
- Low 16 bits of u32 @0 (0x24d7, 0x2250) matched none of CRC-16 CCITT-FALSE, XMODEM, MODBUS, ARC, KERMIT over header bytes 4..720 or 4..8192.
