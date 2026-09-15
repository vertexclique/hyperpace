# SUMMARY: Pulsar .NET desktop updaters (decompilable flashing logic)

- Claim: Pulsar's desktop firmware updaters are decompilable .NET WinForms apps driving the same Compx DFU over Win32 HID; two embedded images use the HYPACE's exact chips (NRF52833, CX52650N).
- Method: Downloaded 4 zips, extracted with bsdtar, identified PE32 .NET v4.8, extracted embedded images with dnfile, disassembled IL with dncil (UsbUpgradeFile core methods + FormMain.Get_Device_Version), read the bundled real USB flash log.
- Result: Full method set (ResetToBootMode/WaitBootMode/CreateUpgradePacket/DownloadFile) recovered; version check uses op_Inequality + AlwaysUpgrade=1 (re-flashes any version, no signature/rollback). Boot/normal PIDs confirmed from header endpoint strings and the log.
- Evidence tier: 1 (vendor packages, decompilable).
- Performance: same 32 B/packet protocol; log shows full mouse flash in ~10s.
- Correctness: IL matches the WebHID JS byte for byte. Not HYPACE-flashable (Xlite v3, different cid/PID).
- Relevance: Second independent readable implementation for code-to-code verification; nearest-silicon reference (NRF52833 + CX52650N).
