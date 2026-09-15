URL: https://bbb.pulsar.gg/ , https://bbb.pulsar.gg/cMouse/ , https://www.pulsar.gg/pages/download
Retrieval date: 2026-09-15
Source type: vendor WebHID app (readable JS) + vendor firmware images (primary)

## Claim
Pulsar's "Bibimbap" web driver (bbb.pulsar.gg) is a WebHID app on the same Compx template as Lofree's, with a LIVE, working firmware updater and a downloadable firmware set. Its JS is the best readable reference for Hyperpace because it flashes over WebHID exactly as Hyperpace will.

## Readable updater code (saved)
- `research-bin/vendor/pulsar-bbb/sub_cMouse/app.pretty.js` (deminified, 13154 lines).
- Engine extracted to `research-bin/vendor/pulsar-bbb/sub_cMouse/UPDATER-ENGINE-webhid.js`.
- Key functions (line refs in app.pretty.js): header parse ~1050; `headCRC == Be(e,t)` verify 1096; UpgradeHandleInit `Ne` 1295; chunk builder ~1167-1180 (`Cmd_DownLoadFile: 177` at 915, `n.buffer[5..8]` big-endian addr, 32-byte slices); ResetToBootMode loop ~1197 (`sendReport`/`sendFeatureReport`); DownloadFile send loop ~1242-1250; response handler ~1280-1290 (`Cmd_DownLoadFile` echo, RepeatCountMax after 8 retries).
- Firmware fetch: `app.pretty.js:3412` `at.UpgradeHandleInit(i,1e4) && (e.enterUpgrade && await pa.Set_Device_EnterUpgrade(), await at.UpgradeStart())`; the image is fetched by `XMLHttpRequest ... responseType="arraybuffer"` from `window.location.href + this.link`.
- Version gate: `Check_New_Version` = `function W(e,t)` (app.pretty.js:902 ref), componentwise "vX.YZ", returns true only if candidate strictly greater. Dongle-vs-device decision at 4568 and 11696: `Check_New_Version(e.version.device, this.upgrade["device"].version)` for the mouse and `...(e.version.dongle, this.pulsarDongle[type].version)` for the receiver.

## Firmware distribution (WebHID app fetches these)
- cfg.json (sha256 dc85f446e4a3450d741c448ea59bc1eaf5cca61ba5f056924af33644053d9e11), title "Bibimbap" v1.3.12, cid 87, mouse MCU NRF52833, dongle1/2 CX52650N, dongle4 CH32V305. `upgrade` blocks name `/Mouse-*-Vx.yz.bin`; `pulsarDongle` array names dongle images.
- Files are served under `https://bbb.pulsar.gg/cMouse/bin/<name>` (the app prepends location; the real dir is /cMouse/bin/). All archived (see firmware/archive/SOURCES.md). Mouse images: v3.03/v3.04/v3.05 (NRF52840), v4.01 (CH32V305 Merge), v5.09 (CX364/NRF54L15). Dongle images: 1K CX52650N v3.00, 8K CH32V305 v2.24/v2.25, CD01 CX52660 v2.08, CD03 v4.07/v5.04.
- history.json (sha256 5d8360f...): "BETA Firmware Distribution 2026-02-05: Mouse v4.07 / Dongle v4.00 ... Please update the dongle first, followed by the mouse. (The update must be performed using a USB cable.) If the update fails, press the 'Enter' key on your keyboard to try again." Driver versions V1.3.5..V1.3.11 (2026-02 to 2026-08).

## Archived (Wayback) cfg deltas
- 2025-08-31 cMouse/cfg.json ("Bibimbap" 1.2.2): older mouse builds v3.03/v3.04 and dongle v2.24/v2.02 (`Dongle-CD01-V2.02`). Two links now gone from live cfg: `/Dongle-CD01-V2.02-1d9f2040.bin` (404 live and in Wayback = lost) and `/Mouse-DM178-3710-3509-31C4DA87-V3.03.bin` (404). Saved cfg snapshots under `firmware/archive/unknown/web-snapshot-2026-09-15/wayback/`.

## Relevance to Hyperpace
Same VID 0x3554 boot mode, same NRF52833/CX52650N/CH32V305 chips, same report-8 protocol. This is the closest working WebHID flasher to what Hyperpace needs; its JS is the reference implementation to diff against. Not directly flashable to the HYPACE (cid 87 vs 102, different normal PIDs).
