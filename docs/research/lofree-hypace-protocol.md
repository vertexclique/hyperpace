# Lofree HYPACE web driver: protocol and architecture reference

Source: https://lofree.tech, fetched 2026-09-15. A static Vue 2 + Element UI single-page app
served by nginx 1.26.1: `js/app.66015056.js`, `js/chunk-vendors.1d7a523f.js`, `cfg.json`,
`sensor.json`, `lang/en.json`. Every offset and encoding below was read from the bundle.
Nothing has been verified against live frames from the mouse yet (see section 11).

## 1. Backend

None. The page talks to the mouse through WebHID (`navigator.hid`) in the browser.

- Network traffic is limited to same-origin static GETs through axios: `cfg.json`,
  `lang/<code>.json`, `custom.scss`, `sensor.json` and images.
- No POST, WebSocket, analytics, tracker or third-party host appears in `app.js`,
  `chunk-vendors.js` or `app.css`.
- nginx serves `index.html` for unknown paths (`/api`, `/graphql` return the 608 B page),
  so no API sits behind it.
- Browser-local state (`localStorage`): `locale`, `theme`, `macro` (macro library),
  `preset_color`, `bat_<receiver addr>` (battery smoothing), `password`, `deviceInfo`.
- "EncryptionData" (command 1) sends 4 random bytes; the page reads back only model ids and
  link type. No cryptographic check exists on the host side.
- The Upgrade button opens `location.href + cfg.upgrade.link`; `cfg.json` has no `upgrade`
  block, so the button permanently reads "LATEST VERSION" without checking anything.

## 2. Device identification

| Field | Value |
|---|---|
| VID | `0x3554` |
| PID, mouse via 2.4 GHz receiver | `0xFB16` |
| PID, mouse wired | `0xFB14` |
| PID, keyboard (listed in cfg, not HYPACE) | `0xF809` wireless, `0xF50A` wired |
| HID product name | `Compx Hypace@Lofree` |
| Model ids from command 1 | cid `102` (0x66), mid `1` or `2` |
| Hardware (cfg) | mouse MCU NRF52833; 1K/2K receiver CX52650N; 4K receiver CH32V305; sensor PAW3950 |

Each PID exposes three HID interfaces. The vendor interface is the one with usage pages
`0xFF02..0xFF06` and report ID 8 (16 B input + 16 B output). On the development machine:
`hidraw9` (receiver) and `hidraw12` (wired). It also declares report ID 9 (48 B in/out),
which the web driver never uses.

Web selection rule: a collection with exactly one input report and one output report whose
output report ID is 8.

## 3. Frame format

Report ID 8, 16 data bytes:

| Byte | Meaning |
|---|---|
| 0 | command |
| 1 | status: 0 in requests; in responses 0 = ok, 1 = unsupported or error |
| 2..3 | flash address, big-endian (flash commands) |
| 4 | payload length |
| 5..14 | payload (max 10 bytes) |
| 15 | checksum |

Checksum: `(0x08 + sum(byte0..byte15)) mod 256 == 0x55`, so
`byte15 = (0x55 - 0x08 - sum(byte0..byte14)) mod 256`. The report ID takes part in the sum.

Transaction as implemented by the web driver: send, wait for input report 8 (poll every 5 ms,
up to 200 ms), accept when response bytes 0..2 equal the request (bytes 0..4 for command 8),
or when response byte 1 is 1. Retry up to 5 times, 10 ms apart. Traffic is request/response
except command 10, which the device pushes.

## 4. Commands

| Id | Name | Request payload | Response | Web use |
|---|---|---|---|---|
| 1 | EncryptionData | len 8: 4 random bytes, 4 zero | [9] cid, [10] mid, [11] link type | handshake |
| 2 | PCDriverStatus | [0 or 1] | echo | only when cfg `driverOnline` (false for HYPACE) |
| 3 | DeviceOnLine | none | [5] online, [6..8] receiver address | before every setter |
| 4 | BatteryLevel | none | [5] percent, [6] charging, [7..8] mV big-endian | every 5 s |
| 5 | DongleEnterPair | len byte 2, bytes [0, 0, cid] | echo, then poll 6 | pair receiver |
| 6 | GetPairState | none | [5] 1 pairing / 2 fail / 3 success, [6] seconds left | 1 s poll, max 20 |
| 7 | WriteFlashData | [2..3] addr, [4] len <= 10, data | echo | settings |
| 8 | ReadFlashData | [2..3] addr, [4] len | data at [5..] | settings |
| 9 | ClearSetting | none | sent when reset completes | factory reset |
| 10 | StatusChanged | device -> host | [5] bitmask, see below | live sync |
| 14 | GetCurrentConfig | none | [5] profile 0..3; status 1 = no profiles | profiles |
| 15 | SetCurrentConfig | [profile] | echo, then full re-read | profiles |
| 18 | ReadVersionID | none | [5] major, [6] minor, shown `v%d.%02x` | mouse firmware |
| 22 | SetLongRangeMode | len 10, [0] = 0 or 1 | echo | long range |
| 23 | GetLongRangeMode | none | [5] state; status 1 = unsupported | long range |
| 24 | SetDongleLight | len 10: mode, r, g, b, speed, brightness, time | echo | receiver light |
| 25 | GetDongleLight | none | [5] mode, [6..8] rgb, [9] speed, [10] brightness, [11] time; status 1 = unsupported | receiver light |
| 29 | GetDongleVersion | none | [5] major, [6] minor | receiver firmware |

Link type (command 1, byte 11): 0 wireless 1 kHz, 1 wireless 4 kHz, 2 wired 1 kHz,
3 wired 8 kHz, 4 wireless 2 kHz, 5 wireless 8 kHz. It caps the polling-rate choices.

StatusChanged bitmask and the web reaction: `0x01` current DPI stage (read 4, len 2),
`0x02` polling rate (read 0, len 2), `0x04` profile (command 14), `0x08` DPI indicator
(read 76, len 8), `0x20` lighting (read 160, len 7), `0x40` battery (commands 3 + 4).

Defined in the bundle but never sent: 11 SetDeviceVidPid, 12 SetDeviceDescriptorString,
13 EnterUsbUpdateMode, 16 ReadCIDMID, 17 EnterMTKMode, 20 Set4KDongleRGB,
21 Get4KDongleRGBValue, 176 MusicColorful, 177 MusicSingleColor, 240 WriteKBCIdMID,
241 ReadKBCIdMID. The web driver has no firmware update path.

## 5. Flash (settings) map

The web driver keeps an 8192-byte shadow filled with `0xFF` and reads `0x000..0x103` on
connect (26 reads of 10 bytes). Writes go out in 10-byte chunks, address advancing by 10.

Integrity conventions:

- Scalar: two bytes `[v, 0x55 - v]`.
- Struct: data bytes followed by one check byte `(0x55 - sum(data)) mod 256`.

| Offset | Kind | Field | Values |
|---|---|---|---|
| 0 | scalar | polling rate | 1=1000, 2=500, 4=250, 8=125, 16=2000, 32=4000, 64=8000 Hz |
| 2 | scalar | DPI stage count | 1..8 (UI offers 1..5) |
| 4 | scalar | current DPI stage | 0-based |
| 10 | scalar | lift-off distance | 1 = 1 mm, 2 = 2 mm, 3 = 0.7 mm (3950 only) |
| 12 + 4i | struct 3+1 | DPI stage i value, i in 0..7 | see 5.1 |
| 44 + 4i | struct 3+1 | DPI stage i color | r, g, b |
| 76 | scalar | DPI indicator mode | 0 off, 1 steady, 2 breathing |
| 78 | scalar | DPI indicator brightness | raw 16, 30, 60, 90, 128, 150, 180, 210, 230, 255 for levels 1..10 |
| 80 | scalar | DPI indicator speed | |
| 82 | scalar | DPI indicator on/off | |
| 94 | scalar | light power save | exported by the driver, no UI |
| 96 + 4i | struct 3+1 | button i function, i in 0..4 | type, param hi, param lo; see 5.2 |
| 160 | struct 6+1 | lighting | mode, r, g, b, speed 0..9, brightness 0..9 |
| 167 | scalar | lighting on/off | |
| 169 | scalar | debounce | 0..15 ms |
| 171 | scalar | motion sync | 0/1 |
| 173 | scalar | sleep time | tens of seconds: 1, 3, 6, 12, 30, 60, 90 |
| 175 | scalar | angle snap | 0/1 |
| 177 | scalar | ripple control | 0/1 |
| 179 | scalar | lights off while moving | 0/1 |
| 181 | scalar | highest performance on/off | 0/1 |
| 183 | scalar | highest performance timeout | tens of seconds: 1, 3, 6, 12, 30, 60, 90 |
| 185 | scalar | sensor mode | 0 LP, 1 HP |
| 256 + 32i | buffer | button i keystroke | see 5.3 |
| 768 + 384i | buffer | button i macro slot | see 5.4 |

For HYPACE the web UI does not render lighting (cfg has no `lightEffect`), the DPI indicator
(read but no control placed) or sensor mode (component registered, not placed in the page).

### 5.1 DPI stage encoding

`[x_lo, y_lo, flags, check]`; the web writes X = Y.

- flags bits 2..3: X value bits 8..9; bits 6..7: Y value bits 8..9
- flags bit 0: X doubled; bit 4: Y doubled (bits 1 and 5 double again for a 4-range sensor)

`sensor.json` for 3950: range A 100..30000 step 50, `DPIex` 0; range B 30100..60000 step 100,
`DPIex` 0x11. The UI caps at cfg `maxDpi` 40000.

```
encode(dpi):  if dpi >= 30100 { raw = dpi / 2 / 50 - 1; flags = 0x11 } else { raw = dpi / 50 - 1; flags = 0 }
              flags |= (raw >> 8) << 2 | (raw >> 8) << 6
              bytes = [raw & 0xFF, raw & 0xFF, flags, check]
decode(bytes): raw = b0 | ((b2 & 0x0C) << 6); dpi = (raw + 1) * 50
               if b2 & 0x01 { dpi *= 2 }; if b2 & 0x02 { dpi *= 2 }
```

Other sensors (3335, 3311, 3325, 3104, 3212, 8920, 4090) use a lookup table from
`sensor.json` and do not apply to HYPACE.

### 5.2 Button functions

| Type | Function | Param |
|---|---|---|
| 0 | disabled | 0 |
| 1 | mouse button | 0x0100 left, 0x0200 right, 0x0400 middle, 0x0800 back, 0x1000 forward |
| 2 | DPI | 0x0100 cycle, 0x0200 up, 0x0300 down |
| 3 | horizontal scroll | 0x0100 left, 0x0200 right |
| 4 | fire (auto repeat) | interval_ms << 8 or times (times 0..3, 0 = while held; interval 10..255) |
| 5 | keystroke or media key | 0; content in the keystroke buffer |
| 6 | macro | slot << 8 or cycle (slot = button index; cycle 1..250, 254 until released, 255 until any key pressed) |
| 7 | polling rate cycle | 0 |
| 8, 9, 10, 11 | light switch, profile switch, DPI lock, vertical scroll | defined, not offered for HYPACE, untested |

Default map by device index: 0 left, 1 right, 2 middle, 3 back, 4 forward. The web refuses to
remove the last left-click binding. On non-Windows platforms the web UI disables remapping of
the two side buttons (a web restriction, not a firmware one).

UI hotspot coordinates on the 710x380 render (cfg `loc`, image rotated, front facing left):
index 0 left (144, 253), 1 right (144, 100), 2 middle (96, 177), 4 forward (248, 352),
3 back (363, 352). Badges are `img/key/Keys/{1..5}.png` and `{n}-click.png`.

### 5.3 Keystroke buffer (256 + 32i)

`[2n, n press entries, n release entries in reverse order, check]`, each entry
`[flags or kind, lo, hi]`.

- flags `0x80` press, `0x40` release
- kind 0 modifier bitmask (1 LCtrl, 2 LShift, 4 LAlt, 8 LGui, 16 RCtrl, 32 RShift, 64 RAlt,
  128 RGui); kind 1 keyboard usage (HID page 0x07); kind 2 consumer usage (page 0x0C);
  kind 7 menu key
- web limit: two modifiers and one key
- media key: `[2, 0x82, lo, hi, 0x42, lo, hi, check]`
- read size: `3 * byte0 + 2`

### 5.4 Macro slot (768 + 384i)

| Offset | Content |
|---|---|
| +0 | name length, 1..30 |
| +1..+30 | UTF-8 name, `0xFF` padded |
| +31 | event count, <= 70 |
| +32 + 5k | event k: `[flags or kind, value lo, value hi, delay hi, delay lo]` |
| after last event | check byte over count and events |

Value is little-endian and delay is big-endian (10..65535 ms). Flags `0x80` press, `0x40`
release. Kind 1 keyboard usage; kind 4 mouse button (1 left, 2 right, 4 middle, 8 back,
0x10 forward). The mouse stores only macros bound to a button; the macro library lives in
browser `localStorage`.

## 6. Connect sequence

1. Open the vendor interface and attach the input report handler.
2. Command 1 -> cid/mid selects the cfg entry and image `img/devices/mouse/66{01,02}.png`;
   link type sets the polling-rate cap.
3. Poll command 3 every 1.5 s until the mouse reports online (a wireless mouse may sleep).
4. Read flash `0x000..0x103`; for each button of type 5 read its keystroke buffer, for type 6
   read its macro slot.
5. Battery (3 + 4), profile (14), mouse version (18); wireless only: receiver version (29),
   receiver light (25), long range (23).
6. Poll battery every 5 s and handle command 10 pushes. If still connecting after 10 s, close.

Every setter sends command 3 first and aborts with an "offline" dialog when the mouse does
not answer.

- Factory reset: command 9, re-read flash and profile, re-read receiver version and light,
  then apply cfg default long range (true for HYPACE).
- Profile switch: command 15, then full re-read.
- Pair new receiver: the mouse enters pairing when left, right and middle are held 3 s; then
  command 5 with cid and poll command 6.

## 7. Battery display

The device reports percent and millivolts. The web maps millivolts to percent with a 21-point
table (3050, 3420, 3480, 3540, 3600, 3660, 3720, 3760, 3800, 3840, 3880, 3920, 3940, 3960,
3980, 4000, 4020, 4040, 4060, 4080, 4110 mV at 5 percent per step), then rate-limits changes
by the time since the last stored reading (per receiver address, in `localStorage`). This is a
display heuristic, not device data.

## 8. Export and import file (.bin)

8192-byte flash shadow followed by a 64-byte trailer: `Compx Inc` (32 B, NUL padded), device
type `mouse` (16 B), sensor `3950` (16 B). Import checks the trailer, clamps polling rate and
DPI stage fields, writes `0x000..0x0FF`, then writes any keystroke or macro region that
differs. Web bug: when clamping the current stage it writes the check byte from the stage
count (`c[5] = 85 - c[2]` instead of `85 - c[4]`).

## 9. Web UI inventory

- Home: connect, pair a new receiver (Space key), version.
- Buttons: mouse render with five numbered hotspots, function per button, profile 1..4,
  battery ring, export / import / restore, receiver light (mode, color bar, presets,
  brightness and speed dials).
- Performance: sleep time dial, DPI stages 1..5 with slider 100..40000 and per-stage color,
  polling rate slider capped by link type, debounce 0..15 ms with a warning below 8 ms, LOD
  dial, motion sync / angle snap / ripple / long range toggles, highest performance dial and
  switch.
- Macros: library, recording with automatic or fixed delay, insert press / release / mouse
  clicks, cycle mode, save.
- Settings: receiver and mouse firmware versions, pair, upgrade (inert, see section 1).

Assets: 94 PNG files plus `fonts/BarlowCondensed-Bold.ttf` and Element icon fonts. The mouse
render is 710x380 RGBA with a transparent background; 94 of 96 PNGs carry alpha. Background
art (`bg_light.png`, `home_bg.png`, `bg_setting.png`) is baked for the light theme.

## 10. Linux access

Vendor hidraw nodes are `root:root 0600` by default. A native app needs a udev rule such as:

```
KERNEL=="hidraw*", ATTRS{idVendor}=="3554", TAG+="uaccess"
```

On the development machine `hidraw9` is `0606`, most likely a manual chmod for the browser;
that resets on replug.

## 11. Not verified yet

- No live frame capture: checksum, offsets and encodings come from reading the bundle only.
- Report ID 9 (48 B in/out) purpose unknown.
- Button types 8..11 and commands never sent by the web are untested; do not send
  13 EnterUsbUpdateMode or 17 EnterMTKMode.
