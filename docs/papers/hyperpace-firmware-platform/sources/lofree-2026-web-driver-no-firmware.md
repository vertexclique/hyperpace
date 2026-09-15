URL: https://lofree.tech/ , https://www.lofree.tech/home/ , https://hyzen.lofree.tech/ , https://www.lofree.co/pages/download-center , https://www.lofree.co/pages/hypace-custom-hub
Retrieval date: 2026-09-15
Source type: vendor pages and vendor JS bundles (primary)

## Claim
Lofree publishes NO HYPACE firmware image and NO firmware updater anywhere it controls. The only HYPACE software Lofree ships is the browser configuration app at lofree.tech, whose Upgrade button is inert.

## Evidence

### Live Lofree HYPACE driver (lofree.tech root, Vue 2 "Control HUB WEB")
- cfg.json (sha256 a6fe720f1668de619a71607467a1f6e963360eb186fd8c22993d0ee1290d8415) has NO `upgrade` block. Model cid 102 mid 1/2, `"mouse":"NRF52833"`, `"dongle1":"CX52650N","dongle2":"CX52650N","dongle4":"CH32V305"` (no dongle8), `"driverOnline":false`.
- Upgrade component (app.pretty.js): `handleDownloadClick(){window.open(window.location.href + this.upgrade.link)}`. With no `upgrade.link` the button opens the SPA itself. Button text is disabled to `LatestVersion` unless `this.upgrade.device != e.version.device` etc; `this.upgrade` is `{}`, so it stays "LATEST VERSION".
- Path probes (nginx 1.26.1): /download, /upgrade, /update, /firmware, /ota, /dfu, /firmware.bin all return the 608-byte SPA index (HTTP 200 text/html) = not found.

### Newer Lofree ODM template at /home/ (Vite "Control HUB WEB", cid 62)
- www.lofree.tech/home/cfg.json (sha256 3b9c365258018a5fc6de07105d110c17ea076ffe1e53d00ca889896356e5acd2) is a Pulsar-derived sample config left in the ODM template. It DOES contain populated `upgrade` blocks for cid 62 mid 1:
  `"upgrade":{"1000":{"version":"v2.16","link":"/pulsar_dongle_1K.bin"},"4000":{"version":"v2.16","link":"/old.bin"},"8000":{"version":"v2.16","link":"/pulsar_dongle_8k.bin"},"device":{"version":"v2.18","link":"/pulsar_mouse.bin"}}`
  and keyboard entries `KB_RGB_V1.55...bin` etc.
- Every one of those .bin links 404s on Lofree's server (returns the SPA page). So the /home/ deployment carries dead Pulsar sample links, not working Lofree firmware. The filenames ("pulsar_mouse.bin", "pulsar_dongle_8k.bin") are the ODM's Pulsar defaults, confirming the shared platform.

### Lofree download center (lofree.co)
Verbatim entries (2026-09-15 and Wayback 2025-06 to 2025-10): "Hypace Web Application - A web-based application to set up your Hypace Mouse easily, such as key setting, DPI, light effects, polling rate and more." Links: `https://www.lofree.tech/` and `https://www.lofree.tech/home/` (Set Up Online). No firmware, no updater, no .bin/.exe/.zip anywhere on the page across all Wayback snapshots. hyzen.lofree.tech is the Flow Lite keyboard hub, unrelated to HYPACE.

### hyzen.lofree.tech ("Lofree HUB", newer Vite app)
Its bundle (index-BMxm1-XU.js) contains a full online-updater subsystem (strings: `firmwareUpdate`, `onlineUpdater`, `resetToUpdateModeCmd`, `bootInputEndPoint`, `downloadPackageFailed: "Failed to download upgrade package: {path}"`, `upgradeOrderInvalid: "dongle must be upgraded first in multi-target upgrades"`, safety string `productId=36/41`). This is the same Compx updater engine, but its cfg.json/sensor.json 404 and it targets keyboards (Flow Lite), not the HYPACE. It proves Lofree HAS the updater code, but has not wired a HYPACE firmware package to it.

## Conclusion
No official Lofree HYPACE firmware exists in public distribution as of 2026-09-15. Firmware for the HYPACE's silicon must come from the shared Compx platform siblings (see pulsar/scyrox/compx source files) or be dumped from the operator's own hardware.
