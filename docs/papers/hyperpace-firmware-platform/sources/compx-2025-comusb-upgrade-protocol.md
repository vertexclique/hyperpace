URL: (derived from vendor artifacts below; no single web page)
Retrieval date: 2026-09-15
Source type: vendor firmware images + three independent readable updater implementations (primary)

## Claim
The HYPACE's silicon (NRF52833 mouse, CX52650N and CH32V305 receivers) is updated by the Compx "ComUsbUpgradeFile" USB HID DFU protocol. The complete framing, chunking, checksum, verify and version-check logic is recoverable from readable vendor code and is documented below with file:line references. This is the protocol Hyperpace must reproduce; verification is code-to-code against these files.

## Readable updater implementations saved (scratch research-bin/vendor/)
1. WebHID JS, deminified, Lofree ODM template: `lofree.tech-home/index.pretty.js` (functions listed below). Cleanest reference.
2. WebHID JS, deminified, Pulsar web driver: `pulsar-bbb/sub_cMouse/app.pretty.js` and the extracted engine `pulsar-bbb/sub_cMouse/UPDATER-ENGINE-webhid.js` (lines 1040-1300 of the bundle).
3. .NET IL (decompilable, C#), Pulsar Xlite v3 desktop updater: `pulsar/IL-Mouse-Update-v2.18.txt` (full) and `pulsar/IL-UsbUpgradeFile-core.txt` (core methods). Ground-truth USB byte log: `pulsar/x_Pulsar_1K_dongle_update_V0300/.../log/log.txt`.
4. Independent TS re-implementation (cross-check): `openmouse-ref/atk-index.ts` (`atkParseCompxFirmware`, `atkCompxPayloadCrc`).

## Package format (the .bin image)
Header at offset 0 (720 bytes), firmware payload at fixed offset 8192 (`$v = 8192`). Header (little-endian u32 unless noted), from `Kv()` in index.pretty.js:5244 and `atkParseCompxFirmware` atk-index.ts:544:
- [0] headCRC, [4] headLength (=720), [8] fwLength, [12] nextFileAddress, [16] version (major<<8 | minor, shown `v%d.%02x`), [20] DeviceType (210 = mouse, 211 = dongle/receiver), [21] Cid, [22] Mid.
- Then eleven 64-byte NUL-terminated ASCII fields at [23 + 64*i]: 0 fileId ("ComUsbUpgradeFile"), 1 icName (NRF52833 / CX52650N / CH32V305 ...), 2 bootInputEndPoint, 3 bootOutputEndPoint, 4 normalInputEndPoint, 5 normalOutputEndPoint, 6 resetToUpdateModeCmd, 7 prepareDownLoadCmd, 8 dataDownLoadCmd, 9 senserName, 10 productName.
- Endpoint fields encode "vid_XXXX&pid_XXXX&mi_01&col0N" and are parsed by `qv()` (index.pretty.js:5324) to derive the boot-mode and normal-mode VID/PID.
- Multi-image chaining: if nextFileAddress != 0 a second complete image follows at that offset (dongle chip + its 8K radio MCU, or CH32V305 + NRF52840_8K). Handled by `Gv()` UpgradeHandleInit (index.pretty.js:5339).

### Header CRC (verify)
`Wv(e,t)` index.pretty.js:5309: `r = sum(bytes[8 .. headLength]); return 1431655765 - r`. 1431655765 = 0x55555555. `jv()` checks `headCRC == Wv(...)`. Confirmed True for every archived image.

### Payload CRC (verify)
Embedded in prepareDownLoadCmd bytes [5..8] big-endian. Formula `atkCompxPayloadCrc` (atk-index.ts:530): CRC-32, init 0xFFFFFFFF, no final XOR. Verified matching for the CX52650N/CX52660 dongle payloads; does NOT match under that scheme for the NRF/CH32 payloads (payload region or variant differs) - flagged unverified for those. The .NET tool reads this field as the download start/length control, not as a gate.

## The wire protocol (framing, chunk, transaction)
Report ID 8, 64-byte reports here (`Ve = o1 = 64`), distinct from the 16-byte config protocol. Boot chunk size 32 bytes/packet.

- resetToUpdateModeCmd (enter bootloader): report `11 00 08 0D 00 ... 40` i.e. config command 0x0D = EnterUsbUpdateMode sent on report 8. `Ql()` index.pretty.js:5411 (.NET `ResetToBootMode` IL ~line 3488). Device re-enumerates under the bootloader PID (mouse boot PID 0xF400/0xF403, 1K dongle 0xF402, 4K/8K dongle 0xF408/0xF40A).
- WaitBootMode: poll HID device list until the boot PID appears and is stable (.NET `WaitBootMode`).
- prepareDownLoadCmd: first packet `[cmd0, 0xB0, ... start-addr, crc]` announces the download (`Cmd_PrepareDownLoad = 176`).
- dataDownLoadCmd chunk builder `t6()` index.pretty.js:5377 (.NET `CreateUpgradePacket` IL ~3151): for each 32-byte slice, `buffer[0]=dataCmd, buffer[1]=Cmd_DownLoadFile(177), buffer[2]=Usb_NextPacket/Usb_LastPacket(193), buffer[3]=len, buffer[5..8]=addr big-endian, buffer[17..17+len]=payload bytes (zero/0xFF padded)`. Address advances by 32.
- Send loop `lo()` index.pretty.js:5463 (.NET `DownloadFile` IL ~3870): send packet, await input report, `eh()` (index.pretty.js:5497) parses the reply: `Cmd_DeviceState(91)` with subcodes DeviceState_Earse_Backup/Main (erase progress), DeviceState_CheckOK(5), DeviceState_Success(136 = 0x88) -> Mt.Success. On mismatch, retry (RepeatCountMax after 8).
- Payload bytes are sent to the bootloader VERBATIM from the image; they appear encrypted/scrambled at rest (NRF payload entropy ~7.998, no valid ARM vector table; first bytes of image byte-for-byte equal the first data packet in the .NET USB log), so the bootloader decrypts. No asymmetric signature is present; integrity is the header CRC plus the per-packet echo/state handshake.

## Multi-target ORDER and version-check
- Order: dongle/receiver first, then mouse. Verbatim from Pulsar `Read-me-Firmware-Update-Steps.txt`: "Update the dongle firmware before updating the product firmware. If you update the product firmware first, you may need to re-pair the device." hyzen.lofree.tech string: `upgradeOrderInvalid: "dongle must be upgraded first in multi-target upgrades"`. Multi-image files (nextFileAddress) upgrade the dongle radio MCU then the dongle main MCU in one file.
- Version comparison (WebHID) `Check_New_Version` = function `W(e,t)` in the bundles: splits "vX.YZ" on ".", compares componentwise, returns true only when candidate > current; returns false for equal or older -> the web driver never offers a downgrade, but there is NO cryptographic anti-rollback and no rollback counter.
- Version comparison (.NET) `FormMain.Get_Device_Version` IL (IL-Mouse-Update-v2.18.txt): reads `AlwaysUpgrade` from cfg.ini (set to 1), enables the Upgrade button when `currentVersion op_Inequality upgradeInfo.version` (not-equal, not greater-than), so the desktop tool re-flashes even the same or an older version. `Parameter.IntToVersion` formats `(v>>8).ToString("x") + "." + (v&0xff).ToString("x02")`.

## Relevance to Hyperpace
This is the exact update state machine Hyperpace must implement for the HYPACE. Chip families match (NRF52833 mouse, CX52650N 1K/2K receiver, CH32V305 4K receiver). The three readable implementations agree byte-for-byte, so Hyperpace's flasher can be verified line-by-line against them without touching hardware. The unresolved items requiring a real HYPACE image: the HYPACE mouse boot PID, its cid/mid (102/1-2), and the payload encryption/CRC coverage for the NRF variant.
