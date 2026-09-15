# HYPACE firmware archive: source ledger

Retrieval date: 2026-09-15. Static analysis only; no file was executed and nothing was sent to any USB device. sha256 values are stable across the reorg; only paths changed.

## Layout and honesty

- `mouse/`, `receiver-1k/`, `receiver-2k/`, `receiver-4k/`, `receiver-8k/`: reserved for images with concrete evidence they target the operator's hardware (VID 0x3554 with PID 0xFB16/0xFB14, cid 102, HYPACE naming, or a Lofree-published link). ALL ARE EMPTY: no such image was found anywhere (see sources/lofree-2026-web-driver-no-firmware.md).
- `other-devices/<brand-model>/<version>/`: firmware and updater packages for OTHER products on the same Compx ODM platform. Reference material for the update protocol ONLY. These MUST NEVER be bundled with or flashed to the HYPACE: their cid/mid, normal-mode PID and payload differ, so they would mis-target the hardware.
- `other-devices/_web-driver-snapshots/`: readable web-driver page + JS + cfg snapshots (code, not firmware), kept for durable protocol reference.
- No `unknown/`: every archived image's own product is established (Pulsar or Scyrox). What is UNestablished is any link to the HYPACE, which is why none sit under mouse/ or receiver-*/.

## Attached HYPACE passive descriptor read (sysfs cache; nothing sent)

`/sys/bus/usb/devices`: `Compx Hypace@Lofree`, VID 0x3554, wired PID 0xFB14 bcdDevice=0x0300, receiver PID 0xFB16 bcdDevice=0x0216. bcdDevice is the USB release field, not a proven firmware version; 0x0216 vs the archived Lofree receiver string `v2.16` and 0x0300 vs a `v3.00`-class build are unverified correlations. These are the identity bytes Hyperpace should match against, but no image here carries them (all foreign PIDs).

## Files (identity markers are the bytes Hyperpace matches images to hardware with)

### other-devices/pulsar-1k-dongle-tool/v3.00/CX52650N-GMMAX-Dongle-欣春旗IM20A-新协议-PID-F508-TX7DBM_V3.00-6cc13abf-聚成-20250314_DLL升级.bin
- size 43104  sha256 `93068709d187e828af1e03cbc5f9a22f483dc5629f3a595a9d3ff846679564a4`
- original URL: (embedded resource; see sibling package row)
- packaging / extraction: extracted from .NET updater resource (dnfile) OR from zip (bsdtar)
- image[0] @off 0, fwLength 34912: magic=ComUsbUpgradeFile; headCRC=0x555522ff(ok); payloadCRC=0x00008860; ver=v3.00(raw 768); DeviceType=211(dongle); cid/mid=0/0; icName=CX52650N; sensor=; normal=vid_3554&pid_f508&mi_01&col05; boot=vid_3554&pid_f402&mi_01&col02
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-1k-dongle-tool/v3.00/Pulsar_1K_dongle_update_V0300.zip
- size 353263  sha256 `fb9a28def512bf63cbaf698c0054dece62f83681b82da341480be8d35b9acb91`
- original URL: https://www.pulsar.gg/cdn/shop/files/Pulsar_1K_dongle_update_V0300.zip
- packaging / extraction: downloaded from pulsar.gg; bsdtar -x (plain zip)
- type: zip package (PE32 .NET Framework v4.8 WinForms 'UpgradeTool' inside; images stored as .NET resources res.dev0/dev1.upgrade.bin). Not itself an image.
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-crazylight-8k/v4.07/Xlite_CrazyLight_Medium+8K_Dongle.exe__res.dev0.upgrade.bin
- size 63184  sha256 `42ae74b67a787d2002a883dc030f67bbe4382ff663b8c1801ae80a5a77b758f8`
- original URL: (embedded resource; see sibling package row)
- packaging / extraction: extracted from .NET updater resource (dnfile) OR from zip (bsdtar)
- image[0] @off 0, fwLength 29584: magic=ComUsbUpgradeFile; headCRC=0x555524f2(ok); payloadCRC=0x0000738c; ver=v4.00(raw 1024); DeviceType=211(dongle); cid/mid=0/0; icName=CH32V305; sensor=; normal=vid_3710&pid_5406&mi_01&col05; boot=vid_3554&pid_f40a&col01
- image[1] @off 37776, fwLength 17216: magic=ComUsbUpgradeFile; headCRC=0x555522e2(ok); payloadCRC=0x00004340; ver=v4.00(raw 1024); DeviceType=211(dongle); cid/mid=0/0; icName=NRF52820_8K; sensor=; normal=vid_3710&pid_5406&mi_01&col05; boot=vid_3710&pid_5406&mi_01&col07
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-crazylight-8k/v4.07/Xlite_CrazyLight_Medium+8K_Dongle.exe__res.dev1.upgrade.bin
- size 101728  sha256 `861a6cd1fdd2febd521a28848e807d204bf5b9543375edce46ead0820a04dc97`
- original URL: (embedded resource; see sibling package row)
- packaging / extraction: extracted from .NET updater resource (dnfile) OR from zip (bsdtar)
- image[0] @off 0, fwLength 93536: magic=ComUsbUpgradeFile; headCRC=0x555526f1(ok); payloadCRC=0x00016d58; ver=v4.07(raw 1031); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3950; normal=vid_3710&pid_3525&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-crazylight-8k/v4.07/Xlite_CrazyLight_Medium_8K_Dongle.zip
- size 676711  sha256 `eb19980d77e9b1b499d509f16f5a8b237c179b3fee43a98740ee19cb51d508f1`
- original URL: https://www.pulsar.gg/cdn/shop/files/Xlite_CrazyLight_Medium_8K_Dongle.zip
- packaging / extraction: downloaded from pulsar.gg; bsdtar -x (plain zip)
- type: zip package (PE32 .NET Framework v4.8 WinForms 'UpgradeTool' inside; images stored as .NET resources res.dev0/dev1.upgrade.bin). Not itself an image.
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-dongle/v2.08/Dongle-CD01-V2.08-61cd8c18.bin
- size 226944  sha256 `2309311bb305f0a20fd12316bcb94df561a60eb8844345f08c7021704fde0d6b`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Dongle-CD01-V2.08-61cd8c18.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 218752: magic=ComUsbUpgradeFile; headCRC=0x5555230b(ok); payloadCRC=0x00035680; ver=v2.08(raw 520); DeviceType=211(dongle); cid/mid=0/0; icName=CX52660; sensor=; normal=vid_3710&pid_5502&mi_01&col05; boot=vid_3554&pid_f40c&mi_01&col02
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-dongle/v2.24/8K-Dongle-3710-5406-V2.24-DC7B13B0.bin
- size 62736  sha256 `3a8115870f4d1369fbf16c4720ed02ddbc8178fd38307e3b5a89f77510ca1fd6`
- original URL: https://bbb.pulsar.gg/cMouse/bin/8K-Dongle-3710-5406-V2.24-DC7B13B0.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 29584: magic=ComUsbUpgradeFile; headCRC=0x55552456(ok); payloadCRC=0x0000738c; ver=v2.24(raw 548); DeviceType=211(dongle); cid/mid=14/3; icName=CH32V305; sensor=; normal=vid_3710&pid_5406&mi_01&col05; boot=vid_3554&pid_f40a&col01
- image[1] @off 37776, fwLength 16768: magic=ComUsbUpgradeFile; headCRC=0x55552111(ok); payloadCRC=0x0000417c; ver=v2.24(raw 548); DeviceType=211(dongle); cid/mid=14/3; icName=NRF52820_8K; sensor=; normal=vid_3710&pid_5406&mi_01&col05; boot=vid_3710&pid_5406&mi_01&col07
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-dongle/v2.25/8K-Dongle-3710-5406-V2.25-213DD3B7.bin
- size 62720  sha256 `8d35046a21a4aade461238d1febaa83cb63639665c2f813ff5719a8dfc84398b`
- original URL: https://bbb.pulsar.gg/cMouse/bin/8K-Dongle-3710-5406-V2.25-213DD3B7.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 29584: magic=ComUsbUpgradeFile; headCRC=0x555524ab(ok); payloadCRC=0x0000738c; ver=v2.25(raw 549); DeviceType=211(dongle); cid/mid=0/0; icName=CH32V305; sensor=; normal=vid_3710&pid_5406&mi_01&col05; boot=vid_3554&pid_f40a&col01
- image[1] @off 37776, fwLength 16752: magic=ComUsbUpgradeFile; headCRC=0x55552183(ok); payloadCRC=0x0000416c; ver=v2.25(raw 549); DeviceType=211(dongle); cid/mid=0/0; icName=NRF52820_8K; sensor=; normal=vid_3710&pid_5406&mi_01&col05; boot=vid_3710&pid_5406&mi_01&col07
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-dongle/v4.07/Dongle-CD03-V4.07-8b781788.bin
- size 213632  sha256 `2bc97353cb1666374465068b4a6ddf1db05a06e83a6ff623419442a9b82404c6`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Dongle-CD03-V4.07-8b781788.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 205440: magic=ComUsbUpgradeFile; headCRC=0x55552387(ok); payloadCRC=0x00032280; ver=v4.07(raw 1031); DeviceType=211(dongle); cid/mid=0/0; icName=CX52660; sensor=3311; normal=vid_3710&pid_5501&mi_01&col05; boot=vid_3554&pid_f40c&mi_01&col02
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-dongle/v5.04/Dongle-CD03-v5.04-5ED8B972.bin
- size 91808  sha256 `522ac186e14b3bfb131c3da8ea42dbf48912d527f2aa905044f9cfea9eeb107a`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Dongle-CD03-v5.04-5ED8B972.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 19360: magic=ComUsbUpgradeFile; headCRC=0x5555268f(ok); payloadCRC=0x00004b94; ver=v5.04(raw 1284); DeviceType=211(dongle); cid/mid=0/0; icName=CX364; sensor=; normal=vid_3710&pid_5504&mi_01&col05; boot=vid_3710&pid_f005&col01
- image[1] @off 27552, fwLength 56064: magic=ComUsbUpgradeFile; headCRC=0x555522dd(ok); payloadCRC=0x0000db00; ver=v5.04(raw 1284); DeviceType=211(dongle); cid/mid=0/0; icName=NRF54L05_8K; sensor=; normal=vid_3710&pid_5504&mi_01&col05; boot=vid_3710&pid_5504&mi_01&col07
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.03/8K-Mouse-X2F-5th-V3.03-4064453F.bin
- size 139776  sha256 `3f154ee95bbf74714a165ff3ca4ef18db4c8a08f68e0b431e704644c4ea45dae`
- original URL: https://bbb.pulsar.gg/cMouse/bin/8K-Mouse-X2F-5th-V3.03-4064453F.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 26064: magic=ComUsbUpgradeFile; headCRC=0x5555235a(ok); payloadCRC=0x000065d0; ver=v3.03(raw 771); DeviceType=210(mouse); cid/mid=0/0; icName=CH32V305; sensor=3395; normal=vid_3710&pid_7505&mi_01&col05; boot=vid_3554&pid_f408&col01
- image[1] @off 34256, fwLength 97328: magic=ComUsbUpgradeFile; headCRC=0x55552233(ok); payloadCRC=0x00017c28; ver=v3.03(raw 771); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840_8K; sensor=3395; normal=vid_3710&pid_7505&mi_01&col05; boot=vid_3710&pid_7505&mi_01&col07
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.03/8K-Mouse-X2F-V3.03-2C2516F4.bin
- size 139760  sha256 `eb6ea97c57a7c3bad6504de0da1345856ac4be06d58dafb8a1f1122f3fa7c0d2`
- original URL: https://bbb.pulsar.gg/cMouse/bin/8K-Mouse-X2F-V3.03-2C2516F4.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 26048: magic=ComUsbUpgradeFile; headCRC=0x555522f5(ok); payloadCRC=0x000065b8; ver=v0.00(raw 0); DeviceType=210(mouse); cid/mid=0/0; icName=CH32V305; sensor=3395; normal=vid_3710&pid_7505&mi_01&col05; boot=vid_3554&pid_f408&col01
- image[1] @off 34240, fwLength 97328: magic=ComUsbUpgradeFile; headCRC=0x55552239(ok); payloadCRC=0x00017c28; ver=v0.00(raw 0); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840_8K; sensor=3395; normal=vid_3710&pid_7505&mi_01&col05; boot=vid_3710&pid_7505&mi_01&col07
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.03/Mouse-DM169-3770-3415-2E82C72F-V3.03.bin
- size 99104  sha256 `a20ef6f4a998d01a665a963a8312750b736fdd534eefac5e48c3553ab71cc185`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-DM169-3770-3415-2E82C72F-V3.03.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 90912: magic=ComUsbUpgradeFile; headCRC=0x555526b4(ok); payloadCRC=0x00016318; ver=v3.03(raw 771); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3710&pid_3415&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.03/Mouse-DM175-3710-3506-D60C3DFA-V3.03.bin
- size 99104  sha256 `8b892a5c9a75ec2e3043c605c9d3e6f951846509632ef5fd2b0baf9bd0c39c0f`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-DM175-3710-3506-D60C3DFA-V3.03.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 90912: magic=ComUsbUpgradeFile; headCRC=0x55552725(ok); payloadCRC=0x00016318; ver=v3.03(raw 771); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3710&pid_3506&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.03/Mouse-DM176-3710-3507-F8A90748-V3.03.bin
- size 99104  sha256 `1090897c870fd19ca826f11ec50a999db0689a0f454de312afcfd81f750f518a`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-DM176-3710-3507-F8A90748-V3.03.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 90912: magic=ComUsbUpgradeFile; headCRC=0x555526fa(ok); payloadCRC=0x00016318; ver=v3.03(raw 771); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3710&pid_3507&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.03/Mouse-DM177-3710-3508-4094A0A5-V3.03.bin
- size 99104  sha256 `f9b874b2a21266127a9751688f2f7050e364bcabe0566ebb92b36933799c365e`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-DM177-3710-3508-4094A0A5-V3.03.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 90912: magic=ComUsbUpgradeFile; headCRC=0x55552721(ok); payloadCRC=0x00016318; ver=v3.03(raw 771); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3710&pid_3508&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.03/Mouse-DM179-3770-3510-257127B4-V3.03.bin
- size 99104  sha256 `b89e75b485076c6b7193f654cb53c549506f50d9a2843ff322c51d894127c2cd`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-DM179-3770-3510-257127B4-V3.03.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 90912: magic=ComUsbUpgradeFile; headCRC=0x55552687(ok); payloadCRC=0x00016318; ver=v3.03(raw 771); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3710&pid_3510&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.03/Mouse-IM31-3710-7501-AF27F237-V3.03.bin
- size 99376  sha256 `ce59825e9423d867b968774883e316fd3787a43c671bd0e699ec45a4f360499e`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-IM31-3710-7501-AF27F237-V3.03.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 91184: magic=ComUsbUpgradeFile; headCRC=0x555526bb(ok); payloadCRC=0x00016424; ver=v0.00(raw 0); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3710&pid_7501&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.03/Mouse-IM33-3710-7504-V3.03-9B684E62.bin
- size 99376  sha256 `17070b950b3fad9ca06a8964c8fdf4d58b6c74e4a3823ce37607e7b12d5bf047`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-IM33-3710-7504-V3.03-9B684E62.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 91184: magic=ComUsbUpgradeFile; headCRC=0x555525d1(ok); payloadCRC=0x00016424; ver=v0.00(raw 0); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3554&pid_f50f&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.03/Mouse-IM41-3710-7503-V3.03-D1D7EBE7.bin
- size 99392  sha256 `470d8fc13176781edfd57ca7ae8394540c13a2facfb4eb298b093459cf1cb5ff`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-IM41-3710-7503-V3.03-D1D7EBE7.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 91200: magic=ComUsbUpgradeFile; headCRC=0x55552601(ok); payloadCRC=0x00016434; ver=v0.00(raw 0); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3554&pid_f50f&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.04/Mouse-DM161-3710-3414-B65B80B6-V3.04.bin
- size 99104  sha256 `3110f112086a539be35aeb79e6c6690ef9f6f48b72f3944a7dd642798d9a2e36`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-DM161-3710-3414-B65B80B6-V3.04.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 90912: magic=ComUsbUpgradeFile; headCRC=0x55552756(ok); payloadCRC=0x00016318; ver=v3.04(raw 772); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3710&pid_3414&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.04/Mouse-DM176-3710-3507-04A90E51-V3.04.bin
- size 99104  sha256 `762c049db312cd5cf42840ca821fa2ad80da527c7a509b5f7540c10eb60472ff`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-DM176-3710-3507-04A90E51-V3.04.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 90912: magic=ComUsbUpgradeFile; headCRC=0x55552615(ok); payloadCRC=0x00016318; ver=v3.04(raw 772); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3710&pid_3507&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.04/Mouse-DM177-3710-3508-A8094761-V3.04.bin
- size 99104  sha256 `68ce1bd2f8d072df5af6481dc219d6d2b5a3d47fa7bd42d0dd1041e80522990b`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-DM177-3710-3508-A8094761-V3.04.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 90912: magic=ComUsbUpgradeFile; headCRC=0x55552660(ok); payloadCRC=0x00016318; ver=v3.04(raw 772); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3710&pid_3508&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.04/Mouse-DM178-3710-3509-F44C45CF-V3.04.bin
- size 99104  sha256 `fa360ee17072c521d362b480b1b47317c7335054a1b746117b3a244c896da65d`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-DM178-3710-3509-F44C45CF-V3.04.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 90912: magic=ComUsbUpgradeFile; headCRC=0x55552759(ok); payloadCRC=0x00016318; ver=v3.04(raw 772); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3710&pid_3509&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.04/Mouse-DM188-3710-3528-V3.04.bin
- size 99104  sha256 `a6327396ae19c2d1b57f0cf372d2a34498b0988e5d8aefe8cc759fb4e9b6d2a8`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-DM188-3710-3528-V3.04.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 90912: magic=ComUsbUpgradeFile; headCRC=0x55552840(ok); payloadCRC=0x00016318; ver=v3.04(raw 772); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3710&pid_3528&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.04/Mouse-IM33-3710-7504-V3.04-9B684E62.bin
- size 99376  sha256 `391d7f763b5a4d7052a410a49b3162249c76dfe0370010c1473a78dfe1c0fdf3`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-IM33-3710-7504-V3.04-9B684E62.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 91184: magic=ComUsbUpgradeFile; headCRC=0x555525ee(ok); payloadCRC=0x00016424; ver=v0.00(raw 0); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3554&pid_f50f&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.04/Mouse-IM41-3395-3710-7503-22A4F371-V3.04.bin
- size 99392  sha256 `259634d4bc93785cb56c1525128ddbb5e4585fc62ef72102d64a8821973d0805`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-IM41-3395-3710-7503-22A4F371-V3.04.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 91200: magic=ComUsbUpgradeFile; headCRC=0x555525bb(ok); payloadCRC=0x00016434; ver=v3.04(raw 772); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3710&pid_7503&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.05/Mouse-DM161-3710-3414-63CC886F-V3.05.bin
- size 99104  sha256 `11bea44f404240c82350a4405f1212328b580964c7fc6001df8a7b0db13d4861`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-DM161-3710-3414-63CC886F-V3.05.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 90912: magic=ComUsbUpgradeFile; headCRC=0x55552734(ok); payloadCRC=0x00016318; ver=v3.05(raw 773); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3710&pid_3414&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v3.05/Mouse-DM185-3395-3710-3525-643A674C-V3.05.bin
- size 99104  sha256 `d3ba091d6bcb6513488306b6772f13cae0b7f26e6ea04c496913586b308013ea`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-DM185-3395-3710-3525-643A674C-V3.05.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 90912: magic=ComUsbUpgradeFile; headCRC=0x55552655(ok); payloadCRC=0x00016318; ver=v3.05(raw 773); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840; sensor=3395; normal=vid_3710&pid_3518&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v4.01/Mouse-Merge-554-V4.01-F83AD0FF.bin
- size 141088  sha256 `6e44d325052c577c32a938e905442400cd4a6efe1979b76143224696a183a69b`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-Merge-554-V4.01-F83AD0FF.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 26096: magic=ComUsbUpgradeFile; headCRC=0x555523b4(ok); payloadCRC=0x000065e4; ver=v4.01(raw 1025); DeviceType=210(mouse); cid/mid=0/0; icName=CH32V305; sensor=3395; normal=vid_3710&pid_7510&mi_01&col05; boot=vid_3554&pid_f408&col01
- image[1] @off 34288, fwLength 98608: magic=ComUsbUpgradeFile; headCRC=0x55552232(ok); payloadCRC=0x00018124; ver=v4.01(raw 1025); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840_8K; sensor=3395; normal=vid_3710&pid_7510&mi_01&col05; boot=vid_3710&pid_7510&mi_01&col07
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-web-mouse/v5.09/Mouse-fs01-V5.09-55A3EE22.bin
- size 309120  sha256 `2cc916b4b70bc8ab69e24fdbf803dc8771b1aaccc1b105befb9211b73f0a0805`
- original URL: https://bbb.pulsar.gg/cMouse/bin/Mouse-fs01-V5.09-55A3EE22.bin
- packaging / extraction: downloaded via web-driver XHR from bbb.pulsar.gg/cMouse/bin/
- image[0] @off 0, fwLength 17488: magic=ComUsbUpgradeFile; headCRC=0x55552635(ok); payloadCRC=0x0000444c; ver=v5.09(raw 1289); DeviceType=210(mouse); cid/mid=0/0; icName=CX364; sensor=3311; normal=vid_3710&pid_7507&mi_01&col05; boot=vid_3710&pid_f004&col01
- image[1] @off 25680, fwLength 275248: magic=ComUsbUpgradeFile; headCRC=0x555521a8(ok); payloadCRC=0x00043330; ver=v5.09(raw 1289); DeviceType=210(mouse); cid/mid=0/0; icName=NRF54L15_8K; sensor=3311; normal=vid_3710&pid_7507&mi_01&col05; boot=vid_3710&pid_7507&mi_01&col07
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-x2f-8k/v4.07/X2F+8K Dongle Update.exe__res.dev0.upgrade.bin
- size 63184  sha256 `42ae74b67a787d2002a883dc030f67bbe4382ff663b8c1801ae80a5a77b758f8`
- original URL: (embedded resource; see sibling package row)
- packaging / extraction: extracted from .NET updater resource (dnfile) OR from zip (bsdtar)
- image[0] @off 0, fwLength 29584: magic=ComUsbUpgradeFile; headCRC=0x555524f2(ok); payloadCRC=0x0000738c; ver=v4.00(raw 1024); DeviceType=211(dongle); cid/mid=0/0; icName=CH32V305; sensor=; normal=vid_3710&pid_5406&mi_01&col05; boot=vid_3554&pid_f40a&col01
- image[1] @off 37776, fwLength 17216: magic=ComUsbUpgradeFile; headCRC=0x555522e2(ok); payloadCRC=0x00004340; ver=v4.00(raw 1024); DeviceType=211(dongle); cid/mid=0/0; icName=NRF52820_8K; sensor=; normal=vid_3710&pid_5406&mi_01&col05; boot=vid_3710&pid_5406&mi_01&col07
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-x2f-8k/v4.07/X2F+8K Dongle Update.exe__res.dev1.upgrade.bin
- size 142160  sha256 `b60c6b30db1b401d6d0ffb93d20213ac07bcc7cb64dc6ba05546f6c0eb10d5b2`
- original URL: (embedded resource; see sibling package row)
- packaging / extraction: extracted from .NET updater resource (dnfile) OR from zip (bsdtar)
- image[0] @off 0, fwLength 26048: magic=ComUsbUpgradeFile; headCRC=0x55552431(ok); payloadCRC=0x000065b8; ver=v4.07(raw 1031); DeviceType=210(mouse); cid/mid=0/0; icName=CH32V305; sensor=; normal=vid_3710&pid_7505&mi_01&col05; boot=vid_3554&pid_f408&col01
- image[1] @off 34240, fwLength 99728: magic=ComUsbUpgradeFile; headCRC=0x55552127(ok); payloadCRC=0x0001858c; ver=v4.07(raw 1031); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52840_8K; sensor=; normal=vid_3710&pid_7505&mi_01&col05; boot=vid_3710&pid_7505&mi_01&col07
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-x2f-8k/v4.07/X2F_8K_Dongle_Update.zip
- size 713599  sha256 `e20c789f1baf9d1b6cfaf106b11a1645514f9298bb8a8d7309da20ba05fcdde6`
- original URL: https://www.pulsar.gg/cdn/shop/files/X2F_8K_Dongle_Update.zip
- packaging / extraction: downloaded from pulsar.gg; bsdtar -x (plain zip)
- type: zip package (PE32 .NET Framework v4.8 WinForms 'UpgradeTool' inside; images stored as .NET resources res.dev0/dev1.upgrade.bin). Not itself an image.
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-xlite-v3/Mouse_v2.18_1K_Dongle_v3.0.zip
- size 1165200  sha256 `6324fcc9b886c9bb3b074a3f1040e89bec94408b8de26ce9c0550e59955a02f5`
- original URL: https://www.pulsar.gg/cdn/shop/files/Mouse_v2.18_1K_Dongle_v3.0.zip
- packaging / extraction: downloaded from pulsar.gg; bsdtar -x (plain zip)
- type: zip package (PE32 .NET Framework v4.8 WinForms 'UpgradeTool' inside; images stored as .NET resources res.dev0/dev1.upgrade.bin). Not itself an image.
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-xlite-v3/dongle-1k-v3.00/(1) 1K Dongle Update v3.00.exe__res.dev0.upgrade.bin
- size 43104  sha256 `93068709d187e828af1e03cbc5f9a22f483dc5629f3a595a9d3ff846679564a4`
- original URL: (embedded resource; see sibling package row)
- packaging / extraction: extracted from .NET updater resource (dnfile) OR from zip (bsdtar)
- image[0] @off 0, fwLength 34912: magic=ComUsbUpgradeFile; headCRC=0x555522ff(ok); payloadCRC=0x00008860; ver=v3.00(raw 768); DeviceType=211(dongle); cid/mid=0/0; icName=CX52650N; sensor=; normal=vid_3554&pid_f508&mi_01&col05; boot=vid_3554&pid_f402&mi_01&col02
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/pulsar-xlite-v3/mouse-v2.18/(2) Mouse Update v2.18.exe__res.dev0.upgrade.bin
- size 92768  sha256 `63b96e470ad9c054559d77e4ed986426311aee88cd4c43519e64b1dd6d42ff4d`
- original URL: (embedded resource; see sibling package row)
- packaging / extraction: extracted from .NET updater resource (dnfile) OR from zip (bsdtar)
- image[0] @off 0, fwLength 84576: magic=ComUsbUpgradeFile; headCRC=0x5555273a(ok); payloadCRC=0x00014a60; ver=v2.18(raw 536); DeviceType=210(mouse); cid/mid=0/0; icName=NRF52833; sensor=3395; normal=vid_3554&pid_f507&mi_01&col05; boot=vid_3554&pid_f400&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/scyrox-v6/v2.22/NRF52840-AC4FBDA9-GMMouse-欣春旗V6-SCYROX-IR-无线8K-3950-PID-F5F6-V2.22-20250717_DLL升级.bin
- size 96432  sha256 `559df12ec0930bad82add9d25a757ad017937dac935f54349aa78ca9cb76ff22`
- original URL: (embedded resource; see sibling package row)
- packaging / extraction: extracted from .NET updater resource (dnfile) OR from zip (bsdtar)
- image[0] @off 0, fwLength 88240: magic=ComUsbUpgradeFile; headCRC=0x5555251f(ok); payloadCRC=0x000158b0; ver=v2.22(raw 546); DeviceType=210(mouse); cid/mid=62/2; icName=NRF52840; sensor=3395; normal=vid_3554&pid_f5f6&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/scyrox-v6/v2.22/V6_Update_Tool__EN.zip
- size 1005899  sha256 `e4c6d783bb49b9556cf17d37d92b0e2300d9c46d69dadbebb40a45ea80e811a8`
- original URL: https://cdn.shopify.com/s/files/1/0580/1489/2094/files/V6_Update_Tool__EN.zip?v=1766475815
- packaging / extraction: downloaded from scyrox.com (Shopify CDN); bsdtar -x
- type: zip package (PE32 .NET Framework v4.8 WinForms 'UpgradeTool' inside; images stored as .NET resources res.dev0/dev1.upgrade.bin). Not itself an image.
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/scyrox-v8/v2.22/NRF52840-FAF2F792-GMMouse-欣春旗V8-SCYROX-IR-无线8K-3950-PID-F5F6-V2.22-20250717_DLL升级.bin
- size 96432  sha256 `a9055f7519879bd399f15ff0971b268b2eaea53641cc6071e6ca293b091f3dee`
- original URL: (embedded resource; see sibling package row)
- packaging / extraction: extracted from .NET updater resource (dnfile) OR from zip (bsdtar)
- image[0] @off 0, fwLength 88240: magic=ComUsbUpgradeFile; headCRC=0x55552634(ok); payloadCRC=0x000158b0; ver=v2.22(raw 546); DeviceType=210(mouse); cid/mid=62/1; icName=NRF52840; sensor=3395; normal=vid_3554&pid_f5f6&mi_01&col05; boot=vid_3554&pid_f403&col01
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.

### other-devices/scyrox-v8/v2.22/V8_Update_Tool_2.22.zip
- size 1003155  sha256 `15ef891fba2bc1bec79b5b81a61ec15ef2068cf93a574ed2e917b399a2689efa`
- original URL: https://cdn.shopify.com/s/files/1/0580/1489/2094/files/V8_Update_Tool_2.22.zip?v=1752910906
- packaging / extraction: downloaded from scyrox.com (Shopify CDN); bsdtar -x
- type: zip package (PE32 .NET Framework v4.8 WinForms 'UpgradeTool' inside; images stored as .NET resources res.dev0/dev1.upgrade.bin). Not itself an image.
- HYPACE match: OTHER DEVICE (foreign PID / cid; NOT the HYPACE). Evidence missing to place under mouse/ or receiver-*: no PID 0xFB14/0xFB16, no cid 102, no Lofree link.
