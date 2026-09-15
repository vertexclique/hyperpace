# SUMMARY: Pulsar WebHID updater (closest working reference) and firmware set

- Claim: Pulsar's bbb.pulsar.gg Bibimbap web driver is a live, working WebHID flasher on the identical Compx template, and its readable JS is the best code reference for Hyperpace.
- Method: Fetched the SPA, beautified the cMouse bundle, extracted the updater engine, mapped functions to line numbers, downloaded every image its cfg names, parsed each header. Pulled Wayback cfg snapshots to find lost versions.
- Result: Engine functions (framing/chunk/checksum/verify/version) located with line refs; images v3.03-v5.09 (mouse), v2.08-v5.04 (dongles) archived; history.json documents the dongle-first order and USB-cable requirement. Two older images gone from live cfg identified.
- Evidence tier: 1 (vendor WebHID app + vendor images).
- Performance: WebHID sendReport loop, 32 B/packet; same as spec.
- Correctness: cfg/images self-consistent and header-CRC-valid. Not HYPACE-targeted (cid 87, foreign PIDs), so useful as protocol reference only.
- Relevance: Diff Hyperpace's WebHID/Rust flasher against this JS line by line.
