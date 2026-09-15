URL: https://scyrox.com/pages/download
Retrieval date: 2026-09-15
Source type: vendor desktop updater packages (primary)

## Claim
Scyrox V6/V8 are the same Compx platform (cid 62, matching the ODM sample cid in Lofree's /home/ template) and ship the same .NET UpgradeTool. Confirms the platform lineage and the "欣春旗" (Xinchunqi) ODM watermark.

## Evidence (archived; sha256 in firmware/archive/SOURCES.md)
- V8_Update_Tool_2.22.zip -> "V8 Mouse Upgrade Tool.exe" + embedded `NRF52840-FAF2F792-GMMouse-欣春旗V8-SCYROX-IR-无线8K-3950-PID-F5F6-V2.22-20250717_DLL升级.bin` (DeviceType 210 mouse, Cid 62 Mid 1, version v2.22, boot PID 0xF403, normal PID 0xF5F6, senserName 3395).
- V6_Update_Tool__EN.zip -> same tool, image Cid 62 Mid 2 v2.22.
- cfg.ini `ToolName=Scyrox v8 Upgrade Tool`, `AlwaysUpgrade=1`. en.xml labels "Current FW Version:", "New FW Version:", "Tool Version:". Upgrade Tool Manual.docx verbatim: "Extract the folder. Open the 'V8 Mouse Upgrade Tool.' Use a Type-C cable ... Click the 'Upgrade' button ... When the button displays 'Success', the device upgrade is complete."
- log.txt captures a full flash identical in shape to Pulsar's (ResetToBootMode `11 00 08 0D ... 40`, `06 B0`/`06 B1` packets, `06 5B B5 88` Success).
- The "欣春旗" watermark and cid 62 match the sample `upgrade` block cid 62 baked into Lofree's ODM /home/ template, tying Lofree, Pulsar and Scyrox to one ODM (Compx / Shenzhen Xinchunqi).

## Relevance to Hyperpace
Third independent instance of the identical protocol and packaging; confirms the format is ODM-wide, not brand-specific. Scyrox mouse is NRF52840 (not the HYPACE's NRF52833), so images are not HYPACE-flashable.
