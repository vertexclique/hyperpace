# SUMMARY: vendor page deployments (/, /home/, /mouse, /keyboard)

## Claim
The chooser at /home/ routes mouse sessions to /mouse, which serves byte-identical copies of the OLD Vue driver and a config differing from the site root in one flag only (`test`). Production mouse sessions therefore run the OLD driver with its cid 102 config; the chooser's cid 62 entry is not evidence about this mouse.

## Method
Fetched all four deployments directly on 2026-09-16, computed sha256 for each asset, diffed the configs, and located the `test` flag's effect in the pretty-printed OLD bundle.

## Result
- /mouse app bundle, vendor chunk, CSS and sensor.json are hash-identical to the site-root copies; lang/en.json matches in size (13778 bytes).
- Config diff is one line: `"test": false` at /mouse versus `"test": true` at the root. `old:2438` gates auto-reconnect on `!cfg.test`, reading localStorage `deviceInfo` (`old:2440`), else navigating to `cfg.homePage`.
- /mouse config keeps cid 102 with mid 1 and 2, title "Lofree HYPACE", version 1.0.0.
- /keyboard is a third build (index-C3SCpSMc.js, 735103 bytes, Last-Modified 2026-01-23), distinct from the /home/ bundle; both carry the same cid 62 sample mouse entry and a 19000-byte lang/en.json.

## Evidence tier
1 (vendor deployments fetched directly; hashes computed locally).

## Performance
Not applicable.

## Correctness
- Proven: hashes, the one-line config difference, dates, and what `test` gates.
- Not proven: that the chooser routes every firmware revision of this mouse to /mouse.
- Not settled: which cid the physical device reports, which only the device can answer and which is out of bounds until the operator runs Hyperpace themselves.

## Relevance to hyperpace
- Closes the protocol reference's open question about what /mouse serves.
- Hyperpace carries both model tables and picks by the cid and mid reported at runtime; the cid 102 table is the production one for this product page.
- The differing DPI ranges between the two configs mean the model table must drive DPI encoding, never a global constant.
