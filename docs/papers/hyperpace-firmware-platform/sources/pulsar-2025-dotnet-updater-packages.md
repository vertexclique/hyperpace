URL: https://www.pulsar.gg/pages/download (Xlite v3 / 8K dongle firmware zips)
Retrieval date: 2026-09-15
Source type: vendor desktop updater packages, .NET IL decompilable (primary)

## Claim
Pulsar's desktop firmware updaters are .NET WinForms apps ("UpgradeTool") that carry one or two embedded Compx images and drive the same USB DFU protocol over Win32 HID. The IL is decompilable, so the flashing logic is fully readable. Two of these images use the HYPACE's exact chips.

## Packages (archived; see firmware/archive/SOURCES.md for sha256)
- Mouse_v2.18_1K_Dongle_v3.0.zip -> "(1) 1K Dongle Update v3.00.exe" (embeds CX52650N dongle v3.00, DeviceType 211, boot PID 0xF402, normal PID 0xF508) and "(2) Mouse Update v2.18.exe" (embeds NRF52833 mouse v2.18, DeviceType 210, boot PID 0xF400, normal PID 0xF507, senserName 3395). NRF52833 = the HYPACE mouse MCU; CX52650N = the HYPACE 1K/2K receiver. Same silicon, different product (Xlite v3), so not flashable to the HYPACE.
- Pulsar_1K_dongle_update_V0300.zip -> loose "Pulsar 1K dongle update V0300" tree WITH a full USB byte log (log/log.txt) capturing a real successful flash: ResetToBootMode write `11 00 08 0D ... 40`, WaitBootMode, then `06 B0 ...`/`06 B1 ...` data packets and device replies `07 5B B5 ..` ending `07 5B B5 88` (Success). cfg.ini `ToolName=Scyrox v8 Upgrade Tool`, `AlwaysUpgrade=1`, `Version=1.1`.
- X2F_8K_Dongle_Update.zip and Xlite_CrazyLight_Medium_8K_Dongle.zip -> two-image files (CH32V305 4K dongle main + NRF52820_8K/NRF52840_8K radio; and NRF52840 mouse), DeviceType 211/210, v4.07/v1024.

## Readable flashing logic (.NET IL saved)
`research-bin/vendor/pulsar/IL-Mouse-Update-v2.18.txt` (full) + `IL-UsbUpgradeFile-core.txt`. Type map: `UsbUpgradeFile` (methods GetHeaderCRC, IsValidHeader, CreateUpgradePacket, ResetToBootMode, WaitBootMode, DownloadFile, UpgradeHandler, FindBootDevices, ExistBootDevice), `Parameter` (StringToVersion, IntToVersion), `Usb_Cmd`/`Usb_Packet`/`UpgradeFileHeader` structs, `USBHIDControl.USBHID` (Win32 CreateFile/WriteFile/HidP_GetCaps). Packaging: PE32 .NET Framework v4.8, 3 sections; images stored as `.NET resource UpgradeTool.Resources.res.dev0/dev1.upgrade.bin`, extracted with dnfile.
- `Get_Device_Version` IL: reads current version via `GetUsbDeviceAttribute().VersionNumber` -> `IntToVersion` ("x.x02"); enables Upgrade button on `op_Inequality` with the file version (not greater-than); reads `AlwaysUpgrade` cfg key (=1). No signature check, no anti-rollback counter.
- `CreateUpgradePacket` IL builds `Cmd_DownLoadFile` packets from prepareDownLoadCmd/dataDownLoadCmd header fields, matching the WebHID `t6()`.
- Header endpoint strings prove the boot-mode identities: log shows `bootInputEndPoint:vid_3554&pid_f402&mi_01&col01`, `normalInputEndPoint:vid_3554&pid_f508...`.

## Relevance to Hyperpace
The v2.18 NRF52833 + v3.00 CX52650N pair is the nearest-silicon readable reference for the HYPACE. The IL confirms every framing/checksum/ordering detail seen in the WebHID JS, giving a second independent implementation to verify Hyperpace against.
