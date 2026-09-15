# SUMMARY: Lofree publishes no HYPACE firmware

- Claim: No official Lofree HYPACE firmware image or firmware updater exists in public distribution. Lofree ships only the browser config app; its Upgrade button is inert (no `upgrade` block in the live cfg).
- Method: Fetched and read lofree.tech (Vue2 driver), lofree.tech/home (Vite ODM template), hyzen.lofree.tech, lofree.co download center (live + Wayback 2025-2026). Probed ~30 paths. Beautified and grepped both bundles. Diffed archived vs live cfg.json.
- Result: Live HYPACE cfg has no upgrade block. The /home/ template carries a Pulsar sample cfg (cid 62) whose .bin links 404 on Lofree's server. hyzen bundle has a full Compx online-updater engine but wired to keyboards, not the HYPACE. Download center only links the web config apps.
- Evidence tier: 1 (vendor pages and vendor JS).
- Performance: n/a (distribution finding).
- Correctness: Proven that nothing is served now and across all Wayback snapshots checked. Cannot prove Lofree will never publish; a future cfg could add an upgrade block. Chinese social platforms not directly crawled (labeled in search log).
- Relevance: Decides Hyperpace cannot rely on a Lofree download; it must dump firmware from hardware or reuse the Compx DFU protocol. Firmware image identity for matching must come from the device itself.
