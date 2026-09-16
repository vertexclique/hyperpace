# Compx ComUsbUpgradeFile update procedure: implementable spec

**Verification column legend**

| Code | Meaning |
|---|---|
| W | Observed on the wire in a vendor USB log (Pulsar 1K CX52650N run, Scyrox V6 and V8 NRF52840 runs) |
| A | Measured across all 45 archived sibling header instances (35 files, 10 chained second images) |
| S | Read in vendor source only (WebHID JS, .NET IL, reference TypeScript); no device observation |
| U | Unverified for the HYPACE specifically, or unverified generally; stated as such |

No HYPACE image or device observation exists, so **every row carries an implicit U for the HYPACE**. Rows marked U below are additionally unverified even for the siblings.

---

## 0. Scope, sources and preconditions

Four implementations were read; three flash.

| Impl | What it is | Role here |
|---|---|---|
| 1 | Lofree `www.lofree.tech/home` WebHID bundle | Primary readable reference; serves a page titled "Lofree HYPACE" whose filter includes 0xFB16 |
| 2 | Pulsar cMouse WebHID bundle | One template with Impl 1, statement-for-statement in the flasher |
| 3 | Pulsar / Scyrox .NET desktop updater | The only implementation with real USB logs |
| 4a | OpenMouse `atk-index.ts` | Read-only parser, no device IO. Mirror its bounds checks, not its CRC check |
| 4b | Lofree hyzen keyboard hub | A different container and two different bootloader protocols. UX and safety reference only |

**Preconditions (S, W for the enforcement, U for the HYPACE policy)**

| # | Precondition | Source | Verif |
|---|---|---|---|
| P1 | Update the mouse over the cable, not the 2.4 GHz link | Vendor support article, verbatim: "When you update the mouse, make sure the mouse is wired" and "Please keep the mouse in wired mode when updating it." Sibling brand, same ODM | S, U for HYPACE |
| P2 | Exactly one candidate device connected | Verbatim "please disconnect all other mice and dongles from your computer"; the .NET tool enforces `Count == 1` on both the normal input and normal output lookups, and again on both boot lookups, aborting with a multi-device error | S |
| P3 | Sufficient battery | Command 4 returns percent at byte 5, charging flag at byte 6, mV big-endian at bytes 7..8; already polled every 5 s by the vendor driver. **Threshold undefined anywhere** | S, U |
| P4 | The config session must release the device | The vendor engine sends PCDriverStatus, clears five timers and the battery smoothing state, then hands the open handle to the updater. Hyperpace's own connect sequence runs a 5 s battery poll and a command-3 probe before every setter, both on the same report-8 collection that carries the reset | S |
| P5 | Linux: the boot node must be writable | Vendor hidraw nodes are `root:root 0600`; a `TAG+="uaccess"` rule keyed on `ATTRS{idVendor}=="3554"` covers a 0x3554 bootloader node, but 7 of the 13 sibling boot identities are under VID 0x3710 and no rule anywhere covers those. No pre-flight writability check is specified | S, U |

---

## 1. Image format

Little-endian header at offset 0.

| Offset | Size | Field | Verif |
|---|---|---|---|
| 0 | u32 | `headCRC` | A |
| 4 | u32 | `headLength` = 720 in 45/45 | A |
| 8 | u32 | `fwLength` | A |
| 12 | u32 | `nextFileAddress` (0, or absolute offset of image 2) | A |
| 16 | **u32** | `version`, `major<<8 | minor` in the low half | S (all four parsers read 4 bytes), A (high half zero in 45/45, max 0x509) |
| 20 | u8 | `DeviceType`: 210 mouse, 211 dongle. 209 observed on Lofree keyboard images (meaning inferred, unverified) | A |
| 21 | u8 | `Cid` | A |
| 22 | u8 | `Mid` | A |
| 23 + 64*i | 64 | Eleven NUL-padded ASCII fields, i = 0..10 | A |

Field order (i): 0 `fileId` = `ComUsbUpgradeFile`, 1 `icName`, 2 `bootInputEndPoint`, 3 `bootOutputEndPoint`, 4 `normalInputEndPoint`, 5 `normalOutputEndPoint`, 6 `resetToUpdateModeCmd`, 7 `prepareDownLoadCmd`, 8 `dataDownLoadCmd`, 9 `senserName`, 10 `productName`. Corroborated independently by the .NET tool's own log, which prints those names in that sequence (W).

**Correction to carry (A):** the eleven fields span `23 + 64*11 = 727` bytes, **seven more than `headLength` 720**. The last 7 bytes of `productName` lie outside both the declared header and the headCRC span (bytes 8..719). Bytes 720..726 are zero in 45/45, which is the only reason this is invisible. Do not size the header struct at 720.

**Payload:** starts at fixed image offset **8192**, spans `fwLength` bytes. Constant in all four implementations (S). Bytes 720..8191 are zero in all 45 headers, so there is no signature field (A).

**Payload is sent verbatim.** No implementation decrypts, signs, re-hashes or transforms it. Proven byte-for-byte: CX image `[8192:8224]` equals the first logged data packet payload, and the same for the NRF image (W).

**Payload at rest, by chip family (A):** NRF52833/NRF52840 and CH32V305 look like ciphertext (entropy ~7.99; an identical aligned 16-byte block appears at payload+16 across Pulsar and Scyrox products, pointing at one deterministic ODM cipher). CX52660 is **plaintext Thumb code** (entropy 6.96, readable BLE symbol strings). CX52650N is scrambled by an undetermined scheme. That the bootloader decrypts is inference (U).

**Command field sub-encoding (S, W):** each 64-byte command field is `[0]` = cmdlength (total on-wire report length including the report ID), `[1]` = feature flag, `[2..]` = the report bytes beginning with the report ID. Measured across 45 images: reset cmdlength 17 in 45/45, prepare and data cmdlength 49 in 45/45.

---

## 2. Validation, before anything is sent

| # | Check | Rule | Verif |
|---|---|---|---|
| V1 | `fileId` | equals `ComUsbUpgradeFile` | A |
| V2 | `headLength` | equals 720 exactly | A, S (only the TS parser enforces it; .NET enforces only `<= 8192`; both web engines leave the CRC loop unbounded) |
| V3 | headCRC | `(0x55555555 - sum(bytes[8 .. headLength))) & 0xFFFFFFFF` equals the stored u32 | A (45/45 recomputed, zero failures), S |
| V4 | Payload bounds | `fwLength >= 1` and `8192 + fwLength <= file size` | S (TS parser only) |
| V5 | Chain bounds | `nextFileAddress` is 0, or `== 8192 + fwLength` and `< file size`, with a second header there passing V1-V4 | A (holds in all 10 chained files), S |
| V6 | Endpoints | every endpoint field matches `vid_([0-9a-f]+)&pid_([0-9a-f]+)` | S (TS parser only) |
| V7 | Command fields | reset present with cmdlength 17 / report 8; prepare and data present with cmdlength 49 and the same report ID | A |
| V8 | File hash | sha256 equals the ledger value | Hyperpace requirement; no vendor tool does this |

**Do NOT attempt these (A):**
- Do not verify the prepare check value. A raw CRC-32 (init 0xFFFFFFFF, poly 0xEDB88320, no final XOR) over the stored payload reproduces it for exactly the 4 CX images (CX52650N x2, CX52660 x2) and for **none** of the other 41. Twelve CRC-32 variants plus byte-sum and 0x55555555-minus-sum were swept over the NRF52833, NRF52840, CH32V305, CX364, NRF52820_8K and NRF54L15_8K images: no match.
- Do not derive the prepare declared length from `fwLength`. It matches in only 12 of 45; deltas are 0, -4, -8 or -12 with **no chip-family pattern** (the "encrypted families are short" rule is refuted: NRF52833, both Scyrox NRF52840, NRF52820_8K, NRF54L05_8K, NRF54L15_8K and one CH32V305 all have delta 0).
- Do not read the payload CRC at prepare-field offset 19. That is 0x00000000 in 45/45 images; the reference parser's `payloadCrcValid` is structurally always false.

**Where the implementations conflict on strictness (S):** the .NET tool rejects `headLength > 8192`; the TS parser enforces V2, V4 and V6 with thrown errors; **both web engines validate only the CRC and fail silently**, returning false with no state write and no message, which the fetch handler then discards. No implementation validates `fileId`, `icName`, `DeviceType`, `Cid`, `Mid`, `version`, `senserName` or `productName`.

---

## 3. Target matching

| # | Rule | Verif |
|---|---|---|
| T1 | Normal VID/PID comes from `normalOutputEndPoint`; boot VID/PID from `bootOutputEndPoint` | S |
| T2 | HID collection check: a collection with exactly one feature report of the required ID, or exactly one input plus one output report whose output ID matches | S |
| T3 | Required report IDs come from the image: reset ID from `resetToUpdateModeCmd[2]`, prepare ID from `prepareDownLoadCmd[2]` | S |
| T4 | Require **exactly one** device matching the normal input endpoint and exactly one matching the normal output endpoint; likewise on the boot side | S (.NET enforces; both web engines rely on the browser picker instead) |
| T5 | Hyperpace-specific: require `vid_3554` plus `pid_fb16` or `pid_fb14` in the normal endpoints, and `DeviceType` matching the chosen component | Hyperpace rule; A for the fact that no archived image carries either PID |

**Known defect not to copy (S, A):** both web engines push the *previous* required-report-ID values into the filter array before reassigning them from the freshly parsed header, so the first init always filters on 8 (normal) and 6 (boot) regardless of the image. Harmless across this archive only because all 45 images use reset report 8 and image-1 prepare report 6. The same stale-ID path means a chained second image whose prepare uses report 9 is still matched on 6 (all 10 chained second images do use 9).

**The identity handshake, and why it is nearly dead (S, A):** the .NET tool sends report `08 01 00 00 00 08 <4 random bytes>` and requires `reply[10] == header.Cid` and `reply[11] == header.Mid`. Three corrections: it returns "match" **before any USB traffic** when the image's cid and mid are both zero, which is 41 of 45 images; the nonce is `System.Random`, not a CSPRNG; and a device that answers nothing is still accepted, only an actual mismatch excludes it. The web flashers have no equivalent at all. **Once the device is in the bootloader it is anonymous and no implementation checks identity again.**

---

## 4. Transport selection

| # | Fact | Value | Verif |
|---|---|---|---|
| X1 | Report IDs | reset 8; prepare and data 6 for image 1, **9** for a chained second image | A (8/6/6 in 35 images, 8/9/9 in 10) |
| X2 | Feature vs output | `dataDownLoadCmd[1]` selects the send API **for every queued packet including prepare**. 1 -> SET_FEATURE, 0 -> output report | S, A |
| X3 | Which images use feature | exactly 4 of 45: two CX52650N and two CX52660, each `(reset, prepare, data) = (0, 1, 1)`. The other 41 are `(0, 0, 0)`. The reset is never a feature report | A |
| X4 | Write length | the engines send `cmdlength` bytes: report ID plus 48 for prepare/data (49 total), report ID plus 16 for the reset (17 total). **Never 64** | S |
| X5 | The 64 in the code is an allocation, not a length | the web chunk builder allocates the data packet at 64 but sets `len = cmdlength`, and the send loop reads `len`; the .NET tool allocates 64 and hands the whole array to WriteFile, but its matching read is issued for exactly `cmdlength` | S |
| X6 | The 64-byte log lines are host-buffer dumps | each vendor log prints a fixed 64-byte host array; the 19-token reset line is the serialized command struct, not the 17 bytes written. **Neither 64 nor 19 is evidence of a wire length** | W, S |
| X7 | Boot handles | 41 of 45 images have `bootInput == bootOutput`; exactly 4 differ, and they are precisely the CX parts (`col01` in, `col02` out). The .NET tool opens two handles; the web engines never read `bootInputEndPoint` at all, because WebHID delivers input reports from every collection of the opened device | A, S |
| X8 | Report-ID asymmetry | on the CX run the host writes ID 0x06 and every reply arrives as ID 0x07; on the NRF runs both directions use 0x06 | W |

**Linux consequences (S, U):**
- A feature write becomes a `HIDIOCSFEATURE` ioctl per packet, not a `write()` to an interrupt OUT endpoint, on the CX path. Replies still arrive as ordinary input reports, so the CX channel is ioctl-write plus interrupt-read, not a symmetric feature transaction.
- On Windows the HID class driver creates one device object per top-level collection, so a native flasher that selects one collection by output report ID would send correctly and **never receive a reply** on a CX receiver. On Linux both CX boot collections are `mi_01`, so one hidraw node covers both.
- The write length must come from the **boot device's own report descriptor** (`1 + that report ID's output-report size`), read passively from `/sys/class/hidraw/*/device/report_descriptor`, and must be cross-checked against the image's `cmdlength` with a loud failure on disagreement. The boot report ID must come from the image, because on this device report ID 6 is already taken in application mode by a 7-byte feature report. **No Compx bootloader descriptor has ever been observed** (U).

**Echo-comparison trap (S):** the web engines compare the reply starting at `reply[0]` against the queued buffer starting at `buffer[1]`. That is correct *only* because WebHID strips the report ID from `event.data`. On hidraw or Win32 the first byte **is** the report ID, so copying the web comparison into a native port makes every data echo mismatch, and on CX it fails a second time because the reply ID (0x07) differs from the write ID (0x06). The failure is silent: the packet is re-armed, and after 8 non-advancing retries sending stops and only the watchdog fires.

---

## 5. Bootloader entry

| # | Fact | Verif |
|---|---|---|
| B1 | `resetToUpdateModeCmd` is a 17-byte frame on report 8 carrying config command `0x0D` (EnterUsbUpdateMode) | A, W |
| B2 | Single-MCU form: `08 0D 00 00 00 00 ... chk` | A, W (`11 00 08 0D ... 40` in both logs) |
| B3 | Chained form: `08 0D 00 00 00 03 <idx> <n> <D2|D3> ... chk`, where `idx` 00 = main MCU and 01 = sub/radio MCU, and `D2`/`D3` equals the header `DeviceType` (0xD2 on every mouse chain, 0xD3 on every dongle chain) | A |
| B4 | Frame checksum (last byte) = `(0x55 - sum(all preceding frame bytes including the report ID)) & 0xFF`. Verified on all 35 reset frames: `0x0D -> 0x40`, CH32 chain frames -> 0x69 and 0x64, CX364 -> 0x62 and 0x61 | A |
| B5 | The meaning of the middle byte `<n>` (observed 01, 03, 05, 08) is unknown | U |

**The single largest architectural conflict in the corpus (S, W):**

| | Web engines (Impl 1, 2) | .NET tool (Impl 3) |
|---|---|---|
| Sends | repeatedly, on the still-open normal handle: 10 ms tick, a send every ~210 ms after `t > 20`, at most 50 sends (~10.5 s) | once |
| Success criterion | the device must **echo the reset frame back** on that same handle (compare the first `cmdlength - 1` bytes, checksum excluded) | after a successful write it unconditionally reads one input report of `cmdlength` bytes; only the *content* comparison is gated on `dataLength > 0` |
| Then | a second `requestDevice` chooser is the only re-enumeration wait | `WaitBootMode` polls the device list for the boot PID |
| On exhaustion | returns false with **no state change**: the dialog stays at "Upgrading" with the button disabled | emits a coded timeout |

In both captured .NET runs the device **did** return an input report after the 0x0D reset, 37 ms later on the CX dongle and 82 ms on the NRF mouse (W). So the reset does elicit a reply on the siblings. Whether the HYPACE echoes the frame *content* the web path compares is unverified (U); if it does not, a straight port of the web reset loop fails silently.

**Recommendation for Hyperpace:** do not require the echo. Send the reset, then poll for re-enumeration (section 6), and treat any reply as informational. Bound the whole phase and fail loudly on exhaustion.

---

## 6. Re-enumeration

| # | Fact | Verif |
|---|---|---|
| E1 | `WaitBootMode` algorithm: arm a 10-unit timer; retry budget `errorTimeout / 200` = 50; a search fires only when the tick counter exceeds 20; a search requires the boot **input** and boot **output** lookups to each return exactly one device; each matching poll logs "wait stable" and increments a stability counter; success requires that counter to exceed 2, i.e. **three consecutive matching polls** | S |
| E2 | More than one device on either lookup -> multi-boot-device error and abort; a non-matching poll resets the stability counter (and a matching poll also zeroes the attempt counter) | S |
| E3 | Measured settling, NRF: reset write 16:07:57.638; three polls at .451, .787, 16:07:59.125; "usb stable" at 16:07:59.125. **1.487 s** end to end | W |
| E4 | Measured settling, CX: reset write 16:43:31.619; polls at 16:43:32.668, .999, 16:43:33.362; "usb stable" 16:43:33.378. **1.759 s** end to end | W |
| E5 | Polls run ~336 ms apart, not the ~200 ms the `errorTimeout / 200` arithmetic implies, so the effective budget is about 30 searches, not 50 | W |
| E6 | The web engines have no re-enumeration poll at all; the browser chooser is the wait, and the user must pick the boot device | S |

**Linux note (U):** the old hidraw node disappears and a new one appears under a different PID and possibly a different index. The Windows lookup-by-path-substring does not model this; a Linux port must re-enumerate by VID/PID, not by remembered node path.

---

## 7. Prepare packet

| # | Fact | Verif |
|---|---|---|
| Q1 | It is queue entry 0, built by copying `prepareDownLoadCmd` verbatim for `cmdlength` bytes. Nothing is recomputed | S |
| Q2 | Wire form: `[0]` report ID, `[1]` 0xB0, `[2..4]` 0, `command[5..8]` declared payload length big-endian (= raw field bytes 7..10), `command[9..12]` check value big-endian (= raw field bytes 11..14), rest 0 | A, W |
| Q3 | Worked example, CX52650N v3.00: field = `31 01 06 B0 00 00 00 00 00 88 60 FE 41 32 3D`; `0x8860` = 34912 = `fwLength`; check `0xFE41323D`, which equals raw CRC-32 of the payload. The logged write is byte-identical | A, W |
| Q4 | Worked example, NRF52833 v2.18 (the HYPACE's own MCU family): field = `31 00 06 B0 00 00 00 00 01 4A 60 41 32 49 38`; length `0x014A60` = 84576 = `fwLength`; check `0x41324938`, matching **no tested algorithm** | A |
| Q5 | The device echoes the prepare packet (reply command byte 0xB0). **That echo does not advance the queue.** The entry leaves the queue only on an erase-done state | S, W |
| Q6 | Beware a common misreading: the `payloadCRC=` column in the archive ledger is in fact this declared-length field, not a CRC | A |

**Erase timing, corrected (W):**
- **CX pays ~2.4 s twice.** Erase-backup brackets a 2.386 s gap *before* the transfer (`5B B5 01 00` -> `5B B5 01 01`), and erase-main brackets a 2.394 s gap *after the last data packet* (`5B B5 02 00` -> `02 01`).
- **NRF erases main once, up front:** 1.928 s, before the first data write; there is no second erase.

Hyperpace must budget for a silent ~2.4 s window **after** the final data packet on a CX-family target.

---

## 8. Data packets

| # | Fact | Verif |
|---|---|---|
| D1 | Queue length = `1 + ceil(fwLength / 32)` | S, W (CX: 1092 writes = 1 + 1091; NRF: 2759 = 1 + 2758) |
| D2 | Layout: `[0]` report ID, `[1]` 0xB1 forced, `[2]` 0xC0 (full chunk) or 0xC1 (final chunk), `[3]` length (32, or the remainder), `[4]` 0, `[5..8]` flash address big-endian, `[9..16]` 0, `[17 .. 17+len)` payload from `image[8192 + i*32]`, remainder 0xFF | S, W |
| D3 | Flash start address comes from `dataDownLoadCmd[5..8]` and advances by 32 per packet. Observed: `0x00007000` for CX52650N/CX52660, `0` for NRF/CH32/CX364 | A, W |
| D4 | Exactly one C1 frame per image, the last one | W |
| D5 | The web chunk builder does **not** bounds-check the source index; a truncated file silently writes zeros past the end | S |

**Transfer discipline: stop-and-wait, window 1 (W, S).** One packet is sent, the host waits for the device to echo the entire packet, then advances. Proven across all three logs: perfect write/receive alternation, zero consecutive writes without an intervening receive, and **zero echo mismatches** over 1091 CX packets and 2758 NRF packets per run. The compare width is 48 bytes (`cmdlength - 1`), measured directly: the highest nonzero byte index in every logged data report, write and echo, is exactly 48, so the input report is 49 bytes (report ID plus 48).

---

## 9. Replies and state codes

| Opcode | Meaning | Verif |
|---|---|---|
| 0xB0 | prepare echo, ignored by every implementation | S, W |
| 0xB1 | data echo, drives the queue | S, W |
| 0x5B | device state, second byte marker 0xB5 | S, W |
| 0x5A | device error, marker 0xA5 | S |

| State byte | Meaning | Verif |
|---|---|---|
| 1 | Erase_Backup (CX only) | W |
| 2 | Erase_Main | W |
| 3 | **undocumented** progress counter, byte 4 running 0x00..0x64 (101 frames), CX only | W |
| 4 | **undocumented**, CX only, exactly once | W |
| 5 | CheckOK, received before Success, ignored by every host | W |
| 0x88 | Success | W |

Census across the three logs: CX `{1:2, 2:2, 3:101, 4:1, 5:1, 0x88:1}`; each NRF run `{2:2, 5:1, 0x88:1}`. **Correction:** 16 (PrepareDownLoad) and 17 (Device_To_Boot_Mode) are **not** device reply states; they occur in none of the 116 logged device-state frames and are host-side UI codes. The device state set is `{1, 2, 5, 0x88}` plus the undocumented `{3, 4}`.

**Queue advance rule (S):** the queue advances only on an erase state **with reply byte 3 equal to 1**. States 3, 4 and 5 fall through with no shift and no progress increment in both reply parsers.

**Device error is swallowed by all three implementations (S).** The web branch returns false with no state write on either path. The .NET branch tests payload byte **1** against 90, then tests the *same byte* against 165, a condition that can never hold, so its error report is unreachable dead code (the neighbouring 0x5B branch correctly tests byte 1 for 91 and byte **2** for 181). The .NET tool does still abort the transfer on 0x5A; what is lost is the reason. **Hyperpace must write the loud path itself** and must read the marker at the correct index for its transport (index 1 under WebHID's ID-stripped data, index 2 on a raw hidraw frame).

---

## 10. Timing, watchdog and retries

| # | Rule | Value | Verif |
|---|---|---|---|
| R1 | Download watchdog | 10000 ms, supplied by the caller, identical in both families (`1e4` in the web callers; `ldc.i4 10000` at both .NET entry points) | S |
| R2 | Watchdog re-arm | **only after a successful send**, never on an incoming state frame, in the web engines. The .NET tool arms it at entry and again after each write | S |
| R3 | Consequence | the entire post-last-packet tail must fit inside one 10 s window | S |
| R4 | Measured CX tail | 3.920 s from the last data write to Success: state 4 at +0.255 s, the 2.394 s erase-main gap, a 101-frame progress burst over 1.173 s, then CheckOK and Success | W |
| R5 | Measured NRF tail | 0.120 s (CheckOK then Success), with the 1.928 s erase paid up front instead | W |
| R6 | Per-packet resend limit | ten tries at one queue position (the counter fires on `> 8` after resetting at a new position). Identical threshold in both families | S |
| R7 | What happens at the limit | web: sets a repeat-limit state but **does not break the loop**; the entry is marked invalid so sending stops and the 10 s watchdog overwrites the state with a timeout. .NET: marks the entry invalid, emits **no result code at all**, stops writing and falls through to the same timeout | S |
| R8 | Reset-phase bound, web | 10 ms tick, send when `t > 20` (~210 ms), max 50 sends, ~10.5 s | S |
| R9 | Reset-phase bound, .NET | `errorTimeout / 200` = 50 poll cycles at a 10 ms tick, plus a 1000 ms reset-response wait | S |
| R10 | Prepare packet is never resent | its entry is invalidated after its single send; only an erase-done state removes it | S |

Both measured tails fit 10 s, but the margin is a function of image size and erase behaviour and **no HYPACE image exists to size either target**, so the budget is unverified rather than safe (U).

---

## 11. Completion

| # | Fact | Verif |
|---|---|---|
| C1 | On `5B B5 88` the host sets Success and returns; on a non-final chained phase it sets "next" instead | S, W |
| C2 | **No implementation sends any exit, reboot or abort command.** The boot opcode set is exactly `{0xB0, 0xB1, 0x5B, 0x5A}` | S |
| C3 | No read-back, no verify-read, no post-write version re-read anywhere | S |
| C4 | The device is relied on to self-reboot into the application image. This is not observable from any host code | U |
| C5 | The web engines do not check that the queue is empty when Success arrives | S |
| C6 | The vendor "exit upgrade" action is a normal-mode session restart that re-handshakes with command 3, not a bootloader abort. No frame is ever sent to a device sitting in the bootloader | S |

---

## 12. Multi-image chaining and multi-target order

**Within one file (S, A):** if `nextFileAddress != 0`, image 2 (own 720-byte header plus payload) begins at that absolute offset, and `nextFileAddress == 8192 + fwLength` of image 1 in all 10 archived chained files. Both headers must pass the CRC check. Chip pairs observed: CH32V305 + NRF52820_8K, CH32V305 + NRF52840_8K, CX364 + NRF54L05_8K, on both mouse (210) and dongle (211) images.

**Order within a chained file depends on the starting mode (S):**
- From **normal** mode: image 2 (the sub/radio MCU) is reset and downloaded first over the normal PID (report 9, `col07` of the normal PID), then image 1 (the main MCU) through its own boot PID.
- From **boot** mode (recovery): image 1 first, then image 2 through the normal PID.
- Only the last image flashed emits the final Success. Progress is 0-50 % for pass 1 and 50-100 % for pass 2.

**Across separate targets: receiver before mouse.**

| Source | What it says | Verif |
|---|---|---|
| Pulsar readme, verbatim | "Update the dongle firmware before updating the product firmware." and "If you update the product firmware first, you may need to re-pair the device." | S |
| Vendor history text, verbatim | "Please update the dongle first, followed by the mouse. (The update must be performed using a USB cable.)" | S |
| Lofree keyboard hub | machine-enforces it: sorts by `{dongle: 0, wired: 1}` and throws an order-invalid error when a multi-target queue does not start with the dongle | S |
| Compx mouse tools | do **not** machine-enforce it; the web driver only gates the two buttons by connection mode, and the .NET ships two separate executables | S |
| Other vendors | one states users "might need to pair the 4K dongle again after firmware update finish"; another that if only one side is updated, 2.4G may fail, and to re-pair manually afterwards (different protocol, analogous evidence only) | S, U |

**Re-pairing primitives (S, and one independent hardware test):** command 5 takes `[0, 0, cid]` and echoes; command 6 returns 1 pairing / 2 fail / 3 success at byte 5 with seconds remaining at byte 6, polled once a second up to 20 times. The mouse enters pairing when left, right and middle are held for 3 s. An independent test on a sibling device confirms the gesture is required and adds a decisive caveat: **an attempt without the gesture reached status 2 yet the device remained offline, so status 2 alone is not success.**

**Hyperpace requirement:** machine-enforce receiver-before-mouse in a multi-target queue (the hub's pattern), and treat a matched mouse/receiver version pair as a Hyperpace policy, not a proven device constraint.

---

## 13. Version policy and rollback

| # | Fact | Verif |
|---|---|---|
| P1 | The device and protocol enforce **no** anti-rollback, version counter or signature. The only header check is the CRC, and nothing compares versions before flashing | S, A |
| P2 | The .NET desktop permits same-version reflash and downgrade: `AlwaysUpgrade=1` in all seven shipped configs enables the button unconditionally; the inequality test is the `AlwaysUpgrade=0` path | S |
| P3 | The Pulsar web driver is strictly-newer; the keyboard hub skips a target when `current >= latest` | S |
| P4 | The Lofree bundle has **no** version-comparison UI at all (`Check_New_Version` occurs zero times in it); its updater is reachable only from the resume dialog | S |
| P5 | Config version labels can disagree with header versions: a cfg entry labelled "v4.00" links an image whose header parses as v2.24 | S, A |
| P6 | Version read transports differ three ways: web uses config command 18 (mouse) and 29 (receiver), formatted `v{major}.{minor:02x}`; .NET reads USB `bcdDevice`, formatted the same way, plus an in-band `0xB3` query only for multi-image files; the keyboard hub uses its own commands 0x28/0x29 | S |
| P7 | Whether the *bootloader* accepts a downgrade is unverified. No log shows a same-version or older image being accepted | U |

**Hyperpace policy (this is a decision to take, not a fact):** adopt the .NET install-any-version behaviour so rollback is possible at all, and pair it with an explicit downgrade confirmation. Before the first flash, read and **persist** the mouse version (command 18), the receiver version (command 29) and both `bcdDevice` values. Nothing currently specifies this, and combined with the absence of any read-back the shipped version becomes unrecoverable the moment an image is written.

---

## 14. Failure handling and recovery

**How failures actually surface in the vendor tools (S):**

| Failure | Web engines | .NET tool |
|---|---|---|
| Non-200 fetch | silent no-op | n/a |
| Header check fails | silent false, discarded | coded, logged |
| Reset exhaustion | silent; dialog stuck at "Upgrading", button disabled | coded timeout (code 6) |
| Resend limit | sets a repeat-limit state but does not break; degrades into the 10 s timeout | marks invalid, emits **no code**, degrades into the same timeout |
| Device error 0x5A | swallowed | unreachable dead code; transfer aborts without a reason |
| Unplug during flash | masked by a no-op disconnect handler; surfaces only as a send exception or the timeout | n/a |
| No collection matches | assigns the error enum to the **state variable itself** instead of its `.state` field, so the UI never shows it (and under module strict mode the next state write would throw) | reported properly |

Net effect: **three distinct failures converge on one indistinguishable timeout**, and the only failure string the Lofree build ships is a single generic "Upgrade Failed". The .NET tool's real diagnostic advantage is its log file, not its coded UI: its result codes 3 and 4 are never emitted, code 10 is unreachable, and its form handler drops every informational message, so at most 7 codes can ever fire and the UI distinguishes only success from not-success.

**Recovery, and what is actually documented (S):**

| # | Fact | Verif |
|---|---|---|
| RC1 | The device stays in the bootloader until a complete valid image lands. Re-running the updater resumes: both web engines include the boot identity in the device filter, and when the selected device matches the boot VID/PID the single-image path **skips the reset entirely** and goes straight to the download | S |
| RC2 | The web engines persist `{href, link}` to localStorage at the start and remove it only on Success; the next page load shows a coercive "Device upgrade failed, please continue upgrading" dialog that reopens the updater with the saved link | S |
| RC3 | The keyboard hub makes this first-class: a persisted boot-recovery flag, a resume-active state written and cleared around each stage, and a UI that jumps straight back into the updater on mount | S |
| RC4 | The vendor's own documented way out of a bootloader is **to complete a flash from a model-specific recovery URL**, not to exit the bootloader. Lofree's recovery guide states verbatim that the recovery page "only applies to keyboards that have lost functionality due to upgrade failure", that working units cannot use it, and that a factory reset is required afterwards | S, keyboard family only |
| RC5 | A physical button gesture into the bootloader is documented by one unrelated vendor with no model or chip named; unverified for Compx bootloaders | U |
| RC6 | Whether a power cycle alone returns a Compx bootloader to the application image when nothing was written is unobserved for every chip family | U |

**Hyperpace requirements:** implement the resume path (detect a device parked at the boot PID, skip the reset, flash); make the three timeout-convergent failures distinguishable and named; treat a disconnect during flashing as a first-class event; and never write a failure message that claims to know the image was rejected, because **no device-side rejection has ever been observed** in the corpus. The honest surface after a post-transfer stall is that the device stopped responding after the image was written, plus the recovery route.

---

## 15. Conflicts between implementations, collected

| # | Conflict | Resolution for Hyperpace |
|---|---|---|
| 1 | Bootloader entry: echo-on-handle (web) vs write-once-then-poll (.NET) | Poll for re-enumeration. Do not require the echo; the HYPACE's echo behaviour is unknown. |
| 2 | Boot handles: one (web, WebHID merges collections) vs two (.NET, input and output separately) | Open per the image: if `bootInput != bootOutput` (the CX case), use both. |
| 3 | Header strictness: CRC only (web) vs `headLength <= 8192` (.NET) vs full bounds (TS parser) | Mirror the TS parser's bounds, add the Hyperpace identity gate on top. |
| 4 | Payload CRC: nobody checks it, except a TS parser that reads the wrong offset and is always false | Do not check it. Carry it verbatim; the device gates it. |
| 5 | Version policy: strictly-newer (web, hub) vs install-any (.NET `AlwaysUpgrade=1`) vs no UI at all (Lofree bundle) | Install-any, with an explicit downgrade confirmation, because rollback is a requirement. |
| 6 | Multi-target order: enforced by throw (hub) vs documented only (Pulsar, .NET) | Enforce it. |
| 7 | Chaining: single-file `nextFileAddress` (Impl 1/2/3) vs two separate `.bin` files (hub) | Support single-file chaining; a HYPACE receiver package's convention is unknown. |
| 8 | Cross-flash protection: endpoint VID/PID only (web) vs endpoint plus a mostly-skipped cid/mid handshake (.NET) vs filename-token guard plus productId assertions (hub) | Endpoint PID as the hard gate, plus component and icName checks; adopt the hub's explicit-assertion style. |
| 9 | Device error 0x5A: swallowed everywhere, by two different mechanisms | Implement it properly; this is net-new work. |
| 10 | Container: raw Compx (`ComUsbUpgradeFile`) vs the hub's `Eevision` + 64-byte-XOR envelope with a different CRC (zlib CRC-32 **with** final XOR) | Accept only the Compx container for the HYPACE; refuse an `Eevision` file by name and reason. Whether Lofree would wrap a HYPACE package that way is unverified. |

---

## 16. What this spec cannot tell you about the HYPACE

Every item below requires a real HYPACE package or device observation, and none can be guessed:

1. The mouse and receiver **boot-mode VID/PIDs**. Siblings span 16 observed identities (13 in the archive plus `0x3554:0xF401`, `0x3554:0xF510`, `0x3554:0xF406`); none is the HYPACE's. Normal PIDs are known: 0xFB16 receiver, 0xFB14 wired.
2. Whether data packets use **feature or output** reports (the `dataDownLoadCmd` feature flag), and whether the prepare uses report 6 or 9.
3. The **prepare check value algorithm** for the NRF52833 family. Unreproducible, which is why only a complete vendor package is flashable.
4. Whether the HYPACE **echoes** the reset frame on the pre-disconnect handle.
5. Whether a HYPACE receiver package uses single-file chaining or two separate files, and whether the receiver really is a two-MCU part.
6. The receiver silicon. FCC certifies only an **8K** HYPACE dongle (`2AC59-OE923RX`), while the vendor config has rows only for 1K/2K/4K and no 8K row; the protocol can report an 8 kHz link. Block Diagram, Schematics and Operational Description are withheld under long-term confidentiality, so the **internal photographs** (both downloadable, dated 2025-08-25) are the only primary route left that can settle it without touching hardware.
7. The mouse MCU: the vendor config says NRF52833; a Chinese teardown description says 52840. Unresolved, and the board must not be opened.
8. Whether the device self-reboots into the application image after Success.
9. The HYPACE's timeout budget, since it depends on image size and erase behaviour.
10. The meaning of the chained reset frame's `<n>` byte, the erase sub-flag `byte3 != 1`, the undocumented device progress states 3 and 4, and the 0xA5 error marker semantics.
