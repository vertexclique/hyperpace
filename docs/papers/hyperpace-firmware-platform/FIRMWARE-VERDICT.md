> Coverage statement, stated first because it qualifies everything below: the critic still reported new gaps in its final round (9). This verdict is a snapshot of what was verified by 2026-09-16, not a proof of universal absence. Every negative here is "not found on the channels reached", never "does not exist".

# 1. Bottom line: can Hyperpace obtain, bundle, install and roll back genuine HYPACE firmware today?

| Capability | Verdict today | Why |
|---|---|---|
| Obtain | No | 37 candidates checked across every reachable vendor, CDN, archive and code-host channel. Zero HYPACE images. Lofree's own HYPACE app config `www.lofree.tech/mouse/cfg.json` (cid 102, PIDs FB16/FB14) has zero `upgrade`, `link` or `.bin` keys, while the sibling `/keyboard/cfg.json` on the same host carries 12 firmware slots (G110). |
| Bundle | No | `firmware/archive/{mouse,receiver-1k,receiver-2k,receiver-4k,receiver-8k}/` are all empty, re-verified this session by directory listing (G101). The 35 archived `.bin` files (45 header instances) under `other-devices/` are all other products and are ledger-marked "MUST NEVER be flashed" (`firmware/archive/SOURCES.md:7-8`). |
| Install | Not with a genuine image; the mechanism is specified | The Compx procedure is now implementable end to end (see the flashing spec). What is missing is the artifact, plus three HYPACE-specific unknowns readable only from a real image: boot PID, report IDs / feature flag, and the prepare-packet check value. |
| Roll back | No, and structurally blocked | Rollback needs >= 2 versions per target; the archive holds 0 for every HYPACE target (G102). No read-back or backup command exists in any implementation (F030, G103, G219), so the version currently on the operator's mouse cannot be snapshotted before the first erase. Rollback is therefore only "re-flash a different bundled package", and none is bundled. |

Two hard consequences for the product requirement "bundle every firmware version for mouse and receivers, with install and rollback":

1. **Every rollback target must be acquired as a complete vendor-built package.** No implementation synthesises the prepare packet; all three copy `resetToUpdateModeCmd`, `prepareDownLoadCmd` and `dataDownLoadCmd` verbatim out of the header (G213). For the HYPACE mouse's own MCU family the prepare check value matches no tested CRC scheme (G214: the one archived NRF52833 image stores `0x41324938`, which reproduces under none of raw CRC-32, zlib CRC-32 or byte sum). So a raw payload, a flash dump, a payload lifted from a sibling container, or a reconstructed older image is **unflashable**.
2. **The feature ships incomplete until packages arrive.** The project README currently states install and rollback ship fully enabled with four guards and defines no empty-archive or single-version behaviour (G104). That degraded case must be specified, and it must say "no firmware available for this device yet", never a false green.

# 2. Claim verdicts C01 to C14

Ledger: 13 partially_correct, 1 unverifiable, 0 confirmed, 0 refuted.

| ID | Claim (abbreviated) | Final verdict | Correction to carry |
|---|---|---|---|
| C01 | No official HYPACE firmware or updater is publicly distributed | partially_correct | No **image** was found on any Lofree-controlled page or search channel as of 2026-09-16. The **updater** half is wrong: Lofree publicly serves the Compx WebHID flashing engine at `www.lofree.tech/home/`, a page titled "Lofree HYPACE" whose device filter includes PID 0xFB16 (`index.pretty.js:5520-5522`, `:25178-25182`). Only the images are missing; its upgrade links are dead Pulsar sample names. Distribution via customer service, login-gated Chinese platforms or API-driven hubs is unverified. |
| C02 | Four readable implementations share byte-identical framing | partially_correct | Three code paths actually flash: Lofree WebHID JS, Pulsar WebHID JS (one template with the Lofree JS, statement-for-statement), and the Pulsar/Scyrox .NET tool. Given one image they build identical packets, confirmed against two real USB logs. OpenMouse `atk-index.ts` only parses and reads the payload CRC at the wrong offset. Framing is **not** one fixed layout: each header sets report 6 or 9, output or feature report, the flash start address and the reset sub-target. |
| C03 | No signature and no anti-rollback; integrity is header CRC plus per-packet replies | partially_correct | Correct that the container has no signature field (bytes 720..8191 are zero in all 45 headers) and no host enforces anti-rollback. Integrity has three parts, not two: (1) the host-only header CRC; (2) a 4-byte whole-image check value in the prepare packet that the device gates, answering `CheckOK` (0x05) before `Success`; (3) the per-packet echo. Whether the bootloader enforces a MAC or version floor inside the encrypted payload is unverified. |
| C04 | Payloads are encrypted at rest and decrypted by the bootloader; host sends verbatim | partially_correct | "Host sends verbatim" is confirmed byte-for-byte against both USB logs. "Encrypted at rest" is chip-specific: NRF52833/NRF52840 and CH32V305 payloads look like ciphertext (entropy ~7.99, a 16-byte block shared across products and brands); **CX52660 payloads are plaintext Thumb code with readable strings** (entropy 6.96, `hci_le_rx_test_v1_cmd_handle` at offset 70507); CX52650N is scrambled by an undetermined scheme. Bootloader-side decryption is inference, never observed. |
| C05 | Receiver before mouse, and mouse/receiver must stay a matched pair | partially_correct | Receiver-first is vendor **guidance** with a soft consequence (re-pairing), verbatim in the Pulsar readme and hub history text, and machine-enforced only in Lofree's keyboard hub (`upgradeOrderInvalid`). No Compx mouse updater enforces it. "Matched version pair, rollback included" has **no Compx source**; Pulsar ships one shared dongle list across mouse configs. Treat pairing as a conservative Hyperpace policy, not a known device constraint. |
| C06 | The 4K receiver is CH32V305 + a Nordic radio MCU in a chained package | **unverifiable** | The only HYPACE-specific datum is the cfg string `dongle4: CH32V305`, and those cfg chip strings are template defaults (32 of 33 Pulsar entries carry the identical chip quartet while Pulsar actually ships NRF52840/CX364/CX52660 parts). The two-MCU inference comes from siblings only. Settle it from a real receiver image or the FCC internal photographs. |
| C07 | Command 0x0D on report 8 enters the bootloader, which re-enumerates at 0xF4xx | partially_correct | The 17-byte reset frame on report 8 with command 0x0D and the 0x55 checksum is in all 45 archived headers and confirmed on the wire. "0xF4xx" is not universal: CX364 parts boot at `0x3710:0xF004/0xF005`, chained radio MCUs have no boot PID at all (they are flashed through the normal PID's col07 on report 9), and Lamzu uses other ids. The HYPACE's response to 0x0D and its boot PID are unverified. |
| C08 | Sibling images cannot be used: cid, mid, normal PID and payload differ | partially_correct | The prohibition stands and is non-negotiable. The reasoning is largely wrong: 41 of 45 archived images carry cid/mid 0/0, no updater compares cid/mid on the flash path, and the payload key is shared across products and brands. The only vendor protection is the host-side normal-endpoint PID match, and **the bootloader is anonymous once entered** (OpenMouse #97: two images sharing boot id `0x3554:0xf408`, wrong-model flash left the optical sensor producing zero movement). Hyperpace must own its target gate. |
| C09 | The .NET tool re-flashes any version because AlwaysUpgrade=1 and an inequality test | partially_correct | The .NET tools can re-flash only their single embedded image. `AlwaysUpgrade=1` (set in all seven shipped cfg.ini files) enables the button **unconditionally**; inequality is the `AlwaysUpgrade=0` path, not the mechanism. Whether the bootloader accepts a downgrade is unverified. Pulsar web is strictly-newer, and its cfg label can disagree with the header (a "v4.00" entry ships a v2.24 image). |
| C10 | CX52650N is a Beken SoC with no public flashing tool but the Compx protocol | partially_correct | Its role as the 2.4 GHz receiver SoC in Compx dongle images is confirmed (DeviceType 211). The Beken attribution is inferred from snippets about CX52850 and CX52650P, never confirmed for the "N" suffix. Generic Beken tools exist and were not evaluated. Only the confirmed part is load-bearing: the Compx USB HID protocol is the sole confirmed field-update path for a sealed receiver. |
| C11 | A failed update is recoverable via a physical button gesture into the bootloader | partially_correct | What is documented for Compx siblings is a **re-run while the device is still in the bootloader**: both web engines accept the boot VID/PID directly and skip the reset, Pulsar's history text says press Enter to retry, and .NET has `ExistBootDevice`. The button gesture is documented only by Endgame Gear with no model or chip named, and is unverified for Compx bootloaders and for the HYPACE. A counter-case exists (a Lofree keyboard bricked by an official update no longer entered DFU). |
| C12 | The /home cfg mouse entry (cid 62, PID 0xF5F6) is a sibling sample, not the HYPACE | partially_correct | It is a hybrid. Template leftovers: cid 62, PID 0xF5F6 and the dead Pulsar upgrade links. HYPACE-facing: the page title, the 0xFB16 filter, sensor 3950. The app applies this cid-62 entry to whatever device it opens (`Set_Pair_CID(Q.mouse[0].cid)`), so it cannot be dismissed as pure sample data. Whether a HYPACE reports cid 62 or 102 on the wire is unverified. |
| C13 | bcdDevice 0x0216 = receiver v2.16 and 0x0300 = mouse v3.00 | partially_correct | Under the sibling Compx convention it would read that way (the .NET tool formats bcdDevice as hex major.minor; a VXE unit matches its package version exactly). On the HYPACE this is unverified until bcdDevice is compared against the in-band reads (command 18 mouse, command 29 receiver) or a real image. On a multi-MCU receiver bcdDevice may track only the primary chip, which is why the .NET tool falls back to an in-band `0xB3` query for chained files. |
| C14 | No Compx updater offers read-back or backup, so installed firmware cannot be saved | partially_correct | Confirmed for every examined implementation: the boot channel has exactly four opcodes (0xB0, 0xB1, 0x5B, 0x5A) and the .NET method inventory contains no flash read. "Cannot be saved at all" overreaches: other vendors' tools, undocumented commands (the .NET tool already sends an unlisted `0xB3`) and hardware debug readout were not assessed. Even a raw dump would not be a re-flashable container, since NRF payloads travel as ciphertext and the prepare check value cannot be recomputed. |

# 3. Candidate hunt: every candidate checked

37 candidate rows. **0 hypace_confirmed, 0 hypace_possible.** 22 files now sit in `firmware/archive/candidates-unverified/` (verified by listing this session); none is recorded in `SOURCES.md`, which is itself a ledger gap (G234).

## 3a. Retrieved, parsed, and rejected as another product (16 rows)

| # | Candidate | Result | Decisive marker |
|---|---|---|---|
| 1 | `hyzen.lofree.tech/online-updater/OE930_KB_encrypt.bin` (168120 B) | not_hypace | Magic `Eevision` at offset 0, not `ComUsbUpgradeFile`. Registered for vid 0x388D pid 36/37 (a keyboard). |
| 2 | `.../OE930_DG_encrypt.bin` (18196 B) | not_hypace | Same container; OE930 keyboard receiver package. |
| 3 | `.../OE930_KB_LINE_encrypt.bin` (158216 B) | not_hypace | Same container; single-mode OE930, vid 0x388D pid 41. |
| 4 | `.../zephyr_encrypt.bin` (230776 B) | not_hypace | Second wired package of the OE930 config. |
| 5 | `.../zephyr_DG_encrypt.bin` (51600 B) | not_hypace | Second dongle package of the OE930 config. |
| 6 | `lofree.tech/keyboard/bin/cid01_mid01/CX53730_KB_OE921_V1.66_...bin` | not_hypace | Valid ComUsb header, headCRC ok, but normal endpoint `vid_05ac&pid_024f`, icName CX53730, DeviceType 209. |
| 7 | `lofree.tech/home/bin/...` same file (HEAD) | not_hypace | Same name and length; `/home/bin`, `/keyboard/bin` and `/OE921/bin` serve one shared tree. |
| 8 | `/OE921/bin/.../OE921_V1.66` (downloaded, sha256 20521b83...) | not_hypace | No `pid_fb14`/`pid_fb16`, no cid 102, productName empty. Byte scan for "hypace" found nothing. |
| 9 | `/OE922/bin/cid01_mid02/...OE922-4.4V_V1.68` | not_hypace | normal `vid_05ac&pid_024f`. |
| 10 | `/OE921E/bin/cid01_mid03/...V1.57` | not_hypace | Same family. |
| 11 | `/OE922E/bin/cid01_mid05/...V1.57` | not_hypace | Same family. |
| 12 | `/OE921GE/bin/...V1.56` | not_hypace | normal `vid_388d&pid_0008`. |
| 13 | `/OE922GE/bin/...V1.57` | not_hypace | normal `vid_388d&pid_0009`. |
| 14 | `/OE921J/bin/...V1.56` | not_hypace | VID is 0x3554 but PID is `f811`, not fb14/fb16. |
| 15 | `/OE922J/bin/...V1.58` | not_hypace | PID `f811`. |
| 16 | `Compx-4K-Dongle-1101-E8AF2ED4-V1.27-20230824.bin` | not_hypace | Chained ComUsb file, both headers CRC-valid: cid 11, PIDs f401/f510, icNames CH32V305 + NRF52810. |

## 3b. Retrieved, not firmware (5 rows)

| # | Candidate | Result | Note |
|---|---|---|---|
| 17 | `Compx Gaming Mouse_Demo_v1.0.0.4_2023.08.30_setup.exe` (6696742 B) | not_firmware | Inno payload decoded with Python lzma, never executed. `ComUsbUpgradeFile` appears only as code strings inside HIDUsb.dll. Its Config.ini devices are ATLANTIS OG V2 / MINI / Thorn; zero NRF52/CX526/CH32V hits; images are loaded from an external `\bin\` folder, not embedded. |
| 18 | `lofree.tech/cfg.json` (5058 B) | not_firmware | The HYPACE config: title "Lofree HYPACE", cid 102, PIDs FB16/FB14, NRF52833/CX52650N/CH32V305. Zero `upgrade` keys, so it links no firmware. Hash unchanged from the prior snapshot. |
| 19 | `cdn.shopify.com/.../Hypace_Mouse_User_Manual.pdf` | not_firmware | A manual. Image-only single page; `pdftotext` extracts 1 byte; only metadata is the title `HYPACE鼠标-说明书-中英日文-20250211`. |
| 20 | Dropbox `hyzen-offline-recovery-tool` zip (6279820 B) | not_hypace | Lofree OE930 keyboard recovery tool (nRF54L05, WCH boot `1A86:FE07`). 0 ComUsb headers, 0 hypace/fb14/fb16/vid_3554 hits. Unpacked and byte-scanned, never run. |
| 21 | Dropbox `lofree-key-mapper` zip (40994329 B) | not_hypace | Flow Lite Key Mapper plus a DOCX recovery guide listing only the 8 Flow Lite recovery URLs. Gap: the Inno payload was not unpacked (no innoextract), so its bundled images are known only from a third-party README. |

## 3c. Derived addresses probed, absent (16 rows)

| # | Probe | Result |
|---|---|---|
| 22 | `/keyboard/bin/cid102_mid01/` (+ cid102_mid02, cid102_mid1, cid66_mid01/02, cid62_mid01, cid87_mid04) | unreachable. Directory oracle: existing dirs return 403, missing ones return the 608 B SPA. All returned the SPA. |
| 23 | `/home/bin/cid102_mid01/` (+ variants, and `/OE921/bin/cid102_mid01/`) | unreachable. A full sweep of `/home/bin/cidNN_midMM` for cid 00..130 found only cid01_mid01..04 existing. |
| 24 | `/mouse/bin/` | unreachable. `/mouse/img/` and `/mouse/js/` are 403 (exist); `/mouse/bin/` is the SPA. This is exactly where a Vite-template HYPACE app would fetch `bin/<link>`. |
| 25 | `/mouse/history.json`, `devicename.json`, `firmware.json`, `version.json`, `upgrade.json`, `ref.json` | unreachable. nginx 404 on each, and under `/home/`, `/keyboard/`, `/OE921/`. |
| 26 | `hyzen /online-updater/HYPACE_encrypt.bin` plus 21 derived names (case variants, chip names NRF52833/CX52650N/CH32V305, PID names FB14/FB16, `cid102_mid01.bin`, Pulsar sample names) | unreachable. All 22 are nginx 404. The HYPACE is also absent from the hyzen device registry, which matches only vid 0x388D. |
| 27 | `hyzen /online-updater/{version,firmware,upgrade,history,devicename,manifest,index,list,latest,cfg}.json` and 5 at the host root | unreachable. All 404. `/online-updater/` itself is 403, so no index or manifest exists. |
| 28 | `/home/bin/pulsar_mouse.bin` (and `/home/`, `/bin/`, `/keyboard/bin/`, `/mouse/bin/` variants) | unreachable. SPA fallback everywhere. |
| 29 | `/home/bin/pulsar_dongle_1K.bin`, `/old.bin`, `/pulsar_dongle_8k.bin` | unreachable. The cid 62 Pulsar sample block in `/home/cfg.json` is entirely dead. |
| 30 | `/home/cfg.json` upgrade block, all 8 URL forms | unreachable. HEAD 200 text/html 608 for every one. |
| 31 | `cdn.shopify.com/s/files/1/2779/9046/Hypace_Firmware.zip` + 9 name guesses | unreachable. All 404, while the control `Hypace_Mouse_User_Manual.pdf` returns 200, proving the prefix is right. Small guess set, not an enumeration. |
| 32 | `www.lofree.tech/OE923/`, `/OE923TX/`, `/OE923RX/` (and cfg.json, `/bin/cid102_mid01/`) | unreachable. All return the 608 B SPA, sha256 9421338c..., while the 8 Flow Lite model pages return a real "Control HUB WEB" index plus a cfg.json with a live firmware link. |
| 33 | `hyzen /online-updater/OE923{,TX,RX}_{MS,DG,KB}_encrypt.bin` | unreachable. 404, while the three OE930/zephyr controls return 200. |
| 34 | Dropbox Flow 2 firmware folder (reddit-posted) | unreachable. Returned an HTML page containing "Deleted"; probably removed (unverified). |
| 35 | Dropbox Key Mapper folder page (`dl=0`) | not_hypace. Page context is Flow Lite remapping software; folder contents not statically enumerable. |
| 36 | Google Drive links on `lofree.co/pages/download` (2020 Wayback) | not_hypace. "Digit Number Pad Firmware Update v1.00.54.00", 2020, years before the HYPACE existed. Which link is the firmware was not mapped. |
| 37 | Compx `download.s21i.co99.net/21165906/` prefix, full Wayback index | unreachable as a HYPACE source. 19122 CDN rows, exactly one under the Compx account prefix: the same 2023 4K dongle bin. No later first-party vintage exists. |

## 3d. Channel-level negatives worth recording

- Wayback: 51459 lofree.co URLs, 5264 Shopify file URLs, 189 lofree.tech, 484 lofree.com.cn, 96 compx.com.cn. **Zero** `.bin/.zip/.exe/.rar/.7z/.hex/.dfu` in any of them. `hyzen.lofree.tech`, the one Lofree host that actually serves firmware, has never been captured at all.
- archive.today: no mementos for any HYPACE or download page.
- Certificate transparency: `lofree.tech` has only 4 names, `lofree.co` 17, `lofree.com.cn` 9, `lofree.com` none. No firmware or updater subdomain.
- GitHub / GitLab / Gitee: no HYPACE firmware. Only a reverse-engineering repo (one commit, no artifacts), a `linearmouse.json` config naming `Hypace@Lofree`, and one OpenMouse test referencing `ComUsbUpgradeFile`.
- Compx vendor site: exhaustively enumerated. `col.jsp?id=104` links exactly six articles (`nd.jsp?id=` 2, 3, 5, 10, 11, 12); ids 1, 4, 6-9, 13-40 all 404. Only id=11 carries attachments, and exactly the two 2023 files. An earlier "zero links" scan of these pages was a gzip decoding artifact, not evidence.
- Chinese channels reachable without login (official site, WeChat via Sogou, Bilibili, Sina, Sohu, Douyin): no firmware, no updater. Tieba, Xiaohongshu, Zhihu, Taobao and JD were blocked and remain **unverified, not negative**.

# 4. What a genuine HYPACE package must look like

## 4a. Required, in order

| # | Marker | Required value | Why it is decisive |
|---|---|---|---|
| R1 | Container magic | ASCII `ComUsbUpgradeFile` in header field 0 (offset 23), NUL-padded | Present in 45/45 archived headers. A Lofree `Eevision` package is a different container entirely and must be refused with that reason named, not guessed at. |
| R2 | `headLength` | exactly 720 | 45/45. The reference parser enforces `!== 720` as a throw. |
| R3 | `headCRC` | `(0x55555555 - sum(bytes[8 .. headLength))) & 0xFFFFFFFF` equals the stored u32 at offset 0 | 45/45 recomputed, zero failures. It is the only integrity check that covers every image. |
| R4 | **normal endpoint PID** | `normalOutputEndPoint` **and** `normalInputEndPoint` both contain `vid_3554` and either `pid_fb16` (receiver link) or `pid_fb14` (wired) | This is the single load-bearing identity marker. None of the 45 archived images, and none of the 10 Lofree keyboard images, carries either PID. The observed normal-endpoint set spans 23 distinct vid&pid pairs, none of them fb14/fb16. |
| R5 | `DeviceType` | 210 for a mouse image, 211 for a receiver image, and it must match the component being flashed | 210/211 hold across all 45. Note 209 appears on all 10 Lofree keyboard images, so DeviceType alone never proves HYPACE; it only rules out a cross-component flash. |
| R6 | `icName` | consistent with the declared target: `NRF52833` for the mouse per the vendor config (a Chinese teardown says nRF52840; unresolved, and the board must not be opened), `CX52650N` for a 1K/2K receiver, `CH32V305` for the 4K receiver, unknown for an 8K receiver | Guards a mouse image against a receiver slot. Record the string; refuse only on a mismatch against a HYPACE-plausible set, and say so when the set is unknown. |
| R7 | Payload bounds | `fwLength >= 1` and `8192 + fwLength <= file size` | The web parsers have no such bound; the reference TS parser does, and this is the check to mirror. |
| R8 | Chain integrity | `nextFileAddress` is either 0, or equals `8192 + fwLength` of image 1 **and** a second header at that offset passes R1-R3 | Holds in all 10 archived chained files. The web parser allocates `new Uint8Array(len - nextFileAddress)` with no bound and throws a RangeError on a malformed value: bound it first. |
| R9 | Endpoint well-formedness | every endpoint field matches `vid_([0-9a-f]+)&pid_([0-9a-f]+)` | Mirror the reference parser's throw. |
| R10 | Command fields present | `resetToUpdateModeCmd` cmdlength 17 / report 8, `prepareDownLoadCmd` and `dataDownLoadCmd` cmdlength 49 / report 6 (or 9 on a chained second image) | 45/45. A package missing any of the three is unflashable because nothing synthesises them. |

## 4b. Explicitly NOT required (recording these as requirements would reject a genuine package)

- **Header `cid` = 102 / `mid` = 1 or 2.** 41 of 45 archived images carry `cid=0, mid=0`, and all 10 Lofree-hosted keyboard images carry 0/0 even though their server directories are named `cid01_midNN`. Record header cid/mid; never gate on them.
- **Sensor string `3950`.** A sibling NRF52840 image also declares 3950, so the sensor field separates nothing on its own.
- **`productName` containing "HYPACE".** It is empty in every Lofree-hosted image observed.
- **A payload CRC that Hyperpace can verify.** The prepare check value is reproducible as a raw CRC-32 (init 0xFFFFFFFF, poly 0xEDB88320, no final XOR) for exactly the 4 CX images and for none of the other 41, including the one NRF52833 image. Carry the field verbatim and let the device gate it.
- **The declared prepare length equalling `fwLength`.** It matches in only 12 of 45; elsewhere it is short by 4, 8 or 12 bytes with no chip-family pattern. Read the field, never derive it.

## 4c. Provenance requirements on top of the bytes

A package that satisfies R1-R10 is *structurally* HYPACE-shaped. Before it is offered for install it must also carry, recorded in the ledger: source URL or "operator-supplied", retrieval date, sha256, file size, the full parsed header, and whether a Lofree-controlled host served it. A package whose provenance is unknown is installable only behind an explicit operator acknowledgement that names what is unverified.

# 5. How Hyperpace must refuse anything else

## 5a. The gate, evaluated before a single byte is sent

Run in this order and stop at the first failure. **Refuse loudly, with the reason, and never fall through to a permissive default.** "If we cannot determine it, allow it" is the false green the doctrine forbids.

1. Parse the container. Any of R1, R2, R3, R7, R8, R9, R10 failing -> refuse: **"This file is not a firmware package Hyperpace can read."**
2. Match the target. R4 failing -> refuse: **"This firmware is for a different device. Hyperpace will not install it."** This is the rule that blocks every one of the 35 sibling images and all 10 Lofree keyboard images, and it is the only rule that would have blocked the community cross-flash that killed a sensor.
3. Match the component. R5/R6 mismatch against the slot the user chose -> refuse: **"This firmware is for the <mouse|receiver>, not the <receiver|mouse>."**
4. Verify the file against its recorded sha256 before use. A mismatch -> refuse: **"This firmware file is damaged. Download it again."** For 41 of 45 image families the payload cannot be validated any other way, and the erase happens before any data packet, so a corrupt payload discovered after the reset is unrecoverable.
5. Require exactly one matching device. More than one device matching the normal input or output endpoint -> refuse and name it: **"More than one mouse or receiver is connected. Disconnect the others and try again."** The .NET tool machine-enforces this at both stages; the vendor instruction is verbatim "please disconnect all other mice and dongles from your computer".
6. Battery and transport preconditions (threshold and transport policy are open decisions, section 7 below). If the battery cannot be read, say so and stop; do not proceed on an unknown.
7. Only then send the 0x0D reset.

## 5b. Refusals that are absolute, regardless of user override

- **Never flash any of the 35 archived sibling images**, even if a future gate would let one through. They are ledger-marked and live under `other-devices/` precisely so they can never be selected.
- **Never accept a raw payload, hex file, flash dump or reconstructed image.** Nothing synthesises the prepare packet and the HYPACE mouse family's check value algorithm is unknown, so such an input is unflashable by construction. Refuse with: **"Hyperpace needs the complete firmware package, not a firmware image file."**
- **Never present a package as verified on the strength of the header CRC alone.** It covers only the 720-byte header.
- **Never claim a rollback capability that the archive cannot back.** With zero or one package for a target, the UI says so plainly.

## 5c. Where the vendor tools are silently wrong, and what Hyperpace must do instead

| Vendor behaviour | Hyperpace requirement |
|---|---|
| A header-check failure returns false with no state change and no message; a non-200 fetch does nothing | Every parse and fetch failure surfaces a reason. |
| Reset exhaustion (about 10.5 s of retries) leaves the dialog stuck at "Upgrading" with the button disabled | Reset exhaustion is a named failure with a recovery instruction. |
| A device-reported error (0x5A) is swallowed by all three implementations; the .NET branch is literally unreachable dead code | Surface a device-reported error as a distinct, loud failure. |
| Three distinct failures (reset exhaustion, resend limit, device error) all converge on one indistinguishable timeout | Keep them distinguishable, and say which one happened. |
| The reference TS parser's `payloadCrcValid` reads the wrong offset and is structurally always false | Mirror its bounds checks; do not mirror its CRC check. |
| Unplugging during a flash is masked by a no-op disconnect handler | Treat a disconnect during flashing as a first-class, named event. |

# 6. Coverage and honesty

- Coverage is **incomplete**. The critic still found nine new gaps in its final round.
- Every "absent" above is scoped to a hand-picked name set or an enumerable index; none is an exhaustive enumeration of a server's file space.
- Unverified, not negative: Lofree Discord channel content (login), Kickstarter backer posts (403, unarchived), Reddit coverage (archive API rate-limited mid-sweep), Tieba / Xiaohongshu / Zhihu / Taobao / JD (captcha or login walls), Bilibili comments (67 of 695 returned), the Key Mapper Inno payload (not unpacked).
- Hard rules held throughout: no `/dev/hidraw*` or USB device was opened, written or ioctl'd; no downloaded file was executed; no node/npx/tsx/ts-node was run; firmware bodies went only to `candidates-unverified/`; no existing archive file or research note was modified.
- Ledger gap to close: the 22 files now in `firmware/archive/candidates-unverified/` are undocumented in `SOURCES.md`, which documents only the 35 images under `other-devices/`.
