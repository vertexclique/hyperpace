# Lofree HYPACE vendor HID protocol: Hyperpace reference, version 2

Status: byte-exact reference rebuilt from both vendor browser drivers plus the target's own HID
report descriptors. Replaces v1 (`docs/research/lofree-hypace-protocol.md`), which was written
from the old driver only. Static reading only: no JavaScript was executed, no HID device was
opened, nothing was written to `/dev/hidraw*`. Every statement below carries a code citation.

Coverage caveat: the completeness review was still surfacing new gaps in its final round (10 in
the last pass). Treat this document as complete for everything it cites and incomplete as a
survey. Section 15 lists what is known to be missing.

---

## 1. Scope and sources

| Source | What it is | Location | Date |
|---|---|---|---|
| NEW driver | Vite single-page app, device chooser at `/home/` | `https://www.lofree.tech/home/`, bundle `assets/index-BTVblIUr.js` | Last-Modified 2025-11-17 |
| NEW configs | `cfg.json` (from page href), `ref.json` (page href), `sensor.json` (origin root) | `new:27357-27360` | same fetch |
| OLD driver | Vue 2 + Element UI app, full mouse UI | `https://lofree.tech/`, bundle `js/app.js` | Last-Modified 2025-08-06 |
| OLD configs | `cfg.json`, `sensor.json` (origin root), `lang/<code>.json` | `old:8359-8385` | same fetch; `sensor.h` Last-Modified Wed, 06 Aug 2025 07:59:14 GMT |
| Community repo | `PROTOCOL.md`, `README.md`, `macos-remap-sidebuttons-to-desktop-switch.js`, `photo.png`, `LICENSE` | single commit `a4700d10` | 2026-06-09 |
| Descriptors | Decoded `report_descriptor` from sysfs for hidraw8..13, primary data from the target | `docs/papers/hyperpace-firmware-platform/sources/local-2026-target-hid-report-descriptors.md` | retrieved 2026-09-15 |

Local working copies (pretty-printed): NEW `home/index.pretty.js` (27531 lines), OLD
`app.pretty.js` (8511 lines).

### 1.1 Citation convention

| Form | Means |
|---|---|
| `new:6087` | line in the pretty-printed NEW bundle |
| `old:1343` | line in the pretty-printed OLD bundle |
| `repo:PROTOCOL.md:39` | line in the community repo |
| `home/cfg.json#opt.mouse.pid` | NEW config, JSON path |
| `cfg.json#pid.mouse.wired` | OLD config, JSON path |
| `site/lang/en.json#KeyOptions` | OLD site language file |
| `descriptors:25` | line in the descriptor source above |
| `v1:103` | line in the v1 reference being replaced |

### 1.2 Verification method

Four independent read passes over the two bundles plus three gap-closing rounds, each claim
recorded with a code fragment, then re-verified against the source by a second pass that was
allowed to correct or refute it. Claims that could not be settled from the files were not
promoted to fact; they are in section 15. Ledger counts are in section 16.

### 1.3 Critical scope limit

The NEW `/home/` bundle is only a device chooser. After a pick it writes `deviceType`, `rooter`
and `deviceInfo` to `localStorage` and does a full document navigation to
`origin + cfg.opt.<type>.rooter`, which is `/mouse` or `/keyboard`
(`new:24697-24705`, `home/cfg.json#opt.mouse.rooter`). Those page bundles are not among the
local files. Therefore:

- The NEW protocol module is complete and is quoted throughout this document.
- Every NEW setter, the profile, restore, export/import and button UI, the cid/mid lookup and
  the code that sets `deviceInfo.type` for a real session live on `/mouse`, unread.
- `rh` (Get_HistoryDevicesInfo), `nh` (Get_Current_Device_Online), `sh` (Device_Reconnect) and
  `th` (Request_Device) are exported and unreferenced inside `/home/` (`new:7145-7151`).
  `o6` (Device_Connect) is reachable only through `Set_Device_ExitUpgrade`, which `/home/`
  calls with `enterUpgrade: false` (`new:6156`, `new:25195`, `new:25642`).

---

## 2. Driver differences

| Dimension | OLD driver | NEW driver |
|---|---|---|
| Framework | Vue 2 + Element UI | Vite bundle, Vue 3 runtime |
| Scope of local bundle | full mouse UI | device chooser plus pairing and upgrade dialogs only |
| Mouse cfg entry | cid 102, mid 1 and 2 (`cfg.json#mouse[0]`) | cid 62, mid 1 only (`home/cfg.json#mouse[0]`) |
| Keyboard support | none in the protocol module | full second flash map `pe`, 12 key layers, lighting, macros (`new:5574-5592`) |
| Flash shadow | 8192 B (`old:1042`) | 16384 B (`new:5642`) |
| Byte 4 class bit | never set (`old:1341`) | `+0x80` whenever `deviceInfo.type == "keyboard"` (`new:6085`) |
| `deviceInfo.type` default | `"mouse"` (`old:1061`) | `"keyboard"` (`new:5664`) |
| Connect flash read | fixed 26 reads of 10 B, 0x000..0x103, unbounded retry per address (`old:1505-1524`) | `js(0,256)`, 26 reads of min(10, 256-addr), last is 6 B, abort after 5 consecutive mismatches, no timeout (`new:6262-6291`) |
| Transaction wait | poll 5 ms x 40 (~200 ms), then compare even with no reply, 10 ms gap, 5 attempts (`old:1301-1332`) | 200 ms `setInterval` watchdog armed after send, poll 1 ms, resend on timeout or mismatch with no gap, 5 sends (`new:6046-6076`) |
| Connect watchdog reset | only at connect start, so 10 s caps the whole connect (`old:1566`) | reset on every report-8 frame during the bulk read, so it bounds silence only (`new:6277`) |
| Command ids defined | 29 (`old:955-985`) | 16 (`new:5531-5548`) |
| Receiver light (24/25) | present, wired to UI (`old:1455-1478`) | removed entirely |
| Firmware update | none; button does `window.open(href + link)` (`old:8046`) | complete WebHID updater, image parser, boot-mode re-enumeration (`new:5129-5529`) |
| `sendFeatureReport` | absent | upgrade path only, flag from the image header (`new:5427`, `new:5475`) |
| Disconnect handling | `navigator.hid.ondisconnect` clears state on any disconnect (`old:1208-1211`) | matches on `productName`, and is installed only by `sh`/`ah`, neither called in `/home/` (`new:5959-5962`, `new:5933`) |
| `Device_Close` | `q.close()` not awaited (`old:1214`) | `await ft.close()` (`new:5965`) |
| visit flag | starts true, set false when cfg loads (`old:954`, `old:8278`) | starts false (`new:5530`, `new:27388`) |
| Request filters | cfg VID x 4 PID lists, `Number.parseInt` (`old:2493-2504`) | cartesian product of mouse+keyboard lists, 5 VIDs x 17 PIDs = 85 filters, raw hex strings (`new:24666-24679`) |
| Wired PID 0xFB14 | in cfg and in the filters (`cfg.json#pid.mouse.wired`) | absent from every cfg list |
| `maxDpi` (cfg) | 40000 | 32000 |
| `sensor.json` 3950 | two ranges: 100..30000 step 50 DPIex 0; 30100..60000 step 100 DPIex 17 | one range: 50..30000 step 50 DPIex 0 |
| Mouse `keysCount` | from `cfg.keys.length` = 5 (`old:2541`) | hardcoded 6, never reassigned (`new:5743`) |
| cfg `lightEffect` for the mouse | absent, lighting tab hidden (`old:8200`) | present: mode 0, brightness 3, speed 8, movingOffState false |
| cfg `upgrade` block | key absent | present under cid 62 and all 13 keyboard entries |
| localStorage keys | `locale`, `theme`, `macro`, `preset_color`, `bat_<addr>`, `password`, `deviceInfo` (read only) | `locale`, `langArrs`, `theme`, `mouseMacro`, `keyMacro`, `deviceType`, `configIndex`, `rooter`, `deviceInfo`, `upgradingInfo`, `bat_<addr>`, `connectDevice` (written, never read), `hidDevices` (read, discarded) |
| `console.log` | 67 calls, suppressed when `cfg.debug.log` is false (`old:8364`) | zero calls; the flag is inert (9 `console.error` remain) |

---

## 3. Device identification and model table

### 3.1 USB and HID identity (primary data)

| Field | Value | Source |
|---|---|---|
| VID | 0x3554 | `cfg.json#vid`, descriptors |
| PID, receiver (2.4 GHz) | 0xFB16 | `descriptors:10-12` |
| PID, wired | 0xFB14 | `descriptors:13-15` |
| HID product string | `Compx Hypace@Lofree` | v1:33, unchanged |
| Interfaces per PID | 3: keyboard (1.0), vendor (1.1), mouse (1.2) | `descriptors:8-15` |
| Config channel | vendor interface, TLC 4, usage page 0xff02 usage 0x0002, report 8 = 16 B in and 16 B out | `descriptors:25` |
| Receiver vendor TLCs | 8 (adds TLC 7, mouse collection, report 11) | `descriptors:11`, `descriptors:28` |
| Wired vendor TLCs | 7 | `descriptors:14` |
| Shared layout | collections 0..6 identical on both links | `descriptors:49` |

Physical input reports (not the config channel): the mouse collection declares a 7-byte report
with no report id (5 buttons, 3 padding bits, 16-bit X/Y, 8-bit wheel, 8-bit AC Pan, Consumer
0x0238). The keyboard collection declares an 8-bit modifier field (usages 0xE0..0xE7) plus a
6-byte usage array 0..255. The vendor interface also carries Consumer Control report 5 with one
16-bit usage array bounded at 0x023C, and System Control report 3 with usages 0x81..0x83
(`descriptors:35`, `descriptors:41`).

### 3.2 Model identity from the protocol

`EncryptionData` (command 1) returns cid at byte 9, mid at byte 10 and link type at byte 11
(`new:5975`, `old:1224`). The old driver then matches cid, then mid, against `cfg.mouse[]`
exactly; no match shows an "unsupported device" dialog and does not connect (`old:2519-2544`).
The NEW `/home/` bundle performs no cid/mid lookup at all; it uses `cfg.mouse[0].cid` only as
the pairing cid (`new:25589`).

### 3.3 Every cid and mid in both configs

| cfg | Class | cid | mid | maxDpi | Sensor | Keys | Upgrade block |
|---|---|---|---|---|---|---|---|
| OLD `cfg.json#mouse[0]` | mouse | 102 | 1 | 40000 | 3950 | 5 | absent |
| OLD `cfg.json#mouse[0]` | mouse | 102 | 2 | 40000 | 3950 | 5 | absent |
| NEW `home/cfg.json#mouse[0]` | mouse | 62 | 1 | 32000 | 3950 | 6 | present (1000, 4000, 8000, device) |
| NEW `home/cfg.json#keyboard[0]` | keyboard | 87 | 4 | n/a | n/a | 128 rects, systems [win, mac], layouts [normal, fn] | `upgrade.device` |
| NEW `home/cfg.json#keyboard[1]` | keyboard | 1 | 1 .. 12 (12 entries) | n/a | n/a | 128 rects each, same 2x2 matrix | `upgrade.device`, some with `compel` |

NEW cid 62 upgrade links: `1000 -> {v2.16, /pulsar_dongle_1K.bin}`, `4000 -> {v2.16, /old.bin}`,
`8000 -> {v2.16, /pulsar_dongle_8k.bin}`, `device -> {v2.18, /pulsar_mouse.bin}`. The OLD driver
reads keys named `device`, `dongle1`, `dongle2`, `dongle4`, `dongle8` (`old:8053`), which do not
match those names; nothing in the NEW bundle reads the block at all (0 property accesses).

OLD cid 102 mid 1 vs mid 2 differ only in `debounce` (8 vs 0). Chip names in the OLD cfg entry:
mouse `NRF52833`, dongle1 and dongle2 `CX52650N`, dongle4 `CH32V305`; NEW cid 62 carries the
same chip names.

Unresolved and material: whether the target reports cid 62 or cid 102. Both configs are titled
`Lofree HYPACE` and both name PID 0xFB16, yet their only mouse entries carry different cids and
different key counts. Section 15 carries this.

### 3.4 Link type, byte 11 of the EncryptionData reply

| Byte 11 | Link | maxReportRate set by the host |
|---|---|---|
| 0 | wireless | 1000 Hz |
| 1 | wireless | 4000 Hz |
| 2 | wired | 1000 Hz |
| 3 | wired | 8000 Hz |
| 4 | wireless | 2000 Hz |
| 5 | wireless | 8000 Hz |
| other | wireless (`isWired = false`) | previous value kept (initially 1000) |

`new:5975`, duplicated at `new:5889`; `old:1224`. The cap is applied to the *displayed* rate
only (`new:6324`); the setter itself never clamps.

### 3.5 Chooser filters

| Driver | Filter construction | Result |
|---|---|---|
| OLD | `cfg.vid` x [pid.mouse.wireless, pid.mouse.wired, pid.keyboard.wireless, pid.keyboard.wired], `Number.parseInt` (`old:2493-2504`) | 0x3554 with 0xFB16, 0xFB14, 0xF809, 0xF50A; wired directly requestable |
| NEW, Connect button | `[...opt.mouse.vid, ...opt.keyboard.vid]` x `[...opt.mouse.pid, ...opt.keyboard.pid]`, raw hex strings (`new:24670`) | 5 x 17 = 85 filters, no dedupe, 0xFB14 in neither list |
| NEW, Space key | `requestDevice({filters: []})` (`new:25594-25597`) | unfiltered chooser, so a wired unit can be picked but then matches no cfg list |
| NEW, upgrade | `[normal, boot]` VID/PID parsed from `vid_XXXX&pid_XXXX` strings in the firmware image header (`new:5324-5337`) | image-defined |

When a picked device matches neither `opt.mouse` nor `opt.keyboard`, the NEW router leaves its
accumulator at `{}`, stores `rooter` as `undefined` and navigates to `origin + "undefined"`
(`new:24697-24705`). The wired path is therefore reachable into a broken navigation, not merely
unreachable. `opt.mouse.vid` is `["0x373B","0x3554"]`; `opt.keyboard.vid` is
`["0x05AC","0x3554","0x388D"]`.

---

## 4. Transport

### 4.1 Interface and collection selection

| Rule | Where | Predicate |
|---|---|---|
| Strict (protocol module) | `new:5884`, `old:1182`, `old:1371` | collection with exactly one input report AND exactly one output report AND `outputReports[0].reportId == 8` |
| Loose (NEW `/home/` router) | `new:24689` | non-empty input list AND non-empty output list AND `outputReports[0].reportId == 8`; keeps the LAST matching device |
| None | `nh` (`new:5916`), `sh` (`new:5940`), OLD `Device_Reconnect` (`old:1197`, `collections.length > 2`) | adopts whatever device the caller passes |
| Upgrade | `new:5206` | a collection with exactly one feature report matching the expected id, else the one-in/one-out output-id rule |

On the target only TLC 4 satisfies the report-8 rule: TLC 6 carries report 9 and TLC 5 carries
feature report 6 (`descriptors:25-27`). Both links expose identical collections
(`descriptors:49`), so one code path serves both.

### 4.2 Report id and lengths

| Property | Value | Citation |
|---|---|---|
| Config report id | 8, a module constant | `new:5637` `cr = 8`, `old:1039` `re = 8` |
| Data length | 16 bytes, always | every builder is a 16-element `Uint8Array` (`new:6083`, `new:6092`, `new:6130`, `new:6144`, `new:6187`, `new:6202`, `new:6208`, `new:6264`) |
| Wire length | 17 bytes with the id, matching the descriptor's Windows `OutputReportByteLength` | `descriptors:25` |
| Transport | `sendReport(8, data)` for every config frame, always an output report | `new:6052`, `new:6147`, `new:6271`; `old:1308`, `old:1423` |
| Feature reports | upgrade path only, per-command flag from the image header | `new:5427`, `new:5475`; absent in OLD |
| `receiveFeatureReport` | never called in either driver | grep |
| Report 9 | never used by the config channel; upgrade report ids come from the image, so id 9 is not excluded there | `new:5395`, `new:5427`, `new:5475` |

WebHID delivers the report id as `event.reportId` and the payload with the id stripped, so the
vendor code indexes byte 0 as the opcode with no offset arithmetic (`new:5969-5971`,
`old:1218-1220`). A native implementation on hidraw must prepend the id itself, as the community
script does (`repo:macos-remap-sidebuttons-to-desktop-switch.js:45`).

### 4.3 Frame layout

| Byte | Request | Response |
|---|---|---|
| 0 | command id | command id (echoed) |
| 1 | 0 | status: 0 decode normally, 1 unsupported, anything else ignored |
| 2..3 | flash address, big-endian (commands 7 and 8 only; other builders leave 0) | address echo for command 8 |
| 4 | length, plus 0x80 in the NEW driver when `deviceInfo.type == "keyboard"` | payload length (command 8; echoes the class bit) |
| 5..14 | payload, zero padded | payload |
| 15 | checksum | not checked by either host |

Byte 4 is not always a payload length:

- command 8: the number of bytes to read, with no payload (`new:6208-6209`, `old:1501`).
- command 5: fixed at 2 while the cid is written to byte 7, outside that length
  (`new:6132`, `old:1409`).
- zero-payload commands: the NEW driver writes only the class flag (`new:6093`), the OLD driver
  writes nothing and leaves the template's 0 (`old:1348-1350`).

Template filler bytes are always overwritten: byte 15 starts as 239 (0xEF) in every builder and
`nt()` never reads it (`new:6079` loops `r < e.length - 1`); the scalar writer's template bytes
8 and 77 at [5] and [6] are replaced by `r[5] = t, r[6] = 85 - t` (`new:6203`).

### 4.4 The class bit (NEW only)

`new:6085` `r[4] = t.length + (A.type == "keyboard" ? 128 : 0)`, repeated at `new:6093`,
`new:6132`, `new:6145`, `new:6194`, `new:6203`, `new:6209`, `new:6269`. The test is
positive-for-keyboard, so any other value (including `null`) clears the bit. The state default is
`"keyboard"` (`new:5664`), and the only assignment anywhere in the bundle is
`Ht.deviceInfo.type = sr("deviceType")` on the Space-key path (`new:25598`), where `sr` returns
`null` for a missing key (`new:24659-24661`). `deviceType` is written by the router only on a
cfg vid+pid match (`new:24697`).

Two consequences a reimplementation must not inherit: with type `"keyboard"` the bit is set on
mouse frames, and with type `null` the bit is clear but the connect dispatch still takes the
keyboard branch and both live-decode dispatches (which test `== "keyboard"` explicitly) go dead
(`new:6000`, `new:6007`, `new:6214`). For the mouse, bit 7 of byte 4 must be clear.

Reply side, and this is a real hazard: the ReadFlashData handler copies `Ie[4]` bytes without
masking (`new:5999`). A 10-byte keyboard read whose reply echoes 0x8A copies 138 bytes: 11 real
bytes from the 16-byte report and 127 out-of-range reads that store as zeros. That the device
echoes the bit is provable from the code itself, since `js` advances only when reply bytes 0..4
equal the request (`new:6278-6282`) and `zo` widens its compare to 5 bytes for opcode 8
(`new:6062`); independently the driver's own re-decode hook tests `r == 130` for a 2-byte
keyboard read (`new:6000`). The OLD driver cannot do this: it never sets the bit.

### 4.5 Checksum

```
nt(frame):  s = sum(frame[0..14]) & 0xFF
            return 85 - s                  # may be negative
frame[15] = (nt(frame) - 8) mod 256         # stored into a Uint8Array
```

Equivalently `byte15 = (0x55 - 0x08 - sum(byte0..byte14)) mod 256`, and on the wire
`(0x08 + sum(byte0..byte15)) mod 256 == 0x55`. `new:6078-6081` and `new:6087`; `old:1334-1337`
and `old:1343`. The community script computes the same byte as `(85 - REPORT_ID - s) & 0xFF`
(`repo:macos-remap-sidebuttons-to-desktop-switch.js:31-35`).

Independent hardware-side corroboration: a vendor upgrade image in the local research corpus
embeds a ready-made 16-byte config frame,
`08 | 0d 00 00 00 03 00 01 d3 00*7 69`; sum(b0..b14) = 228 and (85 - 228 - 8) mod 256 = 0x69,
matching the stored byte, with a second sub-image at `...03 01 01 d3 ... 68` (sum 229 -> 0x68).

Neither host ever verifies a checksum on a received frame: the input handler contains no `nt()`
call (`new:5968-6040`).

The same `nt()` without the `-8` is the *block* check byte for multi-byte records; see 7.2.

### 4.6 Transaction, timing and retries

NEW `zo(frame)` (`new:6046-6076`):

| Step | Behavior |
|---|---|
| visit mode | returns true without sending (`new:6048`) |
| send | `Ls = true; await sendReport(8, frame); yh(200)` (`new:6052`) |
| watchdog | `clearTimeout(ai); c2 = false; ai = setInterval(wh, 200)` armed after the send resolves, never cleared on success (`new:6259-6260`) |
| poll | `await _2(1)` per iteration (`new:6073`) |
| timeout | `c2` set -> attempt++ and resend on the next pass (`new:6058`) |
| mismatch | attempt++ and resend (`new:6071`) |
| accept | reply bytes 0..2 equal the request, or 0..4 for opcode 8, or `Ie[1] === 1` at any position in the compare loop (`new:6061-6070`) |
| exhaustion | 5 sends (attempts 0..4), then returns false (`new:6049`, `new:6075`) |
| send throw | logs and returns false immediately, no retry (`new:6053-6054`) |

OLD `ke(frame)` (`old:1301-1332`): 5 attempts; a send exception is only logged; polls 5 ms up to
40 times (~200 ms); then compares against the last received buffer *even when no reply arrived*,
so a stale matching buffer passes; 10 ms gap before the next attempt.

Both accept a status-1 reply as success before any byte compare, so a setter cannot distinguish
an error reply from an ack, and both update the flash shadow on such a reply
(`new:6064`, `old:1319-1321`, `new:6205`).

For any command except 8 the accepted echo is only 3 bytes (command, status, address high). A
write ack for a different low address byte or a different length is accepted.

### 4.7 Device-pushed reports and cross-talk

`StatusChanged` (10) is never sent by either host; it appears only as a case in the input
dispatch (`new:6005`, `old:1255`). `EncryptionData` is a host request answered by the device, not
a push, despite the community doc listing it under "inbound ... Sent on connect"
(`repo:PROTOCOL.md:97`).

The wait flag is cleared for **every** report-8 frame, outside the status switch
(`new:6038`, `old:1293`). So a push arriving during a pending transaction is compared against
that request: in the NEW driver a mismatch consumes one of the 5 attempts and resends the pending
frame immediately, meaning the same write can reach the device more than once; in the OLD driver
it ends the 200 ms wait early, fails the compare and resends after 10 ms. Exception in the NEW
driver: on the keyboard `StatusChanged` bit 0x80 path the clear happens only after three further
awaited transactions, or not at all if one throws (`new:6007`, `new:6038`).

### 4.8 No serialization

`Ie` (last response), `Ls` (wait flag) and the watchdog counter `ri` are module globals with no
lock, queue or in-flight test (`new:5640-5644`). `zo` and `js` share them. Timer callbacks (1.5 s
online poll, 5 s battery poll, 1 s pairing poll) and the input handler itself can enter the
transport while a user setter is awaiting. Two concurrent `zo` callers additionally clobber each
other's timeout flag and timer (`ai`, `c2`, armed only at `new:6052`). Correctness rests entirely
on the 3-byte compare plus the retries. The OLD driver shares `ce` and `he` the same way.

### 4.9 Bulk read `js(start, endExclusive)`

The second parameter is an exclusive END ADDRESS, not a count: the loop condition and the chunk
length are both computed against it (`new:6265-6268`). The keyboard callers make sense only this
way (`js(pe.EffectPara, 9376)`, `js(pe.CustomLightMaps, 7456)`, `js(pe.ReportRate, 9456)`).

| Property | Value |
|---|---|
| Chunk size | `min(10, end - addr)` (`new:6268`) |
| Mouse connect | `js(0, 256)`: 26 reads, addresses 0, 10, ... 250, last is 6 bytes |
| Acceptance | reply bytes 0..4 must equal the request; a status-1 reply is NOT accepted here |
| Mismatch | resend the same address; 5 consecutive mismatches abort the range |
| No reply | loops forever on 1 ms sleeps; the only exit is the 1 s watchdog closing the device |
| Send throw | `console.error` then `break`, aborting the whole range (`new:6266-6275`) |
| Return | always `false`; every call site discards it (`new:6263`, `new:6290`) |
| Pre-fill | `ee.fill(255)` in `m2` before the read, so an aborted range stays 0xFF (`new:6214`) |
| Watchdog | resets `ri` on any report-8 frame while running (`new:6277`) |

OLD `tt()` always asks for 10 bytes and therefore reads 0x000..0x103 (26 reads); on a mismatch it
logs `read fail` and retries the same address with no bound (`old:1505-1524`).

Address truncation: both `Bt` and `st` store the address into `Uint8Array` elements
(`new:6187`, `new:6194`, `new:6208`), so an address at or above 65536 aliases modulo 65536 and
the reply is written back into the shadow at the aliased offset. No call site in either bundle
can reach such an address; record it as a reimplementation hazard.

### 4.10 Block write `st(addr, bytes)`

Splits into `ceil(len/10)` frames at `addr + 10k`, byte 4 carries the chunk length, unused
payload bytes are zeroed, and it **stops at the first failed chunk**; the shadow is mirrored only
if the last chunk succeeded (`new:6186-6199`, `old:1479-1493`). `st` has no return statement, so
no caller can detect a partial write. A zero-length payload computes a chunk count of zero: no
frame is sent, no shadow byte is touched, and the call resolves normally.

Torn-record exposure: within a write that starts at 0 (the 256-byte profile import), chunk
boundaries fall on multiples of 10, so the 4-byte records at 28, 48, 68 and 108, and the scalars
at 169 and 179, straddle a frame boundary. Single-field setters start at the record's own base
and are one frame, so they are atomic.

### 4.11 Response matching summary for a reimplementation

Do not copy the vendor acceptance rule. It accepts on 3 bytes, accepts any status-1 frame for any
command, and accepts stale buffers in the OLD driver. A correct implementation should match
command, status 0, full address and length, and should reject unsolicited frames instead of
counting them as failed attempts.

---

## 5. Commands

NEW enum `Fe`, 16 ids (`new:5531-5548`). All are referenced by a builder or by the input handler.

| Id | Name | Request byte 4 and payload | Response bytes read | Notes |
|---|---|---|---|---|
| 1 | EncryptionData | 8; [5..8] random 0..255, [9..12] zero | [9] cid, [10] mid, [11] link type | challenge bytes never compared on return (`new:6097-6108`, `old:1378-1389`) |
| 2 | PCDriverStatus | 1; [5] state | ignored | sent only when the driverOnline flag is set; false in both cfgs and NEW never calls the setter (`new:6109-6112`) |
| 3 | DeviceOnLine | 0 (+class) | [5] online, [6..8] receiver address stored reversed | precedes almost every setter (`new:6113-6118`) |
| 4 | BatteryLevel | 0 | [5] percent, [6] charging==1, [7..8] mV big-endian | byte 4 is never read (`new:5983`) |
| 5 | DongleEnterPair | 2; [5]=0, [6]=0, [7]=cid | status-0 echo starts the pair poll | the cid sits outside the declared length (`new:6129-6133`) |
| 6 | GetPairState | 0 | [5] 1 pairing, 2 fail, 3 success; [6] seconds left | 1 s poll, max 20 (`new:6134-6138`) |
| 7 | WriteFlashData | chunk length; [2..3] addr, [5..] data | echo only | handler decodes [3..4] and [4] into unused locals (`new:5991-5995`) |
| 8 | ReadFlashData | byte count; [2..3] addr | [2..3] addr, [4] length, [5..] data | the only command matched on 5 bytes (`new:6207-6212`) |
| 9 | ClearSetting | 0 (+class) | a status-0 command-9 frame clears `isRestoring` | bypasses the retry loop, single raw `sendReport` (`new:6139-6151`) |
| 10 | StatusChanged | never sent | [5] bitmask, see 9.4 | device push only |
| 14 | GetCurrentConfig | 0 | [5] active profile | status 1 marks profiles unsupported (`new:6009-6011`, `new:6031-6033`) |
| 15 | SetCurrentConfig | 1; [5] profile index | ignored | followed by a full flash re-read (`new:6158-6165`) |
| 18 | ReadVersionID | 0 | [5] major decimal, [6] minor as 2-digit lowercase hex | rendered `v<dec>.<hex2>` (`new:6014-6017`) |
| 22 | SetLongRangeMode | 10; [5] mode, [6..14] zero | ignored | 10-byte payload with one significant byte (`new:6172-6179`) |
| 23 | GetLongRangeMode | 0 | [5] == 1 means on | status 1 marks it unsupported (`new:6180-6182`) |
| 29 | GetDongleVersion | 0 | same format as 18 | NEW sends it on every link and also once while offline; status 1 renders `v1.0` (`new:6183-6185`, `new:5859`, `new:6035`) |

OLD-only commands that are actually sent:

| Id | Name | Request | Response | Notes |
|---|---|---|---|---|
| 24 | SetDongleLight | 10; [5] mode, [6..8] rgb, [9] speed, [10] brightness, [11] time | ignored | not gated by DeviceOnLine (`old:1455-1460`) |
| 25 | GetDongleLight | 0 | [5] mode, [6..8] rgb, [9] speed, [10] brightness, [11] time | sent on wireless connect and after reset; status 1 marks unsupported (`old:1473-1478`) |

OLD enum-only, never referenced outside the enum (`old:966-984`): 11 SetDeviceVidPid,
12 SetDeviceDescriptorString, 13 EnterUsbUpdateMode, 16 ReadCIDMID, 17 EnterMTKMode,
20 Set4KDongleRGB, 21 Get4KDongleRGBValue, 176 MusicColorful, 177 MusicSingleColor,
240 WriteKBCIdMID, 241 ReadKBCIdMID. The NEW enum drops all of these plus 24 and 25.

Byte-value collision to keep straight: 176 and 177 are `MusicColorful` and `MusicSingleColor` in
the OLD config protocol, and `Cmd_PrepareDownLoad` and `Cmd_DownLoadFile` in the NEW upgrade
protocol (`new:5138-5139`). They are different channels with different report ids.

Status-1 handling differs by bundle: NEW covers 23, 14 and 29 (`new:6027-6037`); OLD covers 23,
14 and 25 (`old:1282-1292`). Any other status value reaches no decoder in either driver but still
clears the wait flag.

---

## 6. Settings memory map

### 6.1 Mouse map `de` (identical in both drivers)

`new:5549-5573`, `old:986-1010`.

| Offset | Size | Field | Kind |
|---|---|---|---|
| 0 | 2 | polling rate | scalar |
| 2 | 2 | DPI stage count | scalar |
| 4 | 2 | current DPI stage (0-based) | scalar |
| 6..9 | 4 | no named or literal host use | unknown |
| 10 | 2 | lift-off distance | scalar |
| 12 + 4i | 4 | DPI stage i value, i in 0..7 | struct |
| 44 + 4i | 4 | DPI stage i color | struct |
| 76 | 2 | DPI indicator mode | scalar |
| 78 | 2 | DPI indicator brightness (raw) | scalar |
| 80 | 2 | DPI indicator speed (raw) | scalar |
| 82 | 2 | DPI indicator on/off | scalar |
| 84..93 | 10 | no host use | unknown |
| 94 | 2 | light power save (offsets 94 and 95) | scalar, exported, no UI caller in either driver |
| 96 + 4i | 4 | button i function | struct |
| 120..127 | 8 | key records 6 and 7 | reached only if keysCount is 8 |
| 128..159 | 32 | no host use | unknown |
| 160 | 7 | lighting struct: mode, r, g, b, speed, brightness, check | struct |
| 167 | 2 | lighting on/off (offsets 167 and 168) | scalar |
| 169 | 2 | debounce, ms | scalar |
| 171 | 2 | motion sync | scalar 0/1 |
| 173 | 2 | sleep time, tens of seconds | scalar |
| 175 | 2 | angle snap | scalar 0/1 |
| 177 | 2 | ripple control | scalar 0/1 |
| 179 | 2 | lights off while moving | scalar 0/1 |
| 181 | 2 | highest performance on/off | scalar 0/1 |
| 183 | 2 | highest performance timeout, tens of seconds | scalar |
| 185 | 2 | sensor mode | scalar |
| 187..255 | 69 | no host use | unknown |
| 256 + 32i | 32 | button i keystroke slot | buffer |
| 768 + 384i | 384 | button i macro slot | buffer |

Literal offsets outside the map: 94 and 167 in setters (each a 2-byte scalar, so 95 and 168 are
covered); 161, 164, 165 and 167 in the decoder (161..163 read through `Buffer_To_Color`). Offset
166 never appears as a literal: it is the check byte of the 7-byte lighting struct at 160.

Offsets 0x100..0x103 are read by the OLD connect loop and not by the NEW one; no decoder in
either driver reads them.

### 6.2 Keyboard map `pe` (NEW only)

`new:5574-5592`. 17 entries, disjoint from `de`.

| Offset | Size | Field |
|---|---|---|
| 0 .. 6143 | 12 x 512 | key layers: system x 1536 + layout x 512 + index x 4, 128 keys of 4 bytes |
| 6144..6175 | 32 | hole |
| 6176 | 1280 | CustomLightMaps: 128 records of 10 bytes |
| 7456..8495 | 1040 | hole |
| 8496 | 2 | CurrentLightMode |
| 8498 | 2 | LightState |
| 8500 | 2 | OffLightTime |
| 8502..8543 | 42 | hole |
| 8544 | 512 decoded (832 read) | EffectPara: 16 blocks of 32 bytes |
| 9056..9375 | 320 | read into the shadow, never decoded |
| 9376..9407 | 32 | hole |
| 9408 | 2 | ReportRate |
| 9410 | 2 | BootAnimation |
| 9412 | 2 | BootAnimationState |
| 9414 | 2 | ForbidKeyFunction |
| 9416 | 2 | CurrentSystem |
| 9418 | 2 | FNLockState |
| 9420 | 2 | FullKeyLockState |
| 9422 | 2 | MusicState |
| 9424 | 2 | ScanKeyTime |
| 9426 | 2 | MultikeyType |
| 9428 | 2 | WASDKey |
| 9430..9455 | 26 | read into the shadow, never decoded; a profile import writes zeros to 9430..9439 |
| 9456..10063 | 608 | hole |
| 10064 + 384i | 384 | macro slot i, i in 0..14 (ends at 15823) |
| 15824..16383 | 560 | past every map and every bounded read |

The keyboard map is why the NEW shadow is 16384 bytes: slot 14 ends at 15823, far past the OLD
driver's 8192.

Keyboard layer addressing uses `aa = ["win","mac","iOS","android"]` while the `CurrentSystem`
scalar at 9416 is encoded and decoded through `["win","mac","android","iOS"]` (`new:5648`,
`new:6875-6876`, `new:7036-7038`, `new:7130-7131`). Codes 0 and 1 agree; code 2 means iOS for the
layer address and android for the flash scalar.

The per-model `systems`/`layouts` lists in `home/cfg.json` (2 x 2 for every keyboard model) are
never read: the only reads are against the hardcoded 4 x 3 protocol state, which nothing narrows
(`new:5799-5800`, `new:6799`, `new:6803`).

---

## 7. Encodings, with worked byte examples

### 7.1 Scalar

Two bytes `[v, (0x55 - v) mod 256]`. Written by `Pe(addr, v)` as a length-2 WriteFlashData
(`new:6201-6206`, `old:1494-1499`):

```
07 00 <hi> <lo> 02 <v> <85-v> 00 00 00 00 00 00 00 00 <ck>
ck = (0xEF - hi - lo) mod 256          # since v + (85-v) = 85
```

Neither driver ever verifies the complement on read; scalars decode from the first byte only
(`new:6324`).

### 7.2 Struct check byte

`(0x55 - (sum(data bytes) & 0xFF)) mod 256`, computed by the same `nt()` without the `-8`
(`new:6078-6081`). Used for the DPI value record, DPI color, key function, the lighting struct,
keystroke buffers and macro contexts. The full-macro writer uses `nt(events) - count`, which is
the same value mod 256 because the count byte is excluded from its array (`new:6780`).

Neither driver checks a stored check byte on read.

### 7.3 Polling rate

| Rate | Code |
|---|---|
| 125 | 8 |
| 250 | 4 |
| 500 | 2 |
| 1000 | 1 |
| 2000 | 16 |
| 4000 | 32 |
| 8000 | 64 |

Setter: `rate <= 1000 ? 1000/rate : rate/2000*16` (`new:6415`, `old:1687`).
Decoder: `code >= 16 ? code/16*2000 : 1000/code`, then clamped to the link cap for display only
(`new:4877-4880`, `new:6324`).

Trap: the shared helper `ReportRate_To_FlashData` maps `rate > 1000 ? rate/1000*16 : 1000/rate`,
giving 2000 -> 32, 4000 -> 64, 8000 -> 128 (`new:4872-4875`, `old:865-868`). It is used only by
profile imports (OLD mouse import `old:3251`, NEW keyboard import `new:7129`). On any link whose
cap is above 1000 Hz a clamped import therefore writes a code that reads back doubled, and 128 is
not a listed value at all. Hyperpace must use 16/32/64.

Erased and zero bytes decode as 31875 and Infinity respectively; only the display clamp hides
them.

Worked frames at offset 0 (all end in 0xEF):

| Rate | Record | Frame |
|---|---|---|
| 1000 | `01 54` | `07 00 00 00 02 01 54 00*8 EF` |
| 2000 | `10 45` | `07 00 00 00 02 10 45 00*8 EF` |
| 4000 | `20 35` | ... |
| 8000 | `40 15` | ... |
| 500 | `02 53` | ... |
| 250 | `04 51` | ... |
| 125 | `08 4D` | ... |

### 7.4 DPI stage record

`[idxLo, idxLo, flags, check]` at 12 + 4i (both drivers write X = Y).

- flags bits 2-3 and bits 6-7 each carry index bits 8-9 (X and Y).
- flags bit 0 and bit 4 are the doubling flags (`DPIex` 0x11); bits 1 and 5 double again
  (range index 3 hardcodes 0x33 with divisor 4).
- The decoder reads only byte 0 and flag bits 0..3; byte 1 and the high nibble are ignored on
  read (`new:6311-6320`).

Encoder (`new:6443-6446`): pick the highest sensor range whose `min <= dpi`; range 1 or 2 give
divisor 2 with `flags = range.DPIex`, range 3 gives divisor 4 with flags 0x33; then
`idx = (dpi/div)/range[0].step - 1`, or a table lookup if the sensor has a `values` array. Note
the step always comes from range 0 even for range 1.

Sensor tables for 3950:

| Config | Ranges |
|---|---|
| OLD `sensor.json#3950` | 100..30000 step 50 DPIex 0; 30100..60000 step 100 DPIex 17 |
| NEW `home/sensor.json#3950` | 50..30000 step 50 DPIex 0 (single range, no `values`) |

Worked examples (stage 0, frame check `(0xE1 - 4i) mod 256`, so 0xE1 at stage 0):

| DPI | Table | Record | Frame |
|---|---|---|---|
| 100 | either | `01 01 00 53` | `07 00 00 0C 04 01 01 00 53 00*6 E1` |
| 800 | either | `0F 0F 00 37` | `07 00 00 0C 04 0F 0F 00 37 00*6 E1` |
| 30000 | either | `57 57 88 1F` | `07 00 00 0C 04 57 57 88 1F 00*6 E1` |
| 30100 | NEW (1 range) | `59 59 88 1B` | ... |
| 30100 | OLD (2 ranges) | `2C 2C 55 A8` | ... |
| 32000 | NEW | `7F 7F 88 CF` | ... |
| 32000 | OLD | `3F 3F 55 82` | ... |
| 40000 | OLD | `8F 8F 55 E2` | ... |
| 50 | NEW only | `00 00 00 55` | ... |

The decoder is the same function in both drivers, so both `7F 7F 88 CF` and `3F 3F 55 82` read
back as 32000. An erased record `FF FF FF FF` decodes as 204800 (index 1023, both doubling bits
set). Note that NEW's index 639 (32000) is past its own table maximum of 599 (30000); which form
the firmware accepts is not visible in host code.

Missing-value fallback: the `values`-table branch is a bare scan whose loop variable is used
directly as the index afterwards, so a no-match yields `values.length` and a DPI computed from a
past-the-end index rather than an error (`new:6316-6319`). The branch is dead for the 3950, which
has no `values` table, but eight of the eleven models in `home/sensor.json` do carry one.

Beyond 3950, the NEW `sensor.json` adds only sensor 8960 (100..16000 step 100; 16200..32000 step
200 DPIex 17; values 1..160). The feature lists `Setting`, `ModeSelect`, `LOD`, `Ripple`, `Angle`
and `MotionSync` are textually identical in both files, and 3950 is in all of `LOD`, `Ripple`,
`Angle` and `MotionSync`.

Host-side bound the NEW bundle cannot satisfy: `mouseCfg.sensor.cfg` initialises to `{}` and
nothing in the `/home/` bundle ever assigns it (`new:5693`), so the decoder's non-table branch
would dereference `undefined.range[0]`. The OLD driver assigns it from `sensor.json` before
connecting (`old:2541`).

### 7.5 DPI stage color

`[r, g, b, check]` at 44 + 4i. `Color_To_Buffer` accepts only `rgb(r, g, b)` strings and returns
`[0,0,0]` for anything else, including hex (`new:4890-4893`). Example, `rgb(255,0,0)` at stage 0:
record `FF 00 00 56`, frame `07 00 00 2C 04 FF 00 00 56 00*6 C1`.

### 7.6 DPI indicator

| Offset | Field | Encoding |
|---|---|---|
| 76 | mode | 0 off, 1 steady, 2 breathing (names from `site/lang/en.json#DPIEffectOptions`) |
| 78 | brightness | levels 1..10 map to raw 16, 30, 60, 90, 128, 150, 180, 210, 230, 255 |
| 80 | speed | raw, no mapping |
| 82 | on/off | 0/1 |

Decode of brightness: multiples of 30 map to `raw/30 + 1`; 16 -> 1, 128 -> 5, 230 -> 9, 255 -> 10,
anything else -> 5. So 0 -> 1, 120 -> 5, 240 -> 9 (`new:6476-6527`). Setting a mode also writes
state 1 if the indicator was off; `Set_MS_DPILightOff` writes state 0
(`new:6463-6466`, `new:6532-6535`). Level 3 gives record `3C 19` at offset 78.

### 7.7 Lighting

7-byte struct at 160: `[mode, r, g, b, speed, brightness, check]`. Every color, brightness and
speed setter rewrites the whole struct from cached state; `Set_MS_LightMode(0)` writes only the
scalar at 167 (and only if the light was on) and leaves the struct untouched
(`new:6543-6564`, `old:1814-1835`). The decoder clamps speed and brightness above 9 to 9.

Worked: mode 3, `rgb(255,0,0)`, speed 8, brightness 3 gives record `03 FF 00 00 08 03 48` and
frame `07 00 00 A0 07 03 FF 00 00 08 03 48 00 00 00 4A`.

Mode names from the OLD language file: 0 Off, 1 Rainbow, 2 Single Color Breath, 3 Fixed Color,
4 Neon, 5 Rainbow Breath. The shared `LightMode_To_Disable` disables all controls for 0, color
for 1/4/5, nothing for 2, speed for 3, color and speed for 6 (`new:4895-4920`, `old:894-921`);
mode 6 exists in that function but not in the language list. Control sliders run 0..9.

### 7.8 Other scalars

| Field | Offset | Values |
|---|---|---|
| Lift-off distance | 10 | 1 = 1 mm, 2 = 2 mm, 3 = 0.7 mm (the UI drops 3 for sensors other than 3950) |
| Debounce | 169 | 0..`cfg.maxDebounce` (15); a warning shows below `tipsDebounce` (8). Example 8 ms: `07 00 00 A9 02 08 4D 00*8 46` |
| Sleep time | 173 | tens of seconds: 1, 3, 6, 12, 30, 60, 90 (10S, 30S, 1MIN, 2MIN, 5MIN, 10MIN, 15MIN). Example 6: `07 00 00 AD 02 06 4F 00*8 42` |
| Motion sync / angle snap / ripple | 171 / 175 / 177 | 0/1; records `01 54` and `00 55` |
| Highest performance | 181 on/off, 183 timeout | same tens-of-seconds codes as sleep time, NOT seconds |
| Sensor mode | 185 | 0 LP, 1 HP; the language list also carries 256 CORDED, which the OLD UI refuses to write (it restores the previous value instead) |

### 7.9 Long range

Not in the mapped flash. `SetLongRangeMode` (22) carries a 10-byte payload with the value in byte
0 of the payload; `GetLongRangeMode` (23) returns state at [5] and marks the feature unsupported
on status 1.

```
on   16 00 00 00 0A 01 00 00 00 00 00 00 00 00 00 2C
off  16 00 00 00 0A 00 00 00 00 00 00 00 00 00 00 2D
get  17 00 00 00 00 00 00 00 00 00 00 00 00 00 00 36
```

### 7.10 Read and reset frames

```
first connect read   08 00 00 00 0A 00*10 3B
NEW last read (250)  08 00 00 FA 06 00*10 45
OLD last read (250)  08 00 00 FA 0A 00*10 41
factory reset (mouse) 09 00 00 00 00 00*10 44
```

---

## 8. Buttons, keystrokes, media keys, macros

### 8.1 Action type enum `Ts`

`new:5593-5607`, `old:1011-1025`, identical in both drivers.

| Id | Name | Param |
|---|---|---|
| 0 | Disable | none (the UI sends NaN, which stores as 0) |
| 1 | MouseKey | 0x0100 left, 0x0200 right, 0x0400 wheel, 0x0800 backward, 0x1000 forward |
| 2 | DPISwitch | 0x0100 loop, 0x0200 DPI+, 0x0300 DPI- |
| 3 | LeftRightRoll | 0x0100 scroll left, 0x0200 scroll right |
| 4 | FireKey | `(interval << 8) + times`, times 0..3 (0 = repeat while held), interval 10..255, unit not in code |
| 5 | ShortcutKey | 0; the content lives in the keystroke slot at the button's own index |
| 6 | Macro | `(buttonIndex << 8) + cycle`; cycle 1..250, 254 until released, 255 until any button pressed |
| 7 | ReportRateSwitch | none |
| 8 | LightSwitch | enum name only |
| 9 | ProfileSwitch | enum name only |
| 10 | DPILock | enum name only |
| 11 | UpDownRoll | enum name only |

`Ts.LeftKey = 256` sits in the same object but is the MouseKey **param** for left click, not a
type: the UI compares it against `keys[i][1]` to refuse unbinding the last left click
(`old:4391`). Types 8..11 have no menu entry and no param encoding anywhere; only 7 is offered in
the language file. `MouseFireKey = 13` belongs to the separate keyboard enum `u2`
(`new:5608-5622`), not to `Ts`.

### 8.2 KeyFunction record

4 bytes at 96 + 4n: `[type, param >> 8, param & 0xFF, (0x55 - sum) mod 256]`, written as one
`st` chunk after a DeviceOnLine check (`new:6605-6612`, `old:1876-1885`). The param is
big-endian. `Set_MS_KeyFunction` does not bound the index by `keysCount`.

Button count: NEW hardcodes `keysCount: 6` and never reassigns it (`new:5743`); OLD sets it from
`cfg.keys.length`, which is 5 (`old:2541`). Index 5's record is at 116..119, its keystroke slot
at 416, its macro slot at 2688.

Read-back stores the type as a base-16 **string** (`new:6332-6333`, `old:1536`). Consequence for
a reimplementation: a consumer that loose-compares that string against the numeric enum aliases
16 -> `"10"` == DPILock and 17 -> `"11"` == UpDownRoll, while genuine types 10 and 11 render
`"a"` and `"b"` and can never match their own enum members. The two comparisons the vendor
actually makes (`== Ts.ShortcutKey`, `== Ts.Macro`, i.e. 5 and 6) are single hex digits and do not
alias. The param is stored as `"0x%04x"`, and `"0x0100" == 256` is true under loose equality,
which is how the left-click guard works.

### 8.3 Default key layouts

NEW `home/cfg.json#mouse[0].cfg[0].keys`, 6 entries (array order, with the device index):

| loc | index | value |
|---|---|---|
| (160, 280) | 0 | 1 / 0x0100 |
| (100, 230) | 1 | 1 / 0x0200 |
| (220, 180) | 2 | 1 / 0x0400 |
| (306, 260) | 4 | 1 / 0x1000 |
| (360, 250) | 3 | 1 / 0x0800 |
| (290, 155) | 5 | 2 / 0x0100 |

OLD `cfg.json#mouse[0].cfg[0].keys` and `cfg[1].keys`, 5 entries: (144,253) idx0 1/0x0100,
(144,100) idx1 1/0x0200, (96,177) idx2 1/0x0400, (248,352) idx4 1/0x1000, (363,352) idx3
1/0x0800. These are the values the OLD driver writes when a macro is deleted
(`old:4531-4543`).

Both bundles also carry an 8-entry in-code placeholder list with a different param scale
(`[1,0x0001]`, `[1,0x0002]`, `[1,0x0004]`, `[1,0x0010]`, `[1,0x0008]`, `[2,0x0001]`,
`[2,0x0002]`, `[2,0x0003]`) at `new:5744-5760` and `old:1149-1165`. Only the first `keysCount`
entries are overwritten from flash on connect, and nothing ever sends the placeholder params. The
0x0100 scale is the live one.

Model shape hazard: the state default and cfg hold `{value: [type, param]}` objects while the
decoder and both setters replace element n with a bare two-element array, so the array mixes
shapes for indices a connect never reached. The OLD UI seeds itself from the cfg objects and
thereafter consumes bare arrays.

### 8.4 Keystroke slot (256 + 32n)

```
[2k, k press entries, the same k entries in reverse as releases, check]
entry = [kind | 0x80 (press) or kind | 0x40 (release), value lo, value hi]
length = 6k + 2
```

`new:6632-6647`, `old:1903-1925`. Decode reads only the first `b0/2` entries with
`kind = byte & 0x0F` and a little-endian value (`new:6354-6376`). Binding is
`KeyFunction {type 5, param 0}` plus the slot at the same button index; the param never selects
a slot.

Continuation read: the slot is read as 10 bytes, then, if `b0 > 2`, up to `3*b0 + 2` bytes in
10-byte steps (`new:6656-6667`). `b0` is device-supplied and uncapped: a count byte of 255 gives
767 bytes of reads, 735 past the 32-byte slot. For button 0 that covers 256..1022 and for button
5, 416..1182, both reaching into the macro region at 768. A reimplementation must clamp the read
to the slot geometry.

Writers do not pad: a media key writes 8 bytes and a chord writes 6k+2 into a 32-byte slot, and
the shadow is mirrored for exactly that length, so the tail of a previous longer binding survives
on the device and in the host copy. The host bets on the firmware honouring the count byte.

### 8.5 Media keys

Stored as a one-entry keystroke slot of kind 2 (consumer):

```
02 82 <lo> <hi> 42 <lo> <hi> <check>
```

Example, volume up (0x00E9): `02 82 E9 00 42 E9 00 BD` (sum 664, `664 & 255 = 152`,
`85 - 152 = -67` stored as 0xBD). The button is bound as type 5 with param 0
(`new:6616-6623`, `old:1894`, `old:4428`).

The `"0x03"` in `home/cfg.json#medias[n].value[0]` never reaches the device: the writer takes only
the usage string and hardcodes kind 2. 3 is the keyboard enum's `Media` value.

NEW `medias` list, 20 entries:

| icon | usage | icon | usage |
|---|---|---|---|
| light-up | 0x006F | player | 0x0183 |
| light-down | 0x0070 | email | 0x018A |
| mute | 0x00E2 | computer | 0x0194 |
| volume-up | 0x00E9 | calculator | 0x0192 |
| volume-down | 0x00EA | homepage | 0x0223 |
| play | 0x00CD | search | 0x0221 |
| previous | 0x00B6 | favorites | 0x022A |
| next | 0x00B5 | backward | 0x0225 |
| stop | 0x00B7 | forward | 0x0224 |
| | | web-stop | 0x0226 |
| | | refresh | 0x0227 |

Label warning: the NEW config's `backward`/`forward` icons are reversed with respect to the HID
Consumer page (0x0224 is AC Back, 0x0225 is AC Forward) and with respect to the OLD language file
(0x0225 "Next page", 0x0224 "Previous page"). Hyperpace should label by usage id, not by the
config icon name.

The OLD driver has no `medias` block; its Multimedia menu uses a UI-only pseudo type `"1005"`
mapped to 5 at write time, offering 18 usages (the two brightness usages 0x006F and 0x0070 appear
only in the NEW config).

Device-side capability (primary data): the vendor interface declares a Consumer Control
collection, report 5, with one 16-bit usage array bounded at 0x023C, which covers every usage in
the list. Whether kind-2 keystrokes are emitted through report 5 is an inference, not an
observation.

### 8.6 Keymap

104 entries keyed by `KeyboardEvent.code`, with `{value, text, type}` (`new:4244-4765`,
`old:236-757`):

- 8 entries of type 0, the modifier bitmask: LCtrl 1, LShift 2, LAlt 4, LWin 8, RCtrl 16,
  RShift 32, RAlt 64, RWin 128.
- 95 entries of type 1, HID keyboard page 0x07 usage ids: A..Z 4..29, 1..0 30..39, Enter 40,
  Esc 41, Backspace 42, Tab 43, Space 44, Minus 45, Equal 46, `[` 47, `]` 48, Backslash 49,
  Semicolon 51, Quote 52, Backquote 53, Comma 54, Period 55, Slash 56, CapsLock 57, F1..F12
  58..69, PrintScreen 70, ScrollLock 71, Pause 72, Insert 73, Home 74, PageUp 75, Delete 76,
  End 77, PageDown 78, Right 79, Left 80, Down 81, Up 82, NumLock 83, keypad 84..99.
- 1 entry of type 7: ContextMenu, value 1.

No F13+, no IntlBackslash (0x64), no media keys in this table.

The two bundles differ in exactly two entries: ContextMenu is `{1, "Menu", type 7}` in OLD and
`{1, "FN", type "7"}` (a string) in NEW; Backslash's display text is `"|"` in OLD and `"]"` in
NEW. Both differences are load-bearing because the shortcut writer looks keys up by display text
(`new:6640`, `old:1911`):

- In NEW, `"]"` resolves to BracketRight (usage 0x30) before it reaches Backslash (0x31).
- In NEW, the string `type: "7"` breaks the macro encoders, which use `+` not `|`:
  `(2 << 6) + "7"` is the string `"1287"`, which stores as `1287 & 255 = 0x07` and decodes as
  release, while `(1 << 6) + "7"` = `"647"` stores as 0x87 and decodes as press. Press and
  release are inverted through the round trip, and because the block checksum also sums with
  `+`, the check byte becomes garbage. The shortcut encoder uses `| 128`, so it is unaffected.

### 8.7 Macro slot (768 + 384n for the mouse, 10064 + 384n for the keyboard)

| Offset | Content |
|---|---|
| +0 | name length in UTF-8 bytes, accepted 1..30 on read |
| +1..+30 | UTF-8 name, 0xFF padded, no check byte |
| +31 | event count, accepted <= 70 on read |
| +32 + 5k | event k: `[(status << 6) + kind, value lo, value hi, delay hi, delay lo]` |
| after the last event | check byte over the count plus the events |

Status 2 (0x80) is press and 1 (0x40) is release on write; the decoder maps only the top-bit
value 0b10 to press and **everything else**, including 0b00 and 0b11, to release, so an erased
0xFF byte decodes as a plausible `{status: release, kind: 15}` with no error path
(`new:6385-6388`, `old:1662-1664`). The kind is masked to 4 bits on read while both writers build
the byte unmasked.

Maximum slot: 33 + 5 x 70 = 383 bytes. Value is little-endian, delay is big-endian.

Name writer hazard: the writer records the full UTF-8 length in byte 0 but the buffer holds only
30 name bytes, and the length store is itself mod 256. A name of 31..255 bytes makes the reader
reject the slot forever (it returns null); a name of 257..286 bytes wraps byte 0 back into 1..30
and decodes a truncated or corrupt name. Both bundles do this (`new:6733-6739`, `old:1942-1945`).

Null handling: every consumer of the macro decode dereferences the result without a guard
(`new:6346-6347`, `new:6245-6246`, `new:6825-6826`). A slot whose byte 0 is 0 or 255 (an
unanswered read leaves 0xFF, since the shadow is pre-filled) therefore throws a TypeError inside
the connect read, which the connect wrapper catches by closing the device, leaving the state at
Connecting and never reaching Connected (`new:6293-6300`). Hyperpace must tolerate invalid slots.

Continuation reads, both device-controlled and unvalidated:

| Reader | Sizing | Worst case |
|---|---|---|
| name (`m6`) | `b0 + 1` bytes | 256 bytes, 225 past the 31-byte field but inside the slot |
| context (`g6`) | `5 * count + 2` bytes from slot base + 31 | 1277 bytes, 924 past the 384-byte slot into following slots |

Validation (`name 1..30`, `count <= 70`) happens only afterwards, in the decoder.

Delete/restore writes 384 zero bytes (not 0xFF) to the slot, then rewrites the button's cfg
default function (`new:6695-6708`, `old:2006-2018`). Both drivers' `Restore_MS_Macro` call the
online check **without awaiting it**, so the test is a Promise and always truthy: the write goes
ahead regardless of whether the device is online.

### 8.8 Macro authoring limits (OLD UI, the only one available)

- Recording captures keyboard keydown/keyup only, mapped through the keymap, so events are kind
  0, 1 or 7. Mouse buttons enter only through Insert Command as kind 4 with value 0x0001 left,
  0x0002 right, 0x0004 middle, 0x0010 forward, 0x0008 backward, inserted as a press+release pair
  with delay 10 each. Kind 2 never appears in a web-authored macro.
- The 70-event cap applies to **recording only**; Insert Command and Insert Press/Release are
  uncapped, so the UI can save more than 70 events, at which point the payload overruns the
  384-byte region and every reader rejects the macro.
- Auto delay is `floor(timestamp delta)` assigned to the previous event, with the last event at
  0. Delays are clamped to 10..65535 only when edited or set as the default, never when
  auto-recorded, so recorded delays below 10 are real. The unit is not in code.
- Names are stripped of punctuation and whitespace and cut to 30 UTF-8 bytes by the UI.
- The macro library is browser `localStorage` (`macro` in OLD, `mouseMacro`/`keyMacro` in NEW);
  the device stores only macros bound to a button.

### 8.9 Keyboard key records (NEW only)

4 bytes at `system*1536 + layout*512 + index*4`: `[type, value lo, value hi, check]`. The param
is **little-endian**, the opposite of the mouse KeyFunction record; the decoder also computes a
big-endian local that it never uses (`new:6810-6814`, `new:6918-6922`). The type byte's low
nibble is the type (enum `u2`, values 0..13) and the high nibble round-trips untouched with no
meaning assigned anywhere in the bundle. A macro binding carries its slot index in byte 1, and a
slot byte of 15..255 is silently dropped (no slot read, no macro entry) while the key record is
still surfaced.

`Set_KB_KeyRestore` writes a whole 512-byte layer as 52 frames through one `st`, then updates the
UI model unconditionally; since `st` returns undefined and aborts at the first failed chunk, a
mid-layer abort leaves flash half-written, the shadow untouched and the model claiming success.

### 8.10 Platform gating (OLD UI)

The OLD key page disables the dropdowns for UI rows 3 and 4 (device indices 4 forward and 3 back)
whenever `navigator.userAgentData.platform` is not `"Windows"` (`old:4377-4386`, `old:2804`). The
setters themselves have no platform check, and no polling-rate cap by platform exists anywhere in
the code despite the home tip claiming one. This is a UI restriction, not a firmware one.

---

## 9. Connection lifecycle, battery, status changes, reconnect

### 9.1 Connect sequence

NEW `Device_Connect` (`o6`, `new:5858-5860`) and `iu` (`new:6292-6303`):

1. Query DeviceOnLine. If offline: poll it every 1.5 s and send GetDongleVersion once.
2. Once online: clear the online poll, reset the tick counter, arm a 1 s watchdog.
3. If the driverOnline flag is set, send PCDriverStatus(1). It never is on `/home/`.
4. `m2()`: set Connecting, fill the shadow with 0xFF, then branch on `deviceInfo.type`:
   - `"mouse"`: `js(0, 256)`, decode, then the per-button slot reads.
   - anything else (including the default `"keyboard"` and `null`): the four keyboard readers.
5. GetCurrentConfig (14), ReadVersionID (18), GetDongleVersion (29) on every link.
6. Enable battery polling, poll the battery once (DeviceOnLine then BatteryLevel).
7. Long range: wired sets unsupported without sending; wireless sends GetLongRangeMode (23),
   gated on `defaultLongDistance` being defined (it defaults to `false`, which is defined).
8. Set Connected, clear the watchdog, start the 5 s battery interval.

Any exception in steps 4..8 is caught, logged as `Get_Online_Interval` and closes the device.
Because `iu` clears the 1.5 s reconnect interval before the try block and `Device_Connect` arms
that interval only when `iu` returns false, one throw permanently ends auto-reconnect: nothing in
the bundle restarts the timer afterwards.

OLD order (`old:1563-1574`): flash read (its sub-reader arms the 5 s battery interval), battery,
GetCurrentConfig, ReadVersionID, then on wireless only GetDongleVersion, GetDongleLight and
GetLongRangeMode; wired clears `supportLongDistance` and `supportDongleLight` without sending.

### 9.2 Per-button reads after the base read

For each button index below `keysCount`:

| Record type | Reads |
|---|---|
| 5 (ShortcutKey) | slot at 256 + 32i, 10 bytes, then up to `3*b0 + 2` in 10-byte steps when `b0 > 2` |
| 6 (Macro) | name at 768 + 384i, 10 bytes, then up to `b0 + 1` when `b0 + 1 > 10`; context at +31, 10 bytes, then up to `5*count + 2` when `count > 2` |

A 2-event macro context is 12 bytes but fails the `count > 2` test, so bytes +41 and +42 are never
read at connect and keep 0xFF: event 1's delay decodes as `(hi << 8) + 255`.

### 9.3 Keyboard connect cost (NEW)

842 fixed ReadFlashData frames per keyboard connect, every one carrying the class bit:

| Step | Range | Frames |
|---|---|---|
| 12 key layers (4 systems x 3 layouts x 512 B) | 0..6143 | 624 |
| light head | 8496..8501 | 1 |
| effect area | 8544..9375 | 84 |
| custom light maps | 6176..7455 | 128 |
| scalar block | 9408..9455 | 5 |

Plus up to 15 macro slots at roughly 40 frames each, issued between the 624 and the remaining 218.
The macro list is appended to and never cleared, so every reconnect adds 15 more entries to
`keyboard.macros` while all indexing stays on the stale first 15 (`new:6795-6829`); the mouse path
does reset its arrays.

### 9.4 StatusChanged bitmask (command 10, byte 5)

| Bit | Mouse | Keyboard (NEW) |
|---|---|---|
| 0x01 | read (4, 2) current DPI | ignored |
| 0x02 | read (0, 2) polling rate | read (9408, 2) |
| 0x04 | GetCurrentConfig | read (9416, 2) |
| 0x08 | read (76, 8) DPI indicator | clear `isRestoring` (no frame) |
| 0x10 | ignored | read (8498, 2) |
| 0x20 | read (160, 7) lighting | ignored |
| 0x40 | DeviceOnLine then BatteryLevel (only while the battery gate is set) | same |
| 0x80 | ignored | three reads: (8496, 4), (8544 + mode*32, 4), (9422, 2) |

`new:6005-6008`, `old:1255-1258`. None of the mouse handlers is awaited, so they run alongside
any transaction in flight. A ReadFlashData reply for exactly (0,2), (4,2), (76,8) or (160,7)
re-runs the full mouse shadow decode; the keyboard equivalents key on a length byte of 130, i.e.
`0x80 | 2`.

### 9.5 Battery

Wire format: `[5]` percent, `[6]` charging (== 1), `[7..8]` millivolts big-endian. The polled
value is never displayed raw. The display pipeline:

1. On the first reading, latch `useVoltage = (voltage > 0)`.
2. If latched, map millivolts through a 21-point table (3050, 3420, 3480, 3540, 3600, 3660,
   3720, 3760, 3800, 3840, 3880, 3920, 3940, 3960, 3980, 4000, 4020, 4040, 4060, 4080, 4110)
   in 5 percent steps. Above 4110: 99 if charging else 100. Exactly 4110 matches no index and
   returns 0. A "bump" makes results of exactly 0 or 15 become 1 or 16, so below 3050 mV the
   result is 1 percent, not 0.
3. Seed the displayed value from a per-receiver-address `localStorage` record
   (`bat_` + lowercase 2-hex of addr[0], addr[1], addr[2]), clamped by elapsed time: over 1800 s
   use the raw value, under 60 s keep the stored one, otherwise clamp into
   `[stored - 0.014*elapsed, min(100, stored + 0.028*elapsed)]` and return the stored value if
   raw falls inside that band. NEW does not round this result; OLD does.
4. Thereafter step the display by 1 every 10 s toward the target, and by 1 every 60 s while
   charging in slow mode (entered at display >= 85 with raw >= 95 while charging). Discharging:
   a device percent of 0 forces 0, and the first device percent at or below 15 with display at or
   above 15 drops the display to 15. Charging: 8 consecutive device readings of 100 force 100
   (NEW sets it only on the 8th; OLD on every reading from the 8th).

OLD additionally re-polls the battery up to 10 times while charging at 100 percent, and carries a
+/-30 jump rule whose guard flag is never set (dead code).

The smoothing timers are created only while their handles are null and are cleared without
nulling them, so after a disconnect or an upgrade entry a later reconnect in the same page never
restarts them.

### 9.6 Periodic traffic

| Timer | Period | Content |
|---|---|---|
| Online poll | 1500 ms until online | DeviceOnLine, then the whole connect on success |
| Battery poll | 5000 ms after connect | DeviceOnLine, then BatteryLevel if online; if offline, disable the gate and restart the 1.5 s poll |
| Pairing poll | 1000 ms, max 20 ticks | GetPairState |
| Connect watchdog | 1000 ms tick during connect | close after 10 ticks |

Each wake from sleep therefore costs a full flash re-read, and `connectState` stays Connected
while the mouse is asleep.

The pairing poll's "failed poll" exit can never fire: the send helper has no return statement, so
`undefined == false` is false, and only the 20-tick limit or a Fail/Success reply stops it. A
second status-0 pair reply arms a new interval without clearing the old one, orphaning it.

### 9.7 Watchdog, close and disconnect

- Watchdog: 1 s tick, and at 10 ticks it logs `Get_Flash_Time_Tick` and closes the device if the
  state is still Connecting. Neither driver ever assigns the TimeOut state: NEW writes
  `A.connectState == Tr.TimeOut` (a comparison) and OLD writes a bare comma expression.
- `connectState` never returns to Disconnected in either driver; Disconnected is only the initial
  value.
- `Device_Close`: send PCDriverStatus(0) if the driverOnline flag is set, clear every timer, exit
  battery smoothing, mark the device closed (NEW also resets both version strings to `"--"`),
  then close the device unless in visit mode. NEW awaits the close; OLD does not.
- After a close, a pending bulk read keeps spinning: it never times out and its resend is
  suppressed.
- NEW's `ondisconnect` fires only when the removed device's `productName` equals the open one's,
  and it neither closes nor clears the device handle. It is installed only by `Device_Reconnect`,
  which nothing in `/home/` calls, so opens reachable from `/home/` carry no disconnect handling
  at all. The upgrade module overwrites `navigator.hid.ondisconnect` with a no-op.
- NEW's `connect`/`disconnect` listeners only raise a flag (`hidDeviceChangeEvent`) that nothing
  reads; the connect listener is a 200 ms leading-edge throttle, not a debounce.

### 9.8 Reconnect paths

| Path | Driver | Behavior |
|---|---|---|
| `Get_HistoryDevicesInfo` | NEW | enumerates `getDevices()` with no VID/PID test, applies the report-8 collection rule, opens every match in turn without closing the previous, runs EncryptionData + DeviceOnLine per device when the granted list changed, otherwise only DeviceOnLine. The handle ends on the last device. Its re-entrancy flag leaks on an empty list or any throw, after which every later call returns the stale list. No caller in `/home/`. |
| `Device_Reconnect` | NEW | opens the passed device, attaches the handler and the disconnect handler, records the vendor id, runs EncryptionData. No caller. |
| `Get_Current_Device_Online` | NEW | opens whatever device is passed, attaches the handler, returns the online probe. No collection test, no handshake, no flash read, no timers. Leaves cid/mid/type at their defaults (1, 1, 1) and the rate cap at 1000. No caller. |
| `Device_Reconnect` | OLD | takes the first granted device with more than 2 collections and starts connect without opening it or attaching the handler. No caller. |
| `Get_ConnectDevice_Info` | OLD | filters `getDevices()` by vendorId, productId and productName from `localStorage.deviceInfo`, applies the collection rule, opens, handshakes. Reached only from `init()`, which exits immediately because `cfg.json#test` is true; and OLD never writes that key. Dead in practice. |

The live OLD path is Space or the Connect button -> `Request_Device` -> `Device_Connect`.

### 9.9 Exported state that is stale by construction

`devicePID`, `visit` and `driverOnlineFlag` are copied into the export object when it is built,
before any device is open, so later assignments never show through them. The variable behind
`devicePID` is assigned a **vendorId**, not a productId. As a result the OLD driver's PID-based
mouse/keyboard classification after `requestDevice` never matches and falls back to the component
default `"mouse"`.

---

## 10. Pairing, profiles, factory reset, long range, receiver light

### 10.1 Pairing

Request: `05 00 00 00 02 00 00 <cid> 00*7 <ck>`. The length byte says 2 while the cid sits at
byte 7, outside it.

| Driver | cid source |
|---|---|
| OLD | always `info.cid` from the handshake; default 1 if no handshake ran in that page load |
| NEW | `pairCID` when non-zero, else `info.cid`; `/home/` sets `pairCID = cfg.mouse[0].cid` = 62 |

A status-0 reply clears the battery gate and starts a 1 s `GetPairState` poll; a Fail or Success
reply stops the poll and re-enables the gate if connected; 20 ticks force Fail. Reply fields:
`[5]` status (1 pairing, 2 fail, 3 success), `[6]` seconds left (read into state, displayed
nowhere in either bundle).

UI flow (OLD): the user holds left + right + middle for 3 s until the light flashes, plugs in the
new receiver, presses Space, selects the receiver in the chooser, then START PAIRING or Space
sends command 5 (`site/lang/en.json#DialogPairTips`, `#PairTips`).

NEW `/home/` runs the same dialog but never assigns the device handle (only `rh`, `nh` and `sh`
do, and none is called), so the command-5 send throws inside the try and returns false. The
navigation to `/mouse` also starts before the dialog opens.

### 10.2 Profiles

`GetCurrentConfig` (14) returns the active profile at byte 5 and marks profile switching
supported; status 1 marks it unsupported. `SetCurrentConfig` (15) writes a 1-byte payload after a
DeviceOnLine check, ignores the reply, then re-reads all of flash and sets Connected; the function
returns the online flag, not the ack. A device-pushed profile change (StatusChanged 0x04) only
re-reads the profile index, never the flash.

The OLD UI offers four profiles (values 0..3), disables the selector when status 1 was seen, and
reverts the selection when the setter returns false. If no option matches, the loop index equals
the list length (4) and that value is sent.

### 10.3 Factory reset

1. DeviceOnLine gate; skipped entirely in visit mode.
2. Set `isRestoring`, clear the battery gate.
3. Send command 9 once with a raw `sendReport`, outside the retry loop, with no echo check and no
   resend. The send is not wrapped in try/catch, so a send failure leaves `isRestoring` true.
4. Poll every 300 ms up to 4 times (1.2 s) for the reply to clear `isRestoring`. For keyboards the
   StatusChanged bit 0x08 also clears it.
5. If cleared: re-read flash, GetCurrentConfig, and on a wireless link that supports it write
   SetLongRangeMode(defaultLongDistance). OLD additionally re-reads the receiver version and the
   receiver light first.
6. Re-enable the battery gate either way. If no reply arrived, nothing is re-read.

`defaultLongDistance` is copied from `cfg.longDistance` (true) in OLD, so an OLD reset writes
mode 1. In the NEW bundle it is only ever the hardcoded `false`, so a NEW reset would write 0
unless the unread `/mouse` page sets it.

The OLD "device is restoring" dialog never appears: the settings component copies the flag once at
creation and nothing updates the copy, which also means the guard against re-triggering a reset is
always open.

### 10.4 Long range

See 7.9 for the bytes. Queried at connect on wireless only; wired forces unsupported without
sending. The OLD UI exposes it as a Performance toggle and as a Settings switch with a
confirmation dialog, shown only when `supportLongDistance` is true. The NEW `/home/` bundle has no
UI caller.

### 10.5 Receiver light (OLD only)

`SetDongleLight` (24) writes `[mode, r, g, b, speed, brightness, time]` as a 10-byte payload;
`GetDongleLight` (25) returns the same fields at bytes 5..11; status 1 marks the feature
unsupported. Command 25 is sent on a wireless connect and after a reset. Every mode, color,
brightness and speed control re-sends the whole record, and none of these setters is gated by
DeviceOnLine. Defaults: mode 0, `#ff0000`, brightness 5, speed 5, time 1. UI quirk: the speed dial
sends `speed - 1` in one angle branch and `speed` in the other, while the brightness dial sends
`- 1` in both.

The NEW driver has no receiver-light commands and no receiver-light state.

---

## 11. Export and import file format

Only the OLD driver has export/import UI. The NEW bundle exports a `Write_Mouse_Flash` writer but
has no caller for it.

### 11.1 File layout (OLD)

```
[0 .. 8191]        full flash shadow (unread areas are 0xFF, since connect pre-fills with 255)
[8192 .. 8223]     "Compx Inc", 32 bytes, NUL padded
[8224 .. 8239]     device type string (16 bytes)
[8240 .. 8255]     sensor type string (16 bytes, mouse only)
```
Total 8192 + 64. Saved as `<name>.bin`.

### 11.2 Import checks and clamps

| Check | Behavior |
|---|---|
| Trailer string | must be `Compx Inc`, else "config error" |
| Type or sensor mismatch | "config mismatch" |
| Length | not checked; missing bytes become `undefined` and are written as 0 |
| Per-record checksums | not checked |
| Polling rate | clamped to the link cap via `ReportRate_To_FlashData`, which is the wrong encoder above 1000 Hz (see 7.3) |
| DPI stage count | clamped to `cfg.dpis.length` (6) |
| Current DPI stage | clamped to count - 1, but its check byte is written from the **count** (`c[5] = 85 - c[2]`), a real vendor bug |

### 11.3 Write sequence

1. DeviceOnLine gate, clear the battery gate, set Connecting.
2. Write bytes 0..255 as 26 chunked frames through the normal retrying transaction. Stops at the
   first failed chunk with no error reported.
3. Re-decode the shadow regardless.
4. Per button, compare the 32-byte keystroke region and the 384-byte macro region against the
   shadow and rewrite any region that differs **in full** (the list of differing offsets is built
   and never used). A region whose imported bytes equal 0xFF where the connect never read is
   skipped even if flash differs.
5. Set Connected and re-enable the battery gate without checking any write result.

Loop bound: 6 buttons in NEW, 5 in OLD.

### 11.4 Keyboard profile import (NEW)

Writes, in order: 12 key blocks of 512 B, the 6-byte light head, 16 effect blocks of 32 B, 128
custom-light records of 10 B, and a 32-byte scalar block at 9408 (of which only bytes 0..21 are
assigned, so 9430..9439 are zeroed). Then it fires the per-macro writes **without awaiting them**
and immediately re-enables battery polling and replaces the whole keyboard state object, so up to
15 macro writes, each of which begins with its own online probe, race that commit and then index
into the newly swapped object. The key loop is bounded by the imported profile's array lengths but
indexes by the module's own system/layout order, so a differently ordered profile is written to
the wrong regions.

---

## 12. Upgrade code in the new driver

### 12.1 What it can do

The NEW bundle contains a complete WebHID firmware updater: it fetches a `.bin` over XHR, parses
its header, re-enumerates the device by boot-mode VID/PID, sends a reset-to-update command, a
prepare/erase command and 32-byte data chunks, and waits for device acks and a success state
(`new:5129-5529`). v1's "the web driver has no firmware update path" is true of the OLD driver
only.

### 12.2 Image header

Little-endian u32 at 0 headCRC, 4 headLength, 8 fwLength, 12 nextFileAddress, 16 version; u8 at 20
DeviceType, 21 Cid, 22 Mid; then eleven 64-byte fields from offset 23: fileId, icName,
bootInputEndPoint, bootOutputEndPoint, normalInputEndPoint, normalOutputEndPoint,
resetToUpdateModeCmd, prepareDownLoadCmd, dataDownLoadCmd, senserName, productName (ending at
726). String forms drop every 0x00 byte, not just trailing ones.

Integrity: `headCRC == 0x55555555 - sum(bytes 8 .. headLength-1)`. Verified against 13 header
instances in the local research corpus; all match. If `nextFileAddress != 0` the file holds two
images and both headers must pass.

Nothing in the code checks `version`, `DeviceType`, `Cid`, `Mid` or any name field: there is no
model match and no newer-than check. The firmware body is read from offset 8192 within each
sub-image.

### 12.3 Command templates

Each 64-byte command field is `[0] cmdlength (including the report id), [1] feature flag,
[2..] command, where command[0] is the report id`. On the wire: report id `command[0]`, payload
`command[1 .. cmdlength-1]`. Only two feature flags are consulted: the reset command's and the
data command's (the latter also governs the prepare packet, so the prepare flag is dead). This
is the only per-command feature flag in either driver, and it belongs to the upgrade channel, not
to the settings channel.

In all 13 local vendor images the reset command is report id 8 with cmdlength 17 (16 bytes on the
wire), and prepare/data are report id 6 or 9 with cmdlength 49 (48 bytes on the wire).

### 12.4 Data packet

64-byte scratch buffer; `cmdlength - 1` bytes go out after the report id.

| Byte | Content |
|---|---|
| 0 | report id from `dataDownLoadCmd` |
| 1 | 0xB1 `Cmd_DownLoadFile` |
| 2 | 0xC0 next packet, 0xC1 last packet |
| 3 | chunk length (32, or the remainder) |
| 4 | 0 |
| 5..8 | flash address, big-endian, starting at `dataDownLoadCmd.command[5..8]`, +32 per packet |
| 9..16 | 0 |
| 17..48 | image bytes, 0xFF padded |

The queue starts with the `prepareDownLoadCmd` template verbatim, then one packet per 32-byte
chunk of `fwLength` bytes from image offset 8192.

### 12.5 Wait loops and replies

| Phase | Behavior |
|---|---|
| Reset to update | resent every 21 ticks of a 10 ms interval (~210 ms), up to `timeout/(10*20)` = 50 sends for the UI's 10 s, including a ~210 ms delay before the first send. Succeeds when an input report equals the sent bytes except the last. A send exception sets DeviceError; exhaustion returns false with no state change. |
| Download | a 10 s inactivity timer restarts after every send (not on replies), so a long erase can trip TimeOutError |
| 0x5B reply | requires byte 1 == 0xB5. Byte 2 in {1, 2} with byte 3 == 1 dequeues the queue head (the prepare packet in the expected order). Byte 2 == 5 (CheckOK) is ignored. Byte 2 == 0x88 is success. |
| 0xB1 reply | must echo the sent packet from index 1 over the reply length, else the packet is resent; the 10th consecutive mismatch at one queue length sets RepeatCountMax, which is transient and is overwritten by TimeOutError 10 s later |
| 0xB0 reply | ignored outright |
| 0x5A reply | ignored; the upgrade continues |

The upgrade input handler does not filter by report id.

### 12.6 Device selection during upgrade

Filters are `[normal, boot]` VID/PID parsed from the header's output-endpoint strings
(`vid_XXXX&pid_XXXX`, case-sensitive lowercase tokens, `parseInt(..., 16)`). Collection rule: a
collection with exactly one feature report matching the expected id, otherwise exactly one input
and one output report with a matching output id. The expected ids start as `[8, 6]` and are
replaced by the image's own report ids only **after** being pushed, so the first chooser after a
load still looks for the previous ids. On no match the code assigns DeviceInvalid to the reactive
object itself rather than its state field, so the UI never sees it.

### 12.7 Entry and exit from the config channel

`Set_Device_EnterUpgrade` sends PCDriverStatus(0) if enabled, clears all timers and hands the open
device to the updater. If its VID/PID match the header's normal VID/PID, that same device is
reused and its input handler is replaced by the upgrade handler and `ondisconnect` by a no-op, so
config replies are no longer decoded. `Set_Device_ExitUpgrade` re-runs Device_Connect **without**
re-attaching the protocol handler, after which transactions can only time out, the online check
reads a stale buffer, and a bulk read waits until the watchdog closes the device.

### 12.8 What it cannot do here

- No firmware image for this device is available and neither config references one for cid 102.
  The NEW config's upgrade links sit under cid 62 and are named for a different product line.
- Nothing in the NEW bundle reads the cfg upgrade block (0 property accesses for the `1000`,
  `4000`, `8000`, `device` or `compel` keys), and nothing compares firmware versions: both
  version strings are only ever assigned.
- The only reachable entry point in `/home/` is a **resume**: pressing Enter, or OK in the
  "upgrade first" tip, when `localStorage.upgradingInfo.href` equals the current URL. The dialog
  is mounted with `enterUpgrade: false`, so `Set_Device_EnterUpgrade` is never called from
  `/home/`. A 10 s countdown must elapse before Start is clickable, and a `beforeunload` guard is
  then armed.
- The download URL is `window.location.href + link` by string concatenation, so links beginning
  with `/` produce a double slash. Nothing was fetched.

Whether the feature-report flag ever applies to this device cannot be settled locally. The
descriptor does expose feature report 6 (7 bytes) in TLC 5, and the module's default boot report
id is 6, but every local image moves 48 bytes on report 6 or 9, which matches the 48-byte report 9
in TLC 6 rather than the 7-byte feature report.

---

## 13. Community repo audit

Repo: single commit `a4700d10` (2026-06-09) with `PROTOCOL.md`, `README.md`,
`macos-remap-sidebuttons-to-desktop-switch.js`, `photo.png`, `LICENSE`.

### 13.1 Provenance

`PROTOCOL.md` was written from the same NEW build as our copy: it names `assets/index-BTVblIUr.js`
under `/home/`, and every minified symbol it cites resolves to the matching function in our
pretty-printed bundle.

### 13.2 Claim-by-claim

| Repo claim | Verdict |
|---|---|
| Report ID = 8 | agrees |
| 16 bytes of payload per frame | agrees |
| Checksum `(85 - REPORT_ID - sum(b0..b14)) & 0xFF` | agrees, byte-exact |
| Byte 1 is a constant 0 | wrong: byte 1 is a status byte in responses, and status 1 is accepted as success before any echo compare |
| Byte 4 `length + classBit`, bit 7 = keyboard | agrees, and it documents this ahead of v1 |
| Bytes 2..3 big-endian address | agrees |
| "The mouse uses OutputReport mode by default; some devices use FeatureReport, the firmware-config blob has a `feature: bool` flag per command" | misfiled: the flag is real but belongs to the firmware image's upgrade command records; settings traffic is always `sendReport(8)` |
| Filter table, VID 0x373B and 0x3554 with 8 PIDs | agrees with the NEW config; its "16 combinations" holds for the mouse lists alone while the page builds 85 |
| Filter table omits 0xFB14 | matches the NEW config; the repo's own script adds 0xFB14, which is right for the hardware |
| Keyboard VIDs 0x05AC, 0x3554, 0x388D | agrees |
| Command enum, 16 ids | agrees, identical to the NEW enum |
| EncryptionData "Sent on connect" | ambiguous rather than wrong: it is a reply to a host request, listed under inbound decoders |
| DeviceOnLine, BatteryLevel, GetPairState, GetCurrentConfig, ReadVersionID, GetDongleVersion reply layouts | agree |
| PCDriverStatus treated as a no-op by the host | agrees |
| StatusChanged bit table | agrees; its 0x40 row omits the preceding DeviceOnLine and the battery gate |
| WriteFlashData ack "Ie[3..4] address echo, Ie[4] length echo" | restates two unused locals in the input handler; the real acceptance rule compares bytes 0..2 |
| ClearSetting = factory reset ack | agrees |
| Flash offset table (23 fields) | offsets all agree |
| `ReportRate` at 0 is uint16 | wrong: a 1-byte scalar with a complement byte |
| `CurrentDPI` at 4 is uint16 | wrong: same, and it is a 0-based stage index |
| `maxDpiStage` "probably index of the highest enabled stage" | wrong: it is the stage count; the current stage is clamped to count - 1 |
| DPI value "uint16 big-endian + 0x55 parity?" | wrong: `[idxLo, idxLo, flags, check]` with index bits 8-9 in the flag nibbles and doubling flags |
| DPI color "(R,G,B + parity)" | agrees |
| LOD "1 or 2 mm" | incomplete: 3 = 0.7 mm exists for this sensor |
| Performance time at 183 "seconds" | wrong: tens of seconds |
| Sensor mode "one of sensor.json mode codes" | wrong: `sensor.json#ModeSelect` is a list of sensor ids; the codes are 0 LP, 1 HP (256 CORDED is UI-only and never written) |
| `keysCount = 6 (verified against cfg.json)` | agrees with the NEW config, which is cid 62; v1's 5-button model came from the OLD config (cid 102) |
| Default key map and hotspots | agrees with the NEW config, ordered by index |
| Action type list | agrees for 0..11 |
| `MouseFireKey = 13` listed as a mouse type | wrong: it belongs to the keyboard enum |
| `LeftKey = 256` listed as an action type | wrong: it is the MouseKey param for left click |
| FireKey param "(count<<8) or mouseBitmask" | wrong: `(interval << 8) + times` |
| ShortcutKey/media `param = slot index` | wrong: the host writes param 0 and uses the button index as the slot; the repo's own script does the right thing |
| Macro `param = slot index` | incomplete: the high byte is the index, the low byte is the cycle mode |
| Behaviors for types 8..11 ("toggles light effects", "sniper", "scroll-wheel direction") | unsupported by any code; guesses |
| Keystroke slot `[0] keyCount * 2` and entry layout | agrees, but the prose omits the reverse-order release half and the check byte |
| Media slot bytes `02 82 lo hi 42 lo hi ck` | bytes right, reading wrong: the leading 2 is the usual `2k` count and 0x82/0x42 are press/release of kind 2 |
| Media type `0x03` | wrong for the mouse: the stored kind nibble is always 2 |
| Media usage list | values right; only 14 of 20 rows, `stop` left unresolved (it is 0x00B7), and backward/forward copied from the config's swapped icon names |
| Macro slot: name in `[0..30]`, context at `[31..]` | agrees; the length prefix at +0 is not called out |
| "The exact event-record byte format ... was not recovered" | honest gap; recovered in section 8.7 |
| Keymap: 104 entries, two type classes | count right, classes wrong: there are three (0, 1 and a lone type 7) |
| `keymap_Jn.txt next to this doc` | the file does not exist in the repo and the single commit never added it |
| Per-byte parity "firmware verifies it" | unverified: no host code checks a complement or a check byte on read, and the OLD import writes a wrong complement without apparent consequence |
| Script: key constants ControlLeft/ArrowLeft/ArrowRight | agree with the keymap |
| Script: frame builder and checksum | byte-exact with the vendor |
| Script: KeyFunction record and address math | agree |
| Script: `--restore` writes 1/0x0800 and 1/0x1000 | agrees with the NEW config defaults |
| Script: device selection by usage page 0xFF02..0xFF06 with a fallback to the first PID match | looser than the vendor rule, which requires output report id 8; platform-dependent and unverified |
| Script: read accepted on opcode + address only | weaker than the vendor: no status byte, no length check |
| Script: writes are fire-and-forget with a fixed 30 ms gap | diverges: the vendor waits for a per-chunk echo and aborts the block on failure. The 30 ms figure appears nowhere in either bundle |
| Script: no EncryptionData handshake, no DeviceOnLine check | diverges from both drivers; whether the firmware requires them is unknown |
| Script: read-back verification of buttons 3 and 4 | covers only the two 4-byte KeyFunction records, never the 14-byte slots, and prints values without comparing them |
| Script: reply offset inferred from `reply.length === 17` | uses length as a proxy for a fact the HID layer reports directly; the repo's own doc says the payload arrives with the id stripped |
| Script: slot bytes for Ctrl+Left / Ctrl+Right | byte-exact under the vendor algorithms. Note the OLD web UI would produce different bytes for Ctrl+Left because its encoder looks keys up by display text and `"<-"` resolves to Backspace first |
| README "web based HID firmware flasher" / "flash firmware on your own" | wrong: the script writes settings records with WriteFlashData; real firmware flashing is a separate module it does not touch |
| README "doesn't support certain features on macOS" | agrees: a UI gate on key rows 3 and 4 for any non-Windows platform, matching the screenshot's home tip. The tip's claimed 1000 Hz macOS cap is not implemented anywhere |

### 13.3 What the repo adds beyond v1 and the bundles

The class bit in byte 4; the 6-button model with a top DPI button at index 5; the reduced opcode
set; the NEW config's filter lists, keyboard VIDs, hotspot coordinates and media usage table; and
an unverified field report that type-5/param-0 records written over hidraw on macOS do remap the
side buttons. Everything else it gets right is explicit in the NEW bundle or its config.

---

## 14. Corrections to v1

| # | v1 said | Correction |
|---|---|---|
| 1 | v1:103 "8192-byte shadow" | OLD only. The NEW driver's shadow is 16384 bytes, because the keyboard map runs to 15823 |
| 2 | v1:103-104 "reads 0x000..0x103 on connect (26 reads of 10 bytes)" | OLD only. NEW reads 0x000..0x0FF: 26 reads of min(10, 256-addr), the last one 6 bytes |
| 3 | v1:61-63 "poll every 5 ms, up to 200 ms ... retry up to 5 times, 10 ms apart" | OLD only. NEW arms a 200 ms interval after the send, polls every 1 ms, resends on timeout or mismatch with no backoff, 5 sends total, and returns false on a send throw |
| 4 | v1:54 byte 4 = "payload length" | NEW adds 0x80 when the device type is keyboard. Byte 4 is also not always a payload length: for command 8 it is the read count, and for command 5 it is 2 while the cid sits at byte 7 |
| 5 | v1:63-64 request/response except the pushed command 10 | Correct, but any report-8 frame clears the wait flag, so a push arriving mid-transaction is compared against the pending request and consumes a retry, which can duplicate a write |
| 6 | v1:68-87 command table including 24 and 25 | The NEW enum has 16 ids and drops 11, 12, 13, 16, 17, 20, 21, 24, 25, 176, 177, 240, 241. It also adds a status-1 case for 29 that renders "v1.0" |
| 7 | v1:99 "The web driver has no firmware update path" | True of OLD only. The NEW bundle ships a complete WebHID updater (section 12) |
| 8 | v1:22-23 "cfg.json has no upgrade block, so the button permanently reads LATEST VERSION" | True of OLD (the key is absent, not null). The NEW config has an upgrade block under cid 62 and under all 13 keyboard entries, though no NEW code reads it |
| 9 | v1:124 "button i function, i in 0..4" | The NEW config has 6 keys (index 5 = DPI switch 0x0100) and the NEW driver hardcodes `keysCount = 6`, reaching records at 116..119, slot 416 and macro slot 2688 |
| 10 | v1:34 "cid 102 (0x66), mid 1 or 2" | OLD config only. The NEW config's only mouse entry is cid 62 mid 1. Which the hardware reports is unresolved |
| 11 | v1:149-150 3950 ranges and maxDpi 40000 | OLD config only. The NEW `sensor.json` has a single range 50..30000 step 50 DPIex 0 and `maxDpi` 32000, so 30100 and 32000 encode to different bytes under the two configs |
| 12 | v1:139 "cfg has no lightEffect" | OLD only. The NEW config adds `lightEffect {mode 0, brightness 3, speed 8, movingOffState false}` |
| 13 | v1:29-31 filters | The NEW chooser builds 85 filters from the combined mouse and keyboard lists and omits 0xFB14 entirely; an unfiltered Space-key chooser can still pick a wired unit, which then routes to `origin + "undefined"` |
| 14 | v1:196 "read size: 3 * byte0 + 2" | Correct as a formula, but `byte0` is device-supplied and uncapped: a value of 255 reads 767 bytes, 735 past the 32-byte slot and into the macro region |
| 15 | v1:202-204 name length 1..30, count <= 70 | Those are read-side validations only. Neither writer enforces them, and the continuation-read sizes are taken from the device bytes before any validation |
| 16 | v1:208 delays "10..65535 ms" | Clamped only on manual edit or as the default; auto-recorded delays can be below 10. The unit is not in code |
| 17 | v1:215-223 connect sequence | OLD order. NEW: flash, 14, 18, 29 (every link), battery, then 23 on wireless; and NEW dispatches the whole read by `deviceInfo.type`, taking a keyboard path for any non-"mouse" value |
| 18 | v1:223 "If still connecting after 10 s, close" | The close happens, but neither driver assigns the TimeOut state (NEW writes a comparison, OLD a comma expression), and NEW's counter resets on every report-8 frame during the bulk read, so it bounds silence rather than total connect time |
| 19 | v1:228-229 factory reset sequence | OLD. NEW drops the receiver-version and receiver-light re-reads, and writes SetLongRangeMode from a `defaultLongDistance` that the NEW bundle never sets (so 0) |
| 20 | v1:231-232 "command 5 with cid" | NEW uses `pairCID` when non-zero, which `/home/` sets to `cfg.mouse[0].cid` (62); OLD always uses the handshake cid, which is the default 1 if no handshake ran in that page load |
| 21 | v1:221 "wireless only: receiver version (29)" | NEW sends 29 on every link, including wired, and also once while the mouse is offline |
| 22 | v1:39-40 "report ID 9 ... the web driver never uses" | Still true for the config channel. The NEW upgrade module takes its report ids from the firmware image header, so id 9 is not excluded there |
| 23 | v1:52 status 0 = ok, 1 = unsupported or error | Correct, plus: both drivers return success on a status-1 reply before any echo compare, and scalar and block writes update the host shadow on such a reply |
| 24 | v1:113 polling rate codes | Correct for the setter and decoder. The shared helper used by profile imports maps 2000 -> 32, 4000 -> 64, 8000 -> 128, which the decoder reads back doubled |
| 25 | v1:42-43 collection selection | The exact rule holds for the protocol module. The NEW router uses a looser rule (non-empty report lists), and three adopt paths apply no collection test at all |
| 26 | v1:104 "writes go out in 10-byte chunks" | Correct, plus: the block stops at the first failed chunk, the shadow is mirrored only if the last chunk succeeded, and the writer returns nothing so no caller can detect a partial write |
| 27 | v1:71 PCDriverStatus "only when cfg driverOnline" | Correct; and the flag is false in both configs, and the NEW bundle never calls the setter that would set it |
| 28 | v1:18-19 localStorage keys | Correct for OLD. NEW uses a different set including `deviceType`, `rooter`, `upgradingInfo`, `mouseMacro`, `keyMacro`, a `connectDevice` key that is written and never read, and a `hidDevices` key that is read and discarded; an HTTP 401 on its axios instance clears all of it |
| 29 | v1:181-183 hotspot coordinates | OLD config. The NEW config's coordinates differ and it has a sixth hotspot |
| 30 | v1:178-179 macOS side-button restriction | Correct as a UI restriction, on key rows 3 and 4, for any non-Windows platform; the claimed macOS 1000 Hz cap is not implemented anywhere in the code |
| 31 | v1:248 import clamp bug `c[5] = 85 - c[2]` | Confirmed |
| 32 | v1:20-21 "no cryptographic check on the host side" | Confirmed, and stronger: the reply's echo of the 4 challenge bytes is never compared |
| 33 | v1:116 LOD "3 = 0.7 mm (3950 only)" | Confirmed (the UI removes option 3 for other sensors) |
| 34 | v1:134 performance timeout in tens of seconds | Confirmed (the community doc's "seconds" is wrong) |
| 35 | v1:144 "the web writes X = Y" for DPI | Confirmed |

---

## 15. Open questions

### 15.1 Only the missing page bundles can settle these

1. What `https://www.lofree.tech/mouse` serves. Every NEW setter, the cid/mid lookup, the
   export/import and restore UI, the button and macro editors, the DPI slider bounds, the lighting
   panel, and the code that sets `deviceInfo.type` for a real session live there.
2. Whether that page assigns `mouseCfg.sensor.cfg` (without it the NEW DPI decoder throws),
   `keysCount`, `defaultLongDistance` and `driverOnline`.
3. The NEW language file (`lang/<code>.json`, fetched at runtime): whether its button menu offers
   the brightness media usages, types 8..11, or different DPI and scroll params.
4. Whether any page calls `Get_Current_Device_Online`, `Get_HistoryDevicesInfo` or
   `Device_Reconnect`, which are exported and unreferenced in `/home/`.
5. Whether the keyboard page narrows `systems`/`layouts` from the config (2 x 2) or keeps the
   hardcoded 4 x 3, i.e. whether a keyboard connect reads 4 layers or 12.

### 15.2 Only hardware could settle these

6. Which cid and mid the device reports in the EncryptionData reply: 62 (NEW config) or 102 (OLD
   config). This decides whether a sixth remappable button exists and which config entry, sensor
   table and `maxDpi` apply.
7. Whether the firmware verifies the `[v, 85-v]` complement and the block check bytes, and what it
   does with a bad one. No host code checks either on read, and the OLD import writes a wrong
   complement with no visible consequence.
8. What the firmware returns for status byte values other than 0 and 1, and whether replies echo
   bytes 0..4 exactly (including the class bit in byte 4).
9. Whether the firmware requires the EncryptionData handshake or a DeviceOnLine probe before it
   accepts a write. The community script skips both; only an unverified field report suggests it
   works.
10. How the firmware handles the duplicate writes the NEW retry loop can produce when an
    unsolicited push lands mid-transaction.
11. How long the ClearSetting reply takes compared with the host's 1.2 s wait.
12. Whether the firmware honours the count byte of a keystroke slot and ignores the untouched tail
    of a shorter rebinding (the host writes only 8 or 6k+2 bytes into a 32-byte slot).
13. What the firmware stores at the unmapped offsets: mouse 6..9, 84..93, 128..159, 187..255 and
    0x100..0x103; keyboard 6144..6175, 7456..8495, 8502..8543, 9376..9407, 9430..9455 (which every
    keyboard import zeroes in part) and 9456..10063.
14. What DPI flag bits 1 and 5 (the second doubling, x4) do, and whether the firmware accepts an
    undoubled index above 599 (the NEW encoding for 32000).
15. What rate code 128 means (writable by an OLD import clamp on an 8 kHz link).
16. Whether the firmware accepts a type-6 button whose macro slot is zeroed or invalid, and what
    it emits. The web driver aborts the connection when it reads such a slot.
17. How the firmware treats keystroke kind 7 and kind 0 modifiers inside a macro.
18. Whether kind-2 (consumer) keystrokes are emitted through Consumer report 5, and whether
    horizontal scroll uses the AC Pan field of the mouse report.
19. The unit of the FireKey interval and of macro delays, the polling-rate cycle order for type 7,
    and the DPI loop wrap behavior.
20. Whether the receiver and the wired interface report the same `productName` (the NEW disconnect
    handler matches on it alone).
21. Whether a real key record can carry a keyboard macro slot index of 15..255, which the host
    silently drops.
22. What the two images in a two-image firmware file target, and whether a wired unit answers
    GetDongleVersion.

### 15.3 Unresolvable from static reading for other reasons

23. No firmware image for this device is available (the local corpus has 13 images, none matching
    PID 0xFB16/0xFB14 or cid 102), so the HYPACE bootloader VID/PID, report ids, feature flags and
    command bytes are unknown.
24. Browser behavior: whether `requestDevice({filters: []})` lists every HID device; whether
    WebHID coerces the NEW chooser's unparsed hex-string filters; whether each input event carries
    a fresh `ArrayBuffer` at offset 0 (the handler builds a view over the whole buffer).
25. What `node-hid`/hidapi delivers as a reply length on Linux and macOS, which is what the
    community script's offset heuristic guesses at.
26. Whether the NEW `home/sensor.json` copy in hand is the file the page actually receives: the
    page requests it from the origin root while the config comes from the page href, and no
    response header was saved for the local copy.
27. Whether the OLD app is the page the NEW chooser hands off to: it reads the same `deviceInfo`
    key, but the origins differ (`lofree.tech` vs `www.lofree.tech`) and the NEW rooter is
    `/mouse`, not `/`.
28. Coverage: the completeness review was still reporting gaps (10 in its final round), so
    features present in the bundles may be undocumented here.

---

## 16. Verification ledger

| Metric | Value |
|---|---|
| Claims extracted and verified | 401 |
| Confirmed as stated | 304 |
| Partially correct, restated with a correction | 96 |
| Refuted, not stated in this document | 1 |
| Unverifiable | 0 |
| Dropped dimensions | none |

### 16.1 By dimension

| Dimension | Claims | Focus |
|---|---|---|
| D1 | 40 | transport, framing, checksum, chooser, transaction |
| D2 | 48 | command set and per-command semantics |
| D3 | 58 | mouse settings memory map and encodings |
| D4 | 40 | buttons, keystrokes, media keys, macros |
| D5 | 52 | connection lifecycle, battery, reconnect, export/import |
| D6 | 38 | firmware upgrade module |
| D7 | 71 | community repo audit |
| G1 | 24 | keyboard path, class bit, keyboard flash map |
| G2 | 12 | writer divergences, unread config keys, input-handler dispatch |
| G3 | 18 | unbounded reads, failure paths, dead exports |

### 16.2 Method and its limits

- Static reading only. No JavaScript from either bundle or from the community repo was executed,
  no HID device was opened, nothing was written to `/dev/hidraw*`, nothing was installed, and no
  file was fetched during verification.
- Every claim carries a code citation with a quoted fragment, and each was re-checked against the
  source by a second pass empowered to correct or refute it. Ninety-six claims were corrected in
  that pass; one was refuted and is not stated here.
- The single refuted claim concerned an alleged unbounded keyboard macro-slot enumeration. The
  enumeration is in fact bounded at 15 by a short-circuit write guard. Nothing in this document
  relies on it.
- Arithmetic in section 7 (checksums, worked frames, DPI indices, battery table boundaries) was
  recomputed by hand from the quoted algorithms rather than copied.
- What this ledger does **not** cover: device behavior. Every statement about what the firmware
  does with a frame is either quoted host code or listed in section 15. The honest summary is that
  this document specifies what the vendor host sends and how it decodes what comes back, with no
  frame ever observed on the wire.
