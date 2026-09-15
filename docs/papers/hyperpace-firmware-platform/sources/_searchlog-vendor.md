# Vendor firmware/updater search log (stream: vendor)

Retrieval date: 2026-09-15. Static analysis only; nothing sent to the attached mouse.

## Lofree official distribution
- WebSearch "Lofree HYPACE firmware update" -> only the dmtrKovalenko RE repo and the lofree.co download center. No firmware.
- WebFetch lofree.co/pages/download-center -> "Hypace Web Application" links only to www.lofree.tech/ and /home/. No .bin/.exe/.zip. Wayback download-center history 2025-05..2026-09 (20 snapshots) all reviewed: still only the web-app links; user manuals are PDFs; no firmware ever.
- WebFetch lofree.co/products/hypace-wireless-gaming-mouse, /pages/hypace-custom-hub -> "no software to download", web app only.
- Probed lofree.tech and www.lofree.tech paths: / /home /download /downloads /upgrade /update /firmware /ota /dfu /hub /driver /fw /bootloader /firmware.bin /upgrade.html + /mouse/ /keyboard/ router paths + candidate .bin names (pulsar_mouse.bin etc). All return the 608B SPA index (200 text/html) = not found. /download/ returns 403. cfg.json/sensor.json 200.
- Wayback CDX for lofree.tech, www.lofree.tech, hyzen.lofree.tech, lofree.co, lofree.com, lofree.com.cn, lofreekeyboard.com: no .bin/.exe/.zip firmware. Archived www.lofree.tech/home/cfg.json (2026-01-21) has a Pulsar sample upgrade block (cid 62) with dead links. Archived lofree.tech root cfg (2025-06) = HYPACE cid 102, no upgrade block.
- WebSearches: "lofree hypace firmware reddit"; "Lofree HYPACE 固件 升级"; "lofree hypace firmware discord"; "lofree.tech hypace upgrade receiver 4K 8K firmware bilibili/知乎/小红书" -> no firmware links, only product pages and the RE repo. Chinese social platforms not directly crawled (behind login/anti-bot); labeled unverified.

## Sibling Compx-template sites
- WebSearch "Control HUB WEB mouse driver" / "Compx mouse web driver firmware upgrade 3554" -> MCHOSE, WLMouse, Pulsar, Scyrox, ATK/VXE, Lamzu named.
- Pulsar: pulsar.gg/pages/download (full firmware + updater set); bbb.pulsar.gg web driver (WebHID, live updater) + /cMouse cfg.json/history.json/devicename.json. Downloaded and archived all cMouse images and 4 desktop packages. Wayback bbb.pulsar.gg/cMouse cfg 2025-08 and 2026-07 -> older versions, 2 lost images identified.
- Scyrox: scyrox.com/pages/download -> V8/V6 update tools (Shopify CDN) + S-Center (Huawei Cloud). Downloaded V8/V6 zips.
- ATK/VXE: desktop.vxe.com/download -> redirects to desktop.atkgear.com (ATK HUB web app, images behind API, not statically fetchable). Not downloaded (dynamic).
- OpenMouse-Project/mouse-protocol: independent TS Compx parser + issue #97 (cross-flash recovery). Read via gh API.
- MCHOSE, WLMouse, Lamzu, VGN: named as same-platform but not deep-crawled (time; Pulsar+Scyrox already give three readable implementations). Flagged as further leads.

## Compx company
- WebSearch "Compx gaming mouse ODM Shenzhen" -> Compx = fabless wireless-chip vendor (CX52850/CX52650 2.4GHz); ODM assemblers Shenzhen F&G, Newunite, Senix. "欣春旗" (Xinchunqi) watermark inside every firmware image is the actual ODM.

## Static analysis performed
- Beautified 3 WebHID bundles (lofree/home, pulsar/cMouse) and located the updater engine functions.
- Decompiled .NET updaters with dnfile (resources) + dncil (IL): UsbUpgradeFile + FormMain methods.
- Parsed all 45 firmware images (Python struct): header CRC, payload CRC, version, DeviceType, cid/mid, endpoints, entropy, vector table.
- Read the real USB flash byte-log bundled in the Pulsar 1K dongle tool.

## Dead ends
- lofree.com / lofree.com.cn / lofreekeyboard.com: no useful CDX rows (redirect/parked).
- Kickstarter FAQ/posts: 403 to WebFetch.
- Direct firmware URL guesses on lofree.tech (pulsar_mouse.bin etc): all 404 (SPA).
- bbb.pulsar.gg firmware path: images are NOT at cfg link root; real dir is /cMouse/bin/ (found by probing).
