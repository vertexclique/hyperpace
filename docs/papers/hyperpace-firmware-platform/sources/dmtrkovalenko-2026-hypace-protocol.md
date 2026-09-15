URL: https://github.com/dmtrKovalenko/lofree-hypace-reverse-engineer (README.md, PROTOCOL.md)
Retrieved: 2026-09-15
Source type: Community reverse-engineering notes (Unlicense; created and last pushed 2026-06-09; 8 stars), derived from the minified Lofree "Control HUB WEB" WebHID bundle.
Retrieval method: `gh api repos/dmtrKovalenko/lofree-hypace-reverse-engineer/readme` and `.../contents/PROTOCOL.md` (verbatim).
Character note: em-dash characters in quoted text were replaced with ASCII hyphens; nothing else changed.

# README

> This project is a reverse engineering result of my lofree hypace mouse. For some reason their web based HID firmware flasher doesn't support certain features on macos

# PROTOCOL.md section 1 (Transport)

> Source: reverse-engineered from `https://www.lofree.tech/home/` (Vite SPA "Control HUB WEB"), bundle `assets/index-BTVblIUr.js`, plus the JSON files served alongside it (`cfg.json`, `ref.json`, `sensor.json`).
>
> The web app uses **WebHID** to talk to the mouse.

> - API: WebHID (`navigator.hid.requestDevice` + `device.sendReport` / `device.sendFeatureReport`).
> - HID **Report ID = 8** (constant `cr` in JS).
> - Each report is **16 bytes** of payload following the report ID.
> - The mouse uses **OutputReport** mode by default. Some devices use **FeatureReport** - the firmware-config blob has a `feature: bool` flag per command.

> ### Request-device filter (from `cfg.json` -> `opt.mouse`)
>
> | VendorID | ProductID candidates |
> |----------|---|
> | 0x373B | 0x101B, 0xF5F4, 0xF590, 0xF5D5, 0xF53E, 0xF501, 0xF5F6, 0xFB16 |
> | 0x3554 | (same set) |

> The HID stack prepends the report ID (8) to the buffer before sending it on the wire, so on the firmware side the validating constant is:
> `sum(reportId + buf[0..15])  mod 256  ===  85  (decimal) === 0x55`

Grep of PROTOCOL.md for `firmware|upgrade|update|boot|isp|ota|dfu|reconnect` found only configuration-flash and version-read references (for example `ReadVersionID` (18), `GetDongleVersion` (29)); no bootloader VID/PID or re-enumeration description is present in this document.
