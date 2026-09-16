URL: https://lofree.tech/ , https://www.lofree.tech/home/ , https://www.lofree.tech/mouse , https://www.lofree.tech/keyboard
Retrieved: 2026-09-16
Source type: vendor deployments fetched directly (primary), compared by sha256 against the copies already analyzed.

## Claim

The device chooser at /home/ hands mouse sessions to /mouse, and /mouse serves byte-identical copies of the OLD Vue driver, with a config that differs from the site root in exactly one flag. The mouse the operator owns is therefore driven in production by the OLD driver and its cid 102 config, not by the cid 62 entry in the chooser's own config.

## Evidence

### /mouse is the OLD driver

`https://www.lofree.tech/mouse` answers 301, then serves an index (Last-Modified 2025-08-06) referencing `js/app.66015056.js`, `js/chunk-vendors.1d7a523f.js`, `css/app.33f46f1c.css`. sha256 comparison against the copies fetched from the site root on 2026-09-15:

| File | sha256 | Same as site root |
|---|---|---|
| app.66015056.js | 185c71661a9163992338188b0d6fd95e228037ebdcea3c60533f54fbb1e4b94b | yes |
| chunk-vendors.1d7a523f.js | 960b11d5800c921ea2627bc5c1036b81e293828ab687ca098a44a2ce70d1a1fd | yes |
| css/app.33f46f1c.css | 6d6c5fc9ea4414da6aae5a47d964e44b65a3d0555a1b09507f19c7f8af44b177 | yes |
| sensor.json | 00ad93853cce215c068d02e1e5fca6bb8f5da4534777cd3e46210928498e1cc1 | yes |
| lang/en.json | (13778 bytes, same size as the root copy) | size match |

### The one config difference

`/mouse/cfg.json` (sha256 a2fbbac81bdcf2f224ec75ae549d80fb4e4e49ab227fbdb50e8655c60de26774) versus root `cfg.json`
(sha256 a6fe720f1668de619a71607467a1f6e963360eb186fd8c22993d0ee1290d8415) differ in exactly one line:

```
47c47
<     "test": false,      (/mouse)
---
>     "test": true,       (site root)
```

Everything else matches, including `"mouse": [{cid: 102, cfg: [{mid: 1}, {mid: 2}]}]`, `title` "Lofree HYPACE", `version` "1.0.0", `homePage` "https://www.lofree.tech/home".

`test` gates the auto-connect path in the OLD driver: `old:2438` reads `const e = !this.driverCfg.test;` and, when true, parses `localStorage` key `deviceInfo` (`old:2440`) to reconnect the last device, otherwise navigates to `cfg.homePage`. So the root deployment (`test: true`) is a bench build that never auto-connects, and `/mouse` (`test: false`) is the production deployment.

### /keyboard is a third, separate build

`https://www.lofree.tech/keyboard` (Last-Modified 2026-01-23) serves `assets/index-C3SCpSMc.js`
(735103 bytes, sha256 538c201900cdf4cfb223f13ecf0ffe4323af293b80d31c05cad1b3046505f997), which is NOT the
`/home/` bundle (`index-BTVblIUr.js`, 590728 bytes, sha256 dbec61a52a85d2739a00c596bbeb3506a2f4869a65e8a078376fa0f2a572f1c5).
Its `cfg.json` is 458382 bytes titled "Control HUB WEB 1.1.0_8/8/15"; the `/home/` one is 457625 bytes titled
"Lofree HYPACE". Both carry the same single mouse entry (cid 62, mid 1). `/home/` and `/keyboard` share a
19000-byte `lang/en.json`; the OLD driver's is 13778 bytes.

## Evidence tier

1 (vendor deployments fetched directly; hashes computed locally).

## Correctness

- Proven: the hashes, the single-line config difference, the Last-Modified dates, and that `test` gates the auto-connect branch in the OLD bundle.
- Not proven: that the chooser always routes this mouse to /mouse for every firmware or model. The chooser navigates to `origin + cfg.opt.<type>.rooter` after the user picks a device type, and the reported cid is looked up by the destination page.
- Not settled here: which cid the physical device reports. That is only knowable by asking the device, which is out of bounds.

## Relevance to hyperpace

- Resolves the protocol reference's open question "what does /mouse serve": the OLD driver, with cid 102 and mid 1 or 2, which matches this mouse's identity in every other respect.
- Hyperpace should carry both model tables (cid 102 from the OLD config, cid 62 from the chooser config) and select at runtime by the reported cid and mid, since both are vendor-published for a page titled "Lofree HYPACE".
- The chooser's cid 62 entry (6 buttons, maxDpi 32000, single-range 3950 table) is not evidence about this mouse; the production mouse page ships cid 102 (5 buttons, maxDpi 40000, two-range table).
